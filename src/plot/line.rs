/// Stroke style for a line plot.
///
/// The default is [`LineStyle::Solid`].
#[derive(Debug, Clone)]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
    DashDot,
    /// Arbitrary SVG `stroke-dasharray` value, e.g. `"10 5 2 5"`.
    Custom(String),
}

impl Default for LineStyle {
    fn default() -> Self {
        LineStyle::Solid
    }
}

impl LineStyle {
    pub fn dasharray(&self) -> Option<String> {
        match self {
            LineStyle::Solid => None,
            LineStyle::Dashed => Some("8 4".into()),
            LineStyle::Dotted => Some("2 4".into()),
            LineStyle::DashDot => Some("8 4 2 4".into()),
            LineStyle::Custom(s) => Some(s.clone()),
        }
    }
}

/// A single (x, y) data point with optional asymmetric error bars.
///
/// Error bars are stored as `(negative_half, positive_half)` — the
/// magnitude of each arm, not the absolute bounds.
#[derive(Debug, Clone, Copy)]
pub struct ScatterPoint {
    pub x: f64,
    pub y: f64,
    pub x_err: Option<(f64, f64)>, // (negative, positive)
    pub y_err: Option<(f64, f64)>,
}

impl From<&ScatterPoint> for (f64, f64) {
    fn from(p: &ScatterPoint) -> (f64, f64) {
        (p.x, p.y)
    }
}

impl ScatterPoint {
    pub fn with_y_error(mut self, err: f64) -> Self {
        self.y_err = Some((err, err));
        self
    }

    pub fn with_y_error_asymmetric(mut self, neg: f64, pos: f64) -> Self {
        self.y_err = Some((neg, pos));
        self
    }
}


use crate::plot::band::BandPlot;

/// Builder for a line plot.
///
/// Connects (x, y) data points with a continuous line. Supports multiple
/// line styles, filled area regions, step interpolation, confidence bands,
/// and error bars.
///
/// # Example
///
/// ```rust,no_run
/// use visus::plot::LinePlot;
/// use visus::backend::svg::SvgBackend;
/// use visus::render::render::render_multiple;
/// use visus::render::layout::Layout;
/// use visus::render::plots::Plot;
///
/// let data: Vec<(f64, f64)> = (0..=100)
///     .map(|i| { let x = i as f64 * 0.1; (x, x.sin()) })
///     .collect();
///
/// let plot = LinePlot::new()
///     .with_data(data)
///     .with_color("steelblue")
///     .with_stroke_width(2.0);
///
/// let plots = vec![Plot::Line(plot)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Sine Wave")
///     .with_x_label("X")
///     .with_y_label("Y");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("line.svg", svg).unwrap();
/// ```
#[derive(Debug, Clone,)]
pub struct LinePlot {
    pub data: Vec<ScatterPoint>,
    pub color: String,
    pub stroke_width: f64,
    pub legend_label: Option<String>,
    pub band: Option<BandPlot>,
    pub line_style: LineStyle,
    pub step: bool,
    pub fill: bool,
    pub fill_opacity: f64,
}

impl LinePlot {
    /// Create a line plot with default settings.
    ///
    /// Defaults: color `"black"`, stroke width `2.0`, [`LineStyle::Solid`],
    /// no fill, no step interpolation.
    pub fn new() -> Self {
        Self {
            data: vec![],
            color: "black".into(),
            stroke_width: 2.0,
            legend_label: None,
            band: None,
            line_style: LineStyle::default(),
            step: false,
            fill: false,
            fill_opacity: 0.3,
        }
    }

    /// Set the (x, y) data points.
    ///
    /// Accepts any iterator of `(T, U)` pairs where `T` and `U` implement
    /// `Into<f64>`, so integer and float types all work without casting.
    ///
    /// ```rust,no_run
    /// # use visus::plot::LinePlot;
    /// // integer input
    /// let plot = LinePlot::new()
    ///     .with_data(vec![(0_i32, 0_i32), (1, 2), (2, 1)]);
    /// ```
    pub fn with_data<T, U, I>(mut self, points: I) -> Self
    where
        I: IntoIterator<Item = (T, U)>,
        T: Into<f64>,
        U: Into<f64>,
    {
        self.data = points
            .into_iter()
            .map(|(x, y)| ScatterPoint {
                x: x.into(),
                y: y.into(),
                x_err: None,
                y_err: None,
            })
            .collect();

        self
    }

