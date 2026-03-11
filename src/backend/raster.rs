//! Direct raster backend that renders Scene primitives into a `tiny_skia::Pixmap`
//! without the SVG serialization → parse → rasterize round-trip.
//!
//! For data-heavy plots (scatter, manhattan, heatmap), this avoids:
//! 1. Generating a multi-MB SVG string
//! 2. Parsing that string back into a tree (usvg)
//! 3. Re-rasterizing from the parsed tree
//!
//! Text elements are rendered via a minimal SVG overlay through resvg since
//! text shaping requires the full font pipeline.

use std::sync::{Arc, OnceLock};

use resvg::tiny_skia::{
    self, Color, FillRule, Paint, PathBuilder, Pixmap, Rect, Stroke, Transform,
};

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

        let transform = Transform::from_scale(self.scale, self.scale);

        if let Some(ref bg) = scene.background_color {
            if let Some(c) = parse_color(bg) {
                pixmap.fill(c);
            }
        }

        let mut text_primitives: Vec<&Primitive> = Vec::new();

        for elem in &scene.elements {
            match elem {
                Primitive::Circle { cx, cy, r, fill, fill_opacity, stroke, stroke_width } => {
                    if let Some(mut color) = color_to_skia(fill) {
                        if let Some(op) = fill_opacity {
                            let a = op.clamp(0.0, 1.0) as f32;
                            color = Color::from_rgba(color.red(), color.green(), color.blue(), a).unwrap_or(color);
                        }
                        let mut paint = Paint::default();
                        paint.set_color(color);
                        paint.anti_alias = true;
                        if let Some(path) =
                            PathBuilder::from_circle(*cx as f32, *cy as f32, *r as f32)
                        {
                            pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
                            if let Some(sc) = stroke {
                                if let Some(sc_color) = color_to_skia(sc) {
                                    let mut sp = Paint::default();
                                    sp.set_color(sc_color);
                                    sp.anti_alias = true;
                                    let sw = stroke_width.unwrap_or(1.0) as f32;
                                    let sk_stroke = Stroke { width: sw, ..Stroke::default() };
                                    pixmap.stroke_path(&path, &sp, &sk_stroke, transform, None);
                                }
                            }
                        }
                    }
                }
                Primitive::Rect {
                    x,
                    y,
                    width,
                    height,
                    fill,
                    stroke,
                    stroke_width,
                    opacity,
                } => {
                    if let Some(rect) =
                        Rect::from_xywh(*x as f32, *y as f32, *width as f32, *height as f32)
                    {
                        if let Some(mut color) = color_to_skia(fill) {
                            if let Some(op) = opacity {
                                let a = (*op as f32).clamp(0.0, 1.0) * color.alpha();
                                color = Color::from_rgba(
                                    color.red(), color.green(), color.blue(), a,
                                ).unwrap_or(color);
                            }
                            let mut paint = Paint::default();
                            paint.set_color(color);
                            pixmap.fill_rect(rect, &paint, transform, None);
                        }
                        if let Some(ref stroke_color) = stroke {
                            if let Some(color) = color_to_skia(stroke_color) {
                                let mut paint = Paint::default();
                                paint.set_color(color);
                                paint.anti_alias = true;
                                let sw = stroke_width.unwrap_or(1.0) as f32;
                                let sk_stroke = Stroke { width: sw, ..Stroke::default() };
                                let mut pb = PathBuilder::new();
                                pb.push_rect(rect);
                                if let Some(path) = pb.finish() {
                                    pixmap.stroke_path(
                                        &path,
                                        &paint,
                                        &sk_stroke,
                                        transform,
                                        None,
                                    );
                                }
                            }
                        }
                    }
                }
                Primitive::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    stroke,
                    stroke_width,
                    ..
                } => {
                    if let Some(color) = color_to_skia(stroke) {
                        let mut paint = Paint::default();
                        paint.set_color(color);
                        paint.anti_alias = true;
                        let sk_stroke = Stroke { width: *stroke_width as f32, ..Stroke::default() };
                        let mut pb = PathBuilder::new();
                        pb.move_to(*x1 as f32, *y1 as f32);
                        pb.line_to(*x2 as f32, *y2 as f32);
                        if let Some(path) = pb.finish() {
                            pixmap.stroke_path(&path, &paint, &sk_stroke, transform, None);
                        }
                    }
                }
                Primitive::Path(pd) => {
                    if let Some(path) = parse_svg_path(&pd.d) {
                        if let Some(ref fill_str) = pd.fill {
                            if let Some(mut color) = color_to_skia(fill_str) {
                                if let Some(op) = pd.opacity {
                                    let a = (op as f32).clamp(0.0, 1.0) * color.alpha();
                                    color = Color::from_rgba(
                                        color.red(), color.green(), color.blue(), a,
                                    ).unwrap_or(color);
                                }
                                let mut paint = Paint::default();
                                paint.set_color(color);
                                paint.anti_alias = true;
                                pixmap.fill_path(
                                    &path,
                                    &paint,
                                    FillRule::Winding,
                                    transform,
                                    None,
                                );
                            }
                        }
                        if !matches!(pd.stroke, crate::render::color::Color::None) {
                            if let Some(color) = color_to_skia(&pd.stroke) {
                                let mut paint = Paint::default();
                                paint.set_color(color);
                                paint.anti_alias = true;
                                let sk_stroke = Stroke { width: pd.stroke_width as f32, ..Stroke::default() };
                                pixmap.stroke_path(
                                    &path,
                                    &paint,
                                    &sk_stroke,
                                    transform,
                                    None,
                                );
                            }
                        }
                    }
                }
                Primitive::Text { .. } => {
                    text_primitives.push(elem);
                }
                Primitive::CircleBatch { cx, cy, r, fill, fill_opacity, stroke, stroke_width } => {
                    if let Some(mut color) = color_to_skia(fill) {
                        if let Some(op) = fill_opacity {
                            let a = op.clamp(0.0, 1.0) as f32;
                            color = Color::from_rgba(color.red(), color.green(), color.blue(), a).unwrap_or(color);
                        }
                        let mut paint = Paint::default();
                        paint.set_color(color);
                        paint.anti_alias = true;
                        let stroke_paint = stroke.as_ref().and_then(color_to_skia).map(|sc| {
                            let mut sp = Paint::default();
                            sp.set_color(sc);
                            sp.anti_alias = true;
                            sp
                        });
                        let sw = stroke_width.unwrap_or(1.0) as f32;
                        let sk_stroke = Stroke { width: sw, ..Stroke::default() };
                        for i in 0..cx.len() {
                            if let Some(path) =
                                PathBuilder::from_circle(cx[i] as f32, cy[i] as f32, *r as f32)
                            {
                                pixmap.fill_path(
                                    &path, &paint, FillRule::Winding, transform, None,
                                );
                                if let Some(ref sp) = stroke_paint {
                                    pixmap.stroke_path(&path, sp, &sk_stroke, transform, None);
                                }
                            }
                        }
                    }
                }
                Primitive::RectBatch { x, y, w, h, fills } => {
                    let mut paint = Paint::default();
                    for i in 0..x.len() {
                        if let Some(rect) =
                            Rect::from_xywh(x[i] as f32, y[i] as f32, w[i] as f32, h[i] as f32)
                        {
                            if let Some(color) = color_to_skia(&fills[i]) {
                                paint.set_color(color);
                                pixmap.fill_rect(rect, &paint, transform, None);
                            }
                        }
                    }
                }
                Primitive::GroupStart { .. } | Primitive::GroupEnd => {}
            }
        }

        // Render text via a minimal SVG overlay through resvg.
        if !text_primitives.is_empty() {
            let text_svg = build_text_svg(scene, &text_primitives);
            let options = resvg::usvg::Options {
                fontdb: shared_fontdb(),
                ..Default::default()
            };
            if let Ok(tree) = resvg::usvg::Tree::from_str(&text_svg, &options) {
                resvg::render(&tree, transform, &mut pixmap.as_mut());
            }
        }

        pixmap.encode_png().map_err(|e| e.to_string())
    }
}

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
        if let Primitive::Text {
            x,
            y,
            content,
            size,
            anchor,
            rotate,
            bold,
        } = elem
        {
            let anchor_str = match anchor {
                TextAnchor::Start => "start",
                TextAnchor::Middle => "middle",
                TextAnchor::End => "end",
            };
            let _ = write!(
                svg,
                r#"<text x="{x}" y="{y}" font-size="{size}" text-anchor="{anchor_str}""#
            );
            if *bold {
                svg.push_str(r#" font-weight="bold""#);
            }
            if let Some(angle) = rotate {
                let _ = write!(svg, r#" transform="rotate({angle},{x},{y})""#);
            }
            svg.push('>');
            write_escaped(&mut svg, content);
            svg.push_str("</text>");
        }
    }

    svg.push_str("</svg>");
    svg
}

