use crate::plot::heatmap::ColorMap;

/// Builder for a contour plot.
///
/// A contour plot draws iso-lines (or filled iso-bands) on a 2D scalar field,
/// connecting all points that share the same z value. It is well suited for
/// visualising any continuous surface: density functions, spatial expression
/// gradients, topographic elevation, or any field that varies over an x–y plane.
///
/// # Input modes
///
/// Two input modes are available:
///
/// - **Regular grid** ([`with_grid`](Self::with_grid)): provide `z[row][col]`
///   plus explicit x and y coordinate vectors. Use this when your data is
///   already gridded (e.g. from a simulation or a rasterised image).
/// - **Scattered points** ([`with_points`](Self::with_points)): provide an
///   iterator of `(x, y, z)` triples at arbitrary positions. The values are
///   interpolated onto an internal 50×50 grid using **inverse-distance
///   weighting (IDW)** before iso-lines are computed. Use this for spatial
///   data that does not come pre-gridded.
///
/// # Iso-levels
///
/// By default, `n_levels` evenly spaced iso-levels are chosen automatically
/// from the data range (default `n_levels = 8`). Supply explicit values with
/// [`with_levels`](Self::with_levels) when iso-lines should correspond to
/// specific thresholds.
///
/// # Line vs. filled mode
///
/// - **Line mode** (default): iso-lines are drawn using the active colormap or
///   a fixed color set with [`with_line_color`](Self::with_line_color).
/// - **Filled mode** ([`with_filled`](Self::with_filled)): the region between
///   each pair of adjacent iso-levels is filled with the corresponding colormap
///   color. Calling [`with_legend`](Self::with_legend) in filled mode also
///   renders a colorbar in the right margin.
///
/// # Example
///
/// ```rust,no_run
/// use visus::plot::{ContourPlot, ColorMap};
/// use visus::backend::svg::SvgBackend;
/// use visus::render::render::render_multiple;
/// use visus::render::layout::Layout;
/// use visus::render::plots::Plot;
///
/// // Build a 40×40 Gaussian grid
/// let n = 40_usize;
/// let coords: Vec<f64> = (0..n)
///     .map(|i| -3.0 + i as f64 / (n - 1) as f64 * 6.0)
///     .collect();
/// let z: Vec<Vec<f64>> = coords.iter()
///     .map(|&y| coords.iter()
///         .map(|&x| (-(x * x + y * y) / 2.0).exp())
///         .collect())
///     .collect();
///
/// let cp = ContourPlot::new()
///     .with_grid(z, coords.clone(), coords)
///     .with_n_levels(8)
///     .with_filled()
///     .with_legend("Density");
///
/// let plots = vec![Plot::Contour(cp)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Gaussian Density")
///     .with_x_label("x")
///     .with_y_label("y");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("contour.svg", svg).unwrap();
/// ```
#[derive(Clone)]
pub struct ContourPlot {
    /// The scalar field as a row-major grid: `z[row][col]` where `row`
    /// corresponds to `y_coords[row]` and `col` to `x_coords[col]`.
    pub z: Vec<Vec<f64>>,
    /// X coordinate for each column — must have length equal to `z[0].len()`.
    pub x_coords: Vec<f64>,
    /// Y coordinate for each row — must have length equal to `z.len()`.
    pub y_coords: Vec<f64>,
    /// Explicit iso-levels. When non-empty, overrides `n_levels`.
    pub levels: Vec<f64>,
    /// Number of auto-spaced iso-levels when `levels` is empty (default `8`).
    pub n_levels: usize,
    /// When `true`, the area between adjacent iso-levels is filled with the
    /// colormap color. When `false` (default) only iso-lines are drawn.
    pub filled: bool,
    /// Color map used to assign colors to iso-levels / filled bands.
    /// Default [`ColorMap::Viridis`].
    pub color_map: ColorMap,
    /// Iso-line stroke width in pixels (default `1.0`).
    pub line_width: f64,
    /// Fixed color for all iso-lines. `None` = derive from `color_map`.
    pub line_color: Option<String>,
    /// Legend label. In filled mode this triggers a colorbar; in line mode
    /// it adds a line entry to the legend box.
    pub legend_label: Option<String>,
}

impl Default for ContourPlot {
    fn default() -> Self { Self::new() }
}

impl ContourPlot {
    /// Create a contour plot with default settings.
    ///
    /// Defaults: 8 auto-spaced iso-levels, Viridis colormap, line mode,
    /// stroke width 1.0, no fixed line color, no legend.
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

    /// Supply data as a pre-computed regular grid.
    ///
    /// `z[row][col]` is the scalar value at position (`x_coords[col]`,
    /// `y_coords[row]`). The coordinate vectors define the physical extents
    /// of the grid and set the x/y axis ranges.
    ///
    /// ```rust,no_run
    /// # use visus::plot::ContourPlot;
    /// // 5×5 Gaussian grid
    /// let coords: Vec<f64> = (-2..=2).map(|i| i as f64).collect();
    /// let z: Vec<Vec<f64>> = coords.iter()
    ///     .map(|&y| coords.iter()
    ///         .map(|&x| (-(x * x + y * y) / 2.0).exp())
    ///         .collect())
    ///     .collect();
    ///
    /// let cp = ContourPlot::new()
    ///     .with_grid(z, coords.clone(), coords);
    /// ```
    pub fn with_grid(mut self, z: Vec<Vec<f64>>, x_coords: Vec<f64>, y_coords: Vec<f64>) -> Self {
        self.z = z;
        self.x_coords = x_coords;
        self.y_coords = y_coords;
        self
    }

