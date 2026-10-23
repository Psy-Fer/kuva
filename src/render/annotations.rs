use crate::render::color::Color;
use crate::render::layout::{ComputedLayout, Layout};
use crate::render::render::{PathData, Primitive, Scene, TextAnchor};

/// Annotation data types moved to `crate::plot::annotations` (per the plot -> render
/// architecture rule); re-exported here so existing `render::annotations::*` paths work.
pub use crate::plot::annotations::{
    BracketAnchor, BracketLabel, Orientation, ReferenceLine, ShadedRegion, SignificanceBracket,
    TextAnnotation,
};

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

/// Draw the layout's significance brackets: a three-segment bracket between
/// two groups on the category axis with its label centred beyond it. Spans,
/// orientation and auto-stacked positions come from `computed`, which also
/// grew the value axis so the stack fits.
pub fn add_brackets(layout: &Layout, scene: &mut Scene, computed: &ComputedLayout) {
    for (i, b) in layout.brackets.iter().enumerate() {
        let Some((lo, hi)) = computed.bracket_spans.get(i).copied().flatten() else {
            continue;
        };
        let horizontal = computed.bracket_horizontal.get(i).copied().unwrap_or(false);
        let pos = match b.position {
            Some(p) => p,
            None => {
                let Some(level) = computed.bracket_levels.get(i).copied().flatten() else {
                    continue;
                };
                if horizontal {
                    computed.bracket_stack_x.position(level)
                } else {
                    computed.bracket_stack_y.position(level)
                }
            }
        };

        let font = b.font_size.unwrap_or(computed.body_size);
        let font_px = font as f64;
        let tick = (font_px * 0.45).max(3.0);
        // No explicit colour: follow the theme's text colour for both the
        // stroke and the label.
        let stroke = b
            .color
            .as_ref()
            .map(Color::from)
            .unwrap_or_else(|| Color::from(&computed.theme.text_color));
        let label = b.label();
        // Asterisks have no descenders and sit high in the em box, so a star
        // code hugs the bar; text with descenders needs clearance.
        let gap = if b.label_is_stars() {
            font_px * 0.12
        } else {
            font_px * 0.3
        };
        // Terminal output snaps text to rows and columns: keep the label a
        // full row above the bar, or a clear column beside a horizontal
        // bracket (a row is about two columns), so it never touches the
        // bracket's own cells.
        let cell_h = computed.bracket_cell_h;
        let gap_v = cell_h.map_or(gap, |ch| gap.max(ch));
        let gap_h = cell_h.map_or(gap + font_px * 0.1, |ch| (gap + font_px * 0.1).max(ch));

        let (d, tx, ty, anchor) = if horizontal {
            let y_lo = computed.map_y(lo);
            let y_hi = computed.map_y(hi);
            let px = computed.map_x(pos);
            (
                format!(
                    "M {:.2} {:.2} L {:.2} {:.2} L {:.2} {:.2} L {:.2} {:.2}",
                    px - tick,
                    y_lo,
                    px,
                    y_lo,
                    px,
                    y_hi,
                    px - tick,
                    y_hi
                ),
                px + gap_h,
                (y_lo + y_hi) / 2.0 + font_px * 0.35,
                TextAnchor::Start,
            )
        } else {
            let x_lo = computed.map_x(lo);
            let x_hi = computed.map_x(hi);
            let py = computed.map_y(pos);
            (
                format!(
                    "M {:.2} {:.2} L {:.2} {:.2} L {:.2} {:.2} L {:.2} {:.2}",
                    x_lo,
                    py + tick,
                    x_lo,
                    py,
                    x_hi,
                    py,
                    x_hi,
                    py + tick
                ),
                (x_lo + x_hi) / 2.0,
                py - gap_v,
                TextAnchor::Middle,
            )
        };

        scene.add(Primitive::Path(Box::new(PathData {
            d,
            fill: None,
            stroke,
            stroke_width: b.stroke_width * computed.axis_stroke_width,
            opacity: None,
            stroke_dasharray: None,
        })));

        if !label.is_empty() {
            scene.add(Primitive::Text {
                x: tx,
                y: ty,
                content: label,
                size: font,
                anchor,
                rotate: None,
                bold: false,
                color: b.color.as_ref().map(Color::from),
            });
        }
    }
}
