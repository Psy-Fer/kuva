
pub struct ScatterPlot {
    pub data: Vec<(f64, f64)>,
    pub color: String,
    pub size: f64, // radius of circle point...diff markers will be interesting to add lol
    pub legend_label: Option<String>,
}

impl ScatterPlot {
    pub fn new() -> Self {
        Self {
            data: vec![],
            color: "black".into(),
            size: 3.0,
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

    pub fn with_size(mut self, size: f64) -> Self {
        self.size = size;
        self
    }

    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }
}