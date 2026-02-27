use crate::plot::strip::StripStyle;

/// Builder for a violin plot.
///
/// Estimates the probability density of each group using kernel density
/// estimation (KDE) and renders the result as a symmetric shape — wider
/// where data is dense, narrower where it is sparse. Unlike box plots,
/// violins reveal multi-modal and skewed distributions.
///
/// Bandwidth defaults to Silverman's rule-of-thumb. Individual data
/// points can be overlaid as a jittered strip or beeswarm.
///
/// # Example
///
/// ```rust,no_run
/// use visus::plot::ViolinPlot;
/// use visus::backend::svg::SvgBackend;
/// use visus::render::render::render_multiple;
/// use visus::render::layout::Layout;
/// use visus::render::plots::Plot;
///
/// let plot = ViolinPlot::new()
///     .with_group("Control", vec![4.1, 5.0, 5.3, 5.8, 6.2, 7.0, 5.5, 4.8])
///     .with_group("Treated", vec![5.5, 6.1, 6.4, 7.2, 7.8, 8.5, 6.9, 7.0])
///     .with_color("steelblue")
///     .with_width(30.0);
///
/// let plots = vec![Plot::Violin(plot)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Violin Plot")
///     .with_y_label("Value");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("violin.svg", svg).unwrap();
/// ```
pub struct ViolinPlot {
    pub groups: Vec<ViolinGroup>,
    pub color: String,
    /// Half-width of each violin in pixels (default `30.0`).
    pub width: f64,
    pub legend_label: Option<String>,
    /// KDE bandwidth. `None` uses Silverman's rule-of-thumb.
    pub bandwidth: Option<f64>,
    /// Number of KDE evaluation points (default `200`).
    pub kde_samples: usize,
    pub overlay: Option<StripStyle>,
    pub overlay_color: String,
    pub overlay_size: f64,
    pub overlay_seed: u64,
}

/// A single group (one violin) with a category label and raw values.
pub struct ViolinGroup {
    pub label: String,
    pub values: Vec<f64>,
}

impl Default for ViolinPlot {
    fn default() -> Self { Self::new() }
}

impl ViolinPlot {
    /// Create a violin plot with default settings.
    ///
    /// Defaults: color `"black"`, width `30.0` px, Silverman bandwidth,
    /// 200 KDE evaluation points, no overlay.
    pub fn new() -> Self {
        Self {
            groups: vec![],
            color: "black".into(),
            width: 30.0,
            legend_label: None,
            bandwidth: None,
            kde_samples: 200,
            overlay: None,
            overlay_color: "rgba(0,0,0,0.45)".into(),
            overlay_size: 3.0,
            overlay_seed: 42,
        }
    }

    /// Add a group (one violin) with a label and raw values.
    ///
    /// Groups are rendered left-to-right in the order they are added.
    /// More data points produce a smoother, more accurate density estimate.
    ///
    /// ```rust,no_run
    /// # use visus::plot::ViolinPlot;
    /// let plot = ViolinPlot::new()
    ///     .with_group("A", vec![1.0, 2.5, 3.0, 3.5, 4.0, 5.0])
    ///     .with_group("B", vec![2.0, 3.0, 3.8, 4.2, 4.8, 6.0]);
    /// ```
    pub fn with_group<T, U, I>(mut self, label: T, values: I) -> Self
    where
        T: Into<String>,
        I: IntoIterator<Item = U>,
        U: Into<f64>,
    {
        self.groups.push(ViolinGroup {
            label: label.into(),
            values: values.into_iter().map(|x| x.into()).collect(),
        });
        self
    }

    /// Set the violin fill color (CSS color string, e.g. `"steelblue"`).
    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    /// Set the maximum half-width of each violin in pixels (default `30.0`).
    ///
    /// The widest point of the violin is scaled to this value. Increase
    /// it to make violins more prominent, decrease it for a narrower look.
    /// Note this is in pixel units, unlike bar-width which is a fractional slot.
    pub fn with_width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// Attach a legend label to this violin plot.
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    /// Set the KDE bandwidth manually.
    ///
    /// Bandwidth controls the smoothness of the density estimate. Smaller
    /// values reveal finer structure (but may be noisy); larger values
    /// produce a smoother shape (but may hide modes). When not set,
    /// Silverman's rule-of-thumb is applied automatically — a good
    /// starting point for unimodal, roughly normal data.
    ///
    /// ```rust,no_run
    /// # use visus::plot::ViolinPlot;
    /// let plot = ViolinPlot::new()
    ///     .with_group("A", vec![1.0, 2.0, 3.0, 4.0, 5.0])
    ///     .with_bandwidth(0.5);  // tighter than the default
    /// ```
    pub fn with_bandwidth(mut self, h: f64) -> Self {
        self.bandwidth = Some(h);
        self
    }

    /// Set the number of points at which the KDE is evaluated (default `200`).
    ///
    /// Higher values produce a smoother curve at the cost of slightly more
    /// computation. The default is adequate for most use cases.
    pub fn with_kde_samples(mut self, n: usize) -> Self {
        self.kde_samples = n;
        self
    }

    /// Overlay individual data points as a jittered strip.
    ///
    /// `jitter` controls the horizontal spread (in data-axis units).
    /// A value of `0.15`–`0.2` is typical. Use a semi-transparent
    /// [`with_overlay_color`](Self::with_overlay_color) so the violin
    /// shape remains visible underneath.
    pub fn with_strip(mut self, jitter: f64) -> Self {
        self.overlay = Some(StripStyle::Strip { jitter });
        self
    }

    /// Overlay individual data points as a beeswarm.
    ///
    /// Points are spread horizontally to avoid overlap, giving a clearer
    /// picture of density than a jittered strip. Works best with smaller
    /// datasets (roughly N < 200 per group).
    pub fn with_swarm_overlay(mut self) -> Self {
        self.overlay = Some(StripStyle::Swarm);
        self
    }

    /// Set the fill color for overlay points (default `"rgba(0,0,0,0.45)"`).
    ///
    /// A semi-transparent color is strongly recommended so the KDE shape
    /// behind the points remains legible.
    pub fn with_overlay_color<S: Into<String>>(mut self, color: S) -> Self {
        self.overlay_color = color.into();
        self
    }

    /// Set the radius of overlay points in pixels (default `3.0`).
    pub fn with_overlay_size(mut self, size: f64) -> Self {
        self.overlay_size = size;
        self
    }
}
