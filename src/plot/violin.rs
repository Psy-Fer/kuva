

pub struct ViolinPlot {
    pub groups: Vec<ViolinGroup>,
    pub color: String,
    pub width: f64,
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

    pub fn with_color<T: Into<String>>(mut self, color: T) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }
}
