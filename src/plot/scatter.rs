use crate::plot::types::Point;

pub struct ScatterPlot {
    pub points: Vec<Point>,
}

impl ScatterPlot {
    pub fn new(points: Vec<Point>) -> Self {
        Self { points }
    }
}