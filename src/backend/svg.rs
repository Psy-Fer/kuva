use crate::render::render::{Scene, Primitive, TextAnchor};


fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}

// I should probably use the SVG lib for this backend in future.
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
        let nl = if self.pretty { "\n" } else { "" };
        let indent = |d: usize| -> String {
            if self.pretty { "  ".repeat(d) } else { String::new() }
        };

        // create svg with width and height
        let font_attr = if let Some(ref family) = scene.font_family {
            format!(r#" font-family="{family}""#)
        } else {
            String::new()
        };
        let fill_attr = if let Some(ref color) = scene.text_color {
            format!(r#" fill="{color}""#)
        } else {
            String::new()
        };
        // Pre-allocate: ~80 bytes per primitive avoids repeated reallocs at scale.
        let estimated_capacity = 200 + scene.elements.len() * 80;
        let mut svg = String::with_capacity(estimated_capacity);
        svg.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}"{font_attr}{fill_attr}>"#,
            w = scene.width,
            h = scene.height
        ));
        svg.push_str(nl);

        // Add a background rect if specified: .with_background(Some("white"))
        // "none" for transparent
        if let Some(color) = &scene.background_color {
            svg.push_str(&indent(1));
            svg.push_str(&format!(
                r#"<rect width="100%" height="100%" fill="{color}" />"#
            ));
            svg.push_str(nl);
        }

        // Emit any SVG defs (e.g. linearGradients for Sankey ribbons)
        if !scene.defs.is_empty() {
            svg.push_str(&indent(1));
            svg.push_str("<defs>");
            for d in &scene.defs {
                svg.push_str(d);
            }
            svg.push_str("</defs>");
            svg.push_str(nl);
        }

        // go through each element, and add it to the SVG
        let mut depth: usize = 1;
        for elem in &scene.elements {
            match elem {
                Primitive::Circle { cx, cy, r, fill } => {
                    svg.push_str(&indent(depth));
                    svg.push_str(&format!(
                        r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{fill}" />"#,
                    ));
                    svg.push_str(nl);
                }
                Primitive::Text { x, y, content, size, anchor, rotate, bold } => {
                    let anchor_str = match anchor {
                        TextAnchor::Start => "start",
                        TextAnchor::Middle => "middle",
                        TextAnchor::End => "end",
                    };

                    let transform = if let Some(angle) = rotate {
                        format!(r#" transform="rotate({angle},{x},{y})""#)
                    } else {
                        "".into()
                    };

                    let bold_str = if *bold { r#" font-weight="bold""# } else { "" };

                    let escaped = escape_xml(content);
                    svg.push_str(&indent(depth));
                    svg.push_str(&format!(
                        r#"<text x="{x}" y="{y}" font-size="{size}" text-anchor="{anchor_str}"{bold_str}{transform}>{escaped}</text>"#
                    ));
                    svg.push_str(nl);
                }
                Primitive::Line { x1, y1, x2, y2, stroke, stroke_width, stroke_dasharray } => {
                    svg.push_str(&indent(depth));
                    svg.push_str(&format!(
                        r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{stroke}" stroke-width="{stroke_width}""#,
                    ));
                    if let Some(dash) = stroke_dasharray {
                        svg.push_str(&format!(r#" stroke-dasharray="{dash}""#));
                    }
                    svg.push_str(" />");
                    svg.push_str(nl);
                }
                Primitive::Path { d, fill, stroke, stroke_width, opacity, stroke_dasharray } => {
                    svg.push_str(&indent(depth));
                    svg.push_str(&format!(
                        r#"<path d="{d}" stroke="{stroke}" stroke-width="{stroke_width}""#
                    ));

                    if let Some(fill) = fill {
                        svg.push_str(&format!(r#" fill="{fill}""#));
                    }
                    else {
                        svg.push_str(r#" fill="none""#);
                    }

                    if let Some(opacity) = opacity {
                        svg.push_str(&format!(r#" fill-opacity="{opacity}""#));
                    }

                    if let Some(dash) = stroke_dasharray {
                        svg.push_str(&format!(r#" stroke-dasharray="{dash}""#));
                    }

                    svg.push_str(" />");
                    svg.push_str(nl);
                }
                Primitive::GroupStart { transform } => {
                    svg.push_str(&indent(depth));
                    svg.push_str("<g");
                    if let Some(t) = transform {
                        svg.push_str(&format!(r#" transform="{t}""#));
                    }
                    svg.push('>');
                    svg.push_str(nl);
                    depth += 1;
                }
                Primitive::GroupEnd => {
                    depth -= 1;
                    svg.push_str(&indent(depth));
                    svg.push_str("</g>");
                    svg.push_str(nl);
                }
                Primitive::Rect { x, y, width, height, fill, stroke, stroke_width, opacity} => {
                    svg.push_str(&indent(depth));
                    svg.push_str(&format!(
                        r#"<rect x="{x}" y="{y}" width="{width}" height="{height}" fill="{fill}""#
                    ));

                    if let Some(stroke) = stroke {
                        svg.push_str(&format!(r#" stroke="{stroke}""#));
                    }
                    if let Some(width) = stroke_width {
                        svg.push_str(&format!(r#" stroke-width="{width}""#));
                    }
                    if let Some(opacity) = opacity {
                        svg.push_str(&format!(r#" fill-opacity="{opacity}""#));
                    }

                    svg.push_str(" />");
                    svg.push_str(nl);
                }
            }
        }

        // push the end string
        svg.push_str("</svg>");
        svg.push_str(nl);
        svg
    }
}

// Backward-compat shim in the value namespace.
// `SvgBackend.render_scene(...)` in old code: `SvgBackend` resolves to this const.
// `SvgBackend::new()` in new code: `SvgBackend` resolves to the type.
// TODO: To phase out later: add #[deprecated(note = "Use SvgBackend::new()")] here.
#[allow(non_upper_case_globals)]
pub const SvgBackend: SvgBackend = SvgBackend::new();
