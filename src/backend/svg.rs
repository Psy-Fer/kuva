use std::fmt::Write;

use crate::backend::interactive_js;
use crate::render::color::Color;
use crate::render::render::{Primitive, Scene, TextAnchor};

/// Fast float-to-string conversion using Ryu.
/// This will round floats to 2 decimal places.
/// If whole number, strip the decimal e.g. "3" instead of "3.0".
#[inline]
fn write_float(buf: &mut String, v: f64) {
    // Divide, don't multiply by 0.01: 0.01 is inexact in binary, so `6130.0 * 0.01`
    // is 61.300000000000004 and ryu prints all 17 digits (~15% of a big brick SVG).
    let v = (v * 100.0).round() / 100.0;
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
    embed_font: bool,
    rect_batching: bool,
}

impl Default for SvgBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl SvgBackend {
    pub const fn new() -> Self {
        Self {
            pretty: false,
            embed_font: false,
            rect_batching: true,
        }
    }

    pub fn with_pretty(mut self, v: bool) -> Self {
        self.pretty = v;
        self
    }

    /// Embed DejaVu Sans as a base64 `@font-face` in the SVG `<style>` block.
    ///
    /// Use this when the SVG will be rendered in an environment with no system
    /// fonts (headless servers, containers, CI pipelines). The embedded font
    /// makes the SVG fully self-contained at the cost of ~1 MB of extra size.
    pub fn with_embedded_font(mut self, v: bool) -> Self {
        self.embed_font = v;
        self
    }

    /// Write runs of plain rectangles compactly (on by default).
    ///
    /// A run of consecutive rectangles with no stroke, opacity or tooltip, in which no two
    /// rectangles overlap, is written as one `<path>` per fill colour instead of one `<rect>`
    /// per rectangle. Brick plots, heatmaps and bar charts are made of exactly such runs, so
    /// their SVGs come out about half the size and compress much better. Because nothing in the
    /// run overlaps, paint order cannot matter and the image is unchanged; every rectangle stays
    /// its own closed subpath, so gaps between neighbours are kept. A run with any overlap is
    /// written as individual `<rect>`s, exactly as with batching off.
    ///
    /// Turn it off (`false`) when something downstream needs each rectangle as its own element,
    /// e.g. to select or restyle single cells in an editor or with CSS.
    pub fn with_rect_batching(mut self, v: bool) -> Self {
        self.rect_batching = v;
        self
    }

