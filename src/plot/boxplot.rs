use crate::plot::strip::StripStyle;

/// Builder for a box plot (box-and-whisker plot).
///
/// Displays the five-number summary for one or more groups of values.
/// Whiskers use the Tukey 1.5×IQR rule; values outside the whiskers
/// are not drawn automatically (use an overlay to show individual
/// points). Groups are rendered side-by-side in the order they are
/// added.
///
/// # Example
///
/// ```rust,no_run
/// use visus::plot::BoxPlot;
/// use visus::backend::svg::SvgBackend;
/// use visus::render::render::render_multiple;
/// use visus::render::layout::Layout;
/// use visus::render::plots::Plot;
///
/// let plot = BoxPlot::new()
///     .with_group("Control",   vec![4.1, 5.0, 5.3, 5.8, 6.2, 7.0])
///     .with_group("Treated",   vec![5.5, 6.1, 6.4, 7.2, 7.8, 8.5])
///     .with_color("steelblue");
///
/// let plots = vec![Plot::Box(plot)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Box Plot")
///     .with_y_label("Value");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("boxplot.svg", svg).unwrap();
/// ```
pub struct BoxPlot {
    pub groups: Vec<BoxGroup>,
    pub color: String,
    pub width: f64,
    pub legend_label: Option<String>,
    pub overlay: Option<StripStyle>,
    pub overlay_color: String,
    pub overlay_size: f64,
    pub overlay_seed: u64,
}

/// A single group (one box) with a category label and raw values.
pub struct BoxGroup {
    pub label: String,
    pub values: Vec<f64>,
}

impl BoxPlot {
    /// Create a box plot with default settings.
    ///
    /// Defaults: color `"black"`, box width `0.8`,
    /// overlay color `"rgba(0,0,0,0.45)"`, overlay point size `3.0`.
    pub fn new() -> Self {
        Self {
            groups: vec![],
            color: "black".into(),
            width: 0.8,
            legend_label: None,
            overlay: None,
            overlay_color: "rgba(0,0,0,0.45)".into(),
            overlay_size: 3.0,
            overlay_seed: 42,
        }
    }

    /// Add a group (one box) with a label and raw values.
    ///
    /// Groups are rendered left-to-right in the order they are added.
    /// The renderer computes Q1, median, Q3, and Tukey 1.5×IQR whiskers
    /// from the supplied values.
    ///
    /// ```rust,no_run
    /// # use visus::plot::BoxPlot;
    /// let plot = BoxPlot::new()
    ///     .with_group("A", vec![1.0, 2.5, 3.0, 3.5, 4.0, 5.0])
    ///     .with_group("B", vec![2.0, 3.0, 3.8, 4.2, 4.8, 6.0]);
    /// ```
    pub fn with_group<T, U, I>(mut self, label: T, values: I) -> Self
    where
        T: Into<String>,
        I: IntoIterator<Item = U>,
        U: Into<f64>,
    {
        self.groups.push(BoxGroup {
            label: label.into(),
            values: values.into_iter().map(Into::into).collect(),
        });
        self
    }

    /// Set the box fill color (CSS color string, e.g. `"steelblue"`).
    ///
    /// This color is applied to all boxes. Use the same color for all
    /// groups and distinguish them by position, or layer multiple
    /// `BoxPlot` instances in a `Vec<Plot>` with different colors.
    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    /// Set the box width as a fraction of the category slot (default `0.8`).
    pub fn with_width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// Attach a legend label to this box plot.
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    /// Overlay individual data points as a jittered strip.
    ///
    /// `jitter` controls the horizontal spread of the points (in data
    /// units). A value of `0.2` is a reasonable default. Points are
    /// placed on top of the box — use a semi-transparent
    /// [`with_overlay_color`](Self::with_overlay_color) so the box
    /// remains visible underneath.
    pub fn with_strip(mut self, jitter: f64) -> Self {
        self.overlay = Some(StripStyle::Strip { jitter });
        self
    }

    /// Overlay individual data points as a beeswarm.
    ///
    /// Points are spread horizontally to avoid overlap, giving a clearer
    /// view of the data density than a jittered strip. Useful for smaller
    /// datasets (N < ~200 per group) where individual points are
    /// meaningful.
    pub fn with_swarm_overlay(mut self) -> Self {
        self.overlay = Some(StripStyle::Swarm);
        self
    }

    /// Set the fill color for overlay points (default `"rgba(0,0,0,0.45)"`).
    ///
    /// A semi-transparent color is recommended so the box underneath
    /// remains visible.
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
