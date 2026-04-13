use std::collections::BTreeMap;
use crate::plot::colormap::ColorMap;

/// Dice face positions (1-indexed in a 3×3 grid) for 1–6 dots.
///
/// ```text
/// 1 2 3
/// 4 5 6
/// 7 8 9
/// ```
///
/// Positions follow row-major ordering so the legend key matches visual layout.
const DICE_POSITIONS: [&[usize]; 7] = [
    &[],                       // 0 dots
    &[5],                      // 1 dot  – centre
    &[1, 9],                   // 2 dots – diagonal
    &[1, 5, 9],                // 3 dots – diagonal + centre
    &[1, 7, 3, 9],             // 4 dots – corners (row-major so legend matches)
    &[1, 7, 5, 3, 9],          // 5 dots – corners + centre
    &[1, 4, 7, 3, 6, 9],       // 6 dots – two columns
];

/// One data point: which grid cell, which categories are present, and visual encodings.
pub struct DicePoint {
    /// X-axis category label.
    pub x_cat: String,
    /// Y-axis category label.
    pub y_cat: String,
    /// Which of the `ndots` categories are present (0-indexed, values in `0..ndots`).
    pub present: Vec<usize>,
    /// Continuous value encoded as tile background colour via the colour map.
    pub fill: Option<f64>,
    /// Continuous value encoded as dot radius.
    pub size: Option<f64>,
    /// Per-position categorical dot colours.  Length should equal `ndots`.
    pub dot_colors: Vec<Option<String>>,
    /// Per-position continuous fill values.  Length must equal `ndots`.
    pub dot_fills: Vec<Option<f64>>,
    /// Per-position continuous size values.  Length must equal `ndots`.
    pub dot_sizes: Vec<Option<f64>>,
}

/// A DicePlot: a grid of cells where each cell shows up to 6 dots arranged like a die face.
pub struct DicePlot {
    pub points:             Vec<DicePoint>,
    pub x_categories:       Vec<String>,
    pub y_categories:       Vec<String>,
    pub category_labels:    Vec<String>,
    pub ndots:              usize,
    pub cell_width:         f64,
    pub cell_height:        f64,
    pub pad:                f64,
    pub dot_radius:         f64,
    pub color_map:          ColorMap,
    pub fill_range:         Option<(f64, f64)>,
    pub size_range:         Option<(f64, f64)>,
    pub fill_legend_label:  Option<String>,
    pub size_legend_label:  Option<String>,
    pub dot_legend:         Vec<(String, String)>,
    pub position_legend_label: Option<String>,
    /// Draw a 3×3 sub-grid inside each die tile, showing the pip slot boundaries.
    pub grid_lines: bool,
}

impl Default for DicePlot {
    fn default() -> Self { Self::new(4) }
}

impl DicePlot {
    pub fn new(ndots: usize) -> Self {
        let ndots = ndots.clamp(1, 6);
        Self {
            points: Vec::new(),
            x_categories: Vec::new(),
            y_categories: Vec::new(),
            category_labels: (0..ndots).map(|i| format!("Cat {}", i + 1)).collect(),
            ndots,
            cell_width: 0.8,
            cell_height: 0.8,
            pad: 0.1,
            dot_radius: 0.0,
            color_map: ColorMap::Viridis,
            fill_range: None,
            size_range: None,
            fill_legend_label: None,
            size_legend_label: None,
            dot_legend: Vec::new(),
            position_legend_label: None,
            grid_lines: false,
        }
    }

    pub fn with_points<I, Sx, Sy>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (Sx, Sy, Vec<usize>, Option<f64>, Option<f64>)>,
        Sx: Into<String>,
        Sy: Into<String>,
    {
        for (x_cat, y_cat, present, fill, size) in iter {
            let x_cat: String = x_cat.into();
            let y_cat: String = y_cat.into();
            if !self.x_categories.contains(&x_cat) {
                self.x_categories.push(x_cat.clone());
            }
            if !self.y_categories.contains(&y_cat) {
                self.y_categories.push(y_cat.clone());
            }
            self.points.push(DicePoint {
                x_cat, y_cat, present, fill, size,
                dot_colors: Vec::new(),
                dot_fills: Vec::new(),
                dot_sizes: Vec::new(),
            });
        }
        self
    }

    pub fn with_records<I, Sx, Sy, Sd, Sc>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (Sx, Sy, Sd, Sc)>,
        Sx: Into<String>,
        Sy: Into<String>,
        Sd: Into<String>,
        Sc: Into<String>,
    {
        let mut cell_map: BTreeMap<(String, String), Vec<(usize, String)>> = BTreeMap::new();
        for (x_cat, y_cat, dot_cat, color) in iter {
            let x_cat: String = x_cat.into();
            let y_cat: String = y_cat.into();
            let dot_cat: String = dot_cat.into();
            let color: String = color.into();
            let dot_idx = self.category_labels.iter().position(|l| l == &dot_cat);
            if let Some(dot_idx) = dot_idx {
                if !self.x_categories.contains(&x_cat) {
                    self.x_categories.push(x_cat.clone());
                }
                if !self.y_categories.contains(&y_cat) {
                    self.y_categories.push(y_cat.clone());
                }
                cell_map.entry((x_cat, y_cat)).or_default().push((dot_idx, color));
            }
        }
        for ((x_cat, y_cat), dot_entries) in cell_map {
            let mut dot_colors: Vec<Option<String>> = vec![None; self.ndots];
            for (idx, color) in dot_entries {
                if idx < self.ndots {
                    dot_colors[idx] = Some(color);
                }
            }
            self.points.push(DicePoint {
                x_cat, y_cat,
                present: Vec::new(), fill: None, size: None,
                dot_colors,
                dot_fills: Vec::new(),
                dot_sizes: Vec::new(),
            });
        }
        self
    }