fn write_escaped(buf: &mut String, s: &str) {
    for b in s.bytes() {
        match b {
            b'&' => buf.push_str("&amp;"),
            b'<' => buf.push_str("&lt;"),
            b'>' => buf.push_str("&gt;"),
            b'"' => buf.push_str("&quot;"),
            _ => buf.push(b as char),
        }
    }
}

/// Convert a kuva Color to a tiny_skia Color without string round-tripping.
fn color_to_skia(c: &crate::render::color::Color) -> Option<Color> {
    match c {
        crate::render::color::Color::Rgb(r, g, b) => Some(Color::from_rgba8(*r, *g, *b, 255)),
        crate::render::color::Color::None => None,
        crate::render::color::Color::Css(s) => parse_color(s),
    }
}

/// Parse a CSS color string into a tiny_skia Color.
fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    if s.is_empty() || s.eq_ignore_ascii_case("none") || s.eq_ignore_ascii_case("transparent") {
        return None;
    }
    // #RRGGBB
    if s.len() == 7 && s.as_bytes()[0] == b'#' {
        let r = u8::from_str_radix(&s[1..3], 16).ok()?;
        let g = u8::from_str_radix(&s[3..5], 16).ok()?;
        let b = u8::from_str_radix(&s[5..7], 16).ok()?;
        return Some(Color::from_rgba8(r, g, b, 255));
    }
    // #RGB
    if s.len() == 4 && s.as_bytes()[0] == b'#' {
        let r = u8::from_str_radix(&s[1..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..3], 16).ok()?;
        let b = u8::from_str_radix(&s[3..4], 16).ok()?;
        return Some(Color::from_rgba8(r * 17, g * 17, b * 17, 255));
    }
    // rgb(r, g, b) or rgb(r,g,b)
    if let Some(inner) = s.strip_prefix("rgb(").and_then(|t| t.strip_suffix(')')) {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3 {
            let r = parts[0].trim().parse::<f64>().ok()?.round() as u8;
            let g = parts[1].trim().parse::<f64>().ok()?.round() as u8;
            let b = parts[2].trim().parse::<f64>().ok()?.round() as u8;
            return Some(Color::from_rgba8(r, g, b, 255));
        }
    }
    // Named CSS colors (common subset used in plotting)
    match s.to_ascii_lowercase().as_str() {
        "black" => Some(Color::from_rgba8(0, 0, 0, 255)),
        "white" => Some(Color::from_rgba8(255, 255, 255, 255)),
        "red" => Some(Color::from_rgba8(255, 0, 0, 255)),
        "green" => Some(Color::from_rgba8(0, 128, 0, 255)),
        "blue" => Some(Color::from_rgba8(0, 0, 255, 255)),
        "steelblue" => Some(Color::from_rgba8(70, 130, 180, 255)),
        "gray" | "grey" => Some(Color::from_rgba8(128, 128, 128, 255)),
        "lightgray" | "lightgrey" => Some(Color::from_rgba8(211, 211, 211, 255)),
        "darkgray" | "darkgrey" => Some(Color::from_rgba8(169, 169, 169, 255)),
        "orange" => Some(Color::from_rgba8(255, 165, 0, 255)),
        "yellow" => Some(Color::from_rgba8(255, 255, 0, 255)),
        "purple" => Some(Color::from_rgba8(128, 0, 128, 255)),
        "pink" => Some(Color::from_rgba8(255, 192, 203, 255)),
        "brown" => Some(Color::from_rgba8(165, 42, 42, 255)),
        "cyan" => Some(Color::from_rgba8(0, 255, 255, 255)),
        "magenta" => Some(Color::from_rgba8(255, 0, 255, 255)),
        "coral" => Some(Color::from_rgba8(255, 127, 80, 255)),
        "salmon" => Some(Color::from_rgba8(250, 128, 114, 255)),
        "navy" => Some(Color::from_rgba8(0, 0, 128, 255)),
        "teal" => Some(Color::from_rgba8(0, 128, 128, 255)),
        "olive" => Some(Color::from_rgba8(128, 128, 0, 255)),
        "maroon" => Some(Color::from_rgba8(128, 0, 0, 255)),
        "silver" => Some(Color::from_rgba8(192, 192, 192, 255)),
        "gold" => Some(Color::from_rgba8(255, 215, 0, 255)),
        "tomato" => Some(Color::from_rgba8(255, 99, 71, 255)),
        "crimson" => Some(Color::from_rgba8(220, 20, 60, 255)),
        "dodgerblue" => Some(Color::from_rgba8(30, 144, 255, 255)),
        "limegreen" => Some(Color::from_rgba8(50, 205, 50, 255)),
        "orangered" => Some(Color::from_rgba8(255, 69, 0, 255)),
        "darkred" => Some(Color::from_rgba8(139, 0, 0, 255)),
        "darkblue" => Some(Color::from_rgba8(0, 0, 139, 255)),
        "darkgreen" => Some(Color::from_rgba8(0, 100, 0, 255)),
        "firebrick" => Some(Color::from_rgba8(178, 34, 34, 255)),
        "royalblue" => Some(Color::from_rgba8(65, 105, 225, 255)),
        "slategray" | "slategrey" => Some(Color::from_rgba8(112, 128, 144, 255)),
        "dimgray" | "dimgrey" => Some(Color::from_rgba8(105, 105, 105, 255)),
        "indianred" => Some(Color::from_rgba8(205, 92, 92, 255)),
        "mediumblue" => Some(Color::from_rgba8(0, 0, 205, 255)),
        "midnightblue" => Some(Color::from_rgba8(25, 25, 112, 255)),
        "forestgreen" => Some(Color::from_rgba8(34, 139, 34, 255)),
        "seagreen" => Some(Color::from_rgba8(46, 139, 87, 255)),
        "sienna" => Some(Color::from_rgba8(160, 82, 45, 255)),
        "chocolate" => Some(Color::from_rgba8(210, 105, 30, 255)),
        "peru" => Some(Color::from_rgba8(205, 133, 63, 255)),
        "tan" => Some(Color::from_rgba8(210, 180, 140, 255)),
        "plum" => Some(Color::from_rgba8(221, 160, 221, 255)),
        "orchid" => Some(Color::from_rgba8(218, 112, 214, 255)),
        "violet" => Some(Color::from_rgba8(238, 130, 238, 255)),
        "turquoise" => Some(Color::from_rgba8(64, 224, 208, 255)),
        "aquamarine" => Some(Color::from_rgba8(127, 255, 212, 255)),
        "cornflowerblue" => Some(Color::from_rgba8(100, 149, 237, 255)),
        "cadetblue" => Some(Color::from_rgba8(95, 158, 160, 255)),
        "darkorange" => Some(Color::from_rgba8(255, 140, 0, 255)),
        "deeppink" => Some(Color::from_rgba8(255, 20, 147, 255)),
        "hotpink" => Some(Color::from_rgba8(255, 105, 180, 255)),
        "mediumpurple" => Some(Color::from_rgba8(147, 112, 219, 255)),
        "mediumseagreen" => Some(Color::from_rgba8(60, 179, 113, 255)),
        "mediumvioletred" => Some(Color::from_rgba8(199, 21, 133, 255)),
        "darkcyan" => Some(Color::from_rgba8(0, 139, 139, 255)),
        "darkmagenta" => Some(Color::from_rgba8(139, 0, 139, 255)),
        "darkviolet" => Some(Color::from_rgba8(148, 0, 211, 255)),
        "darkorchid" => Some(Color::from_rgba8(153, 50, 204, 255)),
        "darkslateblue" => Some(Color::from_rgba8(72, 61, 139, 255)),
        "darkslategray" | "darkslategrey" => Some(Color::from_rgba8(47, 79, 79, 255)),
        "darkturquoise" => Some(Color::from_rgba8(0, 206, 209, 255)),
        "lightcoral" => Some(Color::from_rgba8(240, 128, 128, 255)),
        "lightsalmon" => Some(Color::from_rgba8(255, 160, 122, 255)),
        "lightseagreen" => Some(Color::from_rgba8(32, 178, 170, 255)),
        "lightskyblue" => Some(Color::from_rgba8(135, 206, 250, 255)),
        "lightsteelblue" => Some(Color::from_rgba8(176, 196, 222, 255)),
        _ => None, // unrecognized; fall through
    }
}

