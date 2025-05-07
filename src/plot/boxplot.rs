

pub struct BoxPlot {
    pub groups: Vec<BoxGroup>,
    pub color: String,
    pub width: f64,
}

pub struct BoxGroup {
    pub label: String,
    pub values: Vec<f64>,
}

impl BoxPlot {
    pub fn new() -> Self {
        Self {
            groups: vec![],
            color: "black".into(),
            width: 0.8,
        }
    }

    pub fn with_group<T: Into<String>>(mut self, label: T, values: Vec<f64>) -> Self {
        self.groups.push(BoxGroup {
            label: label.into(),
            values,
        });
        self
    }

    pub fn with_color<T: Into<String>>(mut self, color: T) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }
}
