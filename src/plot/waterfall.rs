pub enum WaterfallKind {
    /// Floating bar from running total; sign determines color.
    Delta,
    /// Bar from zero to current running total; value field ignored.
    Total,
    /// Bar anchored to explicit [from, to] values; does not affect the running
    /// total.  Green when to > from, red when to < from.
    Difference { from: f64, to: f64 },
}

pub struct WaterfallBar {
    pub label: String,
    pub value: f64,
    pub kind: WaterfallKind,
}

pub struct WaterfallPlot {
    pub bars: Vec<WaterfallBar>,
    pub bar_width: f64,
    pub color_positive: String,
    pub color_negative: String,
    pub color_total: String,
    pub show_connectors: bool,
    pub show_values: bool,
    pub legend_label: Option<String>,
}

impl WaterfallPlot {
    pub fn new() -> Self {
        Self {
            bars: Vec::new(),
            bar_width: 0.6,
            color_positive: "rgb(68,170,68)".into(),
            color_negative: "rgb(204,68,68)".into(),
            color_total: "steelblue".into(),
            show_connectors: false,
            show_values: false,
            legend_label: None,
        }
    }

    pub fn with_delta<S: Into<String>>(mut self, label: S, value: f64) -> Self {
        self.bars.push(WaterfallBar {
            label: label.into(),
            value,
            kind: WaterfallKind::Delta,
        });
        self
    }

    /// Add a bar anchored to explicit y-values rather than the running total.
    /// The bar spans [from, to] and is colored green (to > from) or red (to < from).
    /// The running total is unaffected.
    pub fn with_difference<S: Into<String>>(mut self, label: S, from: f64, to: f64) -> Self {
        self.bars.push(WaterfallBar {
            label: label.into(),
            value: 0.0,
            kind: WaterfallKind::Difference { from, to },
        });
        self
    }

    pub fn with_total<S: Into<String>>(mut self, label: S) -> Self {
        self.bars.push(WaterfallBar {
            label: label.into(),
            value: 0.0,
            kind: WaterfallKind::Total,
        });
        self
    }

    pub fn with_bar_width(mut self, width: f64) -> Self {
        self.bar_width = width;
        self
    }

    pub fn with_color_positive<S: Into<String>>(mut self, color: S) -> Self {
        self.color_positive = color.into();
        self
    }

    pub fn with_color_negative<S: Into<String>>(mut self, color: S) -> Self {
        self.color_negative = color.into();
        self
    }

    pub fn with_color_total<S: Into<String>>(mut self, color: S) -> Self {
        self.color_total = color.into();
        self
    }

    pub fn with_connectors(mut self) -> Self {
        self.show_connectors = true;
        self
    }

    pub fn with_values(mut self) -> Self {
        self.show_values = true;
        self
    }

    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }
}
