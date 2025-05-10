
use std::sync::Arc;
use colorous::{VIRIDIS, INFERNO, GREYS};

// Map [0.0, 1.0] to color string
fn viridis(value: f64) -> String {
    let rgb = VIRIDIS.eval_continuous(value.clamp(0.0, 1.0));
    format!("rgb({},{},{})", rgb.r, rgb.g, rgb.b)
}

fn inferno(value: f64) -> String {
    let rgb = INFERNO.eval_continuous(value.clamp(0.0, 1.0));
    format!("rgb({},{},{})", rgb.r, rgb.g, rgb.b)
}

fn greyscale(value: f64) -> String {
    let rgb = GREYS.eval_continuous(value.clamp(0.0, 1.0));
    format!("rgb({},{},{})", rgb.r, rgb.g, rgb.b)
}

#[derive(Clone)]
pub enum ColorMap {
    Grayscale,
    Viridis,
    Inferno,
    Custom(Arc<dyn Fn(f64) -> String + Send + Sync>), //Arc<dyn Fn...> if need to clone
}


impl ColorMap {
    pub fn map(&self, value: f64) -> String {
        match self {
            ColorMap::Grayscale => greyscale(value),
            ColorMap::Viridis => viridis(value),
            ColorMap::Inferno => inferno(value),
            ColorMap::Custom(f) => f(value),
        }
    }
}


#[derive(Clone)]
pub struct Heatmap {
    pub data: Vec<Vec<f64>>,      // Rows x Columns
    pub row_labels: Option<Vec<String>>,
    pub col_labels: Option<Vec<String>>,
    pub color_map: ColorMap,      // Enum for color scale
    pub show_values: bool,        // Optional value overlay
}


impl Heatmap {
    pub fn new(data: Vec<Vec<f64>>) -> Self {
        Self {
            data,
            row_labels: None,
            col_labels: None,
            color_map: ColorMap::Viridis,
            show_values: false,
        }
    }

    pub fn with_labels(mut self, rows: Vec<String>, cols: Vec<String>) -> Self {
        self.row_labels = Some(rows);
        self.col_labels = Some(cols);
        self
    }

    pub fn with_color_map(mut self, map: ColorMap) -> Self {
        self.color_map = map;
        self
    }

    pub fn show_values(mut self, show: bool) -> Self {
        self.show_values = show;
        self
    }
}