/// A single OHLC data point rendered as one candle.
pub struct CandleDataPoint {
    /// Categorical label shown on the x-axis tick (or attached to the candle when
    /// a numeric x position is set via [`CandlestickPlot::with_candle_at`]).
    pub label: String,
    /// Explicit numeric x position. `None` means the candle is placed at its
    /// insertion index (categorical mode).
    pub x: Option<f64>,
    /// Opening price.
    pub open: f64,
    /// Highest price during the period.
    pub high: f64,
    /// Lowest price during the period.
    pub low: f64,
    /// Closing price.
    pub close: f64,
    /// Optional trading volume, used by the volume panel when
    /// [`CandlestickPlot::with_volume_panel`] is enabled.
    pub volume: Option<f64>,
}

/// Builder for a candlestick (OHLC) chart.
///
/// Each candle encodes four values — **open**, **high**, **low**, **close** —
/// for a single period:
///
/// - The **body** spans from open to close. A bullish candle (`close > open`)
///   is filled with [`color_up`](Self::with_color_up) (default green). A
///   bearish candle (`close < open`) is filled with
///   [`color_down`](Self::with_color_down) (default red). A doji
///   (`close == open`) is drawn with [`color_doji`](Self::with_color_doji)
///   (default gray).
/// - The **wicks** are thin vertical lines extending from the body to `high`
///   (upper wick) and `low` (lower wick).
///
/// An optional **volume panel** can be shown below the price chart by
/// attaching volumes with [`with_volume`](Self::with_volume) and enabling the
/// panel with [`with_volume_panel`](Self::with_volume_panel).
///
/// # Categorical vs numeric x-axis
///
/// Two input modes are available:
///
/// - **Categorical** ([`with_candle`](Self::with_candle)): candles are placed
///   at evenly spaced integer positions and the labels are shown as x-axis
///   category ticks.
/// - **Numeric** ([`with_candle_at`](Self::with_candle_at)): each candle is
///   placed at an explicit `f64` x position, enabling uneven spacing and a
///   true numeric x-axis. Useful for quarterly or irregularly spaced data.
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::CandlestickPlot;
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let plot = CandlestickPlot::new()
///     .with_candle("Mon", 100.0, 106.5,  99.2, 105.8)
///     .with_candle("Tue", 105.8, 108.0, 104.1, 104.5)
///     .with_candle("Wed", 104.5, 109.2, 104.0, 108.0)
///     .with_candle("Thu", 108.0, 111.5, 107.3, 110.9)
///     .with_candle("Fri", 110.9, 111.0, 107.8, 108.5)
///     .with_legend("ACME");
///
/// let plots = vec![Plot::Candlestick(plot)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Weekly OHLC")
///     .with_x_label("Day")
///     .with_y_label("Price (USD)");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("candlestick.svg", svg).unwrap();
/// ```
pub struct CandlestickPlot {
    /// All candles in insertion order.
    pub candles: Vec<CandleDataPoint>,
    /// Candle body width as a fraction of the slot width between candles
    /// (default `0.7`; range `0.0`–`1.0`).
    pub candle_width: f64,
    /// Wick stroke width in pixels (default `1.5`).
    pub wick_width: f64,
    /// Fill color for bullish candles (`close > open`). Default green.
    pub color_up: String,
    /// Fill color for bearish candles (`close < open`). Default red.
    pub color_down: String,
    /// Fill color for doji candles (`close == open`). Default `#888888`.
    pub color_doji: String,
    /// Whether to render the volume bar panel below the price chart.
    pub show_volume: bool,
    /// Fraction of the total chart height reserved for the volume panel
    /// (default `0.22`).
    pub volume_ratio: f64,
    /// Optional legend entry label. When set a legend box is drawn inside
    /// the plot area.
    pub legend_label: Option<String>,
}

impl Default for CandlestickPlot {
    fn default() -> Self { Self::new() }
}

impl CandlestickPlot {
    /// Create a candlestick plot with default settings.
    ///
    /// Defaults: candle width `0.7`, wick width `1.5`, green/red/gray colors,
    /// no volume panel, no legend.
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

    /// Append a candle in **categorical** mode.
    ///
    /// The candle is placed at its insertion index and `label` is shown as an
    /// x-axis category tick. Use this when candles are evenly spaced (daily,
    /// weekly data).
    ///
    /// ```rust,no_run
    /// # use kuva::plot::CandlestickPlot;
    /// let plot = CandlestickPlot::new()
    ///     .with_candle("Mon", 100.0, 106.5,  99.2, 105.8)  // open, high, low, close
    ///     .with_candle("Tue", 105.8, 108.0, 104.1, 104.5);
    /// ```
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

