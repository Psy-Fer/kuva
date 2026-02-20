

pub struct ViolinPlot {
    pub groups: Vec<ViolinGroup>,
    pub color: String,
    pub width: f64,
    pub legend_label: Option<String>,
}

pub struct ViolinGroup {
    pub label: String,
    pub values: Vec<f64>,
}

impl ViolinPlot {
    pub fn new() -> Self {
        Self {
            groups: vec![],
            color: "black".into(),
            width: 30.0,
            legend_label: None,
        }
    }

    pub fn with_group<T, U, I>(mut self, label: T, values: I) -> Self 
    where
        T: Into<String>,
        I: IntoIterator<Item = U>,
        U: Into<f64>,
    {
        self.groups.push(ViolinGroup {
            label: label.into(),
            values: values.into_iter().map(|x| x.into()).collect(),
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
}
