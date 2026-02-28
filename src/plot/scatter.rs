/// Marker shape used to render individual scatter points.
///
/// The default is [`MarkerShape::Circle`].
#[derive(Debug, Clone, Copy, Default)]
pub enum MarkerShape {
    #[default]
    Circle,
    Square,
    Triangle,
    Diamond,
    Cross,
    Plus,
}


/// Trend line variant to overlay on a scatter plot.
#[derive(Debug, Clone, Copy)]
pub enum TrendLine {
    /// Ordinary least-squares linear fit: y = mx + b.
    Linear,
    // Polynomial(u8),
    // Exponential,
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


use crate::plot::band::BandPlot;

/// Builder for a scatter plot.
///
/// Constructs a scatter plot from (x, y) data. Supports error bars,
/// trend lines, confidence bands, variable point sizes (bubble plots),
/// and six marker shapes.
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::scatter::ScatterPlot;
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let data = vec![(1.0_f64, 2.0_f64), (3.0, 5.0), (5.0, 4.0)];
///
/// let plot = ScatterPlot::new()
///     .with_data(data)
///     .with_color("steelblue")
///     .with_size(5.0);
///
/// let plots = vec![Plot::Scatter(plot)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("My Scatter")
///     .with_x_label("X")
///     .with_y_label("Y");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("scatter.svg", svg).unwrap();
/// ```
pub struct ScatterPlot {
    pub data: Vec<ScatterPoint>,
    pub color: String,
    pub size: f64,
    pub legend_label: Option<String>,
    pub trend: Option<TrendLine>,
    pub trend_color: String,
    pub show_equation: bool,
    pub show_correlation: bool,
    pub trend_width: f64,
    pub band: Option<BandPlot>,
    pub marker: MarkerShape,
    pub sizes: Option<Vec<f64>>,
}


impl Default for ScatterPlot {
    fn default() -> Self { Self::new() }
}

impl ScatterPlot {
    /// Create a scatter plot with default settings.
    ///
    /// Defaults: color `"black"`, size `3.0`, [`MarkerShape::Circle`],
    /// no trend line, no legend label.
    pub fn new() -> Self {
        Self {
            data: vec![],
            color: "black".into(),
            size: 3.0,
            legend_label: None,
            trend: None,
            trend_color: "black".into(),
            show_equation: false,
            show_correlation: false,
            trend_width: 1.0,
            band: None,
            marker: MarkerShape::default(),
            sizes: None,
        }
    }

    /// Set the (x, y) data points.
    ///
    /// Accepts any iterator of `(T, U)` pairs where `T` and `U` implement
    /// `Into<f64>`, so integer and float types all work without casting.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::scatter::ScatterPlot;
    /// // integer input
    /// let plot = ScatterPlot::new()
    ///     .with_data(vec![(1_i32, 5_i32), (2, 8), (3, 6)]);
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
    /// Each value is the half-width of the error bar (i.e. the bar
    /// extends ± value from the point). Must be called after
    /// [`with_data`](Self::with_data).
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

    /// Set the point color (CSS color string, e.g. `"steelblue"`, `"#4477aa"`).
    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    /// Set the uniform point radius in pixels (default `3.0`).
    ///
    /// For per-point radii use [`with_sizes`](Self::with_sizes).
    pub fn with_size(mut self, size: f64) -> Self {
        self.size = size;
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

    /// Overlay a trend line computed from the scatter data.
    pub fn with_trend(mut self, trend: TrendLine) -> Self {
        self.trend = Some(trend);
        self
    }

    /// Set the trend line color (default `"black"`).
    pub fn with_trend_color<S: Into<String>>(mut self, color: S) -> Self {
        self.trend_color = color.into();
        self
    }

    /// Annotate the plot with the regression equation (y = mx + b).
    ///
    /// Requires a trend line to be set via [`with_trend`](Self::with_trend).
    pub fn with_equation(mut self) -> Self {
        self.show_equation = true;
        self
    }

    /// Annotate the plot with the Pearson R² value.
    ///
    /// Requires a trend line to be set via [`with_trend`](Self::with_trend).
    pub fn with_correlation(mut self) -> Self {
        self.show_correlation = true;
        self
    }

    /// Set the trend line stroke width in pixels (default `1.0`).
    pub fn with_trend_width(mut self, width: f64) -> Self {
        self.trend_width = width;
        self
    }

    /// Attach a shaded confidence band aligned to the scatter x positions.
    ///
    /// `y_lower` and `y_upper` must have the same length as the data.
    /// The band color matches the scatter series color.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::scatter::ScatterPlot;
    /// let data = vec![(1.0_f64, 2.0_f64), (2.0, 4.0), (3.0, 6.0)];
    /// let lower = vec![1.5_f64, 3.5, 5.5];
    /// let upper = vec![2.5_f64, 4.5, 6.5];
    ///
    /// let plot = ScatterPlot::new()
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

    /// Set the marker shape (default [`MarkerShape::Circle`]).
    pub fn with_marker(mut self, marker: MarkerShape) -> Self {
        self.marker = marker;
        self
    }

    /// Set per-point radii for a bubble plot.
    ///
    /// Values are radii in pixels. When set, the uniform `size` value
    /// from [`with_size`](Self::with_size) is ignored.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::scatter::ScatterPlot;
    /// let data = vec![(1.0_f64, 2.0_f64), (3.0, 4.0), (5.0, 3.0)];
    /// let sizes = vec![5.0_f64, 12.0, 8.0];
    ///
    /// let plot = ScatterPlot::new()
    ///     .with_data(data)
    ///     .with_sizes(sizes)
    ///     .with_color("steelblue");
    /// ```
    pub fn with_sizes<T, I>(mut self, sizes: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<f64>,
    {
        self.sizes = Some(sizes.into_iter().map(|s| s.into()).collect());
        self
    }
}
