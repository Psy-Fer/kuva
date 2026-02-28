use crate::plot::heatmap::ColorMap;

/// A single point in a dot plot grid.
pub struct DotPoint {
    pub x_cat: String,
    pub y_cat: String,
    /// Raw value encoded as circle radius.
    pub size: f64,
    /// Raw value encoded as fill color.
    pub color: f64,
}

/// Builder for a dot plot (bubble matrix).
///
/// A dot plot places circles at the intersections of two categorical axes.
/// Each circle encodes two independent continuous variables simultaneously:
/// **size** (radius) and **color**. This makes it well suited for compact
/// display of multi-variable summaries across a grid — the canonical
/// bioinformatics use case is gene expression across cell types, where size
/// shows the fraction of cells expressing the gene and color shows the mean
/// expression level.
///
/// # Data input
///
/// Two modes are supported:
///
/// - **Sparse tuples** — [`with_data`](Self::with_data): pass an iterator of
///   `(x_cat, y_cat, size, color)` tuples. Missing grid positions are simply
///   absent (no circle drawn). Category order follows first-seen insertion order.
///
/// - **Dense matrix** — [`with_matrix`](Self::with_matrix): pass explicit
///   category lists and `sizes[row_i][col_j]` / `colors[row_i][col_j]`
///   matrices. Every grid cell is filled.
///
/// # Legends
///
/// Both legends are optional and independent:
///
/// - [`with_size_legend`](Self::with_size_legend) — adds a size key in the right margin
///   showing representative radii.
/// - [`with_colorbar`](Self::with_colorbar) — adds a colorbar showing the color scale.
///
/// When both are present they are stacked in a single right-margin column.
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::DotPlot;
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let data = vec![
///     ("CD4 T", "CD3E", 88.0_f64, 3.8_f64),
///     ("CD8 T", "CD3E", 91.0,     4.0    ),
///     ("CD4 T", "CD4",  85.0,     3.5    ),
///     ("CD8 T", "CD4",   8.0,     0.3    ),
/// ];
///
/// let dot = DotPlot::new()
///     .with_data(data)
///     .with_size_legend("% Expressing")
///     .with_colorbar("Mean expression");
///
/// let plots = vec![Plot::DotPlot(dot)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Gene Expression")
///     .with_x_label("Cell type")
///     .with_y_label("Gene");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("dotplot.svg", svg).unwrap();
/// ```
pub struct DotPlot {
    pub points:             Vec<DotPoint>,
    /// X-axis category order (insertion order for `with_data`; explicit for `with_matrix`).
    pub x_categories:       Vec<String>,
    /// Y-axis category order (insertion order; rendered top → bottom).
    pub y_categories:       Vec<String>,
    /// Color map applied to the `color` field after normalisation. Default `Viridis`.
    pub color_map:          ColorMap,
    /// Maximum circle radius in pixels (default `12.0`).
    pub max_radius:         f64,
    /// Minimum circle radius in pixels (default `1.0`).
    pub min_radius:         f64,
    /// Clamp the size encoding to this range before normalising. `None` = auto (data extent).
    pub size_range:         Option<(f64, f64)>,
    /// Clamp the color encoding to this range before normalising. `None` = auto (data extent).
    pub color_range:        Option<(f64, f64)>,
    /// When `Some`, a size legend is drawn in the right margin using this label.
    pub size_label:         Option<String>,
    /// When `Some`, a colorbar is drawn in the right margin using this label.
    pub color_legend_label: Option<String>,
}

impl Default for DotPlot {
    fn default() -> Self { Self::new() }
}

impl DotPlot {
    /// Create a dot plot with default settings.
    ///
    /// Defaults: Viridis color map, `max_radius = 12.0`, `min_radius = 1.0`,
    /// auto size and color ranges, no legends.
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

    /// Add data as an iterator of sparse `(x_cat, y_cat, size, color)` tuples.
    ///
    /// Category order on each axis follows first-seen insertion order.
    /// Grid positions with no tuple are left empty — no circle is drawn.
    /// This mode is natural for data that already comes as a list of records.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::DotPlot;
    /// let dot = DotPlot::new().with_data(vec![
    ///     ("CD4 T", "CD3E", 88.0_f64, 3.8_f64),
    ///     ("CD8 T", "CD3E", 91.0,     4.0    ),
    ///     // ("NK", "CD3E") absent — no circle drawn at that position
    /// ]);
    /// ```
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

    /// Add data as explicit category lists and dense `sizes` / `colors` matrices.
    ///
    /// `sizes[row_i][col_j]` corresponds to `y_cats[row_i]` and `x_cats[col_j]`.
    /// Every grid cell is filled. Use this mode when data comes from a matrix or
    /// 2-D array (e.g. output of a differential expression tool).
    ///
    /// ```rust,no_run
    /// # use kuva::plot::DotPlot;
    /// let dot = DotPlot::new().with_matrix(
    ///     vec!["TypeA", "TypeB"],          // x categories
    ///     vec!["Gene1", "Gene2"],          // y categories
    ///     vec![vec![80.0, 25.0],           // sizes[row_i][col_j]
    ///          vec![15.0, 90.0]],
    ///     vec![vec![3.5,  1.2],            // colors[row_i][col_j]
    ///          vec![0.8,  4.1]],
    /// );
    /// ```
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

    /// Set the color map for the color encoding (default `ColorMap::Viridis`).
    ///
    /// See [`ColorMap`] for available options including
    /// `Viridis`, `Inferno`, `Grayscale`, and `Custom`.
    pub fn with_color_map(mut self, map: ColorMap) -> Self {
        self.color_map = map;
        self
    }

    /// Set the maximum circle radius in pixels (default `12.0`).
    ///
    /// The largest `size` value in the data (or `size_range.1`) maps to this radius.
    pub fn with_max_radius(mut self, r: f64) -> Self {
        self.max_radius = r;
        self
    }

    /// Set the minimum circle radius in pixels (default `1.0`).
    ///
    /// The smallest `size` value in the data (or `size_range.0`) maps to this radius.
    pub fn with_min_radius(mut self, r: f64) -> Self {
        self.min_radius = r;
        self
    }

    /// Clamp the size encoding to an explicit `[min, max]` range before normalising.
    ///
    /// Values below `min` map to `min_radius`; values above `max` map to `max_radius`.
    /// Useful when the data has outliers or when comparing across charts that must
    /// use a consistent scale (e.g. always map `0–100` % to the radius range).
    ///
    /// ```rust,no_run
    /// # use kuva::plot::DotPlot;
    /// let dot = DotPlot::new()
    ///     .with_data(vec![("A", "G", 120.0_f64, 1.0_f64)])  // 120 will be clamped
    ///     .with_size_range(0.0, 100.0);
    /// ```
    pub fn with_size_range(mut self, min: f64, max: f64) -> Self {
        self.size_range = Some((min, max));
        self
    }

    /// Clamp the color encoding to an explicit `[min, max]` range before normalising.
    ///
    /// Values outside the range are clamped before the color map is applied.
    /// Useful when comparing across charts that must share the same color scale.
    pub fn with_color_range(mut self, min: f64, max: f64) -> Self {
        self.color_range = Some((min, max));
        self
    }

    /// Enable a size legend in the right margin with the given variable name.
    ///
    /// The legend shows representative circle sizes with their corresponding
    /// values. When combined with [`with_colorbar`](Self::with_colorbar),
    /// the two are stacked in a single right-margin column.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::DotPlot;
    /// let dot = DotPlot::new()
    ///     .with_data(vec![("A", "G", 75.0_f64, 2.5_f64)])
    ///     .with_size_legend("% Expressing");
    /// ```
    pub fn with_size_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.size_label = Some(label.into());
        self
    }

    /// Enable a colorbar in the right margin with the given label.
    ///
    /// The colorbar maps the data range (or [`with_color_range`](Self::with_color_range)
    /// bounds) to the active color map. When combined with
    /// [`with_size_legend`](Self::with_size_legend), the two are stacked in a
    /// single right-margin column.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::DotPlot;
    /// let dot = DotPlot::new()
    ///     .with_data(vec![("A", "G", 75.0_f64, 2.5_f64)])
    ///     .with_colorbar("Mean expression");
    /// ```
    pub fn with_colorbar<S: Into<String>>(mut self, label: S) -> Self {
        self.color_legend_label = Some(label.into());
        self
    }

    /// Returns `(min, max)` of size values across all points.
    pub fn size_extent(&self) -> (f64, f64) {
        if self.points.is_empty() {
            return (0.0, 1.0);
        }
        let min = self.points.iter().map(|p| p.size).fold(f64::INFINITY, f64::min);
        let max = self.points.iter().map(|p| p.size).fold(f64::NEG_INFINITY, f64::max);
        (min, max)
    }

    /// Returns `(min, max)` of color values across all points.
    pub fn color_extent(&self) -> (f64, f64) {
        if self.points.is_empty() {
            return (0.0, 1.0);
        }
        let min = self.points.iter().map(|p| p.color).fold(f64::INFINITY, f64::min);
        let max = self.points.iter().map(|p| p.color).fold(f64::NEG_INFINITY, f64::max);
        (min, max)
    }
}
