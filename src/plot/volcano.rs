pub enum LabelStyle {
    /// Label placed at the exact point position (no nudge, no leader line).
    Exact,
    /// Labels sorted by x, nudged vertically to avoid stacking (default).
    Nudge,
    /// Label offset by (offset_x, offset_y) px with a short leader line to the point.
    Arrow { offset_x: f64, offset_y: f64 },
}

impl Default for LabelStyle {
    fn default() -> Self { LabelStyle::Nudge }
}

pub struct VolcanoPoint {
    pub name: String,
    pub log2fc: f64,
    pub pvalue: f64,
}

pub struct VolcanoPlot {
    pub points: Vec<VolcanoPoint>,
    pub fc_cutoff: f64,
    pub p_cutoff: f64,
    pub color_up: String,
    pub color_down: String,
    pub color_ns: String,
    pub point_size: f64,
    pub label_top: usize,
    pub label_style: LabelStyle,
    pub pvalue_floor: Option<f64>,
    pub legend_label: Option<String>,
}

impl VolcanoPlot {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            fc_cutoff: 1.0,
            p_cutoff: 0.05,
            color_up: "firebrick".into(),
            color_down: "steelblue".into(),
            color_ns: "#aaaaaa".into(),
            point_size: 3.0,
            label_top: 0,
            label_style: LabelStyle::default(),
            pvalue_floor: None,
            legend_label: None,
        }
    }

    /// Compute the p-value floor used for -log10 transformation.
    /// Uses explicit floor if set, otherwise finds minimum non-zero p-value.
    pub fn floor(&self) -> f64 {
        if let Some(f) = self.pvalue_floor { return f; }
        self.points.iter()
            .map(|p| p.pvalue)
            .filter(|&p| p > 0.0)
            .fold(f64::INFINITY, f64::min)
            .max(1e-300)
    }

    pub fn with_point<S, F, G>(mut self, name: S, log2fc: F, pvalue: G) -> Self
    where
        S: Into<String>,
        F: Into<f64>,
        G: Into<f64>,
    {
        self.points.push(VolcanoPoint {
            name: name.into(),
            log2fc: log2fc.into(),
            pvalue: pvalue.into(),
        });
        self
    }

    pub fn with_points<I, S, F, G>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (S, F, G)>,
        S: Into<String>,
        F: Into<f64>,
        G: Into<f64>,
    {
        for (name, log2fc, pvalue) in iter {
            self.points.push(VolcanoPoint {
                name: name.into(),
                log2fc: log2fc.into(),
                pvalue: pvalue.into(),
            });
        }
        self
    }

    pub fn with_fc_cutoff(mut self, cutoff: f64) -> Self {
        self.fc_cutoff = cutoff;
        self
    }

    pub fn with_p_cutoff(mut self, cutoff: f64) -> Self {
        self.p_cutoff = cutoff;
        self
    }

    pub fn with_color_up<S: Into<String>>(mut self, color: S) -> Self {
        self.color_up = color.into();
        self
    }

    pub fn with_color_down<S: Into<String>>(mut self, color: S) -> Self {
        self.color_down = color.into();
        self
    }

    pub fn with_color_ns<S: Into<String>>(mut self, color: S) -> Self {
        self.color_ns = color.into();
        self
    }

    pub fn with_point_size(mut self, size: f64) -> Self {
        self.point_size = size;
        self
    }

    pub fn with_label_top(mut self, n: usize) -> Self {
        self.label_top = n;
        self
    }

    pub fn with_label_style(mut self, style: LabelStyle) -> Self {
        self.label_style = style;
        self
    }

    pub fn with_pvalue_floor(mut self, floor: f64) -> Self {
        self.pvalue_floor = Some(floor);
        self
    }

    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }
}
