
use crate::plot::strip::StripStyle;

pub struct BoxPlot {
    pub groups: Vec<BoxGroup>,
    pub color: String,
    pub width: f64,
    pub legend_label: Option<String>,
    pub overlay: Option<StripStyle>,
    pub overlay_color: String,
    pub overlay_size: f64,
    pub overlay_seed: u64,
}

pub struct BoxGroup {
    pub label: String,
    pub values: Vec<f64>,
}

impl BoxPlot {
    pub fn new() -> Self {
        Self {
            groups: vec![],
            color: "black".into(),
            width: 0.8,
            legend_label: None,
            overlay: None,
            overlay_color: "rgba(0,0,0,0.45)".into(),
            overlay_size: 3.0,
            overlay_seed: 42,
        }
    }

    pub fn with_group<T, U, I>(mut self, label: T, values: I) -> Self
    where
        T: Into<String>,
        I: IntoIterator<Item = U>,
        U: Into<f64>,
    {
        self.groups.push(BoxGroup {
            label: label.into(),
            values: values.into_iter().map(Into::into).collect(),
        });
        self
    }

    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    pub fn with_strip(mut self, jitter: f64) -> Self {
        self.overlay = Some(StripStyle::Strip { jitter });
        self
    }

    pub fn with_swarm_overlay(mut self) -> Self {
        self.overlay = Some(StripStyle::Swarm);
        self
    }

    pub fn with_overlay_color<S: Into<String>>(mut self, color: S) -> Self {
        self.overlay_color = color.into();
        self
    }

    pub fn with_overlay_size(mut self, size: f64) -> Self {
        self.overlay_size = size;
        self
    }
}
