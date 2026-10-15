//! Annotation *data* types (reference lines, shaded regions, text callouts) shared by the
//! plot and render layers. These are pure configuration structs (no render dependencies), so
//! they live in `src/plot/` per the one-way `plot → render` architecture rule. The drawing
//! functions that consume them stay in [`crate::render::annotations`], which re-exports these
//! types for backward compatibility.

#[derive(Clone, Debug)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Clone)]
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

#[derive(Clone, Debug)]
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

#[derive(Clone)]
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
