/// Which theoretical distribution to compare against.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum QQMode {
    /// Compare sample quantiles to the theoretical standard-normal distribution.
    /// x-axis: theoretical quantiles; y-axis: sample quantiles.
    #[default]
    Normal,
    /// GWAS genomic Q-Q: −log₁₀(expected p) vs −log₁₀(observed p).
    /// Input values must be p-values in (0, 1].
    Genomic,
}

/// One group of values for a Q-Q plot.
#[derive(Debug, Clone)]
pub struct QQGroup {
    pub label: String,
    /// Raw data values (normal mode) or p-values in (0, 1] (genomic mode).
    pub data: Vec<f64>,
    pub color: Option<String>,
}

/// Builder for a Q-Q plot.
///
/// Two modes:
/// - **Normal** — sample quantiles vs standard-normal theoretical quantiles
///   with a robust Q1–Q3 reference line.
/// - **Genomic** — −log₁₀(observed p) vs −log₁₀(expected p), with a y = x
///   diagonal, optional 95 % CI band, and genomic inflation factor λ.
///
/// # Example
///
/// ```rust,no_run
/// use kuva::prelude::*;
///
/// // Normal Q-Q
/// let plot = QQPlot::new()
///     .with_data("Sample", vec![0.1, 0.5, 1.2, 2.3, 3.1])
///     .with_color("steelblue");
///
/// // Genomic Q-Q
/// let plot = QQPlot::new()
///     .with_pvalues("GWAS", vec![0.001, 0.01, 0.05, 0.3, 0.9])
///     .with_ci_band()
///     .with_lambda();
/// ```
#[derive(Debug, Clone)]
pub struct QQPlot {
    pub groups: Vec<QQGroup>,
    pub mode: QQMode,
    /// Draw a reference line (default: `true`).
    pub show_reference_line: bool,
    /// Draw a 95 % pointwise CI band around the reference diagonal.
    pub show_ci_band: bool,
    pub ci_alpha: f64,
    /// Annotate λ (genomic inflation factor) on the plot. Genomic mode only.
    pub show_lambda: bool,
    pub marker_size: f64,
    pub stroke_width: f64,
    pub legend_label: Option<String>,
    /// Default color for single-group plots.
    pub color: String,
    pub fill_opacity: Option<f64>,
}

impl Default for QQPlot {
    fn default() -> Self {
        Self {
            groups: Vec::new(),
            mode: QQMode::Normal,
            show_reference_line: true,
            show_ci_band: false,
            ci_alpha: 0.15,
            show_lambda: true,
            marker_size: 3.0,
            stroke_width: 1.5,
            legend_label: None,
            color: "steelblue".into(),
            fill_opacity: None,
        }
    }
}

impl QQPlot {
    pub fn new() -> Self { Self::default() }

    // ── Data ────────────────────────────────────────────────────────────────

    /// Add a group of raw values (normal mode).
    pub fn with_data(mut self, label: impl Into<String>, data: impl IntoIterator<Item = impl Into<f64>>) -> Self {
        self.groups.push(QQGroup {
            label: label.into(),
            data: data.into_iter().map(|v| v.into()).collect(),
            color: None,
        });
        self
    }

    /// Add a group with an explicit color (normal mode).
    pub fn with_data_colored(mut self, label: impl Into<String>, data: impl IntoIterator<Item = impl Into<f64>>, color: impl Into<String>) -> Self {
        self.groups.push(QQGroup {
            label: label.into(),
            data: data.into_iter().map(|v| v.into()).collect(),
            color: Some(color.into()),
        });
        self
    }

    /// Add a group of p-values and switch to genomic mode.
    /// P-values must be in (0, 1]. Values outside this range are silently filtered.
    pub fn with_pvalues(mut self, label: impl Into<String>, pvals: impl IntoIterator<Item = impl Into<f64>>) -> Self {
        self.mode = QQMode::Genomic;
        self.groups.push(QQGroup {
            label: label.into(),
            data: pvals.into_iter().map(|v| v.into()).collect(),
            color: None,
        });
        self
    }

    /// Add a group of p-values with an explicit color and switch to genomic mode.
    pub fn with_pvalues_colored(mut self, label: impl Into<String>, pvals: impl IntoIterator<Item = impl Into<f64>>, color: impl Into<String>) -> Self {
        self.mode = QQMode::Genomic;
        self.groups.push(QQGroup {
            label: label.into(),
            data: pvals.into_iter().map(|v| v.into()).collect(),
            color: Some(color.into()),
        });
        self
    }

    // ── Mode ────────────────────────────────────────────────────────────────

    /// Switch to normal Q-Q mode (default).
    pub fn with_normal(mut self) -> Self { self.mode = QQMode::Normal; self }

    /// Switch to genomic Q-Q mode.
    pub fn with_genomic(mut self) -> Self { self.mode = QQMode::Genomic; self }

    // ── Features ────────────────────────────────────────────────────────────

    /// Show the reference line (default: on).
    pub fn with_reference_line(mut self) -> Self { self.show_reference_line = true; self }

    /// Hide the reference line.
    pub fn without_reference_line(mut self) -> Self { self.show_reference_line = false; self }

    /// Draw a 95 % pointwise CI band around the reference diagonal.
    pub fn with_ci_band(mut self) -> Self { self.show_ci_band = true; self }

    /// Set CI band fill opacity (default: `0.15`).
    pub fn with_ci_alpha(mut self, alpha: f64) -> Self { self.ci_alpha = alpha; self }

    /// Annotate the genomic inflation factor λ on the plot (genomic mode only).
    pub fn with_lambda(mut self) -> Self { self.show_lambda = true; self }

    /// Hide the λ annotation.
    pub fn without_lambda(mut self) -> Self { self.show_lambda = false; self }

    // ── Appearance ──────────────────────────────────────────────────────────

    pub fn with_marker_size(mut self, size: f64) -> Self { self.marker_size = size; self }
    pub fn with_stroke_width(mut self, w: f64) -> Self { self.stroke_width = w; self }
    pub fn with_color(mut self, color: impl Into<String>) -> Self { self.color = color.into(); self }
    pub fn with_fill_opacity(mut self, opacity: f64) -> Self { self.fill_opacity = Some(opacity); self }

    // ── Legend ──────────────────────────────────────────────────────────────

    /// Enable the legend. Pass `""` for no title, or a string for a titled legend.
    pub fn with_legend(mut self, title: impl Into<String>) -> Self {
        self.legend_label = Some(title.into());
        self
    }
}