    /// Supply data as scattered `(x, y, z)` points; IDW interpolates to a grid.
    ///
    /// The input points can be at any positions — they do not need to be on a
    /// regular grid. An inverse-distance weighting (IDW) algorithm interpolates
    /// them onto an internal 50×50 grid before iso-line extraction.
    ///
    /// The grid bounds are set to the bounding box of the input points.
    /// Denser point clouds produce more accurate interpolations.
    ///
    /// ```rust,no_run
    /// # use visus::plot::ContourPlot;
    /// // 11×11 grid of sample points from a cone function
    /// let pts: Vec<(f64, f64, f64)> = (-5..=5)
    ///     .flat_map(|i| (-5..=5).map(move |j| {
    ///         let (x, y) = (i as f64, j as f64);
    ///         let z = 1.0 - (x * x + y * y).sqrt() / 7.0;
    ///         (x, y, z)
    ///     }))
    ///     .collect();
    ///
    /// let cp = ContourPlot::new()
    ///     .with_points(pts)
    ///     .with_n_levels(6);
    /// ```
    #[allow(clippy::needless_range_loop)]
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

    /// Set explicit iso-level values.
    ///
    /// When set, these values override [`with_n_levels`](Self::with_n_levels).
    /// Use this when iso-lines should correspond to specific thresholds — for
    /// example meaningful expression cutoffs, topographic contour intervals,
    /// or specific probability levels.
    ///
    /// ```rust,no_run
    /// # use visus::plot::ContourPlot;
    /// # let (z, xs, ys) = (vec![vec![0.0_f64]], vec![0.0_f64], vec![0.0_f64]);
    /// // Gaussian peaks at z ∈ [0,1]; draw iso-lines at specific fractions
    /// let cp = ContourPlot::new()
    ///     .with_grid(z, xs, ys)
    ///     .with_levels(&[0.1, 0.25, 0.5, 0.75, 0.9]);
    /// ```
    pub fn with_levels(mut self, levels: &[f64]) -> Self {
        self.levels = levels.to_vec();
        self
    }

    /// Set the number of auto-spaced iso-levels (default `8`).
    ///
    /// Levels are distributed evenly within the z data range, excluding the
    /// minimum and maximum. Ignored when explicit levels are set via
    /// [`with_levels`](Self::with_levels).
    pub fn with_n_levels(mut self, n: usize) -> Self {
        self.n_levels = n;
        self
    }

    /// Enable filled mode: fill the area between adjacent iso-levels.
    ///
    /// Each band is colored according to its z value using the active
    /// colormap. Calling [`with_legend`](Self::with_legend) in filled mode
    /// also renders a colorbar in the right margin.
    ///
    /// ```rust,no_run
    /// # use visus::plot::ContourPlot;
    /// # let (z, xs, ys) = (vec![vec![0.0_f64]], vec![0.0_f64], vec![0.0_f64]);
    /// let cp = ContourPlot::new()
    ///     .with_grid(z, xs, ys)
    ///     .with_filled()
    ///     .with_legend("Density");
    /// ```
    pub fn with_filled(mut self) -> Self {
        self.filled = true;
        self
    }

    /// Set the color map used for iso-line or filled-band colors
    /// (default [`ColorMap::Viridis`]).
    ///
    /// The same [`ColorMap`] variants available for heatmaps apply:
    /// `Viridis`, `Inferno`, `Grayscale`, and `Custom`.
    pub fn with_colormap(mut self, map: ColorMap) -> Self {
        self.color_map = map;
        self
    }

    /// Set a fixed color for all iso-lines.
    ///
    /// When set, all iso-lines are drawn in this color instead of being
    /// colored by the colormap. Accepts any CSS color string. Has no effect
    /// on filled bands — band colors always derive from the colormap.
    ///
    /// ```rust,no_run
    /// # use visus::plot::ContourPlot;
    /// # let (z, xs, ys) = (vec![vec![0.0_f64]], vec![0.0_f64], vec![0.0_f64]);
    /// let cp = ContourPlot::new()
    ///     .with_grid(z, xs, ys)
    ///     .with_line_color("steelblue");   // all iso-lines in steelblue
    /// ```
    pub fn with_line_color<S: Into<String>>(mut self, color: S) -> Self {
        self.line_color = Some(color.into());
        self
    }

    /// Set the iso-line stroke width in pixels (default `1.0`).
    pub fn with_line_width(mut self, w: f64) -> Self {
        self.line_width = w;
        self
    }

    /// Set a legend label.
    ///
    /// In **filled** mode (`with_filled`) this triggers a colorbar in the
    /// right margin using the label as the title.
    ///
    /// In **line** mode this adds a line entry to the in-plot legend box,
    /// useful when a contour plot is composed with other plots (e.g. a
    /// scatter plot overlaid on top of a contour background).
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

    /// Returns `(min, max)` of all z values in the grid.
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
