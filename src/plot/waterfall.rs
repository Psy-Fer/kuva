/// The role of a single bar in a waterfall chart.
#[derive(Clone, Debug)]
pub enum WaterfallKind {
    /// A floating increment or decrement applied to the running total.
    ///
    /// The bar floats between the previous running total and the new one.
    /// Positive values are colored with `color_positive`; negative with
    /// `color_negative`.
    Delta,
    /// A summary bar that resets to zero and spans from zero to the current
    /// running total. Colored with `color_total`. The `value` field is ignored.
    ///
    /// Use at the end of a section to show a subtotal, or at the very end to
    /// show the final accumulated value.
    Total,
    /// A standalone comparison bar anchored at explicit `[from, to]` values.
    ///
    /// This bar does **not** affect the running total — it is purely
    /// illustrative. Green when `to > from`, red when `to < from`.
    /// Useful for annotating a period-over-period change at a specific
    /// reference level.
    Difference { from: f64, to: f64 },
}

/// A single bar in a waterfall chart.
pub struct WaterfallBar {
    pub label: String,
    pub value: f64,
    pub kind: WaterfallKind,
}

/// Builder for a waterfall chart.
///
/// A waterfall chart shows a running total as a sequence of floating bars.
/// Each bar extends from the previous accumulated value, rising for positive
/// deltas (green) and falling for negative deltas (red). Optional
/// [`Total`](WaterfallKind::Total) bars reset to zero and show the
/// accumulated value, useful for intermediate subtotals.
///
/// # Bar types
///
/// | Method | Effect |
/// |--------|--------|
/// | `.with_delta(label, value)` | Floating bar; adds `value` to running total |
/// | `.with_total(label)` | Summary bar from zero to current running total |
/// | `.with_difference(label, from, to)` | Anchored comparison bar; does not change running total |
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::WaterfallPlot;
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let wf = WaterfallPlot::new()
///     .with_delta("Revenue",       850.0)
///     .with_delta("Cost of goods",-340.0)
///     .with_total("Gross profit")
///     .with_delta("Operating costs",-200.0)
///     .with_delta("Tax",           -65.0)
///     .with_total("Net income")
///     .with_connectors()
///     .with_values();
///
/// let plots = vec![Plot::Waterfall(wf)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Income Statement")
///     .with_x_label("Stage")
///     .with_y_label("USD (thousands)");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("waterfall.svg", svg).unwrap();
/// ```
pub struct WaterfallPlot {
    pub bars: Vec<WaterfallBar>,
    /// Bar width as a fraction of the category slot (default `0.6`).
    pub bar_width: f64,
    /// Color for positive delta bars (default `"rgb(68,170,68)"`).
    pub color_positive: String,
    /// Color for negative delta bars (default `"rgb(204,68,68)"`).
    pub color_negative: String,
    /// Color for total/subtotal bars (default `"steelblue"`).
    pub color_total: String,
    /// When `true`, dashed connector lines are drawn between consecutive bars.
    pub show_connectors: bool,
    /// When `true`, numeric values are printed on each bar.
    pub show_values: bool,
    pub legend_label: Option<String>,
}

impl Default for WaterfallPlot {
    fn default() -> Self { Self::new() }
}

impl WaterfallPlot {
    /// Create a waterfall chart with default settings.
    ///
    /// Defaults: bar width `0.6`, green/red delta colors, steelblue totals,
    /// no connectors, no value labels.
    pub fn new() -> Self {
        Self {
            bars: Vec::new(),
            bar_width: 0.6,
            color_positive: "rgb(68,170,68)".into(),
            color_negative: "rgb(204,68,68)".into(),
            color_total: "steelblue".into(),
            show_connectors: false,
            show_values: false,
            legend_label: None,
        }
    }

    /// Add a floating delta bar.
    ///
    /// The bar spans from the current running total to `current + value`.
    /// Positive values are colored green; negative values red.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::WaterfallPlot;
    /// let wf = WaterfallPlot::new()
    ///     .with_delta("Revenue",  850.0)
    ///     .with_delta("COGS",    -340.0)
    ///     .with_delta("OpEx",    -200.0);
    /// ```
    pub fn with_delta<S: Into<String>>(mut self, label: S, value: f64) -> Self {
        self.bars.push(WaterfallBar {
            label: label.into(),
            value,
            kind: WaterfallKind::Delta,
        });
        self
    }

    /// Add a standalone comparison bar anchored at explicit y-values.
    ///
    /// The bar spans `[from, to]` and is green when `to > from`, red when
    /// `to < from`. It does **not** affect the running total, so it can be
    /// placed anywhere to annotate a reference comparison without disrupting
    /// the main flow.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::WaterfallPlot;
    /// let wf = WaterfallPlot::new()
    ///     .with_delta("Start", 1000.0)
    ///     .with_delta("Change",  150.0)
    ///     .with_difference("vs target", 1000.0, 1200.0)  // independent reference
    ///     .with_total("End");
    /// ```
    pub fn with_difference<S: Into<String>>(mut self, label: S, from: f64, to: f64) -> Self {
        self.bars.push(WaterfallBar {
            label: label.into(),
            value: 0.0,
            kind: WaterfallKind::Difference { from, to },
        });
        self
    }

    /// Add a summary bar that spans from zero to the current running total.
    ///
    /// Rendered in `color_total` (default `"steelblue"`). The `value` field
    /// is ignored — the bar height is determined by the accumulated total at
    /// this position. Place after a sequence of delta bars to show a subtotal
    /// or at the end to show the final result.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::WaterfallPlot;
    /// let wf = WaterfallPlot::new()
    ///     .with_delta("Revenue",  850.0)
    ///     .with_delta("COGS",    -340.0)
    ///     .with_total("Gross profit")    // subtotal
    ///     .with_delta("OpEx",    -200.0)
    ///     .with_total("Net income");     // final total
    /// ```
    pub fn with_total<S: Into<String>>(mut self, label: S) -> Self {
        self.bars.push(WaterfallBar {
            label: label.into(),
            value: 0.0,
            kind: WaterfallKind::Total,
        });
        self
    }

    /// Set the bar width as a fraction of the category slot (default `0.6`).
    pub fn with_bar_width(mut self, width: f64) -> Self {
        self.bar_width = width;
        self
    }

    /// Set the color for positive delta bars (default `"rgb(68,170,68)"`).
    pub fn with_color_positive<S: Into<String>>(mut self, color: S) -> Self {
        self.color_positive = color.into();
        self
    }

    /// Set the color for negative delta bars (default `"rgb(204,68,68)"`).
    pub fn with_color_negative<S: Into<String>>(mut self, color: S) -> Self {
        self.color_negative = color.into();
        self
    }

    /// Set the color for total/subtotal bars (default `"steelblue"`).
    pub fn with_color_total<S: Into<String>>(mut self, color: S) -> Self {
        self.color_total = color.into();
        self
    }

    /// Draw dashed connector lines between the top (or bottom) of consecutive bars.
    ///
    /// Connectors make it easier to trace the running total across wide charts.
    pub fn with_connectors(mut self) -> Self {
        self.show_connectors = true;
        self
    }

    /// Print the numeric value of each bar as a text label.
    ///
    /// Delta bars show their `value`; total bars show the accumulated total;
    /// difference bars show `to - from`.
    pub fn with_values(mut self) -> Self {
        self.show_values = true;
        self
    }

    /// Attach a legend label to this waterfall chart.
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }
}
