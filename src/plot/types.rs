#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub enum BarX {
    Numeric(f64),
    Category(String),
}