    /// Append a candle at an explicit **numeric** x position.
    ///
    /// The candle body is centred at `x` on a continuous numeric x-axis.
    /// Use this when candles are unevenly spaced — for example quarterly data
    /// where `x` is a fractional year — or when the x position carries meaning
    /// beyond a simple sequence index.
    ///
    /// When using this method, call [`with_candle_width`](Self::with_candle_width)
    /// to set an appropriate body width in data units (e.g. `0.15` for
    /// quarterly data spaced `0.25` units apart).
    ///
    /// ```rust,no_run
    /// # use kuva::plot::CandlestickPlot;
    /// let plot = CandlestickPlot::new()
    ///     // x = fractional year; candles spaced 0.25 apart
    ///     .with_candle_at(2023.00, "Q1", 110.0, 118.0, 108.0, 116.0)
    ///     .with_candle_at(2023.25, "Q2", 116.0, 122.0, 114.0, 121.0)
    ///     .with_candle_at(2023.50, "Q3", 121.0, 126.0, 118.5, 119.5)
    ///     .with_candle_width(0.15);
    /// ```
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

    /// Attach volume values to existing candles.
    ///
    /// Values are matched to candles in insertion order. If there are fewer
    /// volume values than candles, the remaining candles receive no volume.
    /// The volume data is not rendered until [`with_volume_panel`](Self::with_volume_panel)
    /// is also called.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::CandlestickPlot;
    /// let plot = CandlestickPlot::new()
    ///     .with_candle("Mon", 100.0, 106.0, 99.0, 105.0)
    ///     .with_candle("Tue", 105.0, 108.0, 104.0, 104.5)
    ///     .with_volume([1_250_000.0, 980_000.0])
    ///     .with_volume_panel();
    /// ```
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

    /// Enable the volume bar panel below the price chart.
    ///
    /// The panel occupies the bottom portion of the chart area (default 22 %).
    /// Requires volume data attached via [`with_volume`](Self::with_volume).
    /// Volume bars are colored to match their candle (green = up, red = down).
    pub fn with_volume_panel(mut self) -> Self {
        self.show_volume = true;
        self
    }

    /// Set the fraction of the total chart height used by the volume panel
    /// (default `0.22`).
    ///
    /// For example `0.30` gives the volume panel 30 % of the chart height and
    /// leaves 70 % for the price chart. Has no effect unless
    /// [`with_volume_panel`](Self::with_volume_panel) is also called.
    pub fn with_volume_ratio(mut self, ratio: f64) -> Self {
        self.volume_ratio = ratio;
        self
    }

    /// Set the candle body width as a fraction of the slot between candles
    /// (default `0.7`).
    ///
    /// In categorical mode the slot width is `1.0` (one index unit), so `0.7`
    /// gives a body that fills 70 % of the available space. In numeric mode
    /// (`with_candle_at`) this value is in data units — set it to be smaller
    /// than the spacing between candles.
    pub fn with_candle_width(mut self, width: f64) -> Self {
        self.candle_width = width;
        self
    }

    /// Set the wick stroke width in pixels (default `1.5`).
    pub fn with_wick_width(mut self, width: f64) -> Self {
        self.wick_width = width;
        self
    }

    /// Set the fill color for bullish candles where `close > open`
    /// (default `"rgb(68,170,68)"` — green).
    ///
    /// Accepts any CSS color string.
    pub fn with_color_up<S: Into<String>>(mut self, color: S) -> Self {
        self.color_up = color.into();
        self
    }

    /// Set the fill color for bearish candles where `close < open`
    /// (default `"rgb(204,68,68)"` — red).
    ///
    /// Accepts any CSS color string.
    pub fn with_color_down<S: Into<String>>(mut self, color: S) -> Self {
        self.color_down = color.into();
        self
    }

    /// Set the fill color for doji candles where `close == open`
    /// (default `"#888888"` — gray).
    ///
    /// A doji typically signals indecision in the market. Accepts any CSS color string.
    pub fn with_color_doji<S: Into<String>>(mut self, color: S) -> Self {
        self.color_doji = color.into();
        self
    }

    /// Add a legend label, causing a legend box to appear inside the plot area.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::CandlestickPlot;
    /// let plot = CandlestickPlot::new()
    ///     .with_candle("Jan", 100.0, 108.0, 98.0, 106.0)
    ///     .with_legend("ACME Corp");
    /// ```
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }
}
