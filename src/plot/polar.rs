/// Plot mode for a polar series.
#[derive(Debug, Clone, Default)]
pub enum PolarMode {
    #[default]
    Scatter,
    Line,
}

/// A single data series in a polar plot.
#[derive(Debug, Clone)]
pub struct PolarSeries {
    pub r: Vec<f64>,
    pub theta: Vec<f64>, // degrees
    pub label: Option<String>,
    pub color: Option<String>,
    pub mode: PolarMode,
    pub marker_size: f64,
    pub stroke_width: f64,
    pub line_dash: Option<String>,
    /// Fill opacity for scatter markers (0.0 = transparent, 1.0 = solid). `None` = fully opaque.
    pub marker_opacity: Option<f64>,
    /// Stroke (outline) width for scatter markers. `None` = no stroke. Stroke color matches fill.
    pub marker_stroke_width: Option<f64>,
}

impl Default for PolarSeries {
    fn default() -> Self {
        PolarSeries {
            r: Vec::new(),
            theta: Vec::new(),
            label: None,
            color: None,
            mode: PolarMode::Scatter,
            marker_size: 5.0,
            stroke_width: 1.5,
            line_dash: None,
            marker_opacity: None,
            marker_stroke_width: None,
        }
    }
}

/// Polar coordinate scatter/line plot.
///
/// Supports compass convention (θ=0 at north, increasing clockwise) by default.
/// Switch to math convention with `.with_theta_start(90.0).with_clockwise(false)`.
///
/// # Example
/// ```rust,no_run
/// use kuva::plot::polar::{PolarPlot, PolarMode};
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
/// use kuva::render::render::render_multiple;
/// use kuva::backend::svg::SvgBackend;
///
/// let theta: Vec<f64> = (0..36).map(|i| i as f64 * 10.0).collect();
/// let r: Vec<f64> = theta.iter().map(|&t| 1.0 + t.to_radians().cos()).collect();
///
/// let plot = PolarPlot::new()
///     .with_series(r, theta)
///     .with_r_max(2.0);
///
/// let plots = vec![Plot::Polar(plot)];
/// let layout = Layout::auto_from_plots(&plots).with_title("Cardioid");
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// ```
#[derive(Debug, Clone)]
pub struct PolarPlot {
    pub series: Vec<PolarSeries>,
    pub r_max: Option<f64>,
    /// Where θ=0 appears on canvas, degrees CW from north (top). Default 0 = north.
    pub theta_start: f64,
    /// true = clockwise (compass), false = CCW (math). Default: true.
    pub clockwise: bool,
    /// Number of concentric r-grid circles. None = auto (4).
    pub r_grid_lines: Option<usize>,
    /// Angular spoke divisions. Default 12 (every 30°).
    pub theta_divisions: usize,
    pub show_grid: bool,
    pub show_r_labels: bool,
    pub show_legend: bool,
}

impl Default for PolarPlot {
    fn default() -> Self {
        PolarPlot {
            series: Vec::new(),
            r_max: None,
            theta_start: 0.0,
            clockwise: true,
            r_grid_lines: None,
            theta_divisions: 12,
            show_grid: true,
            show_r_labels: true,
            show_legend: false,
        }
    }
}

impl PolarPlot {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a scatter series with radial values `r` and angular values `theta` (degrees).
    pub fn with_series<T, U, IT, IU>(mut self, r: IT, theta: IU) -> Self
    where
        T: Into<f64>,
        U: Into<f64>,
        IT: IntoIterator<Item = T>,
        IU: IntoIterator<Item = U>,
    {
        let r_vals: Vec<f64> = r.into_iter().map(Into::into).collect();
        let theta_vals: Vec<f64> = theta.into_iter().map(Into::into).collect();
        self.series.push(PolarSeries {
            r: r_vals,
            theta: theta_vals,
            mode: PolarMode::Scatter,
            ..Default::default()
        });
        self
    }

    /// Add a line series with radial values `r` and angular values `theta` (degrees).
    pub fn with_series_line<T, U, IT, IU>(mut self, r: IT, theta: IU) -> Self
    where
        T: Into<f64>,
        U: Into<f64>,
        IT: IntoIterator<Item = T>,
        IU: IntoIterator<Item = U>,
    {
        let r_vals: Vec<f64> = r.into_iter().map(Into::into).collect();
        let theta_vals: Vec<f64> = theta.into_iter().map(Into::into).collect();
        self.series.push(PolarSeries {
            r: r_vals,
            theta: theta_vals,
            mode: PolarMode::Line,
            ..Default::default()
        });
        self
    }

    /// Add a labeled series with explicit mode.
    pub fn with_series_labeled<S, T, U, IT, IU>(
        mut self,
        r: IT,
        theta: IU,
        label: S,
        mode: PolarMode,
    ) -> Self
    where
        S: Into<String>,
        T: Into<f64>,
        U: Into<f64>,
        IT: IntoIterator<Item = T>,
        IU: IntoIterator<Item = U>,
    {
        let r_vals: Vec<f64> = r.into_iter().map(Into::into).collect();
        let theta_vals: Vec<f64> = theta.into_iter().map(Into::into).collect();
        self.series.push(PolarSeries {
            r: r_vals,
            theta: theta_vals,
            label: Some(label.into()),
            mode,
            ..Default::default()
        });
        self
    }

    pub fn with_r_max(mut self, r_max: f64) -> Self {
        self.r_max = Some(r_max);
        self
    }

    /// Set where θ=0 appears on the canvas, in degrees CW from north.
    pub fn with_theta_start(mut self, degrees: f64) -> Self {
        self.theta_start = degrees;
        self
    }

    /// Set whether increasing θ goes clockwise (true = compass) or CCW (false = math).
    pub fn with_clockwise(mut self, cw: bool) -> Self {
        self.clockwise = cw;
        self
    }

    /// Set the number of concentric radial grid circles.
    pub fn with_r_grid_lines(mut self, n: usize) -> Self {
        self.r_grid_lines = Some(n);
        self
    }

    /// Set the number of angular spoke divisions (default 12 = every 30°).
    pub fn with_theta_divisions(mut self, n: usize) -> Self {
        self.theta_divisions = n;
        self
    }

    pub fn with_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    pub fn with_r_labels(mut self, show: bool) -> Self {
        self.show_r_labels = show;
        self
    }

    pub fn with_legend(mut self, show: bool) -> Self {
        self.show_legend = show;
        self
    }

    /// Set the color of the last added series.
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        if let Some(s) = self.series.last_mut() {
            s.color = Some(color.into());
        }
        self
    }

    /// Set the fill opacity for scatter markers in the last added series
    /// (0.0 = fully transparent, 1.0 = fully opaque).
    pub fn with_marker_opacity(mut self, opacity: f64) -> Self {
        if let Some(s) = self.series.last_mut() {
            s.marker_opacity = Some(opacity.clamp(0.0, 1.0));
        }
        self
    }

    /// Draw a solid outline around scatter markers in the last added series.
    ///
    /// Stroke color matches the fill color.
    pub fn with_marker_stroke_width(mut self, width: f64) -> Self {
        if let Some(s) = self.series.last_mut() {
            s.marker_stroke_width = Some(width);
        }
        self
    }

    /// Compute the maximum r value across all series (for auto-scaling).
    pub fn r_max_auto(&self) -> f64 {
        self.series
            .iter()
            .flat_map(|s| s.r.iter())
            .cloned()
            .fold(0.0_f64, f64::max)
    }
}
