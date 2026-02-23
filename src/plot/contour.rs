use crate::plot::heatmap::ColorMap;

#[derive(Clone)]
pub struct ContourPlot {
    pub z: Vec<Vec<f64>>,           // [row][col], always the canonical grid
    pub x_coords: Vec<f64>,         // len = cols
    pub y_coords: Vec<f64>,         // len = rows
    pub levels: Vec<f64>,           // explicit iso-levels; empty = auto from n_levels
    pub n_levels: usize,            // default 8
    pub filled: bool,               // draw filled bands
    pub color_map: ColorMap,
    pub line_width: f64,            // iso-line stroke width
    pub line_color: Option<String>, // None = use colormap for lines too
    pub legend_label: Option<String>,
}

impl ContourPlot {
    pub fn new() -> Self {
        Self {
            z: vec![],
            x_coords: vec![],
            y_coords: vec![],
            levels: vec![],
            n_levels: 8,
            filled: false,
            color_map: ColorMap::Viridis,
            line_width: 1.0,
            line_color: None,
            legend_label: None,
        }
    }

    pub fn with_grid(mut self, z: Vec<Vec<f64>>, x_coords: Vec<f64>, y_coords: Vec<f64>) -> Self {
        self.z = z;
        self.x_coords = x_coords;
        self.y_coords = y_coords;
        self
    }

    pub fn with_points<I>(mut self, pts: I) -> Self
    where
        I: IntoIterator<Item = (f64, f64, f64)>,
    {
        let pts: Vec<(f64, f64, f64)> = pts.into_iter().collect();
        if pts.is_empty() {
            return self;
        }

        let x_min = pts.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
        let x_max = pts.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
        let y_min = pts.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let y_max = pts.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);

        const GRID_SIZE: usize = 50;
        const EPSILON: f64 = 1e-10;

        let x_step = (x_max - x_min) / GRID_SIZE as f64;
        let y_step = (y_max - y_min) / GRID_SIZE as f64;

        let mut z = vec![vec![0.0f64; GRID_SIZE]; GRID_SIZE];
        let mut x_coords = vec![0.0f64; GRID_SIZE];
        let mut y_coords = vec![0.0f64; GRID_SIZE];

        for col in 0..GRID_SIZE {
            x_coords[col] = x_min + (col as f64 + 0.5) * x_step;
        }
        for row in 0..GRID_SIZE {
            y_coords[row] = y_min + (row as f64 + 0.5) * y_step;
        }

        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let cx = x_coords[col];
                let cy = y_coords[row];
                let mut weight_sum = 0.0;
                let mut value_sum = 0.0;
                for &(px, py, pz) in &pts {
                    let d2 = (cx - px) * (cx - px) + (cy - py) * (cy - py) + EPSILON;
                    let w = 1.0 / d2;
                    weight_sum += w;
                    value_sum += w * pz;
                }
                z[row][col] = value_sum / weight_sum;
            }
        }

        self.z = z;
        self.x_coords = x_coords;
        self.y_coords = y_coords;
        self
    }

    pub fn with_levels(mut self, levels: &[f64]) -> Self {
        self.levels = levels.to_vec();
        self
    }

    pub fn with_n_levels(mut self, n: usize) -> Self {
        self.n_levels = n;
        self
    }

    pub fn with_filled(mut self) -> Self {
        self.filled = true;
        self
    }

    pub fn with_colormap(mut self, map: ColorMap) -> Self {
        self.color_map = map;
        self
    }

    pub fn with_line_color<S: Into<String>>(mut self, color: S) -> Self {
        self.line_color = Some(color.into());
        self
    }

    pub fn with_line_width(mut self, w: f64) -> Self {
        self.line_width = w;
        self
    }

    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    /// Compute effective iso-levels (respects explicit `levels` or auto from `n_levels`).
    pub fn effective_levels(&self) -> Vec<f64> {
        if !self.levels.is_empty() {
            return self.levels.clone();
        }
        let (z_min, z_max) = self.z_range();
        if z_min >= z_max || self.n_levels == 0 {
            return vec![];
        }
        let n = self.n_levels;
        (0..n)
            .map(|i| z_min + (i as f64 + 1.0) / (n as f64 + 1.0) * (z_max - z_min))
            .collect()
    }

    pub fn z_range(&self) -> (f64, f64) {
        let mut z_min = f64::INFINITY;
        let mut z_max = f64::NEG_INFINITY;
        for row in &self.z {
            for &v in row {
                z_min = z_min.min(v);
                z_max = z_max.max(v);
            }
        }
        (z_min, z_max)
    }
}
