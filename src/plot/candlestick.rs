pub struct CandleDataPoint {
    pub label: String,
    pub x: Option<f64>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: Option<f64>,
}

pub struct CandlestickPlot {
    pub candles: Vec<CandleDataPoint>,
    pub candle_width: f64,
    pub wick_width: f64,
    pub color_up: String,
    pub color_down: String,
    pub color_doji: String,
    pub show_volume: bool,
    pub volume_ratio: f64,
    pub legend_label: Option<String>,
}

impl CandlestickPlot {
    pub fn new() -> Self {
        Self {
            candles: Vec::new(),
            candle_width: 0.7,
            wick_width: 1.5,
            color_up: "rgb(68,170,68)".into(),
            color_down: "rgb(204,68,68)".into(),
            color_doji: "#888888".into(),
            show_volume: false,
            volume_ratio: 0.22,
            legend_label: None,
        }
    }

    pub fn with_candle<S: Into<String>>(
        mut self,
        label: S,
        open: impl Into<f64>,
        high: impl Into<f64>,
        low: impl Into<f64>,
        close: impl Into<f64>,
    ) -> Self {
        self.candles.push(CandleDataPoint {
            label: label.into(),
            x: None,
            open: open.into(),
            high: high.into(),
            low: low.into(),
            close: close.into(),
            volume: None,
        });
        self
    }

    pub fn with_candle_at<S: Into<String>>(
        mut self,
        x: f64,
        label: S,
        open: impl Into<f64>,
        high: impl Into<f64>,
        low: impl Into<f64>,
        close: impl Into<f64>,
    ) -> Self {
        self.candles.push(CandleDataPoint {
            label: label.into(),
            x: Some(x),
            open: open.into(),
            high: high.into(),
            low: low.into(),
            close: close.into(),
            volume: None,
        });
        self
    }

    pub fn with_volume<T, I>(mut self, volumes: I) -> Self
    where
        T: Into<f64>,
        I: IntoIterator<Item = T>,
    {
        for (candle, vol) in self.candles.iter_mut().zip(volumes.into_iter()) {
            candle.volume = Some(vol.into());
        }
        self
    }

    pub fn with_volume_panel(mut self) -> Self {
        self.show_volume = true;
        self
    }

    pub fn with_volume_ratio(mut self, ratio: f64) -> Self {
        self.volume_ratio = ratio;
        self
    }

    pub fn with_candle_width(mut self, width: f64) -> Self {
        self.candle_width = width;
        self
    }

    pub fn with_wick_width(mut self, width: f64) -> Self {
        self.wick_width = width;
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

    pub fn with_color_doji<S: Into<String>>(mut self, color: S) -> Self {
        self.color_doji = color.into();
        self
    }

    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }
}
