//! Direct raster backend that writes pixels into a raw RGBA buffer.
//!
//! Draws circles, rectangles, and lines with simple scanline algorithms
//! (no anti-aliasing), matching the approach of plotters' BitMapBackend.
//! Text is composited via a resvg overlay for correct font shaping.

use std::sync::{Arc, OnceLock};

use resvg::tiny_skia::{self, Pixmap, Transform};

use crate::render::color::Color;
use crate::render::render::{Primitive, Scene, TextAnchor};

fn shared_fontdb() -> Arc<resvg::usvg::fontdb::Database> {
    static FONTDB: OnceLock<Arc<resvg::usvg::fontdb::Database>> = OnceLock::new();
    FONTDB
        .get_or_init(|| {
            let mut db = resvg::usvg::fontdb::Database::new();
            db.load_system_fonts();
            Arc::new(db)
        })
        .clone()
}

pub struct RasterBackend {
    pub scale: f32,
}

impl Default for RasterBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RasterBackend {
    pub fn new() -> Self {
        Self { scale: 2.0 }
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn render_scene(&self, scene: &Scene) -> Result<Vec<u8>, String> {
        let w = (scene.width as f32 * self.scale).ceil() as u32;
        let h = (scene.height as f32 * self.scale).ceil() as u32;
        if w == 0 || h == 0 {
            return Err("scene has zero dimensions".into());
        }

        let mut pixmap =
            Pixmap::new(w, h).ok_or_else(|| "failed to allocate pixmap".to_string())?;

        let s = self.scale;

        if let Some(ref bg) = scene.background_color {
            let bg_color: Color = bg.as_str().into();
            if let Some(rgba) = color_to_rgba(&bg_color) {
                let data = pixmap.data_mut();
                for chunk in data.chunks_exact_mut(4) {
                    chunk.copy_from_slice(&rgba);
                }
            }
        }

        let mut text_primitives: Vec<&Primitive> = Vec::new();
        let mut path_primitives: Vec<&crate::render::render::PathData> = Vec::new();

        // First pass: direct pixel writes for circles, rects, lines, batches.
        // Collect paths and text for second pass.
        {
            let buf = pixmap.data_mut();
            for elem in &scene.elements {
                match elem {
                    Primitive::Circle { cx, cy, r, fill } => {
                        if let Some(rgba) = color_to_rgba(fill) {
                            pixel_circle(buf, w, h, *cx as f32 * s, *cy as f32 * s, *r as f32 * s, rgba);
                        }
                    }
                    Primitive::Rect { x, y, width, height, fill, opacity, .. } => {
                        if let Some(mut rgba) = color_to_rgba(fill) {
                            if let Some(op) = opacity {
                                rgba[3] = ((*op as f32).clamp(0.0, 1.0) * 255.0) as u8;
                            }
                            pixel_rect(buf, w, h,
                                (*x as f32 * s) as i32, (*y as f32 * s) as i32,
                                (*width as f32 * s) as u32, (*height as f32 * s) as u32,
                                rgba);
                        }
                    }
                    Primitive::Line { x1, y1, x2, y2, stroke, stroke_width, .. } => {
                        if let Some(rgba) = color_to_rgba(stroke) {
                            let sw = (*stroke_width as f32 * s).max(1.0);
                            if sw <= 1.5 {
                                pixel_line(buf, w, h,
                                    (*x1 as f32 * s) as i32, (*y1 as f32 * s) as i32,
                                    (*x2 as f32 * s) as i32, (*y2 as f32 * s) as i32,
                                    rgba);
                            } else {
                                pixel_thick_line(buf, w, h,
                                    *x1 as f32 * s, *y1 as f32 * s,
                                    *x2 as f32 * s, *y2 as f32 * s,
                                    sw, rgba);
                            }
                        }
                    }
                    Primitive::Path(pd) => {
                        path_primitives.push(pd);
                    }
                    Primitive::Text { .. } => {
                        text_primitives.push(elem);
                    }
                    Primitive::CircleBatch { cx, cy, r, fill } => {
                        if let Some(rgba) = color_to_rgba(fill) {
                            let sr = *r as f32 * s;
                            for i in 0..cx.len() {
                                pixel_circle(buf, w, h, cx[i] as f32 * s, cy[i] as f32 * s, sr, rgba);
                            }
                        }
                    }
                    Primitive::RectBatch { x, y, w: rw, h: rh, fills } => {
                        for i in 0..x.len() {
                            if let Some(rgba) = color_to_rgba(&fills[i]) {
                                pixel_rect(buf, w, h,
                                    (x[i] as f32 * s) as i32, (y[i] as f32 * s) as i32,
                                    (rw[i] as f32 * s) as u32, (rh[i] as f32 * s) as u32,
                                    rgba);
                            }
                        }
                    }
                    Primitive::GroupStart { .. } | Primitive::GroupEnd => {}
                }
            }
        } // drop buf borrow

        // Second pass: render paths via tiny_skia (needs &mut pixmap).
        for pd in &path_primitives {
            render_path_with_skia(&mut pixmap, s, pd);
        }

        // Render text via a minimal SVG overlay through resvg, then alpha-composite.
        if !text_primitives.is_empty() {
            let text_svg = build_text_svg(scene, &text_primitives);
            let options = resvg::usvg::Options {
                fontdb: shared_fontdb(),
                ..Default::default()
            };
            if let Ok(tree) = resvg::usvg::Tree::from_str(&text_svg, &options) {
                let transform = Transform::from_scale(s, s);
                // Render text into a separate pixmap, then composite.
                if let Some(mut text_pm) = Pixmap::new(w, h) {
                    resvg::render(&tree, transform, &mut text_pm.as_mut());
                    alpha_composite(pixmap.data_mut(), text_pm.data(), w, h);
                }
            }
        }

        pixmap.encode_png().map_err(|e| e.to_string())
    }
}

// ── Direct pixel-buffer drawing primitives ──────────────────────────────────

/// Filled circle via bounding-box scan. No anti-aliasing.
#[inline]
fn pixel_circle(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, rgba: [u8; 4]) {
    let r2 = r * r;
    let x_min = ((cx - r).floor() as i32).max(0) as u32;
    let x_max = ((cx + r).ceil() as i32).min(w as i32 - 1).max(0) as u32;
    let y_min = ((cy - r).floor() as i32).max(0) as u32;
    let y_max = ((cy + r).ceil() as i32).min(h as i32 - 1).max(0) as u32;

    for py in y_min..=y_max {
        let dy = py as f32 + 0.5 - cy;
        let dy2 = dy * dy;
        let row = (py * w) as usize * 4;
        for px in x_min..=x_max {
            let dx = px as f32 + 0.5 - cx;
            if dx * dx + dy2 <= r2 {
                let off = row + px as usize * 4;
                // SAFETY: bounds checked by x_min/x_max/y_min/y_max clamping
                buf[off..off + 4].copy_from_slice(&rgba);
            }
        }
    }
}

/// Filled axis-aligned rectangle via scanline fill. No anti-aliasing.
#[inline]
fn pixel_rect(buf: &mut [u8], bw: u32, bh: u32, x: i32, y: i32, w: u32, h: u32, rgba: [u8; 4]) {
    let x0 = x.max(0) as u32;
    let y0 = y.max(0) as u32;
    let x1 = ((x as i64 + w as i64) as u32).min(bw);
    let y1 = ((y as i64 + h as i64) as u32).min(bh);
    if x0 >= x1 || y0 >= y1 { return; }

    let span = (x1 - x0) as usize;
    for py in y0..y1 {
        let row_start = (py * bw + x0) as usize * 4;
        let row_end = row_start + span * 4;
        for chunk in buf[row_start..row_end].chunks_exact_mut(4) {
            chunk.copy_from_slice(&rgba);
        }
    }
}

/// Bresenham line (1px). No anti-aliasing.
#[inline]
fn pixel_line(buf: &mut [u8], w: u32, h: u32, mut x0: i32, mut y0: i32, x1: i32, y1: i32, rgba: [u8; 4]) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x0 >= 0 && y0 >= 0 && (x0 as u32) < w && (y0 as u32) < h {
            let off = (y0 as u32 * w + x0 as u32) as usize * 4;
            buf[off..off + 4].copy_from_slice(&rgba);
        }
        if x0 == x1 && y0 == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x0 += sx; }
        if e2 <= dx { err += dx; y0 += sy; }
    }
}

