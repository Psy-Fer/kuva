use std::sync::Arc;

use crate::plot::scatter::MarkerShape;

#[derive(Clone)]
pub struct LegendEntry {
    pub label: String,
    pub color: String,
    pub shape: LegendShape, // useful for scatter vs line
    pub dasharray: Option<String>,
}

#[derive(Clone, Copy)]
pub enum LegendShape {
    Rect,
    Line,
    Circle,
    Marker(MarkerShape),
    CircleSize(f64),  // circle with explicit pixel radius; used by the size legend
}

#[derive(Default)]
pub struct Legend {
    pub entries: Vec<LegendEntry>,
    pub position: LegendPosition,
}


#[derive(Default, Clone, Copy)]
pub enum LegendPosition {
    #[default]
    TopRight,
    BottomRight,
    BottomLeft,
    TopLeft,
    /// Legend top edge aligns with the plot-area top; placed in the right margin.
    RightTop,
    /// Legend vertically centred on the plot area; placed in the right margin.
    RightMiddle,
    /// Legend bottom edge aligns with the plot-area bottom; placed in the right margin.
    RightBottom,
}

pub struct ColorBarInfo {
    pub map_fn: Arc<dyn Fn(f64) -> String + Send + Sync>,
    pub min_value: f64,
    pub max_value: f64,
    pub label: Option<String>,
}
