

pub struct SeriesPlot {
    pub values: Vec<f64>,
    pub color: String,
    pub style: SeriesStyle, // e.g., Line, Point, Both
    pub legend_label: Option<String>,
}

pub enum SeriesStyle {
    Line,
    Point,
    Both,
}


impl SeriesPlot {
    pub fn new() -> Self {
        Self {
            values: vec![],
            color: "black".into(),
            style: SeriesStyle::Point,
            legend_label: None,
        }
    }

    pub fn with_data<T, I>(mut self, data: I) -> Self 
    where
        I: IntoIterator<Item = T>,
        T: Into<f64>,
    {
        self.values = data.into_iter().map(|x| x.into()).collect();
        self
    }

    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_line_style(mut self) -> Self {
        self.style = SeriesStyle::Line;
        self
    }
    pub fn with_point_style(mut self) -> Self {
        self.style = SeriesStyle::Point;
        self
    }
    pub fn with_line_point_style(mut self) -> Self {
        self.style = SeriesStyle::Both;
        self
    }

    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }
}