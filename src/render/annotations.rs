use crate::render::color::Color;
use crate::render::layout::ComputedLayout;
use crate::render::render::{PathData, Primitive, Scene, TextAnchor};

/// Annotation data types moved to `crate::plot::annotations` (per the plot -> render
/// architecture rule); re-exported here so existing `render::annotations::*` paths work.
pub use crate::plot::annotations::{Orientation, ReferenceLine, ShadedRegion, TextAnnotation};

pub fn add_shaded_regions(regions: &[ShadedRegion], scene: &mut Scene, computed: &ComputedLayout) {
    let plot_left = computed.margin_left;
    let plot_right = computed.width - computed.margin_right;
    let plot_top = computed.margin_top;
    let plot_bottom = computed.height - computed.margin_bottom;

    for region in regions {
        let (x, y, width, height) = match region.orientation {
            Orientation::Horizontal => {
                let y_top = computed.map_y(region.max_val);
                let y_bottom = computed.map_y(region.min_val);
                (plot_left, y_top, plot_right - plot_left, y_bottom - y_top)
            }
            Orientation::Vertical => {
                let x_left = computed.map_x(region.min_val);
                let x_right = computed.map_x(region.max_val);
                (x_left, plot_top, x_right - x_left, plot_bottom - plot_top)
            }
        };

        scene.add(Primitive::Rect {
            x,
            y,
            width,
            height,
            fill: Color::from(&region.color),
            stroke: None,
            stroke_width: None,
            opacity: Some(region.opacity),
        });
    }
}

pub fn add_reference_lines(lines: &[ReferenceLine], scene: &mut Scene, computed: &ComputedLayout) {
    let plot_left = computed.margin_left;
    let plot_right = computed.width - computed.margin_right;
    let plot_top = computed.margin_top;
    let plot_bottom = computed.height - computed.margin_bottom;

    for line in lines {
        let (x1, y1, x2, y2) = match line.orientation {
            Orientation::Horizontal => {
                let y = computed.map_y(line.value);
                (plot_left, y, plot_right, y)
            }
            Orientation::Vertical => {
                let x = computed.map_x(line.value);
                (x, plot_top, x, plot_bottom)
            }
        };

        scene.add(Primitive::Line {
            x1,
            y1,
            x2,
            y2,
            stroke: Color::from(&line.color),
            stroke_width: line.stroke_width,
            stroke_dasharray: Some(line.dasharray.clone()),
        });

        if let Some(ref label) = line.label {
            let (tx, ty, anchor) = match line.orientation {
                Orientation::Horizontal => (plot_right - 4.0, y1 - 4.0, TextAnchor::End),
                Orientation::Vertical => (x1 + 4.0, plot_top + 12.0, TextAnchor::Start),
            };
            scene.add(Primitive::Text {
                x: tx,
                y: ty,
                content: label.clone(),
                size: computed.tick_size,
                anchor,
                rotate: None,
                bold: false,
                color: None,
            });
        }
    }
}

pub fn add_text_annotations(
    annotations: &[TextAnnotation],
    scene: &mut Scene,
    computed: &ComputedLayout,
) {
    for ann in annotations {
        let tx = computed.map_x(ann.text_x);
        let ty = computed.map_y(ann.text_y);

        // Determine whether text sits above or below the anchor based on
        // the arrow target direction. In SVG coords, smaller y = higher on
        // screen. If the target is above (ay < ty) the text goes below the
        // anchor so the arrow line won't cross through it, and vice versa.
        let text_offset = if let (Some(_), Some(target_y)) = (ann.target_x, ann.target_y) {
            let ay = computed.map_y(target_y);
            if ay < ty {
                // Target is above text anchor -> place text below
                ann.font_size as f64 + 4.0
            } else {
                // Target is below or level -> place text above
                -(6.0)
            }
        } else {
            // No arrow -> default to above
            -(6.0)
        };

        if let (Some(target_x), Some(target_y)) = (ann.target_x, ann.target_y) {
            let ax = computed.map_x(target_x);
            let ay = computed.map_y(target_y);

            let dx = ax - tx;
            let dy = ay - ty;
            let len = (dx * dx + dy * dy).sqrt();
            if len > 0.0 {
                let ux = dx / len;
                let uy = dy / len;

                // Pull the arrow tip back by the padding so it doesn't
                // overlap the data point
                let tip_x = ax - ux * ann.arrow_padding;
                let tip_y = ay - uy * ann.arrow_padding;

                // Draw arrow line from text anchor to the padded tip
                scene.add(Primitive::Line {
                    x1: tx,
                    y1: ty,
                    x2: tip_x,
                    y2: tip_y,
                    stroke: Color::from(&ann.color),
                    stroke_width: computed.axis_stroke_width,
                    stroke_dasharray: None,
                });

                // Draw arrowhead at the padded tip
                let arrow_len = computed.annotation_arrow_len;
                let arrow_half_w = computed.annotation_arrow_half_w;
                let d = crate::render::render_utils::arrow_head_path(
                    tip_x,
                    tip_y,
                    ux,
                    uy,
                    arrow_len,
                    arrow_half_w,
                );

                scene.add(Primitive::Path(Box::new(PathData {
                    d,
                    fill: Some(Color::from(&ann.color)),
                    stroke: Color::from(&ann.color),
                    stroke_width: computed.axis_stroke_width,
                    opacity: None,
                    stroke_dasharray: None,
                })));
            }
        }

        scene.add(Primitive::Text {
            x: tx,
            y: ty + text_offset,
            content: ann.text.clone(),
            size: ann.font_size,
            anchor: TextAnchor::Middle,
            rotate: None,
            bold: false,
            color: None,
        });
    }
}