/// Thick line drawn as a filled rectangle rotated along the line direction.
fn pixel_thick_line(buf: &mut [u8], w: u32, h: u32, x0: f32, y0: f32, x1: f32, y1: f32, thickness: f32, rgba: [u8; 4]) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.5 { return; }

    let half = thickness * 0.5;
    // Normal perpendicular to the line direction
    let nx = -dy / len * half;
    let ny = dx / len * half;

    // Four corners of the thick line rectangle
    let corners = [
        (x0 + nx, y0 + ny), (x0 - nx, y0 - ny),
        (x1 - nx, y1 - ny), (x1 + nx, y1 + ny),
    ];

    // Bounding box
    let min_x = corners.iter().map(|c| c.0).fold(f32::INFINITY, f32::min).floor() as i32;
    let max_x = corners.iter().map(|c| c.0).fold(f32::NEG_INFINITY, f32::max).ceil() as i32;
    let min_y = corners.iter().map(|c| c.1).fold(f32::INFINITY, f32::min).floor() as i32;
    let max_y = corners.iter().map(|c| c.1).fold(f32::NEG_INFINITY, f32::max).ceil() as i32;

    let min_x = min_x.max(0) as u32;
    let max_x = (max_x as u32).min(w.saturating_sub(1));
    let min_y = min_y.max(0) as u32;
    let max_y = (max_y as u32).min(h.saturating_sub(1));

    // Point-in-convex-polygon test via cross products
    for py in min_y..=max_y {
        let row = (py * w) as usize * 4;
        for px in min_x..=max_x {
            let fx = px as f32 + 0.5;
            let fy = py as f32 + 0.5;
            if point_in_quad(fx, fy, &corners) {
                let off = row + px as usize * 4;
                buf[off..off + 4].copy_from_slice(&rgba);
            }
        }
    }
}

