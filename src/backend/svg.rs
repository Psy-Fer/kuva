use std::fmt::Write;

use crate::render::render::{Scene, Primitive, TextAnchor};

/// Fast coordinate writer: rounds to 2 decimal places, formats via ryu.
/// Produces compact output: whole numbers lack a decimal point (e.g. "3" not "3.0").
#[inline]
fn write_coord(buf: &mut String, v: f64) {
    let v = (v * 100.0).round() * 0.01;
    if v.fract() == 0.0 && v.abs() < 1e15 {
        let _ = write!(buf, "{}", v as i64);
    } else {
        let mut rb = ryu::Buffer::new();
        let s = rb.format(v);
        buf.push_str(s);
    }
}

/// Single-pass XML-escape that avoids allocating when the input is clean.
#[inline]
fn write_escaped(buf: &mut String, s: &str) {
    let mut start = 0;
    for (i, b) in s.bytes().enumerate() {
        let esc: &str = match b {
            b'&' => "&amp;",
            b'<' => "&lt;",
            b'>' => "&gt;",
            b'"' => "&quot;",
            b'\'' => "&apos;",
            _ => continue,
        };
        buf.push_str(&s[start..i]);
        buf.push_str(esc);
        start = i + 1;
    }
    buf.push_str(&s[start..]);
}

/// Write `depth` levels of two-space indentation directly into `buf`.
#[inline]
fn write_indent(buf: &mut String, depth: usize, pretty: bool) {
    if pretty {
        for _ in 0..depth {
            buf.push_str("  ");
        }
    }
}

#[inline]
fn push_nl(buf: &mut String, pretty: bool) {
    if pretty {
        buf.push('\n');
    }
}

pub struct SvgBackend {
    pretty: bool,
}

impl Default for SvgBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl SvgBackend {
    pub const fn new() -> Self {
        Self { pretty: false }
    }

    pub fn with_pretty(mut self, v: bool) -> Self {
        self.pretty = v;
        self
    }

