use crate::render::render_utils::NanCmp;

/// Builder for a Pareto chart: a bar chart of category values, sorted descending
/// by default, with a superimposed cumulative-percentage line on a secondary
/// Y-axis (fixed 0-100%). Common in QC, variant analysis, and error-categorisation
/// work — the "80/20 rule" chart.
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::ParetoPlot;
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let plot = ParetoPlot::new()
///     .with_categories(vec![
///         ("Missing field", 42.0),
///         ("Typo", 31.0),
///         ("Timeout", 18.0),
///         ("Other", 9.0),
///     ])
///     .with_legend("Count", "Cumulative %");
///
/// let plots = vec![Plot::Pareto(plot)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Error Categories")
///     .with_y_label("Count");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("pareto.svg", svg).unwrap();
/// ```
#[derive(Clone, Debug)]
pub struct ParetoPlot {
    pub categories: Vec<ParetoCategory>,
    /// Bar fill color. Default `"steelblue"`.
    pub color: String,
    /// Cumulative-percentage line color. Default `"firebrick"`.
    pub line_color: String,
    /// Bar width as a fraction of the category slot (default `0.8`).
    pub width: f64,
    /// Sort categories descending by value before rendering (default `true`,
    /// the classic Pareto presentation). Set `false` to preserve insertion order.
    pub sorted: bool,
    /// Draw a percentage label above each cumulative-line point. Default `false`.
    pub show_cumulative_labels: bool,
    /// Draw a horizontal dashed reference line at `threshold`. Default `true`.
    pub show_threshold: bool,
    /// Reference line value, as a cumulative percentage (default `80.0`).
    pub threshold: f64,
    /// Legend label for the bars. Defaults to `Some("Value")` — bars and the
    /// cumulative line are two encodings that always coexist, so unlike most
    /// other plot types a legend shows by default (see `show_legend`).
    pub bar_legend_label: Option<String>,
    /// Legend label for the cumulative line. Defaults to `Some("Cumulative %")`.
    pub line_legend_label: Option<String>,
    /// Whether to show the legend at all (default `true`).
    pub show_legend: bool,
    /// Collapse categories beyond this count into one stacked "Other" bar (see
    /// [`with_max_categories`](Self::with_max_categories)). `None` (default)
    /// disables bucketing.
    pub max_categories: Option<usize>,
    /// Label for the bucketed bar when `max_categories` is set (default `"Other"`).
    pub other_label: String,
    /// Render categories on the Y-axis and values on the X-axis (default `false`).
    /// The cumulative-% line moves to a secondary X-axis drawn on top, since the
    /// secondary axis always pairs with whichever axis carries *values*.
    pub horizontal: bool,
}

/// A single Pareto chart category: a label and its (non-cumulative) value.
#[derive(Clone, Debug)]
pub struct ParetoCategory {
    pub label: String,
    pub value: f64,
}

/// One rendered bar slot: either a single category, or a bucketed "Other" made
/// from the categories beyond `max_categories`. The bucket renders as a
/// *stacked* bar of its constituent categories (decoded via legend entries)
/// rather than silently collapsing them into one opaque total — see
/// [`ParetoPlot::with_max_categories`].
#[derive(Clone, Debug)]
pub enum ParetoBar {
    Single(ParetoCategory),
    Bucketed {
        label: String,
        segments: Vec<ParetoCategory>,
    },
}

impl ParetoBar {
    pub fn label(&self) -> &str {
        match self {
            ParetoBar::Single(c) => &c.label,
            ParetoBar::Bucketed { label, .. } => label,
        }
    }

    /// Total height of this bar slot (sum of its segments for a bucketed bar).
    pub fn value(&self) -> f64 {
        match self {
            ParetoBar::Single(c) => c.value,
            ParetoBar::Bucketed { segments, .. } => segments.iter().map(|c| c.value).sum(),
        }
    }
}

impl Default for ParetoPlot {
    fn default() -> Self {
        Self::new()
    }
}

impl ParetoPlot {
    /// Create a Pareto chart with default settings.
    pub fn new() -> Self {
        Self {
            categories: vec![],
            color: "steelblue".into(),
            line_color: "firebrick".into(),
            width: 0.8,
            sorted: true,
            show_cumulative_labels: false,
            show_threshold: true,
            threshold: 80.0,
            bar_legend_label: Some("Value".to_string()),
            line_legend_label: Some("Cumulative %".to_string()),
            show_legend: true,
            max_categories: None,
            other_label: "Other".to_string(),
            horizontal: false,
        }
    }

    /// Add a single category.
    pub fn with_category<T: Into<String>>(mut self, label: T, value: impl Into<f64>) -> Self {
        self.categories.push(ParetoCategory {
            label: label.into(),
            value: value.into(),
        });
        self
    }

    /// Add multiple categories at once. Each item is a `(label, value)` pair.
    pub fn with_categories<T, V, I>(mut self, data: I) -> Self
    where
        T: Into<String>,
        V: Into<f64>,
        I: IntoIterator<Item = (T, V)>,
    {
        for (label, value) in data {
            self.categories.push(ParetoCategory {
                label: label.into(),
                value: value.into(),
            });
        }
        self
    }

