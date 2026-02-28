/// Controls how points are spread horizontally within each group slot.
///
/// Used by both [`StripPlot`] (as the primary layout mode) and as the
/// `overlay` field in [`BoxPlot`](crate::plot::BoxPlot) and
/// [`ViolinPlot`](crate::plot::ViolinPlot).
pub enum StripStyle {
    /// Random horizontal jitter. `jitter` is the half-width as a fraction of
    /// the category slot width — `0.3` means points spread ±30 % of the slot.
    Strip { jitter: f64 },
    /// Deterministic beeswarm: points are placed as close to center as
    /// possible without overlapping. Best for N < ~200 per group.
    Swarm,
    /// No horizontal spread — all points placed at the group center.
    /// Creates a vertical density column.
    Center,
}

/// One group (one column of points) within a strip plot.
pub struct StripGroup {
    pub label: String,
    pub values: Vec<f64>,
}

/// Builder for a strip plot (also called a dot plot or univariate scatter).
///
/// Each group is rendered as a vertical cloud of points along a categorical
/// x-axis. Three layout modes are available:
///
/// | Method | Layout | Best for |
/// |--------|--------|----------|
/// | `.with_jitter(j)` | Random horizontal jitter | Large N; fast |
/// | `.with_swarm()` | Non-overlapping beeswarm | N < ~200; clearest structure |
/// | `.with_center()` | All at center | Density columns; stacked look |
///
/// Multiple `StripPlot`s can be layered on the same canvas (e.g. with a
/// [`BoxPlot`](crate::plot::BoxPlot)) by passing them together to
/// [`render_multiple`](crate::render::render::render_multiple). Use
/// `with_palette` on the [`Layout`](crate::render::layout::Layout) to
/// auto-assign distinct colors.
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::StripPlot;
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let strip = StripPlot::new()
///     .with_group("Control",   vec![4.1, 5.0, 5.3, 5.8, 6.2, 4.7, 5.5])
///     .with_group("Treatment", vec![5.5, 6.1, 6.4, 7.2, 7.8, 6.9, 7.0])
///     .with_color("steelblue")
///     .with_jitter(0.3)
///     .with_point_size(3.0);
///
/// let plots = vec![Plot::Strip(strip)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Control vs. Treatment")
///     .with_x_label("Group")
///     .with_y_label("Value");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("strip.svg", svg).unwrap();
/// ```
pub struct StripPlot {
    pub groups: Vec<StripGroup>,
    /// Point fill color (CSS color string). Default `"steelblue"`.
    pub color: String,
    /// Point radius in pixels. Default `4.0`.
    pub point_size: f64,
    /// Horizontal layout mode. Default is `Strip { jitter: 0.3 }`.
    pub style: StripStyle,
    /// RNG seed for jitter and swarm layout. Default `42`.
    pub seed: u64,
    pub legend_label: Option<String>,
}

impl Default for StripPlot {
    fn default() -> Self { Self::new() }
}

impl StripPlot {
    /// Create a strip plot with default settings.
    ///
    /// Defaults: color `"steelblue"`, point size `4.0`, jitter `0.3`, seed `42`.
    pub fn new() -> Self {
        Self {
            groups: vec![],
            color: "steelblue".into(),
            point_size: 4.0,
            style: StripStyle::Strip { jitter: 0.3 },
            seed: 42,
            legend_label: None,
        }
    }

    /// Add a group (one column of points) with a label and values.
    ///
    /// Groups are rendered left-to-right in the order they are added.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::StripPlot;
    /// let strip = StripPlot::new()
    ///     .with_group("Control",   vec![4.1, 5.0, 5.3, 5.8])
    ///     .with_group("Treatment", vec![6.1, 6.4, 7.2, 7.8]);
    /// ```
    pub fn with_group<S, I>(mut self, label: S, values: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator,
        I::Item: Into<f64>,
    {
        self.groups.push(StripGroup {
            label: label.into(),
            values: values.into_iter().map(Into::into).collect(),
        });
        self
    }

    /// Set the point fill color (CSS color string, default `"steelblue"`).
    ///
    /// Use an `rgba(...)` value to make points semi-transparent when
    /// overlaying on a box plot or violin.
    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    /// Set the point radius in pixels (default `4.0`).
    ///
    /// Reduce for large datasets (e.g. `2.0`–`3.0`) to limit overlap.
    pub fn with_point_size(mut self, size: f64) -> Self {
        self.point_size = size;
        self
    }

    /// Use a jittered strip layout with the given horizontal spread.
    ///
    /// `jitter` is the half-width as a fraction of the category slot width.
    /// `0.3` (the default) spreads points ±30 % of the slot. Increase to
    /// spread points further apart; decrease to tighten the column.
    /// The position is randomised using [`with_seed`](Self::with_seed).
    ///
    /// ```rust,no_run
    /// # use kuva::plot::StripPlot;
    /// let strip = StripPlot::new()
    ///     .with_group("A", vec![1.0, 2.0, 3.0])
    ///     .with_jitter(0.4);   // wider spread
    /// ```
    pub fn with_jitter(mut self, jitter: f64) -> Self {
        self.style = StripStyle::Strip { jitter };
        self
    }

    /// Use a beeswarm (non-overlapping) layout.
    ///
    /// Points are placed as close to the group center as possible without
    /// overlapping. The resulting outline traces the density of the
    /// distribution. Works best for N < ~200 per group; with very large
    /// datasets points will be pushed far from center.
    pub fn with_swarm(mut self) -> Self {
        self.style = StripStyle::Swarm;
        self
    }

    /// Place all points at the group center (no horizontal spread).
    ///
    /// Creates a vertical column of overlapping points. Useful when you want
    /// to show the full data cloud without any jitter artifact, or when
    /// combining with a violin to show individual points on the density axis.
    pub fn with_center(mut self) -> Self {
        self.style = StripStyle::Center;
        self
    }

    /// Set the RNG seed used for jitter positions (default `42`).
    ///
    /// Change the seed to get a different random arrangement while keeping
    /// the output reproducible.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Attach a legend label to this strip plot.
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }
}