    pub fn with_dot_data<I, Sx, Sy>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (Sx, Sy, usize, Option<f64>, Option<f64>)>,
        Sx: Into<String>,
        Sy: Into<String>,
    {
        type DotEntry = (usize, Option<f64>, Option<f64>);
        let mut cell_map: BTreeMap<(String, String), Vec<DotEntry>> = BTreeMap::new();
        for (x_cat, y_cat, dot_idx, fill, size) in iter {
            let x_cat: String = x_cat.into();
            let y_cat: String = y_cat.into();
            if !self.x_categories.contains(&x_cat) {
                self.x_categories.push(x_cat.clone());
            }
            if !self.y_categories.contains(&y_cat) {
                self.y_categories.push(y_cat.clone());
            }
            if dot_idx < self.ndots {
                cell_map.entry((x_cat, y_cat)).or_default().push((dot_idx, fill, size));
            }
        }
        for ((x_cat, y_cat), dot_entries) in cell_map {
            let mut dot_fills: Vec<Option<f64>> = vec![None; self.ndots];
            let mut dot_sizes: Vec<Option<f64>> = vec![None; self.ndots];
            for (idx, fill, size) in dot_entries {
                dot_fills[idx] = fill;
                dot_sizes[idx] = size;
            }
            if dot_fills.iter().all(|v| v.is_none()) && dot_sizes.iter().all(|v| v.is_none()) {
                continue;
            }
            self.points.push(DicePoint {
                x_cat, y_cat,
                present: Vec::new(), fill: None, size: None,
                dot_colors: Vec::new(),
                dot_fills, dot_sizes,
            });
        }
        self
    }

    pub fn with_x_categories(mut self, cats: Vec<String>) -> Self {
        self.x_categories = cats; self
    }
    pub fn with_y_categories(mut self, cats: Vec<String>) -> Self {
        self.y_categories = cats; self
    }
    pub fn with_category_labels(mut self, labels: Vec<String>) -> Self {
        self.category_labels = labels; self
    }
    pub fn with_color_map(mut self, map: ColorMap) -> Self {
        self.color_map = map; self
    }
    pub fn with_fill_range(mut self, min: f64, max: f64) -> Self {
        self.fill_range = Some((min, max)); self
    }
    pub fn with_size_range(mut self, min: f64, max: f64) -> Self {
        self.size_range = Some((min, max)); self
    }
    pub fn with_fill_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.fill_legend_label = Some(label.into()); self
    }
    pub fn with_size_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.size_legend_label = Some(label.into()); self
    }
    pub fn with_dot_legend<I, S1, S2>(mut self, entries: I) -> Self
    where
        I: IntoIterator<Item = (S1, S2)>,
        S1: Into<String>,
        S2: Into<String>,
    {
        self.dot_legend = entries.into_iter().map(|(l, c)| (l.into(), c.into())).collect();
        self
    }
    pub fn with_position_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.position_legend_label = Some(label.into()); self
    }
    pub fn with_grid_lines(mut self, v: bool) -> Self {
        self.grid_lines = v; self
    }
    pub fn with_dot_radius(mut self, r: f64) -> Self {
        self.dot_radius = r; self
    }
    pub fn with_cell_size(mut self, width: f64, height: f64) -> Self {
        self.cell_width = width; self.cell_height = height; self
    }
    pub fn with_pad(mut self, pad: f64) -> Self {
        self.pad = pad; self
    }

    /// Returns (grid_row, grid_col) for each pip position in order, 0-indexed, row-major.
    /// grid_row: 0=top, 1=middle, 2=bottom; grid_col: 0=left, 1=center, 2=right.
    pub fn dot_grid_positions(&self) -> Vec<(usize, usize)> {
        let positions = DICE_POSITIONS.get(self.ndots).copied().unwrap_or(&[]);
        positions.iter().map(|&p| ((p - 1) / 3, (p - 1) % 3)).collect()
    }

    pub fn dot_offsets(&self) -> Vec<(f64, f64)> {
        let positions = DICE_POSITIONS.get(self.ndots).copied().unwrap_or(&[]);
        let w = self.cell_width;
        let h = self.cell_height;
        let pad = self.pad;
        let avail_w = w - 2.0 * pad;
        let avail_h = h - 2.0 * pad;
        positions.iter().map(|&p| {
            let col = ((p - 1) / 3) as f64;  // row-major: pos 1-3 → row 0, 4-6 → row 1, 7-9 → row 2
            let row = ((p - 1) % 3) as f64;  // column within row: pos 1,4,7 → col 0 (left)
            let dx = col / 2.0 * avail_w + pad - w / 2.0;
            let dy = row / 2.0 * avail_h + pad - h / 2.0;
            (dx, dy)
        }).collect()
    }

    pub fn fill_extent(&self) -> (f64, f64) {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for p in &self.points {
            if let Some(v) = p.fill { min = min.min(v); max = max.max(v); }
            for v in p.dot_fills.iter().flatten() { min = min.min(*v); max = max.max(*v); }
        }
        if min.is_infinite() { (0.0, 1.0) } else { (min, max) }
    }

    pub fn size_extent(&self) -> (f64, f64) {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for p in &self.points {
            if let Some(v) = p.size { min = min.min(v); max = max.max(v); }
            for v in p.dot_sizes.iter().flatten() { min = min.min(*v); max = max.max(*v); }
        }
        if min.is_infinite() { (0.0, 1.0) } else { (min, max) }
    }
}