fn point_in_quad(px: f32, py: f32, corners: &[(f32, f32); 4]) -> bool {
    for i in 0..4 {
        let (ax, ay) = corners[i];
        let (bx, by) = corners[(i + 1) % 4];
        let cross = (bx - ax) * (py - ay) - (by - ay) * (px - ax);
        if cross < 0.0 { return false; }
    }
    true
}

/// Alpha-composite src (premultiplied RGBA) onto dst (premultiplied RGBA).
fn alpha_composite(dst: &mut [u8], src: &[u8], _w: u32, _h: u32) {
    for (d, s) in dst.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
        let sa = s[3] as u32;
        if sa == 0 { continue; }
        if sa == 255 {
            d.copy_from_slice(s);
            continue;
        }
        let inv = 255 - sa;
        d[0] = ((s[0] as u32 * 255 + d[0] as u32 * inv) / 255) as u8;
        d[1] = ((s[1] as u32 * 255 + d[1] as u32 * inv) / 255) as u8;
        d[2] = ((s[2] as u32 * 255 + d[2] as u32 * inv) / 255) as u8;
        d[3] = ((sa * 255 + d[3] as u32 * inv) / 255) as u8;
    }
}

// ── Path rendering (falls back to tiny_skia for curves) ─────────────────────

fn render_path_with_skia(pixmap: &mut Pixmap, scale: f32, pd: &crate::render::render::PathData) {
    use resvg::tiny_skia::{Color, FillRule, Paint, Stroke};
    let transform = Transform::from_scale(scale, scale);

    if let Some(path) = parse_svg_path(&pd.d) {
        if let Some(ref fill_color) = pd.fill {
            if let Some(mut color) = css_color_to_skia(fill_color) {
                if let Some(op) = pd.opacity {
                    let a = (op as f32).clamp(0.0, 1.0) * color.alpha();
                    color = Color::from_rgba(color.red(), color.green(), color.blue(), a)
                        .unwrap_or(color);
                }
                let mut paint = Paint::default();
                paint.set_color(color);
                paint.anti_alias = true;
                pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
            }
        }
        if !matches!(pd.stroke, crate::render::color::Color::None) {
            if let Some(color) = css_color_to_skia(&pd.stroke) {
                let mut paint = Paint::default();
                paint.set_color(color);
                paint.anti_alias = true;
                let mut sk_stroke = Stroke::default();
                sk_stroke.width = pd.stroke_width as f32;
                pixmap.stroke_path(&path, &paint, &sk_stroke, transform, None);
            }
        }
    }
}