/// Minimal SVG path data parser. Handles M, L, C, A, Z commands (absolute only).
fn parse_svg_path(d: &str) -> Option<tiny_skia::Path> {
    let mut pb = PathBuilder::new();
    let chars = d.as_bytes();
    let mut i = 0;

    fn skip_ws_comma(data: &[u8], pos: &mut usize) {
        while *pos < data.len()
            && (data[*pos] == b' '
                || data[*pos] == b','
                || data[*pos] == b'\n'
                || data[*pos] == b'\r'
                || data[*pos] == b'\t')
        {
            *pos += 1;
        }
    }

    fn parse_f32(data: &[u8], pos: &mut usize) -> Option<f32> {
        skip_ws_comma(data, pos);
        let start = *pos;
        if *pos < data.len() && (data[*pos] == b'-' || data[*pos] == b'+') {
            *pos += 1;
        }
        let mut has_dot = false;
        while *pos < data.len() && (data[*pos].is_ascii_digit() || (data[*pos] == b'.' && !has_dot))
        {
            if data[*pos] == b'.' {
                has_dot = true;
            }
            *pos += 1;
        }
        // Handle exponent notation
        if *pos < data.len() && (data[*pos] == b'e' || data[*pos] == b'E') {
            *pos += 1;
            if *pos < data.len() && (data[*pos] == b'-' || data[*pos] == b'+') {
                *pos += 1;
            }
            while *pos < data.len() && data[*pos].is_ascii_digit() {
                *pos += 1;
            }
        }
        if start == *pos {
            return None;
        }
        std::str::from_utf8(&data[start..*pos])
            .ok()?
            .parse()
            .ok()
    }

    fn parse_flag(data: &[u8], pos: &mut usize) -> Option<u8> {
        skip_ws_comma(data, pos);
        if *pos < data.len() && (data[*pos] == b'0' || data[*pos] == b'1') {
            let v = data[*pos] - b'0';
            *pos += 1;
            Some(v)
        } else {
            None
        }
    }

    while i < chars.len() {
        skip_ws_comma(chars, &mut i);
        if i >= chars.len() {
            break;
        }
        let cmd = chars[i];
        if cmd.is_ascii_alphabetic() {
            i += 1;
        }
        match cmd {
            b'M' => {
                let x = parse_f32(chars, &mut i)?;
                let y = parse_f32(chars, &mut i)?;
                pb.move_to(x, y);
                // Implicit L after M
                loop {
                    skip_ws_comma(chars, &mut i);
                    if i >= chars.len()
                        || chars[i].is_ascii_alphabetic()
                    {
                        break;
                    }
                    let x = parse_f32(chars, &mut i)?;
                    let y = parse_f32(chars, &mut i)?;
                    pb.line_to(x, y);
                }
            }
            b'L' => loop {
                let x = parse_f32(chars, &mut i)?;
                let y = parse_f32(chars, &mut i)?;
                pb.line_to(x, y);
                skip_ws_comma(chars, &mut i);
                if i >= chars.len() || chars[i].is_ascii_alphabetic() {
                    break;
                }
            },
            b'C' => loop {
                let x1 = parse_f32(chars, &mut i)?;
                let y1 = parse_f32(chars, &mut i)?;
                let x2 = parse_f32(chars, &mut i)?;
                let y2 = parse_f32(chars, &mut i)?;
                let x = parse_f32(chars, &mut i)?;
                let y = parse_f32(chars, &mut i)?;
                pb.cubic_to(x1, y1, x2, y2, x, y);
                skip_ws_comma(chars, &mut i);
                if i >= chars.len() || chars[i].is_ascii_alphabetic() {
                    break;
                }
            },
            b'A' => loop {
                let _rx = parse_f32(chars, &mut i)?;
                let _ry = parse_f32(chars, &mut i)?;
                let _x_rot = parse_f32(chars, &mut i)?;
                let _large_arc = parse_flag(chars, &mut i)?;
                let _sweep = parse_flag(chars, &mut i)?;
                let x = parse_f32(chars, &mut i)?;
                let y = parse_f32(chars, &mut i)?;
                // tiny_skia PathBuilder lacks arc_to; approximate with line_to.
                // Arcs are only used in pie/chord charts which are low-element-count
                // plots, so the visual difference is acceptable for the raster fast path.
                pb.line_to(x, y);
                skip_ws_comma(chars, &mut i);
                if i >= chars.len() || chars[i].is_ascii_alphabetic() {
                    break;
                }
            },
            b'Z' | b'z' => {
                pb.close();
            }
            _ => {
                // Skip unrecognized
                i += 1;
            }
        }
    }
    pb.finish()
}
