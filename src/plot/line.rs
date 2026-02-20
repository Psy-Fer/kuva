

#[derive(Debug, Clone, Copy)]
pub struct ScatterPoint {
    pub x: f64,
    pub y: f64,
    pub x_err: Option<(f64, f64)>, // (negative, positive)
    pub y_err: Option<(f64, f64)>,
}

impl From<&ScatterPoint> for (f64, f64) {
    fn from(p: &ScatterPoint) -> (f64, f64) {
        (p.x, p.y)
    }
}

impl ScatterPoint {
    pub fn with_y_error(mut self, err: f64) -> Self {
        self.y_err = Some((err, err));
        self
    }

    pub fn with_y_error_asymmetric(mut self, neg: f64, pos: f64) -> Self {
        self.y_err = Some((neg, pos));
        self
    }
}


use crate::plot::band::BandPlot;

#[derive(Debug, Clone,)]
pub struct LinePlot {
    pub data: Vec<ScatterPoint>,
    pub color: String,
    pub stroke_width: f64,
    pub legend_label: Option<String>,
    pub band: Option<BandPlot>,
}

impl LinePlot {
    pub fn new() -> Self {
        Self {
            data: vec![],
            color: "black".into(),
            stroke_width: 2.0,
            legend_label: None,
            band: None,
        }
    }

    // accept data of any numerical type and push it to f64
    pub fn with_data<T, U, I>(mut self, points: I) -> Self
    where
        I: IntoIterator<Item = (T, U)>,
        T: Into<f64>,
        U: Into<f64>,
    {
        self.data = points
            .into_iter()
            .map(|(x, y)| ScatterPoint {
                x: x.into(),
                y: y.into(),
                x_err: None,
                y_err: None,
            })
            .collect();

        self
    }

    // insert symmetric error
    pub fn with_x_err<T, I>(mut self, errors: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<f64> + Copy,
    {
        for (i, err) in errors.into_iter().enumerate() {
            if i < self.data.len() {
                self.data[i].x_err = Some((err.into(), err.into()));
            }
        }

        self
    }

    // insert asymmetric x error
    pub fn with_x_err_asymmetric<T, U, I>(mut self, errors: I) -> Self
    where
    I: IntoIterator<Item = (T, U)>,
    T: Into<f64>,
    U: Into<f64>,
    {
        for (i, (neg, pos)) in errors.into_iter().enumerate() {
            if i < self.data.len() {
                self.data[i].x_err = Some((neg.into(), pos.into()));
            }
        }
        
        self
    }
    
    // insert symmetric y error
    pub fn with_y_err<T, I>(mut self, errors: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<f64> + Copy,
    {
        for (i, err) in errors.into_iter().enumerate() {
            if i < self.data.len() {
                self.data[i].y_err = Some((err.into(), err.into()));
            }
        }

        self
    }

    // insert asymmetric y error
    pub fn with_y_err_asymmetric<T, U, I>(mut self, errors: I) -> Self
    where
        I: IntoIterator<Item = (T, U)>,
        T: Into<f64>,
        U: Into<f64>,
    {
        for (i, (neg, pos)) in errors.into_iter().enumerate() {
            if i < self.data.len() {
                self.data[i].y_err = Some((neg.into(), pos.into()));
            }
        }

        self
    }


    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    pub fn with_band<T, U, I1, I2>(mut self, y_lower: I1, y_upper: I2) -> Self
    where
        I1: IntoIterator<Item = T>,
        I2: IntoIterator<Item = U>,
        T: Into<f64>,
        U: Into<f64>,
    {
        let x: Vec<f64> = self.data.iter().map(|p| p.x).collect();
        let band = BandPlot::new(x, y_lower, y_upper)
            .with_color(self.color.clone());
        self.band = Some(band);
        self
    }
}