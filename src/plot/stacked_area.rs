use crate::plot::legend::LegendPosition;

const DEFAULT_COLORS: &[&str] = &[
    "steelblue", "orange", "green", "red", "purple",
    "brown", "pink", "gray",
];

#[derive(Clone)]
pub struct StackedAreaPlot {
    pub x: Vec<f64>,
    pub series: Vec<Vec<f64>>,
    pub colors: Vec<Option<String>>,
    pub labels: Vec<Option<String>>,
    pub fill_opacity: f64,
    pub stroke_width: f64,
    pub show_strokes: bool,
    pub normalized: bool,
    pub legend_position: LegendPosition,
}

impl StackedAreaPlot {
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

    pub fn with_x<T, I>(mut self, x: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<f64>,
    {
        self.x = x.into_iter().map(Into::into).collect();
        self
    }

    /// Append a new series. Use `.with_color()` and `.with_legend()` after this
    /// to configure the series that was just added.
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

    /// Set the color of the most recently added series.
    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        if let Some(last) = self.colors.last_mut() {
            *last = Some(color.into());
        }
        self
    }

    /// Set the legend label of the most recently added series.
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        if let Some(last) = self.labels.last_mut() {
            *last = Some(label.into());
        }
        self
    }

    pub fn with_fill_opacity(mut self, opacity: f64) -> Self {
        self.fill_opacity = opacity;
        self
    }

    pub fn with_stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    pub fn with_strokes(mut self, show: bool) -> Self {
        self.show_strokes = show;
        self
    }

    /// Enable percent-stacking: y-axis spans 0–100%.
    pub fn with_normalized(mut self) -> Self {
        self.normalized = true;
        self
    }

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
