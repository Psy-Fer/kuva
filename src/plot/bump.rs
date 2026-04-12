/// Line curve style connecting consecutive rank points.
#[derive(Debug, Clone, Default)]
pub enum CurveStyle {
    /// S-curve (cubic Bézier with horizontal control tangents).  **(default)**
    /// Clearly shows crossing paths between ranks.
    #[default]
    Sigmoid,
    /// Straight lines.
    Straight,
}

/// Tie-breaking strategy when computing ranks from raw values.
#[derive(Debug, Clone, Default)]
pub enum BumpTieBreak {
    /// Tied series share the average of the occupied rank positions (e.g. 2.5, 2.5). **(default)**
    #[default]
    Average,
    /// All tied series receive the minimum (best) rank number.
    Min,
    /// All tied series receive the maximum (worst) rank number.
    Max,
    /// Maintain original insertion order among ties.
    Stable,
}

/// One series in a [`BumpPlot`].
#[derive(Debug, Clone)]
pub struct BumpSeries {
    pub name: String,
    /// Rank at each time point.  `None` = absent / did not qualify at that step.
    /// Fractional ranks are allowed (average-tie convention produces e.g. 2.5).
    pub ranks: Vec<Option<f64>>,
    /// Optional explicit CSS color.  Falls back to the category10 palette.
    pub color: Option<String>,
}

/// Bump chart — rank of each series across discrete time points / conditions.
///
/// # Basic usage (pre-ranked)
///
/// ```rust,no_run
/// use kuva::plot::bump::BumpPlot;
/// use kuva::render::{plots::Plot, layout::Layout, render::render_multiple};
/// use kuva::backend::svg::SvgBackend;
///
/// let plot = BumpPlot::new()
///     .with_series("Alpha",   vec![1, 3, 2, 1])
///     .with_series("Beta",    vec![2, 1, 1, 3])
///     .with_series("Gamma",   vec![3, 2, 3, 2])
///     .with_x_labels(["2021", "2022", "2023", "2024"]);
///
/// let plots = vec![Plot::Bump(plot)];
/// let layout = Layout::auto_from_plots(&plots).with_title("Rank over time");
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("bump.svg", svg).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct BumpPlot {
    /// Pre-ranked series (added via `.with_series()` / `.with_ranked_series()`).
    pub series: Vec<BumpSeries>,
    /// Labels for each x-axis time point / condition.
    pub x_labels: Vec<String>,
    /// Line curve style.  Default: [`CurveStyle::Sigmoid`].
    pub curve_style: CurveStyle,
    /// Show the rank number inside each dot.  Default: `false`.
    pub show_rank_labels: bool,
    /// Show series name labels at the left and right edges.  Default: `true`.
    pub show_series_labels: bool,
    /// Dot radius in pixels.  Default: `6.0`.
    pub dot_radius: f64,
    /// Line stroke width in pixels.  Default: `2.5`.
    pub stroke_width: f64,
    /// Name of the series to highlight; all others are muted.  Default: `None`.
    pub highlight: Option<String>,
    /// Show a legend.  Default: `true`.
    pub legend: bool,
    /// When `true`, a *lower* raw value maps to a *lower* (better) rank number.
    /// When `false` (default), a *higher* raw value = rank 1.
    pub rank_ascending: bool,
    /// Tie-breaking mode used by `.with_raw_series()`.
    pub tie_break: BumpTieBreak,
    /// Raw-value series queued for deferred auto-ranking.
    pub(crate) raw_values: Vec<(String, Vec<Option<f64>>, Option<String>)>,
}

impl Default for BumpPlot {
    fn default() -> Self { Self::new() }
}

impl BumpPlot {
    /// Create a [`BumpPlot`] with default settings.
    pub fn new() -> Self {
        BumpPlot {
            series: vec![],
            x_labels: vec![],
            curve_style: CurveStyle::Sigmoid,
            show_rank_labels: false,
            show_series_labels: true,
            dot_radius: 6.0,
            stroke_width: 2.5,
            highlight: None,
            legend: true,
            rank_ascending: false,
            tie_break: BumpTieBreak::Average,
            raw_values: vec![],
        }
    }

    // ── Data input ────────────────────────────────────────────────────────────

    /// Add a pre-ranked series using integer or float ranks.
    /// Use `None` values (via `.with_ranked_series`) for time points where the series is absent.
    pub fn with_series(mut self, name: impl Into<String>, ranks: impl IntoIterator<Item = impl Into<f64>>) -> Self {
        self.series.push(BumpSeries {
            name: name.into(),
            ranks: ranks.into_iter().map(|r| Some(r.into())).collect(),
            color: None,
        });
        self
    }

    /// Add a pre-ranked series that may have missing time points.
    /// `None` entries cause the line to break at that position.
    pub fn with_ranked_series(mut self, name: impl Into<String>, ranks: impl IntoIterator<Item = Option<f64>>) -> Self {
        self.series.push(BumpSeries {
            name: name.into(),
            ranks: ranks.into_iter().collect(),
            color: None,
        });
        self
    }

