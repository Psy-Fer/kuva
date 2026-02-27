use crate::plot::legend::LegendPosition;

const DEFAULT_COLORS: &[&str] = &[
    "steelblue", "orange", "green", "red", "purple",
    "brown", "pink", "gray",
];

/// Builder for a stacked area chart.
///
/// A stacked area chart places multiple series on top of each other so the
/// reader can see both the individual contribution of each series and the
/// combined total at any x position. It is well suited for showing how a total
/// is composed of parts over a continuous axis — typically time.
///
/// # Building a chart
///
/// 1. Set x values with [`with_x`](Self::with_x).
/// 2. Add each series with [`with_series`](Self::with_series), then immediately
///    chain [`with_color`](Self::with_color) and [`with_legend`](Self::with_legend)
///    to configure that series.
/// 3. Optionally enable [`with_normalized`](Self::with_normalized) for
///    100 % percent-stacking.
///
/// # Example
///
/// ```rust,no_run
/// use visus::plot::StackedAreaPlot;
/// use visus::backend::svg::SvgBackend;
/// use visus::render::render::render_multiple;
/// use visus::render::layout::Layout;
/// use visus::render::plots::Plot;
///
/// let months: Vec<f64> = (1..=12).map(|m| m as f64).collect();
///
/// let sa = StackedAreaPlot::new()
///     .with_x(months)
///     .with_series([10.0, 12.0, 15.0, 18.0, 14.0, 20.0,
///                   22.0, 19.0, 25.0, 28.0, 24.0, 30.0])
///     .with_color("steelblue").with_legend("Group A")
///     .with_series([ 5.0,  6.0,  8.0,  7.0,  9.0, 10.0,
///                   11.0, 10.0, 12.0, 14.0, 13.0, 15.0])
///     .with_color("orange").with_legend("Group B");
///
/// let plots = vec![Plot::StackedArea(sa)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Monthly Counts")
///     .with_x_label("Month")
///     .with_y_label("Count");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("stacked_area.svg", svg).unwrap();
/// ```
#[derive(Clone)]
pub struct StackedAreaPlot {
    /// X-axis values shared across all series.
    pub x: Vec<f64>,
    /// Y values for each series. `series[k][i]` is the value for series `k` at `x[i]`.
    pub series: Vec<Vec<f64>>,
    /// Optional explicit fill color for each series (parallel to `series`).
    /// `None` falls back to the built-in default color palette.
    pub colors: Vec<Option<String>>,
    /// Optional legend label for each series (parallel to `series`).
    /// Series with `None` are omitted from the legend box.
    pub labels: Vec<Option<String>>,
    /// Fill opacity applied to every band (default `0.7`).
    pub fill_opacity: f64,
    /// Stroke width for the top-edge line on each band (default `1.5`).
    pub stroke_width: f64,
    /// Whether to draw a stroke along the top edge of each band (default `true`).
    pub show_strokes: bool,
    /// When `true`, each column is rescaled to sum to 100 %; the y-axis spans 0–100 %.
    pub normalized: bool,
    /// Corner of the plot area where the legend box is placed (default `TopRight`).
    pub legend_position: LegendPosition,
}

impl Default for StackedAreaPlot {
    fn default() -> Self { Self::new() }
}

impl StackedAreaPlot {
    /// Create a stacked area plot with default settings.
    ///
    /// Defaults: fill opacity `0.7`, stroke width `1.5`, strokes enabled,
    /// absolute (non-normalized) stacking, legend at top-right.
    pub fn new() -> Self {
        Self {
            x: Vec::new(),
            series: Vec::new(),
            colors: Vec::new(),
            labels: Vec::new(),
            fill_opacity: 0.7,
            stroke_width: 1.5,
            show_strokes: true,
            normalized: false,
            legend_position: LegendPosition::TopRight,
        }
    }

