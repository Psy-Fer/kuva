
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
pub struct Histogram2D {
    pub data: Vec<(f64, f64)>,
    pub bins: Vec<Vec<usize>>, // [row][col]
    pub x_range: (f64, f64),
    pub y_range: (f64, f64),
    pub bins_x: usize,
    pub bins_y: usize,
    pub color_map: ColorMap,
    pub show_correlation: bool,
}

impl Histogram2D {
    pub fn new() -> Self {
        Self {
            data: vec![],
            bins: vec![],
            x_range: (0.0, 0.0),
            y_range: (0.0, 0.0),
            bins_x: 10,
            bins_y: 10,
            color_map: ColorMap::Viridis,
            show_correlation: false,
        }
    }

    pub fn with_data<T: Into<f64>>(mut self,
                                       data: Vec<(T, T)>,
                                       x_range: (f64, f64),
                                       y_range: (f64, f64),
                                       bins_x: usize,
                                       bins_y: usize)
                                    -> Self {

        let mut bins = vec![vec![0usize; bins_x]; bins_y];

        // I don't think this is controlling the bin segmentation properly
        let x_bin_width = (x_range.1 - x_range.0) / bins_x as f64;
        let y_bin_height = (y_range.1 - y_range.0) / bins_y as f64;

        for (x_raw, y_raw) in data {
            let x = x_raw.into();
            let y = y_raw.into();

            self.data.push((x, y));

            if x < x_range.0 || x >= x_range.1 || y < y_range.0 || y >= y_range.1 {
                continue; // ignore out-of-bounds
            }

            let col = ((x - x_range.0) / x_bin_width).floor() as usize;
            let row = ((y - y_range.0) / y_bin_height).floor() as usize;

            // Safety check to ensure we don't overflow
            if row < bins_y && col < bins_x {
                bins[row][col] += 1;
            }
        }

        // self.data = data;
        self.bins = bins;
        self.x_range = x_range;
        self.y_range = y_range;
        self.bins_x = bins_x;
        self.bins_y = bins_y;

        self
    }

    pub fn with_color_map(mut self, map: ColorMap) -> Self {
        self.color_map = map;
        self
    }

    pub fn show_correlation(mut self) -> Self {
        self.show_correlation = true;
        self
    }
}