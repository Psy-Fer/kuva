/// Controls how gene labels are positioned on a volcano plot.
#[derive(Clone, Default)]
pub enum LabelStyle {
    /// Label placed at the exact point position — no nudge, no leader line.
    ///
    /// Labels may overlap when many points are clustered together. Use this
    /// for sparse data or when downstream SVG editing will handle placement.
    Exact,
    /// Labels sorted by x position and nudged vertically to reduce stacking
    /// **(default)**.
    ///
    /// This is the most readable option for dense data. Labels stay close
    /// to their points without crossing each other.
    #[default]
    Nudge,
    /// Label offset by `(offset_x, offset_y)` pixels with a short gray leader
    /// line drawn from the offset position back to the point.
    ///
    /// Use this when labels need to be moved further from the plot centre to
    /// avoid crowding in the high-significance region.
    ///
    /// ```rust,no_run
    /// use visus::plot::LabelStyle;
    /// let style = LabelStyle::Arrow { offset_x: 14.0, offset_y: 16.0 };
    /// ```
    Arrow { offset_x: f64, offset_y: f64 },
}


/// A single gene (or feature) displayed in a volcano plot.
pub struct VolcanoPoint {
    /// Gene or feature name, shown as a label when selected.
    pub name: String,
    /// log₂ fold change on the x-axis.
    pub log2fc: f64,
    /// Raw p-value (not −log10). Zero p-values are handled automatically.
    pub pvalue: f64,
}

/// Builder for a volcano plot.
///
/// A volcano plot visualises differential expression results by plotting
/// **log₂ fold change** (x-axis) against **−log₁₀(p-value)** (y-axis).
/// Points that pass both the fold-change cutoff and p-value threshold are
/// colored as up-regulated (right) or down-regulated (left); all others are
/// shown as not-significant (gray).
///
/// Dashed threshold lines are drawn automatically at ±`fc_cutoff` and at
/// the y position corresponding to `p_cutoff`.
///
/// # Gene labels
///
/// Call [`with_label_top(n)`](Self::with_label_top) to label the `n` most
/// significant points. Three placement styles are available via
/// [`with_label_style`](Self::with_label_style): [`LabelStyle::Nudge`]
/// (default), [`LabelStyle::Exact`], and [`LabelStyle::Arrow`].
///
/// # Zero p-values
///
/// p-values of exactly `0.0` cannot be log-transformed. They are
/// automatically capped at the smallest non-zero p-value in the data.
/// Set an explicit cap with [`with_pvalue_floor`](Self::with_pvalue_floor)
/// to control the y-axis ceiling across multiple plots.
///
/// # Example
///
/// ```rust,no_run
/// use visus::plot::VolcanoPlot;
/// use visus::backend::svg::SvgBackend;
/// use visus::render::render::render_multiple;
/// use visus::render::layout::Layout;
/// use visus::render::plots::Plot;
///
/// let vp = VolcanoPlot::new()
///     .with_points(vec![
///         ("EGFR",   3.2_f64, 1e-4_f64),
///         ("AKT1",   3.5,     5e-5   ),
///         ("VHL",   -3.0,     5e-4   ),
///         ("GAPDH",  0.3,     0.5    ),
///     ])
///     .with_label_top(3)
///     .with_legend("DEG status");
///
/// let plots = vec![Plot::Volcano(vp)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Tumour vs. Normal")
///     .with_x_label("log₂ fold change")
///     .with_y_label("−log₁₀(p-value)");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("volcano.svg", svg).unwrap();
/// ```
pub struct VolcanoPlot {
    /// All data points.
    pub points: Vec<VolcanoPoint>,
    /// |log₂FC| threshold for up/down classification (default `1.0`).
    /// Dashed vertical lines are drawn at ±`fc_cutoff`.
    pub fc_cutoff: f64,
    /// p-value threshold for significance (default `0.05`).
    /// A dashed horizontal line is drawn at `−log10(p_cutoff)`.
    pub p_cutoff: f64,
    /// Color for up-regulated points: |log2FC| ≥ fc_cutoff **and** p ≤ p_cutoff
    /// (default `"firebrick"`).
    pub color_up: String,
    /// Color for down-regulated points: log2FC ≤ −fc_cutoff **and** p ≤ p_cutoff
    /// (default `"steelblue"`).
    pub color_down: String,
    /// Color for not-significant points (default `"#aaaaaa"`).
    pub color_ns: String,
    /// Circle radius in pixels (default `3.0`).
    pub point_size: f64,
    /// Number of most-significant points to label (default `0` — no labels).
    pub label_top: usize,
    /// Label placement style (default [`LabelStyle::Nudge`]).
    pub label_style: LabelStyle,
    /// Explicit p-value floor for the −log10 transform. When `None`, the
    /// minimum non-zero p-value in the data is used automatically.
    pub pvalue_floor: Option<f64>,
    /// When `Some`, a legend box shows Up / Down / NS entries.
    pub legend_label: Option<String>,
}

