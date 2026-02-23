/// Builder for a histogram.
///
/// Bins a 1-D dataset and renders each bin as a vertical bar. The bin
/// boundaries are computed from the data range (or an explicit range)
/// and the requested bin count.
///
/// # Example
///
/// ```rust,no_run
/// use visus::plot::Histogram;
/// use visus::backend::svg::SvgBackend;
/// use visus::render::render::render_multiple;
/// use visus::render::layout::Layout;
/// use visus::render::plots::Plot;
///
/// let data = vec![1.1, 2.3, 2.7, 3.2, 3.8, 3.9, 4.0, 1.5, 2.1, 3.5];
///
/// let hist = Histogram::new()
///     .with_data(data)
///     .with_bins(10)
///     .with_color("steelblue");
///
/// let plots = vec![Plot::Histogram(hist)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Histogram")
///     .with_x_label("Value")
///     .with_y_label("Count");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("histogram.svg", svg).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Histogram {
    pub data: Vec<f64>,
    pub bins: usize,
    pub range: Option<(f64, f64)>,
    pub color: String,
    pub normalize: bool,
    pub legend_label: Option<String>,
}

impl Histogram {
    /// Create a histogram with default settings.
    ///
    /// Defaults: 10 bins, color `"black"`, no normalization.
    pub fn new() -> Self {
        Self {
            data: vec![],
            bins: 10,
            range: None,
            color: "black".to_string(),
            normalize: false,
            legend_label: None,
        }
    }

    /// Set the input data.
    ///
    /// Accepts any iterator of values implementing `Into<f64>`. Values
    /// outside the active range (data range or explicit range) are
    /// silently ignored.
    ///
    /// ```rust,no_run
    /// # use visus::plot::Histogram;
    /// // integer input
    /// let hist = Histogram::new()
    ///     .with_data(vec![1_i32, 2, 2, 3, 3, 3, 4, 4, 5]);
    /// ```
    pub fn with_data<T, I>(mut self, data: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<f64>,
    {
        self.data = data.into_iter().map(|x| x.into()).collect();
        self
    }

    /// Set the number of equal-width bins (default `10`).
    ///
    /// The bin edges span from `range.min` to `range.max`. Choose a
    /// value that balances resolution against noise for your sample size.
    pub fn with_bins(mut self, bins: usize) -> Self {
        self.bins = bins;
        self
    }

    /// Set an explicit data range instead of computing it from the data.
    ///
    /// Useful when comparing multiple histograms that must share the same
    /// x-axis scale, or when the data contains outliers that would
    /// otherwise warp the bins.
    ///
    /// ```rust,no_run
    /// # use visus::plot::Histogram;
    /// let hist = Histogram::new()
    ///     .with_data(vec![0.1, 0.5, 1.2, 2.8, 3.0])
    ///     .with_range((0.0, 4.0));  // force bins to cover 0–4
    /// ```
    pub fn with_range(mut self, range: (f64, f64)) -> Self {
        self.range = Some(range);
        self
    }

    /// Set the bar fill color (CSS color string, e.g. `"steelblue"`, `"#4682b4"`).
    ///
    /// For overlapping histograms, use an 8-digit hex color with an alpha
    /// channel (`#RRGGBBAA`) so bars from different series show through:
    ///
    /// ```rust,no_run
    /// # use visus::plot::Histogram;
    /// let hist = Histogram::new()
    ///     .with_data(vec![1.0, 2.0, 3.0])
    ///     .with_color("#4682b480");  // steelblue at 50% opacity
    /// ```
    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    /// Normalize bar heights so the tallest bar equals `1.0`.
    ///
    /// This is a peak-normalization — not a probability density. The
    /// y-axis represents relative frequency (tallest bin = 1), not
    /// counts or probability per unit width.
    pub fn with_normalize(mut self) -> Self {
        self.normalize = true;
        self
    }

    /// Attach a legend label to this histogram.
    ///
    /// A legend is rendered automatically when at least one plot in the
    /// `Vec<Plot>` has a label.
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }
}
