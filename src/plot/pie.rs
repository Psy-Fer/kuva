/// Controls where slice labels are placed relative to each slice.
///
/// The default is [`Auto`](PieLabelPosition::Auto), which places labels
/// inside large slices and moves small ones outside with leader lines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PieLabelPosition {
    /// Labels centered between the inner and outer radius (or at mid-radius for
    /// a full pie). Works well when all slices are large enough to fit text.
    Inside,
    /// Labels placed outside the pie with leader lines connecting them to their
    /// slice. Small slices are automatically spaced to avoid label overlap.
    Outside,
    /// Inside for large slices; outside with a leader line for small ones.
    /// This is the default. The threshold is controlled by
    /// [`with_min_label_fraction`](PiePlot::with_min_label_fraction).
    Auto,
    /// No slice labels. Combine with [`with_legend`](PiePlot::with_legend)
    /// to identify slices via a legend instead.
    None,
}

/// Builder for a pie or donut chart.
///
/// Each slice has its own explicit color. Slice labels can be positioned
/// automatically, forced inside or outside, or suppressed entirely in favor
/// of a legend. Percentage values can be appended to labels with
/// [`with_percent`](Self::with_percent).
///
/// Render with [`render_pie`](crate::render::render::render_pie) for most
/// cases, or [`render_multiple`](crate::render::render::render_multiple) when
/// a legend is attached.
///
/// # Example
///
/// ```rust,no_run
/// use visus::plot::{PiePlot, PieLabelPosition};
/// use visus::backend::svg::SvgBackend;
/// use visus::render::render::render_pie;
/// use visus::render::layout::Layout;
/// use visus::render::plots::Plot;
///
/// let pie = PiePlot::new()
///     .with_slice("Rust",   40.0, "steelblue")
///     .with_slice("Python", 30.0, "tomato")
///     .with_slice("R",      20.0, "seagreen")
///     .with_slice("Other",  10.0, "gold")
///     .with_percent();
///
/// let plots = vec![Plot::Pie(pie.clone())];
/// let layout = Layout::auto_from_plots(&plots).with_title("Language usage");
///
/// let svg = SvgBackend.render_scene(&render_pie(&pie, &layout));
/// std::fs::write("pie.svg", svg).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct PiePlot {
    pub slices: Vec<PieSlice>,
    /// Inner radius in pixels. `0.0` renders a full pie; any positive value
    /// cuts a hole in the centre to produce a donut chart.
    pub inner_radius: f64,
    pub legend_label: Option<String>,
    /// Label placement strategy. Defaults to [`PieLabelPosition::Auto`].
    pub label_position: PieLabelPosition,
    /// When `true`, each label is suffixed with the slice's percentage of the total.
    pub show_percent: bool,
    /// Slices whose fraction of the total is below this threshold receive no
    /// label (default `0.05`, i.e. 5 %).
    pub min_label_fraction: f64,
}

/// A single slice of a pie chart.
#[derive(Debug, Clone)]
pub struct PieSlice {
    pub label: String,
    pub value: f64,
    /// Fill color as a CSS color string (e.g. `"steelblue"`, `"#4682b4"`).
    pub color: String,
}

impl Default for PiePlot {
    fn default() -> Self { Self::new() }
}

impl PiePlot {
    /// Create a pie chart with default settings.
    ///
    /// Defaults: full pie (`inner_radius = 0.0`), Auto label positioning,
    /// no percentages, no legend, `min_label_fraction = 0.05`.
    pub fn new() -> Self {
        Self {
            slices: vec![],
            inner_radius: 0.0,
            legend_label: None,
            label_position: PieLabelPosition::Auto,
            show_percent: false,
            min_label_fraction: 0.05,
        }
    }

    /// Add a slice with a label, value, and fill color.
    ///
    /// Slices are drawn clockwise in the order they are added, starting from
    /// the top (12 o'clock). The value is proportional — only the ratio between
    /// values matters, not their absolute magnitude.
    ///
    /// ```rust,no_run
    /// # use visus::plot::PiePlot;
    /// let pie = PiePlot::new()
    ///     .with_slice("A", 60.0, "steelblue")
    ///     .with_slice("B", 40.0, "tomato");
    /// ```
    pub fn with_slice<L, V, C>(mut self, label: L, value: V, color: C) -> Self
    where
        L: Into<String>,
        V: Into<f64>,
        C: Into<String>,
    {
        self.slices.push(PieSlice {
            label: label.into(),
            value: value.into(),
            color: color.into(),
        });
        self
    }

    /// Set the inner radius in pixels to create a donut chart.
    ///
    /// A value of `0.0` (the default) renders a solid pie. Any positive value
    /// cuts a hollow centre. Typical values are in the range `40.0`–`80.0`
    /// depending on the canvas size.
    ///
    /// ```rust,no_run
    /// # use visus::plot::PiePlot;
    /// let donut = PiePlot::new()
    ///     .with_slice("A", 60.0, "steelblue")
    ///     .with_slice("B", 40.0, "tomato")
    ///     .with_inner_radius(60.0);
    /// ```
    pub fn with_inner_radius(mut self, r: f64) -> Self {
        self.inner_radius = r;
        self
    }

    /// Attach a legend to the pie chart.
    ///
    /// When a legend label is set, [`render_multiple`](crate::render::render::render_multiple)
    /// adds a per-slice legend entry (colored square + slice label) in the
    /// right margin. Combine with
    /// [`with_label_position(PieLabelPosition::None)`](Self::with_label_position)
    /// to use the legend as the sole means of identification.
    ///
    /// ```rust,no_run
    /// # use visus::plot::{PiePlot, PieLabelPosition};
    /// # use visus::render::plots::Plot;
    /// let pie = PiePlot::new()
    ///     .with_slice("A", 60.0, "steelblue")
    ///     .with_slice("B", 40.0, "tomato")
    ///     .with_legend("Category")
    ///     .with_label_position(PieLabelPosition::None);
    /// ```
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    /// Set the label placement strategy.
    ///
    /// See [`PieLabelPosition`] for the available options. The default is
    /// [`Auto`](PieLabelPosition::Auto).
    pub fn with_label_position(mut self, pos: PieLabelPosition) -> Self {
        self.label_position = pos;
        self
    }

    /// Append the percentage of the total to each slice label.
    ///
    /// The percentage is computed from the slice values and formatted to one
    /// decimal place (e.g. `"Rust 40.0%"`).
    pub fn with_percent(mut self) -> Self {
        self.show_percent = true;
        self
    }

    /// Set the minimum slice fraction below which no label is drawn.
    ///
    /// Slices whose value is less than `fraction` of the total are silently
    /// skipped. The default is `0.05` (5 %). Set to `0.0` to label every
    /// slice regardless of size.
    ///
    /// ```rust,no_run
    /// # use visus::plot::PiePlot;
    /// // Label all slices, even tiny ones
    /// let pie = PiePlot::new()
    ///     .with_slice("Big",  90.0, "steelblue")
    ///     .with_slice("Tiny",  1.0, "tomato")
    ///     .with_min_label_fraction(0.0);
    /// ```
    pub fn with_min_label_fraction(mut self, fraction: f64) -> Self {
        self.min_label_fraction = fraction;
        self
    }
}
