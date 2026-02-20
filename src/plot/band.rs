

#[derive(Debug, Clone)]
pub struct BandPlot {
    pub x: Vec<f64>,
    pub y_lower: Vec<f64>,
    pub y_upper: Vec<f64>,
    pub color: String,
    pub opacity: f64,
    pub legend_label: Option<String>,
}

impl BandPlot {
    pub fn new<T, U, V, I1, I2, I3>(x: I1, y_lower: I2, y_upper: I3) -> Self
    where
        I1: IntoIterator<Item = T>,
        I2: IntoIterator<Item = U>,
        I3: IntoIterator<Item = V>,
        T: Into<f64>,
        U: Into<f64>,
        V: Into<f64>,
    {
        Self {
            x: x.into_iter().map(Into::into).collect(),
            y_lower: y_lower.into_iter().map(Into::into).collect(),
            y_upper: y_upper.into_iter().map(Into::into).collect(),
            color: "steelblue".into(),
            opacity: 0.2,
            legend_label: None,
        }
    }

    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }
}
