use std::fmt::Write;

use crate::render::render::{Scene, Primitive, TextAnchor};
use crate::backend::interactive_js;

/// Fast float-to-string conversion using Ryu.
/// This will round floats to 2 decimal places.
/// If whole number, strip the decimal e.g. "3" instead of "3.0".
#[inline]
fn write_float(buf: &mut String, v: f64) {
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

/// Write double whitespace indentations with no allocation.
#[inline]
fn write_indent(buf: &mut String, depth: usize, pretty: bool) {
    if pretty {
        for _ in 0..depth {
            buf.push_str("  ");
        }
    }
}

#[inline]
fn write_newline(buf: &mut String, pretty: bool) {
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
        write_float(&mut svg, scene.width);
        svg.push_str(r#"" height=""#);
        write_float(&mut svg, scene.height);
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
        // Emit axis metadata as data-* attrs when interactive.
        if scene.interactive {
            if let Some(ref m) = scene.axis_meta {
                let _ = write!(svg,
                    r#" data-xmin="{xmin}" data-xmax="{xmax}" data-ymin="{ymin}" data-ymax="{ymax}" data-plot-left="{pl}" data-plot-top="{pt}" data-plot-right="{pr}" data-plot-bottom="{pb}" data-log-x="{lx}" data-log-y="{ly}""#,
                    xmin = m.x_min,
                    xmax = m.x_max,
                    ymin = m.y_min,
                    ymax = m.y_max,
                    pl = m.plot_left,
                    pt = m.plot_top,
                    pr = m.plot_right,
                    pb = m.plot_bottom,
                    lx = if m.log_x { 1 } else { 0 },
                    ly = if m.log_y { 1 } else { 0 },
                );
            }
        }
        svg.push('>');
        write_newline(&mut svg, p);

        if let Some(color) = &scene.background_color {
            write_indent(&mut svg, 1, p);
            svg.push_str(r#"<rect width="100%" height="100%" fill=""#);
            svg.push_str(color);
            svg.push_str(r#"" />"#);
            write_newline(&mut svg, p);
        }

        if !scene.defs.is_empty() || scene.has_tooltips || scene.interactive {
            write_indent(&mut svg, 1, p);
            svg.push_str("<defs>");
            for d in &scene.defs {
                svg.push_str(d);
            }
            if scene.interactive {
                svg.push_str(r#"<style>"#);
                svg.push_str("g.tt{cursor:pointer;}");
                svg.push_str("g.tt:hover>*{opacity:0.75;}");
                svg.push_str("g.tt.pinned>*{stroke:gold!important;stroke-width:2px!important;}");
                svg.push_str("g.tt.dim>*{opacity:0.1!important;}");
                svg.push_str("g.tt.muted>*{opacity:0.12!important;}");
                svg.push_str("g.legend-entry{cursor:pointer;}");
                svg.push_str("#kuva-save-btn{cursor:pointer;}");
                svg.push_str("#kuva-save-btn:hover rect{fill:#e8e8e8;}");
                svg.push_str(r#"#kuva-search{position:absolute;font-size:12px;padding:2px 4px;border:1px solid #aaa;border-radius:3px;}"#);
                svg.push_str(r#"#kuva-readout{pointer-events:none;font-size:11px;fill:#555;}"#);
                svg.push_str("</style>");
            } else if scene.has_tooltips {
                svg.push_str(
                    r#"<style>g.tt{cursor:pointer;}g.tt:hover>*{opacity:0.75;}</style>"#,
                );
            }
            svg.push_str("</defs>");
            write_newline(&mut svg, p);
        }

        // Interactive UI is emitted AFTER scene elements (see below) so it renders on top.

        let mut depth: usize = 1;
        for elem in &scene.elements {
            match elem {
                Primitive::Circle { cx, cy, r, fill, fill_opacity, stroke, stroke_width } => {
                    write_indent(&mut svg, depth, p);
                    svg.push_str(r#"<circle cx=""#);
                    write_float(&mut svg, *cx);
                    svg.push_str(r#"" cy=""#);
                    write_float(&mut svg, *cy);
                    svg.push_str(r#"" r=""#);
                    write_float(&mut svg, *r);
                    svg.push_str(r#"" fill=""#);
                    fill.write_svg(&mut svg);
                    svg.push('"');
                    if let Some(op) = fill_opacity {
                        svg.push_str(r#" fill-opacity=""#);
                        write_float(&mut svg, *op);
                        svg.push('"');
                    }
                    if let Some(sc) = stroke {
                        svg.push_str(r#" stroke=""#);
                        sc.write_svg(&mut svg);
                        svg.push('"');
                    }
                    if let Some(sw) = stroke_width {
                        svg.push_str(r#" stroke-width=""#);
                        write_float(&mut svg, *sw);
                        svg.push('"');
                    }
                    svg.push_str(" />");
                    write_newline(&mut svg, p);
                }
                Primitive::Text { x, y, content, size, anchor, rotate, bold } => {
                    let anchor_str = match anchor {
                        TextAnchor::Start => "start",
                        TextAnchor::Middle => "middle",
                        TextAnchor::End => "end",
                    };
                    write_indent(&mut svg, depth, p);
                    svg.push_str(r#"<text x=""#);
                    write_float(&mut svg, *x);
                    svg.push_str(r#"" y=""#);
                    write_float(&mut svg, *y);
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
                        write_float(&mut svg, *angle);
                        svg.push(',');
                        write_float(&mut svg, *x);
                        svg.push(',');
                        write_float(&mut svg, *y);
                        svg.push_str(r#")""#);
                    }
                    svg.push('>');
                    write_escaped(&mut svg, content);
                    svg.push_str("</text>");
                    write_newline(&mut svg, p);
                }
                Primitive::Line { x1, y1, x2, y2, stroke, stroke_width, stroke_dasharray } => {
                    write_indent(&mut svg, depth, p);
                    svg.push_str(r#"<line x1=""#);
                    write_float(&mut svg, *x1);
                    svg.push_str(r#"" y1=""#);
                    write_float(&mut svg, *y1);
                    svg.push_str(r#"" x2=""#);
                    write_float(&mut svg, *x2);
                    svg.push_str(r#"" y2=""#);
                    write_float(&mut svg, *y2);
                    svg.push_str(r#"" stroke=""#);
                    stroke.write_svg(&mut svg);
                    svg.push_str(r#"" stroke-width=""#);
                    write_float(&mut svg, *stroke_width);
                    svg.push('"');
                    if let Some(dash) = stroke_dasharray {
                        svg.push_str(r#" stroke-dasharray=""#);
                        svg.push_str(dash);
                        svg.push('"');
                    }
                    svg.push_str(" />");
                    write_newline(&mut svg, p);
                }
                Primitive::Path(pd) => {
                    write_indent(&mut svg, depth, p);
                    svg.push_str(r#"<path d=""#);
                    svg.push_str(&pd.d);
                    svg.push_str(r#"" stroke=""#);
                    pd.stroke.write_svg(&mut svg);
                    svg.push_str(r#"" stroke-width=""#);
                    write_float(&mut svg, pd.stroke_width);
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
                        write_float(&mut svg, opacity);
                        svg.push('"');
                    }
                    if let Some(ref dash) = pd.stroke_dasharray {
                        svg.push_str(r#" stroke-dasharray=""#);
                        svg.push_str(dash);
                        svg.push('"');
                    }
                    svg.push_str(" />");
                    write_newline(&mut svg, p);
                }
                Primitive::GroupStart { transform, title, extra_attrs } => {
                    write_indent(&mut svg, depth, p);
                    svg.push_str("<g");
                    // Only add class="tt" from title if extra_attrs doesn't already supply a class.
                    let extra_has_class = extra_attrs.as_ref()
                        .is_some_and(|a| a.contains("class="));
                    if title.is_some() && !extra_has_class {
                        svg.push_str(r#" class="tt""#);
                    }
                    if let Some(t) = transform {
                        svg.push_str(r#" transform=""#);
                        svg.push_str(t);
                        svg.push('"');
                    }
                    if let Some(attrs) = extra_attrs {
                        svg.push(' ');
                        svg.push_str(attrs);
                    }
                    svg.push('>');
                    write_newline(&mut svg, p);
                    depth += 1;
                    if let Some(tip) = title {
                        write_indent(&mut svg, depth, p);
                        svg.push_str("<title>");
                        write_escaped(&mut svg, tip);
                        svg.push_str("</title>");
                        write_newline(&mut svg, p);
                    }
                }
                Primitive::GroupEnd => {
                    depth -= 1;
                    write_indent(&mut svg, depth, p);
                    svg.push_str("</g>");
                    write_newline(&mut svg, p);
                }
                Primitive::ClipStart { id, .. } => {
                    // The <clipPath> definition was pushed to scene.defs and is
                    // already emitted in the <defs> block at the top of the SVG.
                    // Here we only open the clipping group.
                    write_indent(&mut svg, depth, p);
                    svg.push_str(r#"<g clip-path="url(#"#);
                    svg.push_str(id);
                    svg.push_str(r#")">"#);
                    write_newline(&mut svg, p);
                    depth += 1;
                }
                Primitive::ClipEnd => {
                    depth -= 1;
                    write_indent(&mut svg, depth, p);
                    svg.push_str("</g>");
                    write_newline(&mut svg, p);
                }
                Primitive::Rect { x, y, width, height, fill, stroke, stroke_width, opacity } => {
                    write_indent(&mut svg, depth, p);
                    svg.push_str(r#"<rect x=""#);
                    write_float(&mut svg, *x);
                    svg.push_str(r#"" y=""#);
                    write_float(&mut svg, *y);
                    svg.push_str(r#"" width=""#);
                    write_float(&mut svg, *width);
                    svg.push_str(r#"" height=""#);
                    write_float(&mut svg, *height);
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
                        write_float(&mut svg, *w);
                        svg.push('"');
                    }
                    if let Some(opacity) = opacity {
                        svg.push_str(r#" fill-opacity=""#);
                        write_float(&mut svg, *opacity);
                        svg.push('"');
                    }
                    svg.push_str(" />");
                    write_newline(&mut svg, p);
                }
                Primitive::CircleBatch { cx, cy, r, fill, fill_opacity, stroke, stroke_width } => {
                    let mut fill_buf = String::with_capacity(7);
                    fill.write_svg(&mut fill_buf);
                    let mut stroke_buf = String::new();
                    if let Some(sc) = stroke {
                        sc.write_svg(&mut stroke_buf);
                    }
                    for i in 0..cx.len() {
                        write_indent(&mut svg, depth, p);
                        svg.push_str(r#"<circle cx=""#);
                        write_float(&mut svg, cx[i]);
                        svg.push_str(r#"" cy=""#);
                        write_float(&mut svg, cy[i]);
                        svg.push_str(r#"" r=""#);
                        write_float(&mut svg, *r);
                        svg.push_str(r#"" fill=""#);
                        svg.push_str(&fill_buf);
                        svg.push('"');
                        if let Some(op) = fill_opacity {
                            svg.push_str(r#" fill-opacity=""#);
                            write_float(&mut svg, *op);
                            svg.push('"');
                        }
                        if !stroke_buf.is_empty() {
                            svg.push_str(r#" stroke=""#);
                            svg.push_str(&stroke_buf);
                            svg.push('"');
                        }
                        if let Some(sw) = stroke_width {
                            svg.push_str(r#" stroke-width=""#);
                            write_float(&mut svg, *sw);
                            svg.push('"');
                        }
                        svg.push_str(" />");
                        write_newline(&mut svg, p);
                    }
                }
                Primitive::RectBatch { x, y, w, h, fills } => {
                    for i in 0..x.len() {
                        write_indent(&mut svg, depth, p);
                        svg.push_str(r#"<rect x=""#);
                        write_float(&mut svg, x[i]);
                        svg.push_str(r#"" y=""#);
                        write_float(&mut svg, y[i]);
                        svg.push_str(r#"" width=""#);
                        write_float(&mut svg, w[i]);
                        svg.push_str(r#"" height=""#);
                        write_float(&mut svg, h[i]);
                        svg.push_str(r#"" fill=""#);
                        fills[i].write_svg(&mut svg);
                        svg.push_str(r#"" />"#);
                        write_newline(&mut svg, p);
                    }
                }
            }
        }

        // Interactive UI + JavaScript just before </svg> so UI renders on top of the plot.
        if scene.interactive {
            // Position the UI strip inside the plot at the top-right corner to avoid
            // overlapping axis labels.
            let ui_w: f64 = 215.0;  // search + gap + save + gap + info circle
            let ui_h: f64 = 22.0;

            // Place the strip in the 32px reserved zone at the very bottom of the SVG,
            // guaranteed to be below all axis tick labels and titles.
            let strip_y = scene.height - ui_h - 4.0;
            let strip_x = if let Some(ref m) = scene.axis_meta {
                m.plot_left
            } else {
                6.0
            };
            // Layout (left to right): [search input FO] [gap] [save btn SVG] [gap] [info icon SVG]
            let save_w: f64  = 38.0;
            let input_fo_w   = ui_w - save_w - 3.0 - 22.0 - 4.0;  // 148px
            let save_x       = strip_x + input_fo_w + 3.0;
            let save_mid_x   = save_x + save_w * 0.5;
            let mid_y        = strip_y + ui_h * 0.5;
            let info_cx      = strip_x + ui_w - 11.0;

            // Search input in its own foreignObject (no flex, no button).
            write_indent(&mut svg, 1, p);
            let _ = write!(svg,
                "<foreignObject x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"><div xmlns=\"http://www.w3.org/1999/xhtml\" style=\"margin:0;padding:0;background:transparent;\"><input id=\"kuva-search\" type=\"text\" placeholder=\"search\u{2026}\" style=\"width:100%;height:{}px;font-size:11px;padding:1px 3px;border:1px solid #ccc;border-radius:2px;box-sizing:border-box;background:white;\"/></div></foreignObject>",
                strip_x, strip_y, input_fo_w, ui_h, ui_h - 2.0,
            );
            write_newline(&mut svg, p);

            // Save button as native SVG (avoids foreignObject layout quirks).
            write_indent(&mut svg, 1, p);
            let _ = write!(svg,
                "<g id=\"kuva-save-btn\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"3\" fill=\"white\" stroke=\"#ccc\" stroke-width=\"1\"/><text x=\"{}\" y=\"{}\" dy=\"0.35em\" text-anchor=\"middle\" font-size=\"11\" fill=\"#333\">save</text></g>",
                save_x, strip_y + 1.0, save_w, ui_h - 2.0,
                save_mid_x, mid_y,
            );
            write_newline(&mut svg, p);

            // Native SVG ⓘ info icon — works in Firefox standalone SVGs.
            write_indent(&mut svg, 1, p);
            let _ = write!(svg,
                "<g style=\"cursor:help\"><circle cx=\"{}\" cy=\"{}\" r=\"9\" fill=\"#ebebeb\" stroke=\"#bbb\" stroke-width=\"1.2\"/><text x=\"{}\" y=\"{}\" dy=\"0.35em\" text-anchor=\"middle\" font-size=\"11\" fill=\"#555\" font-style=\"italic\">i</text><title>Interactive controls:\n\u{2022} Hover inside plot \u{2192} x,y coordinate readout near cursor\n\u{2022} Hover a point \u{2192} native tooltip with values\n\u{2022} Click a point \u{2192} highlight it, dim others; click again to release\n\u{2022} Click legend entry \u{2192} mute/unmute that group\n\u{2022} Click empty area or Esc \u{2192} reset all\n\u{2022} Search \u{2192} type a group name or x/y value to filter</title></g>",
                info_cx, mid_y, info_cx, mid_y,
            );
            write_newline(&mut svg, p);

            // Coordinate readout text: starts off-screen, JS moves it to cursor.
            write_indent(&mut svg, 1, p);
            svg.push_str("<text id=\"kuva-readout\" x=\"-9999\" y=\"-9999\"></text>");
            write_newline(&mut svg, p);
            write_indent(&mut svg, 1, p);
            svg.push_str("<script type=\"text/javascript\"><![CDATA[");
            svg.push_str(interactive_js::JS);
            svg.push_str("]]></script>");
            write_newline(&mut svg, p);
        }

        svg.push_str("</svg>");
        write_newline(&mut svg, p);
        svg
    }
}

// Backward-compat shim in the value namespace.
// `SvgBackend.render_scene(...)` in old code: `SvgBackend` resolves to this const.
// `SvgBackend::new()` in new code: `SvgBackend` resolves to the type.
// TODO: To phase out later: add #[deprecated(note = "Use SvgBackend::new()")] here.
#[allow(non_upper_case_globals)]
pub const SvgBackend: SvgBackend = SvgBackend::new();