    /// Add a raw-value series; ranks are computed automatically across all raw-value
    /// series once all series have been added.
    ///
    /// By default (`.with_rank_ascending(false)`): higher value → rank 1.
    /// Ties are handled by `.with_tie_break()` (default: average).
    pub fn with_raw_series(mut self, name: impl Into<String>, values: impl IntoIterator<Item = impl Into<f64>>) -> Self {
        self.raw_values.push((
            name.into(),
            values.into_iter().map(|v| Some(v.into())).collect(),
            None,
        ));
        self
    }

    /// Add a raw-value series that may have missing time points.
    pub fn with_raw_series_opt(mut self, name: impl Into<String>, values: impl IntoIterator<Item = Option<f64>>) -> Self {
        self.raw_values.push((name.into(), values.into_iter().collect(), None));
        self
    }

    /// Set the x-axis labels (one per time point / condition).
    pub fn with_x_labels(mut self, labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.x_labels = labels.into_iter().map(|l| l.into()).collect();
        self
    }

    // ── Appearance ────────────────────────────────────────────────────────────

    /// Set the curve style (`Sigmoid` or `Straight`).
    pub fn with_curve_style(mut self, style: CurveStyle) -> Self {
        self.curve_style = style;
        self
    }

    /// Show / hide the rank number inside each dot.
    pub fn with_show_rank_labels(mut self, show: bool) -> Self {
        self.show_rank_labels = show;
        self
    }

    /// Show / hide series name labels at the left and right edges.
    pub fn with_show_series_labels(mut self, show: bool) -> Self {
        self.show_series_labels = show;
        self
    }

    /// Set the dot radius in pixels.
    pub fn with_dot_radius(mut self, r: f64) -> Self {
        self.dot_radius = r;
        self
    }

    /// Set the line stroke width in pixels.
    pub fn with_stroke_width(mut self, w: f64) -> Self {
        self.stroke_width = w;
        self
    }

    /// Highlight one series by name.  All other series are muted to 25 % opacity.
    pub fn with_highlight(mut self, name: impl Into<String>) -> Self {
        self.highlight = Some(name.into());
        self
    }

    /// Show / hide the legend.
    pub fn with_legend(mut self, show: bool) -> Self {
        self.legend = show;
        self
    }

    /// If `true`, lower raw value = lower (better) rank number.
    /// If `false` (default), higher raw value = rank 1.
    pub fn with_rank_ascending(mut self, asc: bool) -> Self {
        self.rank_ascending = asc;
        self
    }

    /// Set the tie-breaking mode for auto-ranking from raw values.
    pub fn with_tie_break(mut self, mode: BumpTieBreak) -> Self {
        self.tie_break = mode;
        self
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    /// Total number of series (pre-ranked + raw-value).
    pub(crate) fn total_series_count(&self) -> usize {
        self.series.len() + self.raw_values.len()
    }

    /// Number of time points derived from labels and series data.
    pub(crate) fn n_time_points(&self) -> usize {
        let from_labels = self.x_labels.len();
        let from_series = self.series.iter().map(|s| s.ranks.len()).max().unwrap_or(0);
        let from_raw = self.raw_values.iter().map(|(_, v, _)| v.len()).max().unwrap_or(0);
        from_labels.max(from_series).max(from_raw)
    }

    /// Resolve all series into ranked form, computing ranks from raw values if needed.
    pub(crate) fn resolved_series(&self) -> Vec<BumpSeries> {
        if self.raw_values.is_empty() {
            return self.series.clone();
        }

        let n_raw = self.raw_values.len();
        let n_time = self.raw_values.iter().map(|(_, v, _)| v.len()).max().unwrap_or(0);
        let mut ranked: Vec<Vec<Option<f64>>> = vec![vec![None; n_time]; n_raw];

        #[allow(clippy::needless_range_loop)]
        for t in 0..n_time {
            // Collect (series_idx, value) for series present at this time point
            let mut present: Vec<(usize, f64)> = (0..n_raw)
                .filter_map(|s| {
                    self.raw_values[s].1.get(t)
                        .and_then(|v| *v)
                        .map(|v| (s, v))
                })
                .collect();

            if present.is_empty() { continue; }

            // Sort: higher value = rank 1 when !ascending
            if self.rank_ascending {
                present.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            } else {
                present.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            }

            // Assign ranks with tie handling
            let n = present.len();
            let mut i = 0;
            while i < n {
                let val = present[i].1;
                let mut j = i + 1;
                // Find end of tie group
                while j < n && (present[j].1 - val).abs() < f64::EPSILON * val.abs().max(1.0) * 1000.0 {
                    j += 1;
                }
                for k in i..j {
                    let rank = match self.tie_break {
                        BumpTieBreak::Average => {
                            let sum: f64 = ((i + 1)..=(j)).map(|r| r as f64).sum();
                            sum / (j - i) as f64
                        }
                        BumpTieBreak::Min    => (i + 1) as f64,
                        BumpTieBreak::Max    => j as f64,
                        BumpTieBreak::Stable => (k + 1) as f64,
                    };
                    ranked[present[k].0][t] = Some(rank);
                }
                i = j;
            }
        }

        // Build BumpSeries from computed ranks
        let mut result: Vec<BumpSeries> = self.raw_values.iter().enumerate()
            .map(|(s, (name, _, color))| BumpSeries {
                name: name.clone(),
                ranks: ranked[s].clone(),
                color: color.clone(),
            })
            .collect();

        // Append any additional pre-ranked series
        result.extend(self.series.clone());
        result
    }
}
