use crate::render::render::{Scene, Primitive, TextAnchor};
use crate::render::layout::ComputedLayout;

pub enum Orientation {
    Horizontal,
    Vertical,
}

pub struct TextAnnotation {
    pub text: String,
    pub text_x: f64,
    pub text_y: f64,
    pub target_x: Option<f64>,
    pub target_y: Option<f64>,
    pub font_size: u32,
    pub color: String,
    pub arrow_padding: f64,
}

impl TextAnnotation {
    pub fn new<S: Into<String>>(text: S, x: f64, y: f64) -> Self {
        Self {
            text: text.into(),
            text_x: x,
            text_y: y,
            target_x: None,
            target_y: None,
            font_size: 12,
            color: "black".into(),
            arrow_padding: 6.0,
        }
    }

    pub fn with_arrow(mut self, target_x: f64, target_y: f64) -> Self {
        self.target_x = Some(target_x);
        self.target_y = Some(target_y);
        self
    }

    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_font_size(mut self, size: u32) -> Self {
        self.font_size = size;
        self
    }

    pub fn with_arrow_padding(mut self, padding: f64) -> Self {
        self.arrow_padding = padding;
        self
    }
}

pub struct ReferenceLine {
    pub value: f64,
    pub orientation: Orientation,
    pub color: String,
    pub stroke_width: f64,
    pub dasharray: String,
    pub label: Option<String>,
}

impl ReferenceLine {
    pub fn horizontal(y: f64) -> Self {
        Self {
            value: y,
            orientation: Orientation::Horizontal,
            color: "red".into(),
            stroke_width: 1.0,
            dasharray: "6 4".into(),
            label: None,
        }
    }

    pub fn vertical(x: f64) -> Self {
        Self {
            value: x,
            orientation: Orientation::Vertical,
            color: "red".into(),
            stroke_width: 1.0,
            dasharray: "6 4".into(),
            label: None,
        }
    }

    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    pub fn with_dasharray<S: Into<String>>(mut self, dash: S) -> Self {
        self.dasharray = dash.into();
        self
    }
}

pub struct ShadedRegion {
    pub orientation: Orientation,
    pub min_val: f64,
    pub max_val: f64,
    pub color: String,
    pub opacity: f64,
}

impl ShadedRegion {
    pub fn horizontal(y_min: f64, y_max: f64) -> Self {
        Self {
            orientation: Orientation::Horizontal,
            min_val: y_min,
            max_val: y_max,
            color: "blue".into(),
            opacity: 0.15,
        }
    }

    pub fn vertical(x_min: f64, x_max: f64) -> Self {
        Self {
            orientation: Orientation::Vertical,
            min_val: x_min,
            max_val: x_max,
            color: "blue".into(),
            opacity: 0.15,
        }
    }

    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }
}

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
            fill: region.color.clone(),
            stroke: None,
            stroke_width: None,
            opacity: Some(region.opacity * 100.0),
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
            stroke: line.color.clone(),
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
                size: 10,
                anchor,
                rotate: None,
                bold: false,
            });
        }
    }
}

pub fn add_text_annotations(annotations: &[TextAnnotation], scene: &mut Scene, computed: &ComputedLayout) {
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
                    stroke: ann.color.clone(),
                    stroke_width: 1.0,
                    stroke_dasharray: None,
                });

                // Draw arrowhead at the padded tip
                let arrow_len = 8.0;
                let arrow_half_w = 4.0;

                let base_x = tip_x - ux * arrow_len;
                let base_y = tip_y - uy * arrow_len;
                let left_x = base_x - uy * arrow_half_w;
                let left_y = base_y + ux * arrow_half_w;
                let right_x = base_x + uy * arrow_half_w;
                let right_y = base_y - ux * arrow_half_w;

                scene.add(Primitive::Path {
                    d: format!("M{tip_x},{tip_y} L{left_x},{left_y} L{right_x},{right_y} Z"),
                    fill: Some(ann.color.clone()),
                    stroke: ann.color.clone(),
                    stroke_width: 1.0,
                    opacity: None,
                });
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
        });
    }
}
