/// Re-export the shared `ColorMap` type from the histogram2d module.
pub use crate::plot::histogram2d::ColorMap;

/// Aggregation function applied to a third variable `z` over points that fall
/// into the same hexagonal bin.
///
/// Used by [`HexbinPlot::with_z`] to choose how multiple `z` values in one bin
/// are reduced to a single color-mapped scalar.
#[derive(Debug, Clone, Default)]
pub enum ZReduce {
    /// Number of data points in the bin. **(default)**
    #[default]
    Count,
    /// Arithmetic mean of `z` over all points in the bin.
    Mean,
    /// Sum of `z` over all points in the bin.
    Sum,
    /// Median of `z` over all points in the bin.
    Median,
    /// Minimum `z` value in the bin.
    Min,
    /// Maximum `z` value in the bin.
    Max,
}

/// Builder for a hexbin (hexagonal-bin) density plot.
///
/// A hexbin plot divides a 2-D scatter into a regular hexagonal grid and
/// colors each cell by the number of points (or by an aggregated third
/// variable `z`) it contains.  Hexagonal bins produce a more visually uniform
/// density estimate than rectangular bins because every hex is equidistant
/// from its six neighbors.
///
/// # Basic usage
///
/// ```rust,no_run
/// use kuva::plot::hexbin::HexbinPlot;
/// use kuva::render::plots::Plot;
/// use kuva::render::layout::Layout;
/// use kuva::render::render::render_multiple;
/// use kuva::backend::svg::SvgBackend;
///
/// let x: Vec<f64> = (0..200).map(|i| (i as f64 / 10.0).sin()).collect();
/// let y: Vec<f64> = (0..200).map(|i| (i as f64 / 10.0).cos()).collect();
///
/// let plot = HexbinPlot::new()
///     .with_data(x, y)
///     .with_n_bins(25);
///
/// let plots = vec![Plot::Hexbin(plot)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Hexbin Plot")
///     .with_x_label("X")
///     .with_y_label("Y");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("hexbin.svg", svg).unwrap();
/// ```
pub struct HexbinPlot {
    /// X coordinates of the scatter points.
    pub x: Vec<f64>,
    /// Y coordinates of the scatter points.
    pub y: Vec<f64>,
    /// Optional third variable for aggregation-based coloring.
    pub z: Option<Vec<f64>>,
    /// Target number of hexagonal bins across the x-axis. Default: `20`.
    pub n_bins: usize,
    /// Explicit circumradius of each hex in pixels.  Overrides `n_bins` when set.
    pub bin_size: Option<f64>,
    /// Colormap applied to bin values. Default: [`ColorMap::Viridis`].
    pub color_map: ColorMap,
    /// Aggregation function for the `z` variable. Default: [`ZReduce::Count`].
    pub z_reduce: ZReduce,
    /// Apply log₁₀ scaling to bin values before color mapping. Default: `false`.
    pub log_color: bool,
    /// Minimum number of points required to render a bin. Default: `1`.
    pub min_count: usize,
    /// Divide counts by the total number of points (fractional density). Default: `false`.
    pub normalize: bool,
    /// Show a colorbar legend on the right margin. Default: `true`.
    pub show_colorbar: bool,
    /// Custom label for the colorbar.  Auto-derived from `z_reduce` when `None`.
    pub colorbar_label: Option<String>,
    /// Outline color for each hexagon.  `None` = no outline (default).
    pub stroke_color: Option<String>,
    /// Outline stroke width when `stroke_color` is set. Default: `0.5`.
    pub stroke_width: f64,
    /// `true` = flat-top orientation; `false` = pointy-top (default).
    pub flat_top: bool,
    /// Explicit data-space x extent for binning and axis limits.
    pub x_range: Option<(f64, f64)>,
    /// Explicit data-space y extent for binning and axis limits.
    pub y_range: Option<(f64, f64)>,
    /// Clamp the color scale to `(lo, hi)` instead of using the data range.
    pub color_range: Option<(f64, f64)>,
}

impl Default for HexbinPlot {
    fn default() -> Self { Self::new() }
}

impl HexbinPlot {
    /// Create a hexbin plot with default settings.
    ///
    /// Defaults: 20 bins, Viridis colormap, Count aggregation, pointy-top hexagons,
    /// colorbar shown, no stroke, no z variable.
    pub fn new() -> Self {
        Self {
            x: vec![],
            y: vec![],
            z: None,
            n_bins: 20,
            bin_size: None,
            color_map: ColorMap::Viridis,
            z_reduce: ZReduce::Count,
            log_color: false,
            min_count: 1,
            normalize: false,
            show_colorbar: true,
            colorbar_label: None,
            stroke_color: None,
            stroke_width: 0.5,
            flat_top: false,
            x_range: None,
            y_range: None,
            color_range: None,
        }
    }

