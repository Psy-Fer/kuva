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

#[derive(Clone)]
pub struct LegendGroup {
    pub title: String,
    pub entries: Vec<LegendEntry>,
}

pub struct Legend {
    pub title: Option<String>,
    pub entries: Vec<LegendEntry>,
    pub groups: Option<Vec<LegendGroup>>,
    pub position: LegendPosition,
    pub show_box: bool,
}

impl Default for Legend {
    fn default() -> Self {
        Self {
            title: None,
            entries: Vec::new(),
            groups: None,
            position: LegendPosition::default(),
            show_box: true,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub enum LegendPosition {
    // Inside the plot axes area (overlay, ~8px inset from axis edges)
    InsideTopRight,
    InsideTopLeft,
    InsideBottomRight,
    InsideBottomLeft,
    InsideTopCenter,
    InsideBottomCenter,
    // Outside — right margin (default)
    #[default]
    OutsideRightTop,
    OutsideRightMiddle,
    OutsideRightBottom,
    // Outside — left margin
    OutsideLeftTop,
    OutsideLeftMiddle,
    OutsideLeftBottom,
    // Outside — top margin
    OutsideTopLeft,
    OutsideTopCenter,
    OutsideTopRight,
    // Outside — bottom margin
    OutsideBottomLeft,
    OutsideBottomCenter,
    OutsideBottomRight,
    // Absolute SVG canvas pixel coordinate
    Custom(f64, f64),
    // Data-space coordinate — mapped through map_x/map_y at render time
    DataCoords(f64, f64),
}

pub struct ColorBarInfo {
    pub map_fn: Arc<dyn Fn(f64) -> String + Send + Sync>,
    pub min_value: f64,
    pub max_value: f64,
    pub label: Option<String>,
    /// When set, overrides auto-generated ticks. Each entry is `(position, label)` where
    /// `position` is in `[min_value, max_value]` space.
    pub tick_labels: Option<Vec<(f64, String)>>,
}
