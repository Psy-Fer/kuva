use crate::render::render::{Scene, Primitive, TextAnchor};


// I should probably use the SVG lib for this backend in future.
pub struct SvgBackend;

impl SvgBackend {
    pub fn render_scene(&self, scene: &Scene) -> String {
        // create svg with width and height
        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}">"#,
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
                Primitive::Text { x, y, content, size, anchor, rotate } => {
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
                
                    svg.push_str(&format!(
                        r#"<text x="{x}" y="{y}" font-size="{size}" text-anchor="{anchor_str}"{transform}>{content}</text>"#
                    ));
                }
                Primitive::Line { x1, y1, x2, y2, stroke, stroke_width } => {
                    svg.push_str(&format!(
                        r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{stroke}" stroke-width="{stroke_width}" />"#,
                    ));
                }
                Primitive::Path { d, stroke, stroke_width } => {
                    svg.push_str(&format!(
                        r#"<path d="{d}" stroke="{stroke}" stroke-width="{stroke_width}" fill="none"/>"#
                    ));
                }
                Primitive::Rect { x, y, width, height, fill, stroke, stroke_width } => {
                     svg.push_str(&format!(
                        r#"<rect x="{x}" y="{y}" width="{width}" height="{height}" fill="{fill}""#
                    ));

                    if let Some(stroke) = stroke {
                        svg.push_str(&format!(r#" stroke="{stroke}""#));
                    }
                    if let Some(width) = stroke_width {
                        svg.push_str(&format!(r#" stroke-width="{width}""#));
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