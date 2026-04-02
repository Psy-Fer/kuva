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
    /// Optional per-point colors. When set, overrides both `group_colors` and the
    /// uniform `color` for each point individually. Shorter than `values` → remaining
    /// points fall back to the group/uniform color.
    pub point_colors: Option<Vec<String>>,
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
/// auto-assign distinct colors across plots.
///
/// To color groups within a single `StripPlot` differently, use
/// [`with_group_colors`](Self::with_group_colors). This is an alternative to
/// creating one `StripPlot` per group when the data is already grouped.
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
    pub group_colors: Option<Vec<String>>,
    /// Fill opacity for markers (0.0 = transparent, 1.0 = solid). `None` = fully opaque.
    pub marker_opacity: Option<f64>,
    /// Stroke (outline) width for markers. `None` = no stroke. Stroke color matches fill.
    pub marker_stroke_width: Option<f64>,
    pub show_tooltips: bool,
    pub tooltip_labels: Option<Vec<String>>,
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
            group_colors: None,
            marker_opacity: None,
            marker_stroke_width: None,
            show_tooltips: false,
            tooltip_labels: None,
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
            point_colors: None,
        });
        self
    }

    /// Add a group where each point carries its own color.
    ///
    /// `points` is any iterator of `(value, color)` pairs. Colors are matched to points
    /// by position; the uniform [`with_color`](Self::with_color) / per-group color is
    /// used as a fallback for any point beyond the end of the list.
    ///
    /// Use this when each observation belongs to a distinct category (e.g. a motif type)
    /// and you want to color individual points independently within the same column.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::StripPlot;
    /// let strip = StripPlot::new()
    ///     .with_colored_group("Sample", vec![
    ///         (1.5, "steelblue"),
    ///         (2.3, "tomato"),
    ///         (1.8, "seagreen"),
    ///     ])
    ///     .with_swarm();
    /// ```
    pub fn with_colored_group<S, V, C, I>(mut self, label: S, points: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = (V, C)>,
        V: Into<f64>,
        C: Into<String>,
    {
        let (values, colors): (Vec<f64>, Vec<String>) = points
            .into_iter()
            .map(|(v, c)| (v.into(), c.into()))
            .unzip();
        self.groups.push(StripGroup {
            label: label.into(),
            values,
            point_colors: Some(colors),
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

    /// Set per-group colors.
    ///
    /// Colors are matched to groups by position (first color → first group
    /// added via [`with_group`](Self::with_group), and so on). If the list is
    /// shorter than the number of groups, the uniform color from
    /// [`with_color`](Self::with_color) is used as a fallback.
    ///
    /// Note that the legend is not automatically updated when using this method.
    /// If you need a labeled legend, create one `StripPlot` per group (each
    /// with `.with_legend()`) or supply custom entries via
    /// `Layout::with_legend_entries` (planned).
    ///
    /// ```rust,no_run
    /// # use kuva::plot::StripPlot;
    /// let strip = StripPlot::new()
    ///     .with_group("Control",   vec![1.0, 2.0, 3.0])
    ///     .with_group("Treatment", vec![2.0, 3.0, 4.0])
    ///     .with_group("Placebo",   vec![1.5, 2.5, 3.5])
    ///     .with_group_colors(vec!["steelblue", "tomato", "seagreen"]);
    /// ```
    pub fn with_group_colors<S, I>(mut self, colors: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.group_colors = Some(colors.into_iter().map(|s| s.into()).collect());
        self
    }

    /// Set the fill opacity for all markers (0.0 = fully transparent, 1.0 = fully opaque).
    pub fn with_marker_opacity(mut self, opacity: f64) -> Self {
        self.marker_opacity = Some(opacity.clamp(0.0, 1.0));
        self
    }

    /// Draw a solid outline around each marker at the given stroke width.
    ///
    /// Stroke color matches the fill color.
    pub fn with_marker_stroke_width(mut self, width: f64) -> Self {
        self.marker_stroke_width = Some(width);
        self
    }

    pub fn with_tooltips(mut self) -> Self {
        self.show_tooltips = true;
        self
    }

    pub fn with_tooltip_labels(mut self, labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tooltip_labels = Some(labels.into_iter().map(|s| s.into()).collect());
        self
    }
}