fn css_color_to_skia(c: &Color) -> Option<tiny_skia::Color> {
    match c {
        Color::Rgb(r, g, b) => Some(tiny_skia::Color::from_rgba8(*r, *g, *b, 255)),
        Color::None => None,
        Color::Css(s) => parse_css_color(s),
    }
}

fn parse_css_color(s: &str) -> Option<tiny_skia::Color> {
    let s = s.trim();
    if s.is_empty() || s.eq_ignore_ascii_case("none") { return None; }
    if s.len() == 7 && s.as_bytes()[0] == b'#' {
        let r = u8::from_str_radix(&s[1..3], 16).ok()?;
        let g = u8::from_str_radix(&s[3..5], 16).ok()?;
        let b = u8::from_str_radix(&s[5..7], 16).ok()?;
        return Some(tiny_skia::Color::from_rgba8(r, g, b, 255));
    }
    if let Some(inner) = s.strip_prefix("rgb(").and_then(|t| t.strip_suffix(')')) {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3 {
            let r = parts[0].trim().parse::<f64>().ok()?.round() as u8;
            let g = parts[1].trim().parse::<f64>().ok()?.round() as u8;
            let b = parts[2].trim().parse::<f64>().ok()?.round() as u8;
            return Some(tiny_skia::Color::from_rgba8(r, g, b, 255));
        }
    }
    None
}

// ── Color conversion ────────────────────────────────────────────────────────

/// Convert a kuva Color to premultiplied RGBA bytes for direct pixel writes.
#[inline]
fn color_to_rgba(c: &Color) -> Option<[u8; 4]> {
    match c {
        Color::Rgb(r, g, b) => Some([*r, *g, *b, 255]),
        Color::None => None,
        Color::Css(s) => {
            let s = s.trim();
            if s.is_empty() || s.eq_ignore_ascii_case("none") { return None; }
            if s.len() == 7 && s.as_bytes()[0] == b'#' {
                let r = u8::from_str_radix(&s[1..3], 16).ok()?;
                let g = u8::from_str_radix(&s[3..5], 16).ok()?;
                let b = u8::from_str_radix(&s[5..7], 16).ok()?;
                return Some([r, g, b, 255]);
            }
            if let Some(inner) = s.strip_prefix("rgb(").and_then(|t| t.strip_suffix(')')) {
                let parts: Vec<&str> = inner.split(',').collect();
                if parts.len() == 3 {
                    let r = parts[0].trim().parse::<f64>().ok()?.round() as u8;
                    let g = parts[1].trim().parse::<f64>().ok()?.round() as u8;
                    let b = parts[2].trim().parse::<f64>().ok()?.round() as u8;
                    return Some([r, g, b, 255]);
                }
            }
            None
        }
    }
}

// ── Text overlay SVG ────────────────────────────────────────────────────────