    pub fn render_scene(&self, scene: &Scene) -> String {
        let p = self.pretty;

        let estimated_capacity = 200 + scene.elements.len() * 80;
        let mut svg = String::with_capacity(estimated_capacity);

        svg.push_str(r#"<svg xmlns="http://www.w3.org/2000/svg" width=""#);
        write_coord(&mut svg, scene.width);
        svg.push_str(r#"" height=""#);
        write_coord(&mut svg, scene.height);
        svg.push('"');
        if let Some(ref family) = scene.font_family {
            svg.push_str(r#" font-family=""#);
            svg.push_str(family);
            svg.push('"');
        }
        if let Some(ref color) = scene.text_color {
            svg.push_str(r#" fill=""#);
            svg.push_str(color);
            svg.push('"');
        }
        svg.push('>');
        push_nl(&mut svg, p);

        if let Some(color) = &scene.background_color {
            write_indent(&mut svg, 1, p);
            svg.push_str(r#"<rect width="100%" height="100%" fill=""#);
            svg.push_str(color);
            svg.push_str(r#"" />"#);
            push_nl(&mut svg, p);
        }

        if !scene.defs.is_empty() {
            write_indent(&mut svg, 1, p);
            svg.push_str("<defs>");
            for d in &scene.defs {
                svg.push_str(d);
            }
            svg.push_str("</defs>");
            push_nl(&mut svg, p);
        }

        let mut depth: usize = 1;
        for elem in &scene.elements {
            match elem {
                Primitive::Circle { cx, cy, r, fill } => {
                    write_indent(&mut svg, depth, p);
                    svg.push_str(r#"<circle cx=""#);
                    write_coord(&mut svg, *cx);
                    svg.push_str(r#"" cy=""#);
                    write_coord(&mut svg, *cy);
                    svg.push_str(r#"" r=""#);
                    write_coord(&mut svg, *r);
                    svg.push_str(r#"" fill=""#);
                    fill.write_svg(&mut svg);
                    svg.push_str(r#"" />"#);
                    push_nl(&mut svg, p);
                }
                Primitive::Text { x, y, content, size, anchor, rotate, bold } => {
                    let anchor_str = match anchor {
                        TextAnchor::Start => "start",
                        TextAnchor::Middle => "middle",
                        TextAnchor::End => "end",
                    };
                    write_indent(&mut svg, depth, p);
                    svg.push_str(r#"<text x=""#);
                    write_coord(&mut svg, *x);
                    svg.push_str(r#"" y=""#);
                    write_coord(&mut svg, *y);
                    svg.push_str(r#"" font-size=""#);
                    let _ = write!(svg, "{size}");
                    svg.push_str(r#"" text-anchor=""#);
                    svg.push_str(anchor_str);
                    svg.push('"');
                    if *bold {
                        svg.push_str(r#" font-weight="bold""#);
                    }
                    if let Some(angle) = rotate {
                        svg.push_str(r#" transform="rotate("#);
                        write_coord(&mut svg, *angle);
                        svg.push(',');
                        write_coord(&mut svg, *x);
                        svg.push(',');
                        write_coord(&mut svg, *y);
                        svg.push_str(r#")""#);
                    }
                    svg.push('>');
                    write_escaped(&mut svg, content);
                    svg.push_str("</text>");
                    push_nl(&mut svg, p);
                }
                Primitive::Line { x1, y1, x2, y2, stroke, stroke_width, stroke_dasharray } => {
                    write_indent(&mut svg, depth, p);
                    svg.push_str(r#"<line x1=""#);
                    write_coord(&mut svg, *x1);
                    svg.push_str(r#"" y1=""#);
                    write_coord(&mut svg, *y1);
                    svg.push_str(r#"" x2=""#);
                    write_coord(&mut svg, *x2);
                    svg.push_str(r#"" y2=""#);
                    write_coord(&mut svg, *y2);
                    svg.push_str(r#"" stroke=""#);
                    stroke.write_svg(&mut svg);
                    svg.push_str(r#"" stroke-width=""#);
                    write_coord(&mut svg, *stroke_width);
                    svg.push('"');
                    if let Some(dash) = stroke_dasharray {
                        svg.push_str(r#" stroke-dasharray=""#);
                        svg.push_str(dash);
                        svg.push('"');
                    }
                    svg.push_str(" />");
                    push_nl(&mut svg, p);
                }
                Primitive::Path(pd) => {
                    write_indent(&mut svg, depth, p);
                    svg.push_str(r#"<path d=""#);
                    svg.push_str(&pd.d);
                    svg.push_str(r#"" stroke=""#);
                    pd.stroke.write_svg(&mut svg);
                    svg.push_str(r#"" stroke-width=""#);
                    write_coord(&mut svg, pd.stroke_width);
                    svg.push('"');
                    if let Some(ref fill) = pd.fill {
                        svg.push_str(r#" fill=""#);
                        fill.write_svg(&mut svg);
                        svg.push('"');
                    } else {
                        svg.push_str(r#" fill="none""#);
                    }
                    if let Some(opacity) = pd.opacity {
                        svg.push_str(r#" fill-opacity=""#);
                        write_coord(&mut svg, opacity);
                        svg.push('"');
                    }
                    if let Some(ref dash) = pd.stroke_dasharray {
                        svg.push_str(r#" stroke-dasharray=""#);
                        svg.push_str(dash);
                        svg.push('"');
                    }
                    svg.push_str(" />");
                    push_nl(&mut svg, p);
                }
                Primitive::GroupStart { transform } => {
                    write_indent(&mut svg, depth, p);
                    svg.push_str("<g");
                    if let Some(t) = transform {
                        svg.push_str(r#" transform=""#);
                        svg.push_str(t);
                        svg.push('"');
                    }
                    svg.push('>');
                    push_nl(&mut svg, p);
                    depth += 1;
                }
                Primitive::GroupEnd => {
                    depth -= 1;
                    write_indent(&mut svg, depth, p);
                    svg.push_str("</g>");
                    push_nl(&mut svg, p);
                }
                Primitive::Rect { x, y, width, height, fill, stroke, stroke_width, opacity } => {
                    write_indent(&mut svg, depth, p);
                    svg.push_str(r#"<rect x=""#);
                    write_coord(&mut svg, *x);
                    svg.push_str(r#"" y=""#);
                    write_coord(&mut svg, *y);
                    svg.push_str(r#"" width=""#);
                    write_coord(&mut svg, *width);
                    svg.push_str(r#"" height=""#);
                    write_coord(&mut svg, *height);
                    svg.push_str(r#"" fill=""#);
                    fill.write_svg(&mut svg);
                    svg.push('"');
                    if let Some(stroke) = stroke {
                        svg.push_str(r#" stroke=""#);
                        stroke.write_svg(&mut svg);
                        svg.push('"');
                    }
                    if let Some(w) = stroke_width {
                        svg.push_str(r#" stroke-width=""#);
                        write_coord(&mut svg, *w);
                        svg.push('"');
                    }
                    if let Some(opacity) = opacity {
                        svg.push_str(r#" fill-opacity=""#);
                        write_coord(&mut svg, *opacity);
                        svg.push('"');
                    }
                    svg.push_str(" />");
                    push_nl(&mut svg, p);
                }
                Primitive::CircleBatch { cx, cy, r, fill } => {
                    let mut fill_buf = String::with_capacity(7);
                    fill.write_svg(&mut fill_buf);
                    for i in 0..cx.len() {
                        write_indent(&mut svg, depth, p);
                        svg.push_str(r#"<circle cx=""#);
                        write_coord(&mut svg, cx[i]);
                        svg.push_str(r#"" cy=""#);
                        write_coord(&mut svg, cy[i]);
                        svg.push_str(r#"" r=""#);
                        write_coord(&mut svg, *r);
                        svg.push_str(r#"" fill=""#);
                        svg.push_str(&fill_buf);
                        svg.push_str(r#"" />"#);
                        push_nl(&mut svg, p);
                    }
                }
                Primitive::RectBatch { x, y, w, h, fills } => {
                    for i in 0..x.len() {
                        write_indent(&mut svg, depth, p);
                        svg.push_str(r#"<rect x=""#);
                        write_coord(&mut svg, x[i]);
                        svg.push_str(r#"" y=""#);
                        write_coord(&mut svg, y[i]);
                        svg.push_str(r#"" width=""#);
                        write_coord(&mut svg, w[i]);
                        svg.push_str(r#"" height=""#);
                        write_coord(&mut svg, h[i]);
                        svg.push_str(r#"" fill=""#);
                        fills[i].write_svg(&mut svg);
                        svg.push_str(r#"" />"#);
                        push_nl(&mut svg, p);
                    }
                }
            }
        }

        svg.push_str("</svg>");
        push_nl(&mut svg, p);
        svg
    }
}

// Backward-compat shim in the value namespace.
// `SvgBackend.render_scene(...)` in old code: `SvgBackend` resolves to this const.
// `SvgBackend::new()` in new code: `SvgBackend` resolves to the type.
// TODO: To phase out later: add #[deprecated(note = "Use SvgBackend::new()")] here.
#[allow(non_upper_case_globals)]
pub const SvgBackend: SvgBackend = SvgBackend::new();
