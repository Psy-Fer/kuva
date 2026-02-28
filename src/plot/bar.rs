/// Builder for a bar chart.
///
/// Supports three modes depending on how data is added:
///
/// - **Simple** — one bar per category via [`.with_bar()`](BarPlot::with_bar) or
///   [`.with_bars()`](BarPlot::with_bars).
/// - **Grouped** — multiple side-by-side bars per category via
///   [`.with_group()`](BarPlot::with_group).
/// - **Stacked** — bars stacked vertically via `.with_group()` +
///   [`.with_stacked()`](BarPlot::with_stacked).
///
/// # Simple example
///
/// ```rust,no_run
/// use kuva::plot::BarPlot;
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let plot = BarPlot::new()
///     .with_bars(vec![("Apples", 42.0), ("Bananas", 58.0), ("Cherries", 31.0)])
///     .with_color("steelblue");
///
/// let plots = vec![Plot::Bar(plot)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Fruit Counts")
///     .with_x_label("Fruit")
///     .with_y_label("Count");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("bar.svg", svg).unwrap();
/// ```
pub struct BarPlot {
    pub groups: Vec<BarGroup>,
    pub width: f64,
    pub legend_label: Option<Vec<String>>,
    pub stacked: bool,
}

/// A single category group containing one or more bars.
///
/// In simple mode each group holds one `BarValue`. In grouped and stacked
/// modes each group holds one `BarValue` per series.
#[derive(Debug, Clone)]
pub struct BarGroup {
    pub label: String,
    pub bars: Vec<BarValue>,
}

/// A single bar segment with a value and a fill color.
#[derive(Debug, Clone)]
pub struct BarValue {
    pub value: f64,
    pub color: String,
}

impl Default for BarPlot {
    fn default() -> Self { Self::new() }
}

impl BarPlot {
    /// Create a bar plot with default settings.
    ///
    /// Default bar width is `0.8` (as a fraction of the available slot).
    pub fn new() -> Self {
        Self {
            groups: vec![],
            width: 0.8,
            legend_label: None,
            stacked: false,
        }
    }

    /// Add a group of bars for one category (grouped / stacked mode).
    ///
    /// Each item in `values` is a `(value, color)` pair — one per series.
    /// Call this once per x-axis category. Pair with
    /// [`.with_legend()`](Self::with_legend) to label the series.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::BarPlot;
    /// let plot = BarPlot::new()
    ///     .with_group("Jan", vec![(10.0, "steelblue"), (7.0, "crimson")])
    ///     .with_group("Feb", vec![(13.0, "steelblue"), (9.0, "crimson")])
    ///     .with_legend(vec!["Series A", "Series B"]);
    /// ```
    pub fn with_group<T: Into<String>>(mut self, label: T, values: Vec<(f64, &str)>) -> Self {
        let bars = values
                        .into_iter()
                        .map(|(v, c)| BarValue {
                            value: v,
                            color: c.into(),
                        })
                        .collect();

        self.groups.push(BarGroup {
                        label: label.into(),
                        bars,
                    });
        self
    }

    /// Set legend labels for each series (one per bar within a group).
    ///
    /// Must be called after the groups are defined so the label count
    /// matches the number of bars per group.
    pub fn with_legend(mut self, legend: Vec<&str>) -> Self {
        self.legend_label = Some(legend.into_iter()
                                 .map(|l| l.into())
                                .collect());
        self
    }

    /// Set the bar width as a fraction of the available category slot (default `0.8`).
    ///
    /// Values between `0.0` and `1.0`. A width of `1.0` means bars touch.
    pub fn with_width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// Add a single bar (simple mode).
    ///
    /// The bar is colored with the library default (`"steelblue"`). Use
    /// [`.with_color()`](Self::with_color) afterwards to change all bars at once.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::BarPlot;
    /// let plot = BarPlot::new()
    ///     .with_bar("A", 3.2)
    ///     .with_bar("B", 4.7)
    ///     .with_bar("C", 2.8)
    ///     .with_color("steelblue");
    /// ```
    pub fn with_bar<T: Into<String>>(mut self, label: T, value: f64) -> Self {
        let color = self.default_color();
        let l = label.into();
        self.groups.push(BarGroup {
            label: l.clone(),
            bars: vec![BarValue { value, color }],
        });

        self
    }

    /// Add multiple bars at once (simple mode).
    ///
    /// Equivalent to calling [`.with_bar()`](Self::with_bar) for each item.
    /// Chain [`.with_color()`](Self::with_color) to set a uniform color.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::BarPlot;
    /// let plot = BarPlot::new()
    ///     .with_bars(vec![("A", 3.2), ("B", 4.7), ("C", 2.8)])
    ///     .with_color("steelblue");
    /// ```
    pub fn with_bars<T: Into<String>>(mut self, data: Vec<(T, f64)>) -> Self {
        let color = self.default_color();
        for (label, value) in data.into_iter() {
            self.groups.push(BarGroup {
                label: label.into(),
                bars: vec![BarValue { value, color: color.clone() }],
            });
        }
        self
    }

    /// Set a uniform color for all bars added so far.
    ///
    /// Overwrites the color on every existing bar. In simple mode, call
    /// this after [`.with_bar()`](Self::with_bar) /
    /// [`.with_bars()`](Self::with_bars). Not needed in grouped/stacked
    /// mode, where colors are set per-value in
    /// [`.with_group()`](Self::with_group).
    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        let c = color.into();
        for group in &mut self.groups {
            for bar in &mut group.bars {
                bar.color = c.clone();
            }
        }
        self
    }

    /// Enable stacked mode.
    ///
    /// Instead of placing bars side-by-side, segments are stacked
    /// vertically within each category. Requires groups to be defined
    /// with [`.with_group()`](Self::with_group).
    pub fn with_stacked(mut self) -> Self {
        self.stacked = true;
        self
    }

    fn default_color(&self) -> String {
        "steelblue".into()
    }
}