    /// Render the scene as a gzipped SVG (the bytes of a `.svgz` file).
    ///
    /// Big brick plots and heatmaps compress 10 to 50 times. Image viewers, Inkscape and Firefox
    /// open a `.svgz` directly, but Chrome only shows one served over HTTP with
    /// `Content-Encoding: gzip`, not from a local file. To put a compressed figure in a web page
    /// that works anywhere, including from `file://`, use the [`html`](crate::html) helpers.
    #[cfg(feature = "svgz")]
    pub fn render_scene_svgz(&self, scene: &Scene) -> Vec<u8> {
        crate::encoding::gzip(self.render_scene(scene).as_bytes())
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
            write_escaped(&mut svg, family);
            svg.push('"');
        }
        if let Some(ref color) = scene.text_color {
            svg.push_str(r#" fill=""#);
            write_escaped(&mut svg, color);
            svg.push('"');
        }
        // Emit axis metadata as data-* attrs when interactive.
        if scene.interactive {
            if let Some(ref m) = scene.axis_meta {
                let _ = write!(
                    svg,
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

        // Generator stamp as the first child (kept after the opening tag so the string
        // still starts with `<svg`). Records the kuva version for later issue triage.
        write_indent(&mut svg, 1, p);
        svg.push_str("<!-- Generated by kuva ");
        svg.push_str(crate::VERSION);
        svg.push_str(" -->");
        write_newline(&mut svg, p);

        #[cfg(feature = "embed_font")]
        if self.embed_font {
            write_indent(&mut svg, 1, p);
            svg.push_str(crate::fonts::dejavu_sans_style_block());
            write_newline(&mut svg, p);
        }

        if let Some(color) = &scene.background_color {
            write_indent(&mut svg, 1, p);
            svg.push_str(r#"<rect width="100%" height="100%" fill=""#);
            write_escaped(&mut svg, color);
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
                svg.push_str(r#"<style>g.tt{cursor:pointer;}g.tt:hover>*{opacity:0.75;}</style>"#);
            }
            svg.push_str("</defs>");
            write_newline(&mut svg, p);
        }

        // Interactive UI is emitted AFTER scene elements (see below) so it renders on top.

        let mut depth: usize = 1;
        // Unique tag per embedded math fragment, to namespace Typst's element IDs.
        #[cfg(feature = "typst-math")]
        let mut math_uid: usize = 0;
        // Elements before this index were already checked for a batchable rect run and found
        // none (or an overlapping one), so they are written one by one.
        let mut no_batch_before = 0;
        let mut i = 0;
        while i < scene.elements.len() {
            if self.rect_batching && i >= no_batch_before {
                let end = rect_run_end(&scene.elements, i);
                if end > i {
                    if write_rect_run(&mut svg, &scene.elements[i..end], depth, p) {
                        i = end;
                        continue;
                    }
                    no_batch_before = end;
                }
            }
            let elem = &scene.elements[i];
            i += 1;
            match elem {
                Primitive::Circle {
                    cx,
                    cy,
                    r,
                    fill,
                    fill_opacity,
                    stroke,
                    stroke_width,
                } => {
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
                Primitive::Text {
                    x,
                    y,
                    content,
                    size,
                    anchor,
                    rotate,
                    bold,
                    color,
                } => {
                    // Math routing: a `$...$` label is either typeset by the
                    // typst tier (feature `typst-math`) and embedded as a fragment,
                    // or lowered to inline Unicode by the always-on lookup
                    // tier and emitted as ordinary text.
                    #[cfg(feature = "typst-math")]
                    if crate::render::math::contains_math(content) {
                        if let Some(m) = crate::render::math::render_label_svg(
                            content,
                            *size as f64,
                            color.as_ref(),
                        ) {
                            write_indent(&mut svg, depth, p);
                            crate::backend::svg_math::embed_label(
                                &mut svg, *x, *y, *anchor, *rotate, &m, math_uid,
                            );
                            math_uid += 1;
                            write_newline(&mut svg, p);
                            continue;
                        }
                        // Compile failed — fall through to the lookup tier.
                    }

                    // Lookup tier (or plain text). When the label has math but
                    // the typst tier isn't active/failed, substitute to Unicode
                    // (needs_rewrite also catches escaped `\$`).
                    let lowered;
                    let content: &str = if crate::render::math::needs_rewrite(content) {
                        lowered = crate::render::math::to_unicode(content);
                        &lowered
                    } else {
                        content
                    };

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
                    if let Some(c) = color {
                        svg.push_str(r#" fill=""#);
                        c.write_svg(&mut svg);
                        svg.push('"');
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
                Primitive::RichText {
                    x,
                    y,
                    spans,
                    size,
                    anchor,
                    color,
                } => {
                    // Splice path (typst tier): a line containing math spans
                    // is laid out manually — text runs measured with the
                    // bundled-font metrics, fragments with their typeset
                    // width — so each piece can be positioned absolutely and
                    // the math embeds inline at the shared baseline.
                    #[cfg(feature = "typst-math")]
                    if spans.iter().any(|sp| sp.math) {
                        rich_text_with_math(
                            &mut svg,
                            *x,
                            *y,
                            spans,
                            *size,
                            *anchor,
                            color.as_ref(),
                            depth,
                            p,
                            &mut math_uid,
                        );
                        continue;
                    }

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
                    if let Some(c) = color {
                        svg.push_str(r#" fill=""#);
                        c.write_svg(&mut svg);
                        svg.push('"');
                    }
                    svg.push('>');
                    for span in spans {
                        let styled = span.bold || span.italic || span.underline || span.code;
                        if styled {
                            svg.push_str("<tspan");
                            if span.code {
                                svg.push_str(r#" font-family="DejaVu Sans Mono,monospace""#);
                            }
                            if span.bold {
                                svg.push_str(r#" font-weight="bold""#);
                            }
                            if span.italic {
                                svg.push_str(r#" font-style="italic""#);
                            }
                            if span.underline {
                                svg.push_str(r#" text-decoration="underline""#);
                            }
                            svg.push('>');
                        }
                        write_escaped(&mut svg, &span.text);
                        if styled {
                            svg.push_str("</tspan>");
                        }
                    }
                    svg.push_str("</text>");
                    write_newline(&mut svg, p);
                }
                Primitive::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    stroke,
                    stroke_width,
                    stroke_dasharray,
                } => {
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
                        write_escaped(&mut svg, dash);
                        svg.push('"');
                    }
                    svg.push_str(" />");
                    write_newline(&mut svg, p);
                }
                Primitive::PolyLine {
                    points,
                    stroke,
                    stroke_width,
                    stroke_dasharray,
                } => {
                    write_indent(&mut svg, depth, p);
                    svg.push_str(r#"<path d=""#);
                    for (i, &(x, y)) in points.iter().enumerate() {
                        if i == 0 {
                            svg.push('M');
                        } else {
                            svg.push('L');
                        }
                        svg.push(' ');
                        write_float(&mut svg, x);
                        svg.push(' ');
                        write_float(&mut svg, y);
                        svg.push(' ');
                    }
                    svg.push_str(r#"" stroke=""#);
                    stroke.write_svg(&mut svg);
                    svg.push_str(r#"" stroke-width=""#);
                    write_float(&mut svg, *stroke_width);
                    svg.push_str(r#"" fill="none""#);
                    if let Some(ref dash) = stroke_dasharray {
                        svg.push_str(r#" stroke-dasharray=""#);
                        write_escaped(&mut svg, dash);
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
                        write_escaped(&mut svg, dash);
                        svg.push('"');
                    }
                    svg.push_str(" />");
                    write_newline(&mut svg, p);
                }
                Primitive::GroupStart {
                    transform,
                    title,
                    extra_attrs,
                } => {
                    write_indent(&mut svg, depth, p);
                    svg.push_str("<g");
                    // Only add class="tt" from title if extra_attrs doesn't already supply a class.
                    let extra_has_class =
                        extra_attrs.as_ref().is_some_and(|a| a.contains("class="));
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
                    write_indent(&mut svg, depth, p);
                    write_rect(
                        &mut svg,
                        (*x, *y, *width, *height),
                        fill,
                        stroke.as_ref(),
                        *stroke_width,
                        *opacity,
                    );
                    write_newline(&mut svg, p);
                }
                Primitive::CircleBatch {
                    cx,
                    cy,
                    r,
                    fill,
                    fill_opacity,
                    stroke,
                    stroke_width,
                } => {
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
                    for k in 0..x.len() {
                        write_indent(&mut svg, depth, p);
                        write_rect(
                            &mut svg,
                            (x[k], y[k], w[k], h[k]),
                            &fills[k],
                            None,
                            None,
                            None,
                        );
                        write_newline(&mut svg, p);
                    }
                }
            }
        }

        // Emit script blocks (e.g. CalendarPlot tooltip JS).
        for script in &scene.scripts {
            write_indent(&mut svg, 1, p);
            svg.push_str("<script type=\"text/javascript\"><![CDATA[");
            svg.push_str(script);
            svg.push_str("]]></script>");
            write_newline(&mut svg, p);
        }

        // Interactive UI + JavaScript just before </svg> so UI renders on top of the plot.
        if scene.interactive {
            // Position the UI strip inside the plot at the top-right corner to avoid
            // overlapping axis labels.
            let ui_w: f64 = 215.0; // search + gap + save + gap + info circle
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
            let save_w: f64 = 38.0;
            let input_fo_w = ui_w - save_w - 3.0 - 22.0 - 4.0; // 148px
            let save_x = strip_x + input_fo_w + 3.0;
            let save_mid_x = save_x + save_w * 0.5;
            let mid_y = strip_y + ui_h * 0.5;
            let info_cx = strip_x + ui_w - 11.0;

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

/// Write one `<rect>` element (no indent or newline).
fn write_rect(
    svg: &mut String,
    (x, y, width, height): (f64, f64, f64, f64),
    fill: &Color,
    stroke: Option<&Color>,
    stroke_width: Option<f64>,
    opacity: Option<f64>,
) {
    svg.push_str(r#"<rect x=""#);
    write_float(svg, x);
    svg.push_str(r#"" y=""#);
    write_float(svg, y);
    svg.push_str(r#"" width=""#);
    write_float(svg, width);
    svg.push_str(r#"" height=""#);
    write_float(svg, height);
    svg.push_str(r#"" fill=""#);
    fill.write_svg(svg);
    svg.push('"');
    if let Some(stroke) = stroke {
        svg.push_str(r#" stroke=""#);
        stroke.write_svg(svg);
        svg.push('"');
    }
    if let Some(w) = stroke_width {
        svg.push_str(r#" stroke-width=""#);
        write_float(svg, w);
        svg.push('"');
    }
    if let Some(opacity) = opacity {
        svg.push_str(r#" fill-opacity=""#);
        write_float(svg, opacity);
        svg.push('"');
    }
    svg.push_str(" />");
}

/// A rectangle that rect batching may fold into a per-colour `<path>`.
struct PlainRect<'a> {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    fill: &'a Color,
}

/// Whether a fill can move from a `<rect>` to a `<path>` unchanged. A pattern or gradient
/// (`url(...)`) may be sized to the element's bounding box, which differs between one rect and
/// a path spanning many, and `none` paints nothing, so neither is batched.
fn batchable_fill(fill: &Color) -> bool {
    match fill {
        Color::Rgb(..) => true,
        Color::None => false,
        Color::Css(s) => !s.trim_start().starts_with("url("),
    }
}

/// Append `elem`'s rectangles to `out` if every one of them can be batched; otherwise return
/// `false` (and leave `out` as it was).
fn push_plain_rects<'a>(elem: &'a Primitive, out: &mut Vec<PlainRect<'a>>) -> bool {
    let finite = |v: [f64; 4]| v.iter().all(|c| c.is_finite());
    match elem {
        Primitive::Rect {
            x,
            y,
            width,
            height,
            fill,
            stroke: None,
            opacity: None,
            ..
        } if batchable_fill(fill) && finite([*x, *y, *width, *height]) => {
            out.push(PlainRect {
                x: *x,
                y: *y,
                w: *width,
                h: *height,
                fill,
            });
            true
        }
        Primitive::RectBatch { x, y, w, h, fills } => {
            let n = x.len();
            let ok = y.len() == n
                && w.len() == n
                && h.len() == n
                && fills.len() == n
                && (0..n).all(|k| batchable_fill(&fills[k]) && finite([x[k], y[k], w[k], h[k]]));
            if ok {
                out.extend((0..n).map(|k| PlainRect {
                    x: x[k],
                    y: y[k],
                    w: w[k],
                    h: h[k],
                    fill: &fills[k],
                }));
            }
            ok
        }
        _ => false,
    }
}

/// End (exclusive) of the run of batchable rect primitives starting at `start`, or `start`
/// when the run holds fewer than two rectangles (nothing to gain).
fn rect_run_end(elements: &[Primitive], start: usize) -> usize {
    let mut scratch = Vec::new();
    let mut end = start;
    while end < elements.len() && push_plain_rects(&elements[end], &mut scratch) {
        end += 1;
    }
    if scratch.len() < 2 {
        start
    } else {
        end
    }
}

/// Overlap below this many pixels, in both directions, counts as touching. SVG coordinates are
/// written to two decimals, so a smaller overlap is rounding, not paint.
const OVERLAP_EPS: f64 = 0.01;

/// Whether any two rectangles in the run overlap. Only an overlap-free run may be regrouped by
/// colour: with no shared pixels, paint order (and any translucency) cannot change the image.
///
/// Uses a uniform grid sized from the median rectangle, so a big heatmap or brick plot costs
/// about one bucket check per rectangle instead of comparing every pair. If the grid work grows
/// far past the rectangle count (wildly mixed sizes), it gives up and reports an overlap, which
/// is the safe answer: the run is then written as plain `<rect>`s.
fn any_overlap(rects: &[PlainRect]) -> bool {
    use std::collections::HashMap;

    let live: Vec<&PlainRect> = rects.iter().filter(|r| r.w > 0.0 && r.h > 0.0).collect();
    if live.len() < 2 {
        return false;
    }
    let median = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2].max(OVERLAP_EPS)
    };
    let cw = median(live.iter().map(|r| r.w).collect());
    let ch = median(live.iter().map(|r| r.h).collect());
    let budget = 64 * live.len() + 100_000;
    let mut work = 0usize;
    let mut grid: HashMap<(i64, i64), Vec<u32>> = HashMap::new();
    for (idx, r) in live.iter().enumerate() {
        let (c0, c1) = ((r.x / cw).floor() as i64, ((r.x + r.w) / cw).floor() as i64);
        let (r0, r1) = ((r.y / ch).floor() as i64, ((r.y + r.h) / ch).floor() as i64);
        for gx in c0..=c1 {
            for gy in r0..=r1 {
                work += 1;
                if work > budget {
                    return true;
                }
                let cell = grid.entry((gx, gy)).or_default();
                for &other in cell.iter() {
                    let o = live[other as usize];
                    let xov = (r.x + r.w).min(o.x + o.w) - r.x.max(o.x);
                    let yov = (r.y + r.h).min(o.y + o.h) - r.y.max(o.y);
                    if xov > OVERLAP_EPS && yov > OVERLAP_EPS {
                        return true;
                    }
                    work += 1;
                }
                cell.push(idx as u32);
            }
        }
    }
    false
}

/// Write a run of batchable rect primitives as one `<path>` per fill (first-seen colour order),
/// each rectangle a closed `M x y h w v h h -w z` subpath with relative moves between them. A
/// colour with a single rectangle stays a `<rect>`. Returns `false`, writing nothing, if any two
/// rectangles overlap; the caller then writes the run element by element.
fn write_rect_run(svg: &mut String, run: &[Primitive], depth: usize, p: bool) -> bool {
    use std::collections::HashMap;

    let mut rects = Vec::new();
    for elem in run {
        push_plain_rects(elem, &mut rects);
    }
    if any_overlap(&rects) {
        return false;
    }

    // Group by the fill exactly as it would be written, in first-seen order (never HashMap
    // order, so the output is deterministic).
    // A rect with no positive size paints nothing (and a negative-size subpath would paint), so
    // those are dropped here.
    let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for (k, r) in rects.iter().enumerate() {
        if !(r.w > 0.0 && r.h > 0.0) {
            continue;
        }
        let mut key = String::with_capacity(7);
        r.fill.write_svg(&mut key);
        match index.get(&key) {
            Some(&g) => groups[g].1.push(k),
            None => {
                index.insert(key.clone(), groups.len());
                groups.push((key, vec![k]));
            }
        }
    }

    let round = |v: f64| (v * 100.0).round() / 100.0;
    for (fill, members) in &groups {
        write_indent(svg, depth, p);
        if members.len() == 1 {
            let r = &rects[members[0]];
            write_rect(svg, (r.x, r.y, r.w, r.h), r.fill, None, None, None);
            write_newline(svg, p);
            continue;
        }
        svg.push_str(r#"<path d=""#);
        // After `z` the current point is the subpath's start, so each move is relative to the
        // previous rectangle's corner. Deltas come from the rounded corners, so they add up to
        // exactly the absolute coordinates a `<rect>` would have used.
        let mut prev: Option<(f64, f64)> = None;
        for &k in members {
            let r = &rects[k];
            let (x, y) = (round(r.x), round(r.y));
            match prev {
                None => {
                    svg.push('M');
                    write_float(svg, x);
                    svg.push(' ');
                    write_float(svg, y);
                }
                Some((px, py)) => {
                    svg.push('m');
                    write_float(svg, x - px);
                    svg.push(' ');
                    write_float(svg, y - py);
                }
            }
            let w = round(r.w);
            svg.push('h');
            write_float(svg, w);
            svg.push('v');
            write_float(svg, round(r.h));
            svg.push('h');
            write_float(svg, -w);
            svg.push('z');
            prev = Some((x, y));
        }
        svg.push_str(r#"" fill=""#);
        svg.push_str(fill);
        svg.push_str(r#"" />"#);
        write_newline(svg, p);
    }
    true
}

// Backward-compat shim in the value namespace.
// `SvgBackend.render_scene(...)` in old code: `SvgBackend` resolves to this const.
// `SvgBackend::new()` in new code: `SvgBackend` resolves to the type.
// TODO: To phase out later: add #[deprecated(note = "Use SvgBackend::new()")] here.
#[allow(non_upper_case_globals)]
pub const SvgBackend: SvgBackend = SvgBackend::new();

/// Draw one rich-text line that contains typeset math spans (typst tier).
///
/// The tspan flow used for plain rich text cannot host an embedded SVG
/// fragment, so this path lays the line out itself: every span gets an
/// absolute x from the bundled-font metrics (text) or the fragment's typeset
/// width (math), all sharing the baseline `y`. Anchoring shifts the whole
/// line by its total width. A fragment that fails to compile degrades to its
/// lookup-tier text in place.
#[cfg(feature = "typst-math")]
#[allow(clippy::too_many_arguments)]
fn rich_text_with_math(
    svg: &mut String,
    x: f64,
    y: f64,
    spans: &[crate::render::render::TextSpan],
    size: u32,
    anchor: TextAnchor,
    color: Option<&crate::render::color::Color>,
    depth: usize,
    pretty: bool,
    math_uid: &mut usize,
) {
    use crate::render::math;
    use crate::render::text_metrics::{measure_text_width, FontStyle};

    // Resolve each span to (drawable, advance) up front so anchoring can use
    // the exact total width.
    enum Piece<'a> {
        Text(&'a crate::render::render::TextSpan, String),
        Math(math::MathSvg),
    }
    let fs = size as f64;
    // The fragment page bakes in a margin (descender/overhang safety);
    // inline, the fragment must advance like a word — subtract it from the
    // advance and start the embed a margin early so glyphs align optically.
    let frag_margin = math::FRAGMENT_MARGIN_EM * fs;
    let mut pieces: Vec<(Piece, f64)> = Vec::with_capacity(spans.len());
    for sp in spans {
        if sp.math {
            match math::render_label_svg(&format!("${}$", sp.text), fs, color) {
                Some(m) => {
                    let w = (m.width_pt - 2.0 * frag_margin).max(1.0);
                    pieces.push((Piece::Math(m), w));
                    continue;
                }
                None => {
                    // Lookup-tier fallback, drawn as ordinary text.
                    let lowered = math::to_unicode(&format!("${}$", sp.text));
                    let w =
                        measure_text_width(&lowered, fs, FontStyle::from_flags(sp.bold, sp.italic));
                    pieces.push((Piece::Text(sp, lowered), w));
                    continue;
                }
            }
        }
        let w = measure_text_width(&sp.text, fs, FontStyle::from_flags(sp.bold, sp.italic));
        pieces.push((Piece::Text(sp, sp.text.clone()), w));
    }

    let total: f64 = pieces.iter().map(|(_, w)| w).sum();
    let mut cursor = match anchor {
        TextAnchor::Start => x,
        TextAnchor::Middle => x - total / 2.0,
        TextAnchor::End => x - total,
    };

    for (piece, w) in &pieces {
        match piece {
            Piece::Text(sp, text) => {
                write_indent(svg, depth, pretty);
                svg.push_str(r#"<text x=""#);
                write_float(svg, cursor);
                svg.push_str(r#"" y=""#);
                write_float(svg, y);
                svg.push_str(r#"" font-size=""#);
                let _ = write!(svg, "{size}");
                svg.push_str(r#"" text-anchor="start""#);
                if sp.code {
                    svg.push_str(r#" font-family="DejaVu Sans Mono,monospace""#);
                }
                if sp.bold {
                    svg.push_str(r#" font-weight="bold""#);
                }
                if sp.italic {
                    svg.push_str(r#" font-style="italic""#);
                }
                if sp.underline {
                    svg.push_str(r#" text-decoration="underline""#);
                }
                if let Some(c) = color {
                    svg.push_str(r#" fill=""#);
                    c.write_svg(svg);
                    svg.push('"');
                }
                // Leading/trailing inter-word spaces are significant here
                // (each run is its own <text> element).
                svg.push_str(r#" xml:space="preserve">"#);
                write_escaped(svg, text);
                svg.push_str("</text>");
                write_newline(svg, pretty);
            }
            Piece::Math(m) => {
                write_indent(svg, depth, pretty);
                crate::backend::svg_math::embed_label(
                    svg,
                    cursor - frag_margin,
                    y,
                    TextAnchor::Start,
                    None,
                    m,
                    *math_uid,
                );
                *math_uid += 1;
                write_newline(svg, pretty);
            }
        }
        cursor += w;
    }
}

#[cfg(test)]
mod tests {
    use super::write_float;

    /// `x * 0.01` leaves float noise that ryu prints in full (`61.300000000000004`);
    /// every value must come out as at most two decimals.
    #[test]
    fn write_float_never_emits_float_noise() {
        for (v, want) in [
            (61.3, "61.3"),
            (3.26, "3.26"),
            (449.59, "449.59"),
            (0.29, "0.29"),
            (-12.345, "-12.35"),
            (100.0, "100"),
        ] {
            let mut s = String::new();
            write_float(&mut s, v);
            assert_eq!(s, want, "write_float({v})");
        }
        for i in -200_000..200_000 {
            let mut s = String::new();
            write_float(&mut s, i as f64 * 0.013);
            let decimals = s.split('.').nth(1).map_or(0, str::len);
            assert!(decimals <= 2, "write_float({}) = {s}", i as f64 * 0.013);
        }
    }

    use super::SvgBackend;
    use crate::render::color::Color;
    use crate::render::render::{Primitive, Scene};

    fn rect(x: f64, y: f64, w: f64, h: f64, fill: &str) -> Primitive {
        Primitive::Rect {
            x,
            y,
            width: w,
            height: h,
            fill: Color::from(fill),
            stroke: None,
            stroke_width: None,
            opacity: None,
        }
    }

    fn render(elements: Vec<Primitive>, batching: bool) -> String {
        let mut scene = Scene::new(100.0, 100.0);
        scene.elements = elements;
        SvgBackend::new()
            .with_rect_batching(batching)
            .render_scene(&scene)
    }

    /// Bricks: alternating colours with small gaps. One path per colour, every brick its own
    /// closed subpath (so the gaps survive), colours in first-seen order.
    #[test]
    fn non_overlapping_run_becomes_one_path_per_colour() {
        let els = (0..6)
            .map(|i| {
                let fill = if i % 2 == 0 { "#ff0000" } else { "#0000ff" };
                rect(10.0 + i as f64 * 2.4, 5.0, 2.28, 4.0, fill)
            })
            .collect();
        let svg = render(els, true);
        assert_eq!(svg.matches("<path").count(), 2, "{svg}");
        assert_eq!(
            svg.matches("<rect").count(),
            1,
            "only the background rect: {svg}"
        );
        assert_eq!(svg.matches('z').count(), 6);
        let red = svg.find("#ff0000").unwrap();
        let blue = svg.find("#0000ff").unwrap();
        assert!(red < blue);
        // First red brick absolute, the next two relative to it (4.8 px pitch).
        assert!(
            svg.contains(r#"d="M10 5h2.28v4h-2.28zm4.8 0h2.28v4h-2.28zm4.8 0h2.28v4h-2.28z""#),
            "{svg}"
        );
    }

    /// Two different colours overlapping: paint order matters, so the run stays as `<rect>`s,
    /// byte for byte what batching-off writes.
    #[test]
    fn overlapping_run_is_left_alone() {
        let els = || {
            vec![
                rect(0.0, 0.0, 10.0, 10.0, "#ff0000"),
                rect(5.0, 5.0, 10.0, 10.0, "#0000ff"),
                rect(20.0, 0.0, 10.0, 10.0, "#ff0000"),
            ]
        };
        assert_eq!(render(els(), true), render(els(), false));
    }

    /// Edges that touch (or overlap by less than the 0.01 px written precision) are not an
    /// overlap: flush heatmap cells still batch.
    #[test]
    fn touching_cells_still_batch() {
        let els = vec![
            rect(0.0, 0.0, 10.0, 10.0, "#ff0000"),
            rect(10.0, 0.0, 10.0, 10.0, "#00ff00"),
            rect(0.0, 9.995, 10.0, 10.0, "#00ff00"),
            rect(10.0, 10.0, 10.0, 10.0, "#ff0000"),
        ];
        let svg = render(els, true);
        assert_eq!(svg.matches("<path").count(), 2, "{svg}");
    }

    /// Stroked, translucent, pattern-filled or unfilled rects break the run; a lone plain rect
    /// is never turned into a path.
    #[test]
    fn only_plain_rects_batch() {
        let mut stroked = rect(30.0, 0.0, 5.0, 5.0, "#ff0000");
        if let Primitive::Rect { stroke, .. } = &mut stroked {
            *stroke = Some(Color::from("#000000"));
        }
        let mut faded = rect(40.0, 0.0, 5.0, 5.0, "#ff0000");
        if let Primitive::Rect { opacity, .. } = &mut faded {
            *opacity = Some(0.5);
        }
        let els = vec![
            rect(0.0, 0.0, 5.0, 5.0, "#ff0000"),
            stroked,
            rect(10.0, 0.0, 5.0, 5.0, "url(#hatch)"),
            rect(20.0, 0.0, 5.0, 5.0, "#ff0000"),
            faded,
        ];
        let batched = render(els, true);
        assert!(!batched.contains("<path"), "{batched}");
    }

    /// A RectBatch (heatmap cells) batches, and zero-size cells are dropped rather than drawn.
    #[test]
    fn rect_batch_primitive_batches_and_skips_empty_cells() {
        let els = vec![Primitive::RectBatch {
            x: vec![0.0, 10.0, 20.0, 30.0],
            y: vec![0.0; 4],
            w: vec![10.0, 10.0, 0.0, 10.0],
            h: vec![10.0; 4],
            fills: vec![Color::from("#111111"); 4],
        }];
        let svg = render(els, true);
        assert_eq!(svg.matches("<path").count(), 1, "{svg}");
        assert_eq!(svg.matches('z').count(), 3, "{svg}");
    }

    /// The default is on, and `SvgBackend` the value-namespace const matches `new()`.
    #[test]
    fn batching_is_the_default() {
        let els = || {
            vec![
                rect(0.0, 0.0, 5.0, 5.0, "#ff0000"),
                rect(10.0, 0.0, 5.0, 5.0, "#ff0000"),
            ]
        };
        let mut scene = Scene::new(100.0, 100.0);
        scene.elements = els();
        assert!(super::SvgBackend.render_scene(&scene).contains("<path"));
        assert!(SvgBackend::new().render_scene(&scene).contains("<path"));
    }
}
