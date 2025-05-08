// use crate::plot::types::BarX;

pub struct BarPlot {
    pub groups: Vec<BarGroup>,
    pub color: String,
    pub width: f64,
}

pub struct BarGroup {
    pub label: String,
    pub value: f64,
}

impl BarPlot {
    pub fn new() -> Self {
        Self {
            groups: vec![],
            color: "orange".into(),
            width: 0.8,
        }
    }

    pub fn with_group<T: Into<String>>(mut self, label: T, value: f64) -> Self {
        self.groups.push(BarGroup {
            label: label.into(),
            value,
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