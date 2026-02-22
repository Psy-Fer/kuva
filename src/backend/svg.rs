use crate::render::render::{Scene, Primitive, TextAnchor};


fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}

// I should probably use the SVG lib for this backend in future.
pub struct SvgBackend;

impl SvgBackend {
    pub fn render_scene(&self, scene: &Scene) -> String {
        // create svg with width and height
        let font_attr = if let Some(ref family) = scene.font_family {
            format!(r#" font-family="{family}""#)
        } else {
            String::new()
        };
        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}"{font_attr}>"#,
            w = scene.width,
            h = scene.height
        );

        // Add a background rect if specified: .with_background(Some("white"))
        // "none" for transparent
        if let Some(color) = &scene.background_color {
            svg.push_str(&format!(
                r#"<rect width="100%" height="100%" fill="{}" />"#,
                color
            ));
        }

        // go through each element, and add it to the SVG
        for elem in &scene.elements {
            match elem {
                Primitive::Circle { cx, cy, r, fill } => {
                    svg.push_str(&format!(
                        r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{fill}" />"#,
                    ));
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
                    svg.push_str(&format!(
                        r#"<text x="{x}" y="{y}" font-size="{size}" text-anchor="{anchor_str}"{bold_str}{transform}>{escaped}</text>"#
                    ));
                }
                Primitive::Line { x1, y1, x2, y2, stroke, stroke_width, stroke_dasharray } => {
                    svg.push_str(&format!(
                        r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{stroke}" stroke-width="{stroke_width}""#,
                    ));
                    if let Some(dash) = stroke_dasharray {
                        svg.push_str(&format!(r#" stroke-dasharray="{dash}""#));
                    }
                    svg.push_str(" />");
                }
                Primitive::Path { d, fill, stroke, stroke_width, opacity, stroke_dasharray } => {
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
                }
                Primitive::GroupStart { transform } => {
                    svg.push_str("<g");
                    if let Some(t) = transform {
                        svg.push_str(&format!(r#" transform="{t}""#));
                    }
                    svg.push_str(">");
                }
                Primitive::GroupEnd => {
                    svg.push_str("</g>");
                }
                Primitive::Rect { x, y, width, height, fill, stroke, stroke_width, opacity} => {
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

                
                    svg.push_str(&format!(r#" />"#));
                }
            }
        }

        // push the end string
        svg.push_str("</svg>");
        svg
    }
}