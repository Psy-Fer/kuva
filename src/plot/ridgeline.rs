/// Builder for a ridgeline (joyplot) — stacked KDE density curves.
///
/// Groups are rendered as horizontal ridges stacked vertically. The y-axis
/// shows group labels; the x-axis is the continuous data range.
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::ridgeline::{RidgelinePlot, RidgelineGroup};
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let plot = RidgelinePlot::new()
///     .with_group("Group A", vec![1.0, 1.5, 2.0, 2.5, 2.2, 1.8])
///     .with_group("Group B", vec![3.0, 3.5, 4.0, 4.5, 4.2, 3.8]);
///
/// let plots = vec![Plot::Ridgeline(plot)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_x_label("Value")
///     .with_y_label("Group");
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("ridgeline.svg", svg).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct RidgelinePlot {
    pub groups: Vec<RidgelineGroup>,
    pub filled: bool,
    pub opacity: f64,
    pub bandwidth: Option<f64>,
    pub kde_samples: usize,
    pub stroke_width: f64,
    pub overlap: f64,
    pub normalize: bool,
    pub show_legend: bool,
    pub line_dash: Option<String>,
    /// Draw a thin horizontal baseline at each group's zero-density level.
    ///
    /// The baseline spans the full plot width and makes it easy to associate
    /// each overlapping ridge with its y-axis category label.  Default: `true`.
    pub show_baseline: bool,
}

#[derive(Debug, Clone)]
pub struct RidgelineGroup {
    pub label: String,
    pub values: Vec<f64>,
    pub color: Option<String>,
}

impl Default for RidgelinePlot {
    fn default() -> Self { Self::new() }
}

impl RidgelinePlot {
    pub fn new() -> Self {
        Self {
            groups: vec![],
            filled: true,
            opacity: 0.7,
            bandwidth: None,
            kde_samples: 200,
            stroke_width: 1.5,
            overlap: 0.5,
            normalize: false,
            show_legend: false,
            line_dash: None,
            show_baseline: true,
        }
    }

    /// Append a group with the given label and data values.
    pub fn with_group<S, T, I>(mut self, label: S, data: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = T>,
        T: Into<f64>,
    {
        self.groups.push(RidgelineGroup {
            label: label.into(),
            values: data.into_iter().map(|x| x.into()).collect(),
            color: None,
        });
        self
    }

    /// Append a group with an explicit color override.
    pub fn with_group_color<S, C, T, I>(mut self, label: S, data: I, color: C) -> Self
    where
        S: Into<String>,
        C: Into<String>,
        I: IntoIterator<Item = T>,
        T: Into<f64>,
    {
        self.groups.push(RidgelineGroup {
            label: label.into(),
            values: data.into_iter().map(|x| x.into()).collect(),
            color: Some(color.into()),
        });
        self
    }

    /// Add multiple groups at once (no explicit colors).
    pub fn with_groups<S, T, I, II>(mut self, groups: II) -> Self
    where
        S: Into<String>,
        T: Into<f64>,
        I: IntoIterator<Item = T>,
        II: IntoIterator<Item = (S, I)>,
    {
        for (label, data) in groups {
            self.groups.push(RidgelineGroup {
                label: label.into(),
                values: data.into_iter().map(|x| x.into()).collect(),
                color: None,
            });
        }
        self
    }

    /// Show or hide the horizontal baseline drawn at each group's zero-density level (default `true`).
    pub fn with_baseline(mut self, show: bool) -> Self { self.show_baseline = show; self }
    pub fn with_filled(mut self, filled: bool) -> Self { self.filled = filled; self }
    pub fn with_opacity(mut self, opacity: f64) -> Self { self.opacity = opacity; self }
    pub fn with_bandwidth(mut self, bw: f64) -> Self { self.bandwidth = Some(bw); self }
    pub fn with_kde_samples(mut self, samples: usize) -> Self { self.kde_samples = samples; self }
    pub fn with_stroke_width(mut self, width: f64) -> Self { self.stroke_width = width; self }
    pub fn with_overlap(mut self, overlap: f64) -> Self { self.overlap = overlap; self }
    pub fn with_normalize(mut self, normalize: bool) -> Self { self.normalize = normalize; self }
    pub fn with_legend(mut self, show: bool) -> Self { self.show_legend = show; self }
    pub fn with_line_dash<S: Into<String>>(mut self, dash: S) -> Self { self.line_dash = Some(dash.into()); self }
}
