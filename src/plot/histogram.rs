
#[derive(Debug, Clone)]

pub struct Histogram {
    pub data: Vec<f64>,
    pub bins: usize,
    pub range: Option<(f64, f64)>,
    pub color: String,
    pub normalize: bool,
}

impl Histogram {
    pub fn new() -> Self {
        Self {
            data: vec![],
            bins: 10,
            range: None,
            color: "black".to_string(),
            normalize: false,
        }
    }

    pub fn with_data<T, I>(mut self, data: I) -> Self 
    where 
        I: IntoIterator<Item = T>,
        T: Into<f64>,
    {
        self.data = data.into_iter().map(|x| x.into()).collect();

        self
    }

    pub fn with_bins(mut self, bins: usize) -> Self {
        self.bins = bins;
        self
    }

    pub fn with_range(mut self, range: (f64, f64)) -> Self {
        self.range = Some(range);
        self
    }

    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    pub fn normalize(mut self) -> Self {
        self.normalize = true;
        self
    }
}
