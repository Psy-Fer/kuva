
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

/// Colormap applied to bin counts in a 2D histogram (or cell values in a heatmap).
///
/// The map function receives a normalized value in `[0.0, 1.0]` (where `1.0` is
/// the maximum count / value) and returns a CSS color string.
///
/// # Variants
///
/// | Variant | Description |
/// |---------|-------------|
/// | `Viridis` | Perceptually uniform, blue → green → yellow. Colorblind-safe. **(default)** |
/// | `Inferno` | Dark purple → orange → bright yellow. High contrast. |
/// | `Grayscale` | White (low) → black (high). Print-friendly. |
/// | `Custom(f)` | User-supplied function `f64 → String`. |
///
/// # Example
///
/// ```rust,no_run
/// use visus::plot::Histogram2D;
/// use visus::plot::histogram2d::ColorMap;
///
/// let hist = Histogram2D::new()
///     .with_data(vec![(5.0_f64, 5.0_f64)], (0.0, 10.0), (0.0, 10.0), 10, 10)
///     .with_color_map(ColorMap::Inferno);
/// ```
#[derive(Clone)]
pub enum ColorMap {
    /// White → dark (Greys colormap). Print-friendly.
    Grayscale,
    /// Blue → green → yellow (Viridis colormap). Perceptually uniform, colorblind-safe. **(default)**
    Viridis,
    /// Black → orange → yellow (Inferno colormap). High contrast for dense data.
    Inferno,
    /// User-supplied mapping function `f64 → CSS color string`.
    ///
    /// The function receives a normalized value in `[0.0, 1.0]`.
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use visus::plot::histogram2d::ColorMap;
    ///
    /// let cmap = ColorMap::Custom(Arc::new(|t: f64| {
    ///     let g = (t * 255.0) as u8;
    ///     format!("rgb(0,{g},128)")
    /// }));
    /// ```
    Custom(Arc<dyn Fn(f64) -> String + Send + Sync>),
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

/// Builder for a 2D histogram (density map).
///
/// A 2D histogram bins scatter points `(x, y)` into a rectangular grid and
/// colors each cell by its count. The colorbar (labeled **"Count"**) is added
/// to the right margin automatically.
///
/// # Data loading
///
/// Pass scatter points, explicit axis ranges, and bin counts to
/// [`with_data`](Self::with_data). Points outside the specified ranges are
/// silently discarded. The range should start at `0.0` — see the note below.
///
/// # Range convention
///
/// The x and y axis extents reported to the layout are `[0, bins_x]` and
/// `[0, bins_y]` (bin-index space). The render function then maps each bin's
/// physical coordinate using the same layout. To keep the two spaces aligned,
/// **start the range at `0.0`** and choose a `bins_*` count equal to or a
/// simple fraction of the range width. For example, range `(0.0, 30.0)` with
/// 30 bins gives one bin per unit; range `(0.0, 20.0)` with 25 bins works too.
///
/// # Correlation annotation
///
/// [`with_correlation()`](Self::with_correlation) overlays the Pearson r
/// coefficient in the top-right corner, computed from the raw scatter points.
///
/// # Example
///
/// ```rust,no_run
/// use visus::plot::Histogram2D;
/// use visus::plot::histogram2d::ColorMap;
/// use visus::backend::svg::SvgBackend;
/// use visus::render::render::render_multiple;
/// use visus::render::layout::Layout;
/// use visus::render::plots::Plot;
///
/// // (x, y) scatter points
/// let data: Vec<(f64, f64)> = vec![(5.0, 6.0), (14.0, 15.0), (15.0, 14.0)];
///
/// let hist = Histogram2D::new()
///     .with_data(data, (0.0, 20.0), (0.0, 20.0), 20, 20)
///     .with_color_map(ColorMap::Viridis)
///     .with_correlation();
///
/// let plots = vec![Plot::Histogram2d(hist)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("2D Histogram")
///     .with_x_label("X")
///     .with_y_label("Y");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("hist2d.svg", svg).unwrap();
/// ```
#[derive(Clone)]
pub struct Histogram2D {
    /// Raw scatter points used for correlation computation.
    pub data: Vec<(f64, f64)>,
    /// Pre-computed bin counts indexed as `bins[row][col]`.
    pub bins: Vec<Vec<usize>>,
    /// Physical x-axis range `(min, max)` used for binning.
    pub x_range: (f64, f64),
    /// Physical y-axis range `(min, max)` used for binning.
    pub y_range: (f64, f64),
    /// Number of bins along the x-axis. Default: `10`.
    pub bins_x: usize,
    /// Number of bins along the y-axis. Default: `10`.
    pub bins_y: usize,
    /// Colormap applied to normalized bin counts. Default: [`ColorMap::Viridis`].
    pub color_map: ColorMap,
    /// When `true`, the Pearson r coefficient is printed in the top-right corner.
    pub show_correlation: bool,
}

impl Histogram2D {
    /// Create a 2D histogram with default settings.
    ///
    /// Defaults: 10×10 bins, Viridis colormap, no correlation annotation.
    /// Call [`with_data`](Self::with_data) to load points.
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

    /// Load scatter points and bin them into a grid.
    ///
    /// - `data` — `(x, y)` pairs; any type implementing `Into<f64>`.
    /// - `x_range` / `y_range` — axis extents `(min, max)`. Points outside
    ///   these bounds are silently discarded. Start at `0.0` to keep bin-index
    ///   and layout coordinates aligned.
    /// - `bins_x` / `bins_y` — number of columns / rows in the grid.
    ///
    /// ```rust,no_run
    /// use visus::plot::Histogram2D;
    ///
    /// let data: Vec<(f64, f64)> = vec![(5.0, 8.0), (12.0, 3.0), (7.0, 15.0)];
    /// let hist = Histogram2D::new()
    ///     .with_data(data, (0.0, 20.0), (0.0, 20.0), 20, 20);
    /// ```
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

    /// Set the colormap for bin counts. Default: [`ColorMap::Viridis`].
    ///
    /// ```rust,no_run
    /// use visus::plot::Histogram2D;
    /// use visus::plot::histogram2d::ColorMap;
    ///
    /// let hist = Histogram2D::new()
    ///     .with_data(vec![(5.0_f64, 5.0_f64)], (0.0, 10.0), (0.0, 10.0), 10, 10)
    ///     .with_color_map(ColorMap::Inferno);
    /// ```
    pub fn with_color_map(mut self, map: ColorMap) -> Self {
        self.color_map = map;
        self
    }

    /// Overlay the Pearson correlation coefficient in the top-right corner.
    ///
    /// The coefficient is computed from all points passed to
    /// [`with_data`](Self::with_data), including those clipped outside the
    /// plot range. Displayed as `r = 0.85`.
    ///
    /// ```rust,no_run
    /// use visus::plot::Histogram2D;
    ///
    /// let hist = Histogram2D::new()
    ///     .with_data(vec![(5.0_f64, 6.0_f64)], (0.0, 10.0), (0.0, 10.0), 10, 10)
    ///     .with_correlation();
    /// ```
    pub fn with_correlation(mut self) -> Self {
        self.show_correlation = true;
        self
    }
}
