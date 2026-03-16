/// Builder for a 1-D kernel density estimate curve.
///
/// Estimates the probability density of a numeric dataset via Gaussian KDE
/// and renders it as a smooth curve (optionally filled).
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::DensityPlot;
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let data = vec![1.1, 2.3, 2.7, 3.2, 3.8, 3.9, 4.0, 1.5, 2.1, 3.5];
///
/// let density = DensityPlot::new()
///     .with_data(data)
///     .with_color("steelblue")
///     .with_filled(true);
///
/// let plots = vec![Plot::Density(density)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Density")
///     .with_x_label("Value")
///     .with_y_label("Density");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("density.svg", svg).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct DensityPlot {
    pub data: Vec<f64>,
    pub color: String,
    pub filled: bool,
    pub opacity: f64,
    pub bandwidth: Option<f64>,
    pub kde_samples: usize,
    pub stroke_width: f64,
    pub legend_label: Option<String>,
    pub line_dash: Option<String>,
    /// Pre-smoothed (x, y) curve; bypasses KDE when set.
    pub precomputed: Option<(Vec<f64>, Vec<f64>)>,
    /// Clamp KDE evaluation to this x range. Useful for bounded data (e.g.
    /// methylation β-values or frequencies in [0, 1]) where the default
    /// behaviour of extending 3×bandwidth beyond the data extremes produces a
    /// curve that bleeds into physically impossible negative values.
    pub x_range: Option<(f64, f64)>,
}

impl Default for DensityPlot {
    fn default() -> Self { Self::new() }
}

impl DensityPlot {
    /// Create a density plot with default settings.
    ///
    /// Defaults: color `"steelblue"`, not filled, opacity `0.2`,
    /// Silverman bandwidth, 200 KDE evaluation points, stroke width `1.5`.
    pub fn new() -> Self {
        Self {
            data: vec![],
            color: "steelblue".to_string(),
            filled: false,
            opacity: 0.2,
            bandwidth: None,
            kde_samples: 200,
            stroke_width: 1.5,
            legend_label: None,
            line_dash: None,
            precomputed: None,
            x_range: None,
        }
    }

    /// Create a density plot from a pre-computed (x, y) curve, bypassing KDE.
    ///
    /// Use this when you already have a smoothed curve from another source
    /// (e.g. Python's `scipy.stats.gaussian_kde`).
    ///
    /// ```rust,no_run
    /// # use kuva::plot::DensityPlot;
    /// let x = vec![0.0, 1.0, 2.0, 3.0, 4.0];
    /// let y = vec![0.1, 0.3, 0.5, 0.3, 0.1];
    /// let density = DensityPlot::from_curve(x, y).with_color("coral");
    /// ```
    pub fn from_curve(x: Vec<f64>, y: Vec<f64>) -> Self {
        Self {
            precomputed: Some((x, y)),
            ..Self::new()
        }
    }

    /// Set the input data values.
    ///
    /// Accepts any iterator of values implementing `Into<f64>`.
    pub fn with_data<T, I>(mut self, data: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<f64>,
    {
        self.data = data.into_iter().map(|x| x.into()).collect();
        self
    }

    /// Set the curve color (CSS color string, e.g. `"steelblue"`, `"#4682b4"`).
    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    /// Fill the area under the density curve (default `false`).
    pub fn with_filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }

    /// Set the fill opacity when `filled = true` (default `0.2`).
    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }

    /// Set the KDE bandwidth.
    ///
    /// When not set, Silverman's rule-of-thumb is used automatically.
    pub fn with_bandwidth(mut self, bandwidth: f64) -> Self {
        self.bandwidth = Some(bandwidth);
        self
    }

    /// Set the number of evaluation points for the KDE (default `200`).
    ///
    /// Higher values give a smoother curve but are slower to compute.
    pub fn with_kde_samples(mut self, samples: usize) -> Self {
        self.kde_samples = samples;
        self
    }

    /// Set the outline stroke width (default `1.5`).
    pub fn with_stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    /// Attach a legend label to this density curve.
    ///
    /// A legend is rendered automatically when at least one plot in the
    /// `Vec<Plot>` has a label.
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    /// Set a SVG stroke-dasharray for a dashed or dotted line (e.g. `"4 2"`).
    ///
    /// Pass `None` (the default) for a solid line.
    pub fn with_line_dash<S: Into<String>>(mut self, dash: S) -> Self {
        self.line_dash = Some(dash.into());
        self
    }

    /// Clamp the KDE evaluation range to `[lo, hi]`.
    ///
    /// By default the KDE is evaluated from `data_min - 3×bandwidth` to
    /// `data_max + 3×bandwidth` so the Gaussian tails taper smoothly. For
    /// data that is physically bounded (e.g. methylation β-values or
    /// frequencies in `[0, 1]`) this produces a curve that extends into
    /// impossible negative values. Setting `with_x_range(0.0, 1.0)` prevents
    /// that and gives a cleaner result.
    pub fn with_x_range(mut self, lo: f64, hi: f64) -> Self {
        self.x_range = Some((lo, hi));
        self
    }
}