    /// Set the x-axis values shared by all series.
    ///
    /// Call this before adding any series. Accepts any numeric type via `Into<f64>`.
    ///
    /// ```rust,no_run
    /// # use visus::plot::StackedAreaPlot;
    /// let months: Vec<f64> = (1..=12).map(|m| m as f64).collect();
    /// let sa = StackedAreaPlot::new().with_x(months);
    /// ```
    pub fn with_x<T, I>(mut self, x: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<f64>,
    {
        self.x = x.into_iter().map(Into::into).collect();
        self
    }

    /// Append a new series.
    ///
    /// Call [`with_color`](Self::with_color) and [`with_legend`](Self::with_legend)
    /// immediately after to configure the series that was just added. These methods
    /// always operate on the **most recently added** series.
    ///
    /// ```rust,no_run
    /// # use visus::plot::StackedAreaPlot;
    /// let sa = StackedAreaPlot::new()
    ///     .with_x([1.0, 2.0, 3.0])
    ///     .with_series([10.0, 20.0, 15.0])
    ///     .with_color("steelblue").with_legend("Series A")
    ///     .with_series([5.0, 8.0, 6.0])
    ///     .with_color("orange").with_legend("Series B");
    /// ```
    pub fn with_series<T, I>(mut self, y: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<f64>,
    {
        self.series.push(y.into_iter().map(Into::into).collect());
        self.colors.push(None);
        self.labels.push(None);
        self
    }

    /// Set the fill color of the most recently added series.
    ///
    /// Accepts any CSS color string (named colors, hex, `rgb(…)`). When not
    /// called, the series falls back to the built-in default palette:
    /// `steelblue`, `orange`, `green`, `red`, `purple`, `brown`, `pink`, `gray`
    /// (cycling for more than eight series).
    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        if let Some(last) = self.colors.last_mut() {
            *last = Some(color.into());
        }
        self
    }

    /// Set the legend label of the most recently added series.
    ///
    /// Series without a legend label are not shown in the legend box.
    /// Omit this call to exclude a series from the legend entirely.
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        if let Some(last) = self.labels.last_mut() {
            *last = Some(label.into());
        }
        self
    }

    /// Set the fill opacity applied to every band (default `0.7`).
    ///
    /// Valid range is `0.0` (fully transparent) to `1.0` (fully opaque).
    /// Lower values let the background grid lines show through the bands;
    /// `1.0` gives solid fills.
    pub fn with_fill_opacity(mut self, opacity: f64) -> Self {
        self.fill_opacity = opacity;
        self
    }

    /// Set the stroke width for the top-edge line on each band (default `1.5`).
    ///
    /// Has no effect when [`with_strokes(false)`](Self::with_strokes) is set.
    pub fn with_stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    /// Show or hide the stroke drawn along the top edge of each band (default `true`).
    ///
    /// Setting `false` produces flat, borderless bands — useful when the color
    /// contrast between adjacent bands is sufficient to distinguish them without outlines.
    pub fn with_strokes(mut self, show: bool) -> Self {
        self.show_strokes = show;
        self
    }

    /// Enable 100 % percent-stacking.
    ///
    /// Each column is normalised so all series sum to 100 % at every x value.
    /// The y-axis is rescaled to span 0–100 %. Use this when you want to
    /// emphasise proportional composition rather than absolute magnitude.
    ///
    /// ```rust,no_run
    /// # use visus::plot::StackedAreaPlot;
    /// let sa = StackedAreaPlot::new()
    ///     .with_x([1.0, 2.0, 3.0])
    ///     .with_series([30.0, 40.0, 35.0]).with_legend("A")
    ///     .with_series([20.0, 15.0, 25.0]).with_legend("B")
    ///     .with_normalized();
    /// ```
    pub fn with_normalized(mut self) -> Self {
        self.normalized = true;
        self
    }

    /// Set the corner of the plot area where the legend box is placed.
    ///
    /// The four variants are `TopRight` (default), `TopLeft`, `BottomRight`,
    /// and `BottomLeft` from [`LegendPosition`].
    ///
    /// ```rust,no_run
    /// use visus::plot::{StackedAreaPlot, LegendPosition};
    /// let sa = StackedAreaPlot::new()
    ///     .with_x([1.0, 2.0, 3.0])
    ///     .with_series([10.0, 20.0, 15.0]).with_legend("A")
    ///     .with_legend_position(LegendPosition::BottomLeft);
    /// ```
    pub fn with_legend_position(mut self, pos: LegendPosition) -> Self {
        self.legend_position = pos;
        self
    }

    /// Resolve the display color for series `k`, falling back to a built-in palette.
    pub fn resolve_color(&self, k: usize) -> &str {
        if let Some(Some(ref c)) = self.colors.get(k) {
            c.as_str()
        } else {
            DEFAULT_COLORS[k % DEFAULT_COLORS.len()]
        }
    }
}