    /// Load x and y scatter data.
    ///
    /// Accepts any iterable of values convertible to `f64`.
    ///
    /// ```rust,no_run
    /// use kuva::plot::hexbin::HexbinPlot;
    /// let plot = HexbinPlot::new().with_data(
    ///     vec![1.0_f64, 2.0, 3.0],
    ///     vec![4.0_f64, 5.0, 6.0],
    /// );
    /// ```
    pub fn with_data(
        mut self,
        x: impl IntoIterator<Item = impl Into<f64>>,
        y: impl IntoIterator<Item = impl Into<f64>>,
    ) -> Self {
        self.x = x.into_iter().map(Into::into).collect();
        self.y = y.into_iter().map(Into::into).collect();
        self
    }

    /// Attach a third variable `z` and choose the aggregation function.
    ///
    /// When set, bins are colored by the aggregated `z` value rather than by
    /// point count.
    ///
    /// ```rust,no_run
    /// use kuva::plot::hexbin::{HexbinPlot, ZReduce};
    /// let plot = HexbinPlot::new()
    ///     .with_data(vec![1.0_f64, 2.0], vec![3.0_f64, 4.0])
    ///     .with_z(vec![10.0_f64, 20.0], ZReduce::Mean);
    /// ```
    pub fn with_z(mut self, z: impl IntoIterator<Item = impl Into<f64>>, reduce: ZReduce) -> Self {
        self.z = Some(z.into_iter().map(Into::into).collect());
        self.z_reduce = reduce;
        self
    }

    /// Set the target number of hexagonal bins across the x-axis. Default: `20`.
    pub fn with_n_bins(mut self, n: usize) -> Self {
        self.n_bins = n;
        self
    }

    /// Override the hex circumradius in pixels instead of using `n_bins`.
    pub fn with_bin_size(mut self, s: f64) -> Self {
        self.bin_size = Some(s);
        self
    }

    /// Set the colormap. Default: [`ColorMap::Viridis`].
    pub fn with_color_map(mut self, m: ColorMap) -> Self {
        self.color_map = m;
        self
    }

    /// Apply log₁₀ scaling to bin values before color mapping.
    pub fn with_log_color(mut self, b: bool) -> Self {
        self.log_color = b;
        self
    }

    /// Set the minimum number of points required to render a bin. Default: `1`.
    pub fn with_min_count(mut self, n: usize) -> Self {
        self.min_count = n;
        self
    }

    /// Divide counts by the total number of points (fractional density).
    pub fn with_normalize(mut self, b: bool) -> Self {
        self.normalize = b;
        self
    }

    /// Show or hide the colorbar. Default: `true`.
    pub fn with_colorbar(mut self, b: bool) -> Self {
        self.show_colorbar = b;
        self
    }

    /// Set a custom colorbar label.  Auto-derived when not set.
    pub fn with_colorbar_label(mut self, s: impl Into<String>) -> Self {
        self.colorbar_label = Some(s.into());
        self
    }

    /// Add a hex outline stroke with the given CSS color.
    pub fn with_stroke(mut self, color: impl Into<String>) -> Self {
        self.stroke_color = Some(color.into());
        self
    }

    /// Set the hex outline stroke width. Default: `0.5`.
    pub fn with_stroke_width(mut self, w: f64) -> Self {
        self.stroke_width = w;
        self
    }

    /// Use flat-top hex orientation instead of pointy-top. Default: `false`.
    pub fn with_flat_top(mut self, b: bool) -> Self {
        self.flat_top = b;
        self
    }

    /// Clip data and fix the x-axis extent to `[lo, hi]`.
    pub fn with_x_range(mut self, lo: f64, hi: f64) -> Self {
        self.x_range = Some((lo, hi));
        self
    }

    /// Clip data and fix the y-axis extent to `[lo, hi]`.
    pub fn with_y_range(mut self, lo: f64, hi: f64) -> Self {
        self.y_range = Some((lo, hi));
        self
    }

    /// Clamp the colorbar scale to `(lo, hi)` instead of using the data range.
    pub fn with_color_range(mut self, lo: f64, hi: f64) -> Self {
        self.color_range = Some((lo, hi));
        self
    }
}
