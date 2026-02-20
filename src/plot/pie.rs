

/// The full pie
#[derive(Debug, Clone)]
pub struct PiePlot {
    pub slices: Vec<PieSlice>,
    pub inner_radius: f64, // 0.0 = full pie, >0.0 = donut
    pub legend_label: Option<String>,
}

/// each slice of the pie
#[derive(Debug, Clone)]
pub struct PieSlice {
    pub label: String,
    pub value: f64,
    pub color: String,
}

/// TODO: add a builder method to move labels in or out of pie
/// TODO: move the title into the middle of the donut
/// TODO: when the slice gets really small, push label to a legend
/// TODO: Add % to the labels
/// TODO: add background colours to the labels
impl PiePlot {
    pub fn new() -> Self {
        Self {
            slices: vec![],
            inner_radius: 0.0,
            legend_label: None,
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
}
