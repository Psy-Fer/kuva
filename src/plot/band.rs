/// A shaded area between two y-curves over a shared x-axis — typically used
/// to display confidence intervals, prediction bands, or IQR envelopes.
///
/// # Usage modes
///
/// **Standalone** — create with `BandPlot::new` and add as `Plot::Band`. Pair
/// with a `Plot::Line` or `Plot::Scatter` in the same `plots` vector to draw
/// the band behind the data series.
///
/// **Attached** — use [`LinePlot::with_band`](crate::plot::LinePlot::with_band)
/// or [`ScatterPlot::with_band`](crate::plot::ScatterPlot::with_band) as a
/// one-call shorthand. The band inherits the series color automatically and is
/// rendered behind the line or points.
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::{BandPlot, LinePlot};
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let x: Vec<f64> = (0..50).map(|i| i as f64 * 0.2).collect();
/// let y: Vec<f64> = x.iter().map(|&v| v.sin()).collect();
/// let lower: Vec<f64> = y.iter().map(|&v| v - 0.3).collect();
/// let upper: Vec<f64> = y.iter().map(|&v| v + 0.3).collect();
///
/// let band = BandPlot::new(x.clone(), lower, upper)
///     .with_color("steelblue")
///     .with_opacity(0.25);
///
/// let line = LinePlot::new()
///     .with_data(x.iter().copied().zip(y.iter().copied()))
///     .with_color("steelblue");
///
/// let plots = vec![Plot::Band(band), Plot::Line(line)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Confidence Band")
///     .with_x_label("x")
///     .with_y_label("y");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("band.svg", svg).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct BandPlot {
    /// x coordinates shared by both boundary curves.
    pub x: Vec<f64>,
    /// Lower boundary y values. Must have the same length as `x`.
    pub y_lower: Vec<f64>,
    /// Upper boundary y values. Must have the same length as `x`.
    pub y_upper: Vec<f64>,
    /// Fill color as a CSS color string. Default: `"steelblue"`.
    pub color: String,
    /// Fill opacity in `[0.0, 1.0]`. Default: `0.2`.
    pub opacity: f64,
    /// When `Some`, a legend entry is shown with a filled rectangle swatch.
    pub legend_label: Option<String>,
}

impl BandPlot {
    /// Create a band from parallel x, lower-bound, and upper-bound iterables.
    ///
    /// All three iterables must have the same length. Accepts any type
    /// implementing `Into<f64>`. Default fill: `"steelblue"` at opacity `0.2`.
    ///
    /// ```rust,no_run
    /// use kuva::plot::BandPlot;
    ///
    /// let x = vec![0.0_f64, 1.0, 2.0, 3.0];
    /// let lower = vec![-0.3_f64, 0.7, 1.7, 2.7];
    /// let upper = vec![ 0.3_f64, 1.3, 2.3, 3.3];
    ///
    /// let band = BandPlot::new(x, lower, upper);
    /// ```
    pub fn new<T, U, V, I1, I2, I3>(x: I1, y_lower: I2, y_upper: I3) -> Self
    where
        I1: IntoIterator<Item = T>,
        I2: IntoIterator<Item = U>,
        I3: IntoIterator<Item = V>,
        T: Into<f64>,
        U: Into<f64>,
        V: Into<f64>,
    {
        Self {
            x: x.into_iter().map(Into::into).collect(),
            y_lower: y_lower.into_iter().map(Into::into).collect(),
            y_upper: y_upper.into_iter().map(Into::into).collect(),
            color: "steelblue".into(),
            opacity: 0.2,
            legend_label: None,
        }
    }

    /// Set the fill color. Default: `"steelblue"`.
    ///
    /// Accepts any CSS color string. When using the standalone mode, set this
    /// to match the paired line or scatter color for a cohesive look.
    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    /// Set the fill opacity. Default: `0.2`.
    ///
    /// Values in `[0.0, 1.0]`. Lower values make the band more transparent;
    /// `1.0` gives a fully opaque filled area.
    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }

    /// Enable a legend entry with the given label.
    ///
    /// The legend swatch is a filled rectangle in the band color.
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }
}
