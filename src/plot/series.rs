

pub struct SeriesPlot {
    pub values: Vec<f64>,
    pub color: String,
    pub style: SeriesStyle, // e.g., Line, Point, Both
    pub legend_label: Option<String>,
    pub stroke_width: f64,
    pub point_radius: f64,
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
            stroke_width: 2.0,
            point_radius: 3.0,
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

    pub fn with_stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    pub fn with_point_radius(mut self, radius: f64) -> Self {
        self.point_radius = radius;
        self
    }
}