    /// Set symmetric X error bars.
    ///
    /// Each value is the half-width of the error bar. Must be called
    /// after [`with_data`](Self::with_data).
    pub fn with_x_err<T, I>(mut self, errors: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<f64> + Copy,
    {
        for (i, err) in errors.into_iter().enumerate() {
            if i < self.data.len() {
                self.data[i].x_err = Some((err.into(), err.into()));
            }
        }

        self
    }

    /// Set asymmetric X error bars.
    ///
    /// Each item is a `(negative_arm, positive_arm)` tuple. Must be
    /// called after [`with_data`](Self::with_data).
    pub fn with_x_err_asymmetric<T, U, I>(mut self, errors: I) -> Self
    where
    I: IntoIterator<Item = (T, U)>,
    T: Into<f64>,
    U: Into<f64>,
    {
        for (i, (neg, pos)) in errors.into_iter().enumerate() {
            if i < self.data.len() {
                self.data[i].x_err = Some((neg.into(), pos.into()));
            }
        }

        self
    }

    /// Set symmetric Y error bars.
    ///
    /// Each value is the half-height of the error bar. Must be called
    /// after [`with_data`](Self::with_data).
    pub fn with_y_err<T, I>(mut self, errors: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<f64> + Copy,
    {
        for (i, err) in errors.into_iter().enumerate() {
            if i < self.data.len() {
                self.data[i].y_err = Some((err.into(), err.into()));
            }
        }

        self
    }

    /// Set asymmetric Y error bars.
    ///
    /// Each item is a `(negative_arm, positive_arm)` tuple. Must be
    /// called after [`with_data`](Self::with_data).
    pub fn with_y_err_asymmetric<T, U, I>(mut self, errors: I) -> Self
    where
        I: IntoIterator<Item = (T, U)>,
        T: Into<f64>,
        U: Into<f64>,
    {
        for (i, (neg, pos)) in errors.into_iter().enumerate() {
            if i < self.data.len() {
                self.data[i].y_err = Some((neg.into(), pos.into()));
            }
        }

        self
    }

    /// Set the line color (CSS color string, e.g. `"steelblue"`, `"#4477aa"`).
    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    /// Set the stroke width in pixels (default `2.0`).
    pub fn with_stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    /// Attach a legend label to this series.
    ///
    /// A legend is rendered automatically when at least one series in
    /// the plot has a label.
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    /// Attach a shaded confidence band aligned to the line's x positions.
    ///
    /// `y_lower` and `y_upper` must have the same length as the data.
    /// The band color matches the line color at 30% opacity.
    ///
    /// ```rust,no_run
    /// # use visus::plot::LinePlot;
    /// let data: Vec<(f64, f64)> = (0..=5).map(|i| (i as f64, (i as f64).sin())).collect();
    /// let lower: Vec<f64> = data.iter().map(|&(_, y)| y - 0.2).collect();
    /// let upper: Vec<f64> = data.iter().map(|&(_, y)| y + 0.2).collect();
    ///
    /// let plot = LinePlot::new()
    ///     .with_data(data)
    ///     .with_color("steelblue")
    ///     .with_band(lower, upper);
    /// ```
    pub fn with_band<T, U, I1, I2>(mut self, y_lower: I1, y_upper: I2) -> Self
    where
        I1: IntoIterator<Item = T>,
        I2: IntoIterator<Item = U>,
        T: Into<f64>,
        U: Into<f64>,
    {
        let x: Vec<f64> = self.data.iter().map(|p| p.x).collect();
        let band = BandPlot::new(x, y_lower, y_upper)
            .with_color(self.color.clone());
        self.band = Some(band);
        self
    }

    /// Set the line style explicitly.
    ///
    /// Convenience shortcuts: [`with_dashed`](Self::with_dashed),
    /// [`with_dotted`](Self::with_dotted), [`with_dashdot`](Self::with_dashdot).
    pub fn with_line_style(mut self, style: LineStyle) -> Self {
        self.line_style = style;
        self
    }

    /// Set the line style to [`LineStyle::Dashed`] (`8 4` dasharray).
    pub fn with_dashed(mut self) -> Self {
        self.line_style = LineStyle::Dashed;
        self
    }

    /// Set the line style to [`LineStyle::Dotted`] (`2 4` dasharray).
    pub fn with_dotted(mut self) -> Self {
        self.line_style = LineStyle::Dotted;
        self
    }

    /// Set the line style to [`LineStyle::DashDot`] (`8 4 2 4` dasharray).
    pub fn with_dashdot(mut self) -> Self {
        self.line_style = LineStyle::DashDot;
        self
    }

    /// Use step interpolation between data points.
    ///
    /// Instead of a diagonal segment between (x₁, y₁) and (x₂, y₂), the
    /// renderer draws a horizontal segment at y₁ to x₂, then a vertical
    /// step to y₂. Useful for histograms and discrete-time series.
    pub fn with_step(mut self) -> Self {
        self.step = true;
        self
    }

    /// Fill the area between the line and the x-axis (area chart).
    ///
    /// The fill uses the line color at the opacity set by
    /// [`with_fill_opacity`](Self::with_fill_opacity) (default `0.3`).
    pub fn with_fill(mut self) -> Self {
        self.fill = true;
        self
    }

    /// Set the fill opacity for area charts (default `0.3`).
    ///
    /// Has no effect unless [`with_fill`](Self::with_fill) is also called.
    pub fn with_fill_opacity(mut self, opacity: f64) -> Self {
        self.fill_opacity = opacity;
        self
    }
}
