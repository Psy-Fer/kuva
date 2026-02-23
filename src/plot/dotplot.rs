use crate::plot::heatmap::ColorMap;

pub struct DotPoint {
    pub x_cat: String,
    pub y_cat: String,
    pub size: f64,    // raw value; encoded as circle radius
    pub color: f64,   // raw value; encoded as fill color
}

pub struct DotPlot {
    pub points:             Vec<DotPoint>,
    pub x_categories:       Vec<String>,          // x axis order (insertion order)
    pub y_categories:       Vec<String>,          // y axis order (insertion order; rendered top->bottom)
    pub color_map:          ColorMap,             // default Viridis
    pub max_radius:         f64,                  // max circle radius (px); default 12.0
    pub min_radius:         f64,                  // min circle radius (px); default 1.0
    pub size_range:         Option<(f64, f64)>,   // clamp size values before normalising; None = auto
    pub color_range:        Option<(f64, f64)>,   // clamp color values; None = auto
    pub size_label:         Option<String>,       // enables size legend with this variable name
    pub color_legend_label: Option<String>,       // enables colorbar with this label
}

impl DotPlot {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            x_categories: Vec::new(),
            y_categories: Vec::new(),
            color_map: ColorMap::Viridis,
            max_radius: 12.0,
            min_radius: 1.0,
            size_range: None,
            color_range: None,
            size_label: None,
            color_legend_label: None,
        }
    }

    /// Data input mode 1: sparse tuple iterator.
    /// Categories are inferred from first-seen insertion order.
    pub fn with_data<I, Sx, Sy, F, G>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (Sx, Sy, F, G)>,
        Sx: Into<String>,
        Sy: Into<String>,
        F: Into<f64>,
        G: Into<f64>,
    {
        for (x_cat, y_cat, size, color) in iter {
            let x_cat: String = x_cat.into();
            let y_cat: String = y_cat.into();
            let size: f64 = size.into();
            let color: f64 = color.into();

            if !self.x_categories.contains(&x_cat) {
                self.x_categories.push(x_cat.clone());
            }
            if !self.y_categories.contains(&y_cat) {
                self.y_categories.push(y_cat.clone());
            }

            self.points.push(DotPoint { x_cat, y_cat, size, color });
        }
        self
    }

    /// Data input mode 2: dense matrix.
    /// `sizes[row_i][col_j]` → y_cat[row_i], x_cat[col_j].
    pub fn with_matrix<Sx, Sy, F, G>(
        mut self,
        x_cats: impl IntoIterator<Item = Sx>,
        y_cats: impl IntoIterator<Item = Sy>,
        sizes: Vec<Vec<F>>,
        colors: Vec<Vec<G>>,
    ) -> Self
    where
        Sx: Into<String>,
        Sy: Into<String>,
        F: Into<f64>,
        G: Into<f64>,
    {
        let x_cats: Vec<String> = x_cats.into_iter().map(|s| s.into()).collect();
        let y_cats: Vec<String> = y_cats.into_iter().map(|s| s.into()).collect();

        self.x_categories = x_cats.clone();
        self.y_categories = y_cats.clone();

        for (y_cat, (size_row, color_row)) in y_cats.iter()
            .zip(sizes.into_iter().zip(colors.into_iter()))
        {
            for (col_j, (size, color)) in size_row.into_iter()
                .zip(color_row.into_iter())
                .enumerate()
            {
                if let Some(x_cat) = x_cats.get(col_j) {
                    self.points.push(DotPoint {
                        x_cat: x_cat.clone(),
                        y_cat: y_cat.clone(),
                        size: size.into(),
                        color: color.into(),
                    });
                }
            }
        }
        self
    }

    pub fn with_color_map(mut self, map: ColorMap) -> Self {
        self.color_map = map;
        self
    }

    pub fn with_max_radius(mut self, r: f64) -> Self {
        self.max_radius = r;
        self
    }

    pub fn with_min_radius(mut self, r: f64) -> Self {
        self.min_radius = r;
        self
    }

    pub fn with_size_range(mut self, min: f64, max: f64) -> Self {
        self.size_range = Some((min, max));
        self
    }

    pub fn with_color_range(mut self, min: f64, max: f64) -> Self {
        self.color_range = Some((min, max));
        self
    }

    pub fn with_size_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.size_label = Some(label.into());
        self
    }

    pub fn with_colorbar<S: Into<String>>(mut self, label: S) -> Self {
        self.color_legend_label = Some(label.into());
        self
    }

    /// Returns (min, max) of size values across all points.
    pub fn size_extent(&self) -> (f64, f64) {
        if self.points.is_empty() {
            return (0.0, 1.0);
        }
        let min = self.points.iter().map(|p| p.size).fold(f64::INFINITY, f64::min);
        let max = self.points.iter().map(|p| p.size).fold(f64::NEG_INFINITY, f64::max);
        (min, max)
    }

    /// Returns (min, max) of color values across all points.
    pub fn color_extent(&self) -> (f64, f64) {
        if self.points.is_empty() {
            return (0.0, 1.0);
        }
        let min = self.points.iter().map(|p| p.color).fold(f64::INFINITY, f64::min);
        let max = self.points.iter().map(|p| p.color).fold(f64::NEG_INFINITY, f64::max);
        (min, max)
    }
}
