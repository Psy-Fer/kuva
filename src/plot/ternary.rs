/// A single data point in a ternary (simplex) coordinate system.
#[derive(Debug, Clone)]
pub struct TernaryPoint {
    /// Top vertex component (A).
    pub a: f64,
    /// Bottom-left vertex component (B).
    pub b: f64,
    /// Bottom-right vertex component (C).
    pub c: f64,
    /// Optional group name for color coding.
    pub group: Option<String>,
}

/// Ternary (simplex) scatter plot.
///
/// Each data point is described by three components (a, b, c) that ideally sum to 1.
/// If they don't, use `.with_normalize(true)` to auto-normalize each row.
///
/// # Example
/// ```rust,no_run
/// use kuva::plot::ternary::TernaryPlot;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
/// use kuva::render::render::render_multiple;
/// use kuva::backend::svg::SvgBackend;
///
/// let plot = TernaryPlot::new()
///     .with_point_group(0.7, 0.2, 0.1, "A-rich")
///     .with_point_group(0.1, 0.7, 0.2, "B-rich")
///     .with_point_group(0.2, 0.1, 0.7, "C-rich")
///     .with_corner_labels("A", "B", "C");
///
/// let plots = vec![Plot::Ternary(plot)];
/// let layout = Layout::auto_from_plots(&plots).with_title("Ternary Plot");
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// ```
#[derive(Debug, Clone)]
pub struct TernaryPlot {
    pub points: Vec<TernaryPoint>,
    /// Labels for the three corners: [top (A), bottom-left (B), bottom-right (C)].
    pub corner_labels: [String; 3],
    /// Auto-normalize each (a,b,c) so a+b+c=1. Default: false.
    pub normalize: bool,
    pub marker_size: f64,
    /// Number of grid divisions per axis. Default 5 (tick at 0%, 20%, 40%, 60%, 80%, 100%).
    pub grid_lines: usize,
    pub show_grid: bool,
    pub show_legend: bool,
    /// Show percentage tick labels on each axis edge. Default: true.
    pub show_percentages: bool,
    /// Fill opacity for markers (0.0 = transparent, 1.0 = solid). `None` = fully opaque.
    pub marker_opacity: Option<f64>,
    /// Stroke (outline) width for markers. `None` = no stroke. Stroke color matches fill.
    pub marker_stroke_width: Option<f64>,
    pub show_tooltips: bool,
    pub tooltip_labels: Option<Vec<String>>,
}

impl Default for TernaryPlot {
    fn default() -> Self {
        TernaryPlot {
            points: Vec::new(),
            corner_labels: ["A".to_string(), "B".to_string(), "C".to_string()],
            normalize: false,
            marker_size: 5.0,
            grid_lines: 5,
            show_grid: true,
            show_legend: false,
            show_percentages: true,
            marker_opacity: None,
            marker_stroke_width: None,
            show_tooltips: false,
            tooltip_labels: None,
        }
    }
}

impl TernaryPlot {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a single ungrouped point.
    pub fn with_point(mut self, a: f64, b: f64, c: f64) -> Self {
        self.points.push(TernaryPoint { a, b, c, group: None });
        self
    }

    /// Add a single point with a group label.
    pub fn with_point_group<S: Into<String>>(mut self, a: f64, b: f64, c: f64, group: S) -> Self {
        self.points.push(TernaryPoint {
            a,
            b,
            c,
            group: Some(group.into()),
        });
        self
    }

    /// Add multiple ungrouped points from an iterator of (a, b, c) tuples.
    pub fn with_points<I: IntoIterator<Item = (f64, f64, f64)>>(mut self, pts: I) -> Self {
        for (a, b, c) in pts {
            self.points.push(TernaryPoint { a, b, c, group: None });
        }
        self
    }

    /// Set corner labels for the three vertices.
    pub fn with_corner_labels<S: Into<String>>(mut self, top: S, left: S, right: S) -> Self {
        self.corner_labels = [top.into(), left.into(), right.into()];
        self
    }

    /// Auto-normalize each (a,b,c) so a+b+c=1.
    pub fn with_normalize(mut self, normalize: bool) -> Self {
        self.normalize = normalize;
        self
    }

    pub fn with_marker_size(mut self, size: f64) -> Self {
        self.marker_size = size;
        self
    }

    pub fn with_grid_lines(mut self, n: usize) -> Self {
        self.grid_lines = n;
        self
    }

    pub fn with_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    pub fn with_legend(mut self, show: bool) -> Self {
        self.show_legend = show;
        self
    }

    pub fn with_percentages(mut self, show: bool) -> Self {
        self.show_percentages = show;
        self
    }

    /// Set the fill opacity for all markers (0.0 = fully transparent, 1.0 = fully opaque).
    pub fn with_marker_opacity(mut self, opacity: f64) -> Self {
        self.marker_opacity = Some(opacity.clamp(0.0, 1.0));
        self
    }

    /// Draw a solid outline around each marker at the given stroke width.
    ///
    /// Stroke color matches the fill color.
    pub fn with_marker_stroke_width(mut self, width: f64) -> Self {
        self.marker_stroke_width = Some(width);
        self
    }

    /// Return all unique group names in insertion order.
    pub fn unique_groups(&self) -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();
        for pt in &self.points {
            if let Some(ref g) = pt.group {
                if seen.insert(g.clone()) {
                    result.push(g.clone());
                }
            }
        }
        result
    }

    pub fn with_tooltips(mut self) -> Self {
        self.show_tooltips = true;
        self
    }

    pub fn with_tooltip_labels(mut self, labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tooltip_labels = Some(labels.into_iter().map(|s| s.into()).collect());
        self
    }
}
