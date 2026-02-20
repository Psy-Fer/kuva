use std::sync::Arc;

pub struct LegendEntry {
    pub label: String,
    pub color: String,
    pub shape: LegendShape, // useful for scatter vs line
}

pub enum LegendShape {
    Rect,
    Line,
    Circle,
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
}

pub struct ColorBarInfo {
    pub map_fn: Arc<dyn Fn(f64) -> String + Send + Sync>,
    pub min_value: f64,
    pub max_value: f64,
    pub label: Option<String>,
}
