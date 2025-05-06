use crate::plot::types::Point;

pub struct LinePlot {
    pub data: Vec<Point>,
    pub color: String,
    pub stroke_width: f64,
}

impl LinePlot {
    pub fn new() -> Self {
        Self {
            data: vec![],
            color: "black".into(),
            stroke_width: 2.0,
        }
    }

    pub fn with_data(mut self, data: Vec<Point>) -> Self {
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
}