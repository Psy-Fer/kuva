
pub enum StripStyle {
    /// Random x-jitter; jitter is half-width as fraction of slot width.
    Strip { jitter: f64 },
    /// Deterministic non-overlapping beeswarm layout.
    Swarm,
    /// All points at group center (no jitter).
    Center,
}

pub struct StripGroup {
    pub label: String,
    pub values: Vec<f64>,
}

pub struct StripPlot {
    pub groups: Vec<StripGroup>,
    pub color: String,
    pub point_size: f64,
    pub style: StripStyle,
    pub seed: u64,
    pub legend_label: Option<String>,
}

impl StripPlot {
    pub fn new() -> Self {
        Self {
            groups: vec![],
            color: "steelblue".into(),
            point_size: 4.0,
            style: StripStyle::Strip { jitter: 0.3 },
            seed: 42,
            legend_label: None,
        }
    }

    pub fn with_group<S, I>(mut self, label: S, values: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator,
        I::Item: Into<f64>,
    {
        self.groups.push(StripGroup {
            label: label.into(),
            values: values.into_iter().map(Into::into).collect(),
        });
        self
    }

    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_point_size(mut self, size: f64) -> Self {
        self.point_size = size;
        self
    }

    /// Set jitter half-width as a fraction of the slot width and use strip layout.
    pub fn with_jitter(mut self, jitter: f64) -> Self {
        self.style = StripStyle::Strip { jitter };
        self
    }

    /// Use beeswarm (non-overlapping) layout.
    pub fn with_swarm(mut self) -> Self {
        self.style = StripStyle::Swarm;
        self
    }

    /// Place all points at the group center (no jitter).
    pub fn with_center(mut self) -> Self {
        self.style = StripStyle::Center;
        self
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }
}
