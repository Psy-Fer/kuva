/// One row in a forest plot: a study with an effect estimate and confidence interval.
pub struct ForestRow {
    pub label: String,
    pub estimate: f64,
    pub ci_lower: f64,
    pub ci_upper: f64,
    /// Optional study weight — used to scale the marker radius.
    pub weight: Option<f64>,
    /// Optional per-row color override (CSS color string).
    pub color: Option<String>,
}

impl ForestRow {
    /// Create a row with the required fields; weight and color default to `None`.
    pub fn new<S: Into<String>>(label: S, estimate: f64, ci_lower: f64, ci_upper: f64) -> Self {
        Self {
            label: label.into(),
            estimate,
            ci_lower,
            ci_upper,
            weight: None,
            color: None,
        }
    }
}

/// Builder for a forest plot (meta-analysis).
///
/// Each row represents a study: a point estimate with a confidence interval
/// on a numeric X-axis, and a label on a categorical Y-axis. A vertical
/// reference line (null effect, typically at x = 0) can be shown.
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::ForestPlot;
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let forest = ForestPlot::new()
///     .with_row("Study A", 0.50, 0.10, 0.90)
///     .with_row("Study B", -0.30, -0.80, 0.20)
///     .with_row("Study C", 0.20, -0.10, 0.50)
///     .with_null_value(0.0);
///
/// let plots = vec![Plot::Forest(forest)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Meta-Analysis")
///     .with_x_label("Effect Size");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("forest.svg", svg).unwrap();
/// ```
pub struct ForestPlot {
    pub rows: Vec<ForestRow>,
    /// Point/whisker color (CSS color string). Default `"steelblue"`.
    pub color: String,
    /// Base marker half-width in pixels. Default `6.0`.
    pub marker_size: f64,
    /// Whisker stroke width in pixels. Default `1.5`.
    pub whisker_width: f64,
    /// Null-effect reference value (vertical dashed line). Default `Some(0.0)`.
    pub null_value: Option<f64>,
    /// Whether to draw the null reference line. Default `true`.
    pub show_null_line: bool,
    /// Cap half-height in pixels for whisker end caps. Default `0.0` (no caps).
    pub cap_size: f64,
    pub legend_label: Option<String>,
}

impl Default for ForestPlot {
    fn default() -> Self { Self::new() }
}

impl ForestPlot {
    /// Create a forest plot with default settings.
    ///
    /// Defaults: color `"steelblue"`, marker size `6.0`, whisker width `1.5`,
    /// null value `0.0`, null line shown.
    pub fn new() -> Self {
        Self {
            rows: vec![],
            color: "steelblue".into(),
            marker_size: 6.0,
            whisker_width: 1.5,
            null_value: Some(0.0),
            show_null_line: true,
            cap_size: 0.0,
            legend_label: None,
        }
    }

    /// Add a row (study) with a label, point estimate, and confidence interval bounds.
    ///
    /// Rows are rendered top-to-bottom in the order they are added.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::ForestPlot;
    /// let forest = ForestPlot::new()
    ///     .with_row("Study A", 0.50, 0.10, 0.90)
    ///     .with_row("Study B", -0.30, -0.80, 0.20);
    /// ```
    pub fn with_row<S: Into<String>>(mut self, label: S, estimate: f64, ci_lower: f64, ci_upper: f64) -> Self {
        self.rows.push(ForestRow::new(label, estimate, ci_lower, ci_upper));
        self
    }

    /// Add a weighted row. The marker size scales with `sqrt(weight / max_weight)`.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::ForestPlot;
    /// let forest = ForestPlot::new()
    ///     .with_weighted_row("Study A", 0.50, 0.10, 0.90, 5.2)
    ///     .with_weighted_row("Study B", -0.30, -0.80, 0.20, 3.8);
    /// ```
    pub fn with_weighted_row<S: Into<String>>(mut self, label: S, estimate: f64, ci_lower: f64, ci_upper: f64, weight: f64) -> Self {
        let mut row = ForestRow::new(label, estimate, ci_lower, ci_upper);
        row.weight = Some(weight);
        self.rows.push(row);
        self
    }

    /// Add a row with a per-row color override.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::ForestPlot;
    /// let forest = ForestPlot::new()
    ///     .with_colored_row("Favours treatment", 0.50, 0.10, 0.90, "seagreen")
    ///     .with_colored_row("Favours control",  -0.30, -0.80, 0.20, "tomato");
    /// ```
    pub fn with_colored_row<S: Into<String>, C: Into<String>>(mut self, label: S, estimate: f64, ci_lower: f64, ci_upper: f64, color: C) -> Self {
        let mut row = ForestRow::new(label, estimate, ci_lower, ci_upper);
        row.color = Some(color.into());
        self.rows.push(row);
        self
    }

    /// Add a row with both a weight and a per-row color override.
    pub fn with_weighted_colored_row<S: Into<String>, C: Into<String>>(
        mut self, label: S, estimate: f64, ci_lower: f64, ci_upper: f64, weight: f64, color: C,
    ) -> Self {
        let mut row = ForestRow::new(label, estimate, ci_lower, ci_upper);
        row.weight = Some(weight);
        row.color = Some(color.into());
        self.rows.push(row);
        self
    }

    /// Set the point fill and whisker color (CSS color string, default `"steelblue"`).
    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    /// Set the base marker half-width in pixels (default `6.0`).
    ///
    /// When weights are present, the actual size is scaled by
    /// `sqrt(weight / max_weight)`.
    pub fn with_marker_size(mut self, size: f64) -> Self {
        self.marker_size = size;
        self
    }

    /// Set the whisker (CI line) stroke width in pixels (default `1.5`).
    pub fn with_whisker_width(mut self, width: f64) -> Self {
        self.whisker_width = width;
        self
    }

    /// Set the null-effect reference value (default `0.0`).
    ///
    /// A vertical dashed line is drawn at this value when
    /// [`show_null_line`](Self::with_show_null_line) is `true`.
    pub fn with_null_value(mut self, value: f64) -> Self {
        self.null_value = Some(value);
        self
    }

    /// Toggle the null-effect reference line (default `true`).
    pub fn with_show_null_line(mut self, show: bool) -> Self {
        self.show_null_line = show;
        self
    }

    /// Set the whisker end-cap half-height in pixels (default `0.0`, no caps).
    ///
    /// Set to e.g. `4.0` to add visible serifs at each end of the CI whisker.
    pub fn with_cap_size(mut self, size: f64) -> Self {
        self.cap_size = size;
        self
    }

    /// Attach a legend label to this forest plot.
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }
}
