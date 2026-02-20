

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PieLabelPosition {
    Inside,   // labels centered between inner/outer radius
    Outside,  // labels outside with leader lines
    Auto,     // inside for large slices, outside for small ones
    None,     // no slice labels
}

/// The full pie
#[derive(Debug, Clone)]
pub struct PiePlot {
    pub slices: Vec<PieSlice>,
    pub inner_radius: f64, // 0.0 = full pie, >0.0 = donut
    pub legend_label: Option<String>,
    pub label_position: PieLabelPosition,
    pub show_percent: bool,
    pub min_label_fraction: f64,
}

/// each slice of the pie
#[derive(Debug, Clone)]
pub struct PieSlice {
    pub label: String,
    pub value: f64,
    pub color: String,
}

impl PiePlot {
    pub fn new() -> Self {
        Self {
            slices: vec![],
            inner_radius: 0.0,
            legend_label: None,
            label_position: PieLabelPosition::Auto,
            show_percent: false,
            min_label_fraction: 0.05,
        }
    }

    pub fn with_slice<L, V, C>(mut self, label: L, value: V, color: C) -> Self
    where
        L: Into<String>,
        V: Into<f64>,
        C: Into<String>,
    {
        self.slices.push(PieSlice {
            label: label.into(),
            value: value.into(),
            color: color.into(),
        });
        self
    }

    pub fn with_inner_radius(mut self, r: f64) -> Self {
        self.inner_radius = r;
        self
    }

    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    pub fn with_label_position(mut self, pos: PieLabelPosition) -> Self {
        self.label_position = pos;
        self
    }

    pub fn with_percent(mut self) -> Self {
        self.show_percent = true;
        self
    }

    pub fn with_min_label_fraction(mut self, fraction: f64) -> Self {
        self.min_label_fraction = fraction;
        self
    }
}
