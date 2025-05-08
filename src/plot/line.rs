
pub struct LinePlot {
    pub data: Vec<(f64, f64)>,
    pub color: String,
    pub stroke_width: f64,
    pub legend_label: Option<String>,
}

impl LinePlot {
    pub fn new() -> Self {
        Self {
            data: vec![],
            color: "black".into(),
            stroke_width: 2.0,
            legend_label: None,
        }
    }

    pub fn with_data(mut self, data: Vec<(f64, f64)>) -> Self {
        self.data = data;
        self
    }

    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }
}