fn build_text_svg(scene: &Scene, texts: &[&Primitive]) -> String {
    use std::fmt::Write;

    let mut svg = String::with_capacity(texts.len() * 120 + 200);
    let _ = write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}""#,
        scene.width, scene.height
    );
    if let Some(ref family) = scene.font_family {
        let _ = write!(svg, r#" font-family="{family}""#);
    }
    if let Some(ref color) = scene.text_color {
        let _ = write!(svg, r#" fill="{color}""#);
    }
    svg.push('>');

    for elem in texts {
        if let Primitive::Text { x, y, content, size, anchor, rotate, bold } = elem {
            let anchor_str = match anchor {
                TextAnchor::Start => "start",
                TextAnchor::Middle => "middle",
                TextAnchor::End => "end",
            };
            let _ = write!(svg, r#"<text x="{x}" y="{y}" font-size="{size}" text-anchor="{anchor_str}""#);
            if *bold { svg.push_str(r#" font-weight="bold""#); }
            if let Some(angle) = rotate {
                let _ = write!(svg, r#" transform="rotate({angle},{x},{y})""#);
            }
            svg.push('>');
            for b in content.bytes() {
                match b {
                    b'&' => svg.push_str("&amp;"),
                    b'<' => svg.push_str("&lt;"),
                    b'>' => svg.push_str("&gt;"),
                    b'"' => svg.push_str("&quot;"),
                    _ => svg.push(b as char),
                }
            }
            svg.push_str("</text>");
        }
    }
    svg.push_str("</svg>");
    svg
}

// ── SVG path parser (for Path fallback) ─────────────────────────────────────

fn parse_svg_path(d: &str) -> Option<tiny_skia::Path> {
    use resvg::tiny_skia::PathBuilder;
    let mut pb = PathBuilder::new();
    let chars = d.as_bytes();
    let mut i = 0;

    fn skip_ws(data: &[u8], pos: &mut usize) {
        while *pos < data.len() && matches!(data[*pos], b' ' | b',' | b'\n' | b'\r' | b'\t') {
            *pos += 1;
        }
    }
    fn parse_f32(data: &[u8], pos: &mut usize) -> Option<f32> {
        skip_ws(data, pos);
        let start = *pos;
        if *pos < data.len() && matches!(data[*pos], b'-' | b'+') { *pos += 1; }
        let mut has_dot = false;
        while *pos < data.len() && (data[*pos].is_ascii_digit() || (data[*pos] == b'.' && !has_dot)) {
            if data[*pos] == b'.' { has_dot = true; }
            *pos += 1;
        }
        if *pos < data.len() && matches!(data[*pos], b'e' | b'E') {
            *pos += 1;
            if *pos < data.len() && matches!(data[*pos], b'-' | b'+') { *pos += 1; }
            while *pos < data.len() && data[*pos].is_ascii_digit() { *pos += 1; }
        }
        if start == *pos { return None; }
        std::str::from_utf8(&data[start..*pos]).ok()?.parse().ok()
    }

    while i < chars.len() {
        skip_ws(chars, &mut i);
        if i >= chars.len() { break; }
        let cmd = chars[i];
        if cmd.is_ascii_alphabetic() { i += 1; }
        match cmd {
            b'M' => {
                let x = parse_f32(chars, &mut i)?;
                let y = parse_f32(chars, &mut i)?;
                pb.move_to(x, y);
                loop {
                    skip_ws(chars, &mut i);
                    if i >= chars.len() || chars[i].is_ascii_alphabetic() { break; }
                    let x = parse_f32(chars, &mut i)?;
                    let y = parse_f32(chars, &mut i)?;
                    pb.line_to(x, y);
                }
            }
            b'L' => loop {
                let x = parse_f32(chars, &mut i)?;
                let y = parse_f32(chars, &mut i)?;
                pb.line_to(x, y);
                skip_ws(chars, &mut i);
                if i >= chars.len() || chars[i].is_ascii_alphabetic() { break; }
            },
            b'C' => loop {
                let x1 = parse_f32(chars, &mut i)?; let y1 = parse_f32(chars, &mut i)?;
                let x2 = parse_f32(chars, &mut i)?; let y2 = parse_f32(chars, &mut i)?;
                let x = parse_f32(chars, &mut i)?; let y = parse_f32(chars, &mut i)?;
                pb.cubic_to(x1, y1, x2, y2, x, y);
                skip_ws(chars, &mut i);
                if i >= chars.len() || chars[i].is_ascii_alphabetic() { break; }
            },
            b'A' => loop {
                let _ = parse_f32(chars, &mut i)?; let _ = parse_f32(chars, &mut i)?;
                let _ = parse_f32(chars, &mut i)?;
                skip_ws(chars, &mut i);
                if i < chars.len() && matches!(chars[i], b'0' | b'1') { i += 1; }
                skip_ws(chars, &mut i);
                if i < chars.len() && matches!(chars[i], b'0' | b'1') { i += 1; }
                let x = parse_f32(chars, &mut i)?; let y = parse_f32(chars, &mut i)?;
                pb.line_to(x, y);
                skip_ws(chars, &mut i);
                if i >= chars.len() || chars[i].is_ascii_alphabetic() { break; }
            },
            b'Z' | b'z' => { pb.close(); }
            _ => { i += 1; }
        }
    }
    pb.finish()
}
