//! Direct raster backend that writes pixels into a raw RGBA buffer.
//!
//! Draws circles, rectangles, and lines with simple scanline algorithms
//! (no anti-aliasing), matching the approach of plotters' BitMapBackend.
//! Text is rendered via fontdue glyph rasterization directly into the
//! pixel buffer — no SVG round-trip, no second pixmap allocation.

use std::sync::OnceLock;

use resvg::tiny_skia::{self, Pixmap, Transform};

use crate::render::color::Color;
use crate::render::render::{Primitive, Scene, TextAnchor};

/// Cached fontdue font loaded from system or embedded fallback.
fn shared_font() -> &'static fontdue::Font {
    static FONT: OnceLock<fontdue::Font> = OnceLock::new();
    FONT.get_or_init(|| {
        // Try common system sans-serif fonts
        let candidates = if cfg!(target_os = "macos") {
            vec![
                "/System/Library/Fonts/Helvetica.ttc",
                "/System/Library/Fonts/SFNSText.ttf",
                "/System/Library/Fonts/SFNS.ttf",
                "/Library/Fonts/Arial.ttf",
            ]
        } else {
            vec![
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
                "/usr/share/fonts/TTF/DejaVuSans.ttf",
                "/usr/share/fonts/noto/NotoSans-Regular.ttf",
                "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
            ]
        };

        for path in &candidates {
            if let Ok(data) = std::fs::read(path) {
                if let Ok(font) = fontdue::Font::from_bytes(
                    data,
                    fontdue::FontSettings::default(),
                ) {
                    return font;
                }
            }
        }

        // Fallback: generate a minimal bitmap font by using fontdue with
        // whatever the first available system font is via fontdb
        let mut db = resvg::usvg::fontdb::Database::new();
        db.load_system_fonts();
        for face in db.faces() {
            if let resvg::usvg::fontdb::Source::File(ref path) = face.source {
                if let Ok(data) = std::fs::read(path) {
                    if let Ok(font) = fontdue::Font::from_bytes(
                        data,
                        fontdue::FontSettings {
                            collection_index: face.index,
                            ..Default::default()
                        },
                    ) {
                        return font;
                    }
                }
            }
        }

        panic!("no usable font found on this system");
    })
}

pub struct RasterBackend {
    pub scale: f32,
    /// Skip text rendering (axis labels, titles) for maximum speed.
    /// Useful when the frontend renders its own labels over the image.
    pub skip_text: bool,
}

impl Default for RasterBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RasterBackend {
    pub fn new() -> Self {
        Self { scale: 2.0, skip_text: false }
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Skip text rendering for maximum throughput.
    /// Axis labels and titles will not appear in the output image.
    pub fn with_skip_text(mut self, skip: bool) -> Self {
        self.skip_text = skip;
        self
    }

    /// Render to a raw RGBA byte buffer (no PNG encoding).
    /// Returns `(width, height, rgba_data)`.
    pub fn render_scene_to_rgba(&self, scene: &Scene) -> Result<(u32, u32, Vec<u8>), String> {
        let pixmap = self.render_scene_to_pixmap(scene)?;
        let w = pixmap.width();
        let h = pixmap.height();
        Ok((w, h, pixmap.data().to_vec()))
    }

    /// Render to PNG-encoded bytes.
    pub fn render_scene(&self, scene: &Scene) -> Result<Vec<u8>, String> {
        let pixmap = self.render_scene_to_pixmap(scene)?;
        pixmap.encode_png().map_err(|e| e.to_string())
    }

    /// Render into a tiny_skia Pixmap (raw RGBA). Use this when you need
    /// the pixel data without PNG encoding overhead, or when you want to
    /// encode in a different format (JPEG, WebP, etc.).
    pub fn render_scene_to_pixmap(&self, scene: &Scene) -> Result<Pixmap, String> {
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
        }

        for pd in &path_primitives {
            render_path_with_skia(&mut pixmap, s, pd);
        }

        if !text_primitives.is_empty() && !self.skip_text {
            let font = shared_font();
            let buf = pixmap.data_mut();
            for elem in &text_primitives {
                if let Primitive::Text { x, y, content, size, anchor, rotate, bold: _ } = elem {
                    let px_size = *size as f32 * s;
                    render_text_fontdue(buf, w, h, font, content, *x as f32 * s, *y as f32 * s, px_size, anchor, rotate.map(|a| a as f32));
                }
            }
        }

        Ok(pixmap)
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

/// Render a text string into the RGBA buffer using fontdue glyph rasterization.
fn render_text_fontdue(
    buf: &mut [u8], w: u32, h: u32,
    font: &fontdue::Font,
    text: &str,
    x: f32, y: f32,
    px_size: f32,
    anchor: &TextAnchor,
    rotate: Option<f32>,
) {
    if text.is_empty() || px_size < 1.0 { return; }

    // Measure total advance width for anchoring
    let mut total_width: f32 = 0.0;
    for ch in text.chars() {
        let metrics = font.metrics(ch, px_size);
        total_width += metrics.advance_width;
    }

    let x_offset = match anchor {
        TextAnchor::Start => 0.0,
        TextAnchor::Middle => -total_width / 2.0,
        TextAnchor::End => -total_width,
    };

    // Baseline: SVG text y is the baseline. fontdue metrics give top-relative coords.
    // We need to shift up by the ascent.
    let line_metrics = font.horizontal_line_metrics(px_size);
    let ascent = line_metrics.map(|m| m.ascent).unwrap_or(px_size * 0.8);

    if rotate.is_some() {
        // For rotated text, rasterize to a temp buffer then rotate-blit.
        render_text_rotated(buf, w, h, font, text, x, y, px_size, x_offset, ascent, rotate.unwrap());
        return;
    }

    // Non-rotated fast path: blit glyphs directly
    let text_color: [u8; 3] = [0, 0, 0]; // axis labels are black
    let mut cursor_x = x + x_offset;
    let base_y = y - ascent;

    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, px_size);
        if metrics.width == 0 || metrics.height == 0 {
            cursor_x += metrics.advance_width;
            continue;
        }

        let gx = (cursor_x + metrics.xmin as f32) as i32;
        let gy = (base_y + (px_size - metrics.height as f32 - metrics.ymin as f32)) as i32;

        blit_glyph(buf, w, h, &bitmap, metrics.width, metrics.height, gx, gy, &text_color);
        cursor_x += metrics.advance_width;
    }
}