    /// Set the bar fill color (default `"steelblue"`).
    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    /// Set the cumulative-line color (default `"firebrick"`).
    pub fn with_line_color<S: Into<String>>(mut self, color: S) -> Self {
        self.line_color = color.into();
        self
    }

    /// Set the bar width as a fraction of the category slot (default `0.8`).
    pub fn with_width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// Preserve insertion order instead of sorting descending by value (default sorts).
    pub fn with_sorted(mut self, sorted: bool) -> Self {
        self.sorted = sorted;
        self
    }

    /// Show a percentage label above each cumulative-line point (default `false`).
    pub fn with_cumulative_labels(mut self, show: bool) -> Self {
        self.show_cumulative_labels = show;
        self
    }

    /// Set the reference-line threshold as a cumulative percentage (default `80.0`).
    /// Implies `.with_show_threshold(true)`.
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self.show_threshold = true;
        self
    }

    /// Show or hide the threshold reference line (default `true`).
    pub fn with_show_threshold(mut self, show: bool) -> Self {
        self.show_threshold = show;
        self
    }

    /// Set the legend labels for the bars and the cumulative line (shown by
    /// default as "Value" / "Cumulative %"; use this to customize them).
    pub fn with_legend<S1: Into<String>, S2: Into<String>>(
        mut self,
        bar_label: S1,
        line_label: S2,
    ) -> Self {
        self.bar_legend_label = Some(bar_label.into());
        self.line_legend_label = Some(line_label.into());
        self.show_legend = true;
        self
    }

    /// Show or hide the legend (default `true` — bars and the cumulative line
    /// are two encodings that always coexist, so the legend is on by default
    /// unlike most other plot types).
    pub fn with_show_legend(mut self, show: bool) -> Self {
        self.show_legend = show;
        self
    }

    /// Collapse categories beyond `max` into one stacked "Other" bar instead of
    /// cluttering the x-axis with a long tail of tiny bars — common in
    /// real-world Pareto data (defect logs, error taxonomies) that can have
    /// dozens of categories after the top few.
    ///
    /// The bucketed bar renders as a *stack* of its constituent categories
    /// (like a mini stacked-bar-within-a-bar), each given a legend entry, so
    /// the hidden categories are still identifiable rather than silently
    /// disappearing into one opaque total. It contributes exactly one point
    /// to the cumulative line, matching its one slot on the x-axis.
    ///
    /// `max` counts the bucket itself: `.with_max_categories(5)` keeps the top
    /// 4 categories as their own bars plus one "Other" bar for the rest. No
    /// effect if there are `max` or fewer categories to begin with.
    pub fn with_max_categories(mut self, max: usize) -> Self {
        self.max_categories = Some(max);
        self
    }

    /// Set the label for the bucketed bar (default `"Other"`). No effect unless
    /// [`.with_max_categories()`](Self::with_max_categories) is also set.
    pub fn with_other_label<S: Into<String>>(mut self, label: S) -> Self {
        self.other_label = label.into();
        self
    }

    /// Render categories on the Y-axis and values on the X-axis (default `false`).
    pub fn with_horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
    }

    /// Total of all category values.
    pub fn total(&self) -> f64 {
        self.categories.iter().map(|c| c.value).sum()
    }

    /// Categories in render order: sorted descending by value when `sorted` is
    /// set (default), otherwise insertion order. Ignores `max_categories`
    /// bucketing — see [`render_bars`](Self::render_bars) for the bucketed view
    /// actually used for drawing.
    pub fn ordered_categories(&self) -> Vec<&ParetoCategory> {
        let mut ordered: Vec<&ParetoCategory> = self.categories.iter().collect();
        if self.sorted {
            ordered.sort_by(|a, b| b.value.nan_cmp(&a.value));
        }
        ordered
    }

    /// Bars in render order, after sorting and (if `max_categories` is set)
    /// bucketing the tail into one "Other" bar. This is what's actually drawn —
    /// [`cumulative_percentages`](Self::cumulative_percentages) is computed over
    /// these, not over raw per-category values, so a bucketed bar contributes
    /// exactly one point to the cumulative line.
    pub fn render_bars(&self) -> Vec<ParetoBar> {
        let ordered = self.ordered_categories();
        match self.max_categories {
            Some(max) if max > 0 && ordered.len() > max => {
                let keep = &ordered[..max - 1];
                let tail = &ordered[max - 1..];
                let mut bars: Vec<ParetoBar> = keep
                    .iter()
                    .map(|c| ParetoBar::Single((*c).clone()))
                    .collect();
                bars.push(ParetoBar::Bucketed {
                    label: self.other_label.clone(),
                    segments: tail.iter().map(|c| (*c).clone()).collect(),
                });
                bars
            }
            _ => ordered
                .into_iter()
                .cloned()
                .map(ParetoBar::Single)
                .collect(),
        }
    }

    /// Cumulative percentage at each rendered bar (parallel to
    /// [`render_bars`](Self::render_bars)). Empty if there's no data or the
    /// total is zero.
    pub fn cumulative_percentages(&self) -> Vec<f64> {
        let total = self.total();
        if total <= 0.0 {
            return vec![];
        }
        let mut running = 0.0;
        self.render_bars()
            .iter()
            .map(|b| {
                running += b.value();
                (running / total) * 100.0
            })
            .collect()
    }
}