impl Default for VolcanoPlot {
    fn default() -> Self { Self::new() }
}

impl VolcanoPlot {
    /// Create a volcano plot with default settings.
    ///
    /// Defaults: fc_cutoff `1.0`, p_cutoff `0.05`, firebrick/steelblue/gray
    /// colors, point size `3.0`, no labels, no legend.
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            fc_cutoff: 1.0,
            p_cutoff: 0.05,
            color_up: "firebrick".into(),
            color_down: "steelblue".into(),
            color_ns: "#aaaaaa".into(),
            point_size: 3.0,
            label_top: 0,
            label_style: LabelStyle::default(),
            pvalue_floor: None,
            legend_label: None,
        }
    }

    /// Compute the p-value floor used for -log10 transformation.
    /// Uses explicit floor if set, otherwise finds minimum non-zero p-value.
    pub fn floor(&self) -> f64 {
        if let Some(f) = self.pvalue_floor { return f; }
        self.points.iter()
            .map(|p| p.pvalue)
            .filter(|&p| p > 0.0)
            .fold(f64::INFINITY, f64::min)
            .max(1e-300)
    }

    /// Add a single point by name, log₂FC, and raw p-value.
    ///
    /// Useful when building a plot incrementally. For bulk input prefer
    /// [`with_points`](Self::with_points).
    ///
    /// ```rust,no_run
    /// # use visus::plot::VolcanoPlot;
    /// let vp = VolcanoPlot::new()
    ///     .with_point("EGFR", 3.2_f64, 1e-4_f64)
    ///     .with_point("GAPDH", 0.3, 0.5);
    /// ```
    pub fn with_point<S, F, G>(mut self, name: S, log2fc: F, pvalue: G) -> Self
    where
        S: Into<String>,
        F: Into<f64>,
        G: Into<f64>,
    {
        self.points.push(VolcanoPoint {
            name: name.into(),
            log2fc: log2fc.into(),
            pvalue: pvalue.into(),
        });
        self
    }

    /// Add multiple points from an iterator of `(name, log2fc, pvalue)` tuples.
    ///
    /// This is the primary input method. Accepts any types that implement
    /// `Into<String>` / `Into<f64>`. Slices, `Vec`, or any other iterable
    /// of tuples all work.
    ///
    /// ```rust,no_run
    /// # use visus::plot::VolcanoPlot;
    /// let results = vec![
    ///     ("EGFR",  3.2_f64, 1e-4_f64),
    ///     ("AKT1",  3.5,     5e-5    ),
    ///     ("VHL",  -3.0,     5e-4    ),
    ///     ("GAPDH", 0.3,     0.5     ),
    /// ];
    /// let vp = VolcanoPlot::new().with_points(results);
    /// ```
    pub fn with_points<I, S, F, G>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (S, F, G)>,
        S: Into<String>,
        F: Into<f64>,
        G: Into<f64>,
    {
        for (name, log2fc, pvalue) in iter {
            self.points.push(VolcanoPoint {
                name: name.into(),
                log2fc: log2fc.into(),
                pvalue: pvalue.into(),
            });
        }
        self
    }

    /// Set the absolute log₂FC threshold for up/down classification
    /// (default `1.0` — corresponding to a 2× fold change).
    ///
    /// Dashed vertical lines are drawn at `±fc_cutoff`. Points with
    /// `|log2FC| < fc_cutoff` are always shown as not-significant regardless
    /// of their p-value.
    pub fn with_fc_cutoff(mut self, cutoff: f64) -> Self {
        self.fc_cutoff = cutoff;
        self
    }

    /// Set the p-value significance threshold (default `0.05`).
    ///
    /// A dashed horizontal line is drawn at `−log10(p_cutoff)`. Points with
    /// `pvalue > p_cutoff` are shown as not-significant regardless of their
    /// fold change.
    pub fn with_p_cutoff(mut self, cutoff: f64) -> Self {
        self.p_cutoff = cutoff;
        self
    }

    /// Set the color for up-regulated points (default `"firebrick"`).
    ///
    /// A point is up-regulated when `log2fc ≥ fc_cutoff` **and**
    /// `pvalue ≤ p_cutoff`. Accepts any CSS color string.
    pub fn with_color_up<S: Into<String>>(mut self, color: S) -> Self {
        self.color_up = color.into();
        self
    }

    /// Set the color for down-regulated points (default `"steelblue"`).
    ///
    /// A point is down-regulated when `log2fc ≤ −fc_cutoff` **and**
    /// `pvalue ≤ p_cutoff`. Accepts any CSS color string.
    pub fn with_color_down<S: Into<String>>(mut self, color: S) -> Self {
        self.color_down = color.into();
        self
    }

    /// Set the color for not-significant points (default `"#aaaaaa"`).
    ///
    /// All points that do not meet both the fold-change and p-value thresholds
    /// are drawn in this color. Accepts any CSS color string.
    pub fn with_color_ns<S: Into<String>>(mut self, color: S) -> Self {
        self.color_ns = color.into();
        self
    }

    /// Set the circle radius in pixels (default `3.0`).
    pub fn with_point_size(mut self, size: f64) -> Self {
        self.point_size = size;
        self
    }

    /// Label the `n` most significant points (lowest p-values) with their names.
    ///
    /// When `n = 0` (default) no labels are drawn. Use
    /// [`with_label_style`](Self::with_label_style) to control placement.
    ///
    /// ```rust,no_run
    /// # use visus::plot::VolcanoPlot;
    /// let vp = VolcanoPlot::new()
    ///     .with_points(vec![("EGFR", 3.2_f64, 1e-4_f64)])
    ///     .with_label_top(10);  // label the 10 most significant genes
    /// ```
    pub fn with_label_top(mut self, n: usize) -> Self {
        self.label_top = n;
        self
    }

    /// Set the label placement style (default [`LabelStyle::Nudge`]).
    ///
    /// - [`LabelStyle::Nudge`] — labels are sorted by x and nudged vertically
    ///   to reduce overlap. Best default for most datasets.
    /// - [`LabelStyle::Exact`] — labels are placed at the exact point position
    ///   with no adjustment. May overlap on dense plots.
    /// - [`LabelStyle::Arrow`] — labels are offset by `(offset_x, offset_y)` px
    ///   with a gray leader line back to the point.
    ///
    /// ```rust,no_run
    /// use visus::plot::{VolcanoPlot, LabelStyle};
    /// let vp = VolcanoPlot::new()
    ///     .with_points(vec![("EGFR", 3.2_f64, 1e-4_f64)])
    ///     .with_label_top(10)
    ///     .with_label_style(LabelStyle::Arrow { offset_x: 14.0, offset_y: 16.0 });
    /// ```
    pub fn with_label_style(mut self, style: LabelStyle) -> Self {
        self.label_style = style;
        self
    }

    /// Set an explicit p-value floor for the −log10 transform.
    ///
    /// Points with `pvalue == 0.0` are clamped to this value before
    /// transformation, preventing infinite y positions. Also sets the
    /// y-axis ceiling to `−log10(floor)`.
    ///
    /// When not set, the floor is inferred as the minimum non-zero p-value
    /// in the data. Set it explicitly when comparing multiple plots that
    /// should share the same y-axis scale.
    ///
    /// ```rust,no_run
    /// # use visus::plot::VolcanoPlot;
    /// let vp = VolcanoPlot::new()
    ///     .with_points(vec![("EGFR", 3.2_f64, 0.0_f64)])  // p = 0 is valid input
    ///     .with_pvalue_floor(1e-10);  // y-axis ceiling = 10
    /// ```
    pub fn with_pvalue_floor(mut self, floor: f64) -> Self {
        self.pvalue_floor = Some(floor);
        self
    }

    /// Enable a legend showing **Up**, **Down**, and **NS** entries.
    ///
    /// The legend uses the active up/down/NS colors. The string argument is
    /// not currently used as a title but must be set to enable the legend.
    ///
    /// ```rust,no_run
    /// # use visus::plot::VolcanoPlot;
    /// let vp = VolcanoPlot::new()
    ///     .with_points(vec![("EGFR", 3.2_f64, 1e-4_f64)])
    ///     .with_legend("DEG status");
    /// ```
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }
}