/// Render rotated text by rasterizing into a temp buffer then rotating pixels.
fn render_text_rotated(
    buf: &mut [u8], w: u32, h: u32,
    font: &fontdue::Font,
    text: &str,
    cx: f32, cy: f32,
    px_size: f32,
    x_offset: f32,
    ascent: f32,
    angle_deg: f32,
) {
    let text_color: [u8; 3] = [0, 0, 0];
    let angle_rad = angle_deg * std::f32::consts::PI / 180.0;
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();

    let mut cursor_x = x_offset;
    let base_y = -ascent;

    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, px_size);
        if metrics.width == 0 || metrics.height == 0 {
            cursor_x += metrics.advance_width;
            continue;
        }

        let gx0 = cursor_x + metrics.xmin as f32;
        let gy0 = base_y + (px_size - metrics.height as f32 - metrics.ymin as f32);

        for row in 0..metrics.height {
            for col in 0..metrics.width {
                let alpha = bitmap[row * metrics.width + col];
                if alpha == 0 { continue; }

                let lx = gx0 + col as f32;
                let ly = gy0 + row as f32;
                let rx = cx + lx * cos_a - ly * sin_a;
                let ry = cy + lx * sin_a + ly * cos_a;

                let px = rx as i32;
                let py = ry as i32;
                if px >= 0 && py >= 0 && (px as u32) < w && (py as u32) < h {
                    let off = (py as u32 * w + px as u32) as usize * 4;
                    let inv = 255 - alpha as u32;
                    buf[off]     = ((text_color[0] as u32 * alpha as u32 + buf[off] as u32 * inv) / 255) as u8;
                    buf[off + 1] = ((text_color[1] as u32 * alpha as u32 + buf[off + 1] as u32 * inv) / 255) as u8;
                    buf[off + 2] = ((text_color[2] as u32 * alpha as u32 + buf[off + 2] as u32 * inv) / 255) as u8;
                    buf[off + 3] = 255;
                }
            }
        }
        cursor_x += metrics.advance_width;
    }
}

/// Blit a fontdue glyph bitmap (alpha mask) into the RGBA buffer.
#[inline]
fn blit_glyph(
    buf: &mut [u8], w: u32, h: u32,
    bitmap: &[u8], gw: usize, gh: usize,
    gx: i32, gy: i32,
    color: &[u8; 3],
) {
    for row in 0..gh {
        let py = gy + row as i32;
        if py < 0 || py as u32 >= h { continue; }
        let dst_row = py as u32 * w;
        let src_row = row * gw;
        for col in 0..gw {
            let px = gx + col as i32;
            if px < 0 || px as u32 >= w { continue; }
            let alpha = bitmap[src_row + col];
            if alpha == 0 { continue; }
            let off = (dst_row + px as u32) as usize * 4;
            if alpha == 255 {
                buf[off] = color[0];
                buf[off + 1] = color[1];
                buf[off + 2] = color[2];
                buf[off + 3] = 255;
            } else {
                let inv = 255 - alpha as u32;
                buf[off]     = ((color[0] as u32 * alpha as u32 + buf[off] as u32 * inv) / 255) as u8;
                buf[off + 1] = ((color[1] as u32 * alpha as u32 + buf[off + 1] as u32 * inv) / 255) as u8;
                buf[off + 2] = ((color[2] as u32 * alpha as u32 + buf[off + 2] as u32 * inv) / 255) as u8;
                buf[off + 3] = 255;
            }
        }
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
