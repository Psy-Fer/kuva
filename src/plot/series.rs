/// Display style for a [`SeriesPlot`].
///
/// Controls whether each data point is rendered as a line segment, a dot,
/// or both.
pub enum SeriesStyle {
    /// Connect consecutive values with a polyline. No dots are drawn.
    Line,
    /// Draw a circle at each value. No connecting line is drawn. **(default)**
    Point,
    /// Draw both the connecting polyline and a circle at each value.
    Both,
}

/// Builder for a series plot — a 1D sequence of y-values plotted against
/// their sequential index on the x-axis.
///
/// A series plot is the simplest way to visualise a time series, signal, or
/// any ordered sequence of measurements. The x-axis is assigned automatically
/// as consecutive integers `0, 1, 2, …` matching the index of each value.
///
/// Multiple `SeriesPlot` instances combined in one `plots` vector share the
/// same axes and are drawn in order, enabling overlay of several series.
///
/// # Display styles
///
/// Three rendering styles are available via the `with_*_style()` methods:
///
/// | Method | Style | Description |
/// |--------|-------|-------------|
/// | `.with_line_style()` | `Line` | Polyline connecting consecutive points |
/// | `.with_point_style()` | `Point` | Circle at each value **(default)** |
/// | `.with_line_point_style()` | `Both` | Polyline + circles |
///
/// # Example
///
/// ```rust,no_run
/// use visus::plot::SeriesPlot;
/// use visus::backend::svg::SvgBackend;
/// use visus::render::render::render_multiple;
/// use visus::render::layout::Layout;
/// use visus::render::plots::Plot;
///
/// let data: Vec<f64> = (0..80)
///     .map(|i| (i as f64 * std::f64::consts::TAU / 80.0).sin())
///     .collect();
///
/// let series = SeriesPlot::new()
///     .with_data(data)
///     .with_color("steelblue")
///     .with_line_style();
///
/// let plots = vec![Plot::Series(series)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Sine Wave")
///     .with_x_label("Sample")
///     .with_y_label("Amplitude");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("series.svg", svg).unwrap();
/// ```
pub struct SeriesPlot {
    /// Ordered y-values. The x position of value `i` is `i as f64`.
    pub values: Vec<f64>,
    /// CSS color used for both lines and points. Default: `"black"`.
    pub color: String,
    /// Rendering style. Default: [`SeriesStyle::Point`].
    pub style: SeriesStyle,
    /// When `Some`, a legend entry is shown with this label.
    pub legend_label: Option<String>,
    /// Line stroke width in pixels. Default: `2.0`.
    pub stroke_width: f64,
    /// Circle radius in pixels (used in `Point` and `Both` styles). Default: `3.0`.
    pub point_radius: f64,
}


impl SeriesPlot {
    /// Create a series plot with default settings.
    ///
    /// Defaults: no data, `"black"` color, `Point` style, stroke width `2.0`,
    /// point radius `3.0`, no legend.
    pub fn new() -> Self {
        Self {
            values: vec![],
            color: "black".into(),
            style: SeriesStyle::Point,
            legend_label: None,
            stroke_width: 2.0,
            point_radius: 3.0,
        }
    }

    /// Load y-values from any iterable of numeric values.
    ///
    /// The x position of each value is its index in the sequence (0, 1, 2, …).
    ///
    /// ```rust,no_run
    /// # use visus::plot::SeriesPlot;
    /// let data: Vec<f64> = (0..100)
    ///     .map(|i| (i as f64 / 10.0).sin())
    ///     .collect();
    /// let series = SeriesPlot::new().with_data(data);
    /// ```
    pub fn with_data<T, I>(mut self, data: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<f64>,
    {
        self.values = data.into_iter().map(|x| x.into()).collect();
        self
    }

    /// Set the color for lines and points. Default: `"black"`.
    ///
    /// Accepts any CSS color string (`"steelblue"`, `"#4682b4"`, `"rgb(70,130,180)"`).
    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    /// Render as a polyline only — no circles at data points.
    pub fn with_line_style(mut self) -> Self {
        self.style = SeriesStyle::Line;
        self
    }

    /// Render as circles only — no connecting line. **(default)**
    pub fn with_point_style(mut self) -> Self {
        self.style = SeriesStyle::Point;
        self
    }

    /// Render as a polyline with circles at each data point.
    pub fn with_line_point_style(mut self) -> Self {
        self.style = SeriesStyle::Both;
        self
    }

    /// Enable a legend entry with the given label.
    ///
    /// ```rust,no_run
    /// # use visus::plot::SeriesPlot;
    /// let series = SeriesPlot::new()
    ///     .with_data(vec![1.0_f64, 2.0, 1.5])
    ///     .with_legend("sensor A");
    /// ```
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    /// Set the line stroke width in pixels. Default: `2.0`.
    ///
    /// Only affects `Line` and `Both` styles.
    pub fn with_stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    /// Set the circle radius in pixels. Default: `3.0`.
    ///
    /// Only affects `Point` and `Both` styles.
    pub fn with_point_radius(mut self, radius: f64) -> Self {
        self.point_radius = radius;
        self
    }
}
