use crate::plot::legend::LegendPosition;
use crate::plot::nan_cmp::NanCmp;

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Baseline algorithm for the streamgraph.
///
/// The baseline controls where the "floor" of the entire stack sits at each
/// x position.  Different algorithms produce different visual aesthetics and
/// expose different patterns in the data.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum StreamBaseline {
    /// Byron & Wattenberg (2008) wiggle — minimises the sum of squared first
    /// derivatives of all layer boundaries.  Keeps the silhouette as flat and
    /// undulating as possible.  **Default.**
    #[default]
    Wiggle,
    /// ThemeRiver symmetric — centres the total stack symmetrically around
    /// y = 0 at every x position.  Simple and visually balanced.
    Symmetric,
    /// Standard zero baseline — equivalent to a regular stacked area chart
    /// starting from y = 0.
    Zero,
}

/// Layer ordering for the streamgraph.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum StreamOrder {
    /// D3-style inside-out: sort series by total area descending, then
    /// greedily interleave so the widest streams end up near the centre.
    /// Produces the best visual balance.  **Default.**
    #[default]
    InsideOut,
    /// Sort by total area descending; largest series drawn at the bottom.
    ByTotal,
    /// Preserve the order in which series were added.
    Original,
}

// ── Geometry helper ───────────────────────────────────────────────────────────

/// Pre-computed stream geometry used by both bounds() and the renderer.
pub struct StreamGeometry {
    /// Baseline y-value at each x index.
    pub baseline: Vec<f64>,
    /// `lowers[k][i]` — bottom edge of the k-th rendered stream at x index i.
    pub lowers: Vec<Vec<f64>>,
    /// `uppers[k][i]` — top edge of the k-th rendered stream at x index i.
    pub uppers: Vec<Vec<f64>>,
    /// `render_order[k]` — original series index for the k-th rendered stream.
    pub render_order: Vec<usize>,
}

fn inside_out_order(totals: &[f64]) -> Vec<usize> {
    let n = totals.len();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| totals[b].nan_cmp(&totals[a]));

    let mut top_sum = 0.0_f64;
    let mut bottom_sum = 0.0_f64;
    let mut tops: Vec<usize> = Vec::new();
    let mut bottoms: Vec<usize> = Vec::new();

    for &j in &order {
        if top_sum < bottom_sum {
            top_sum += totals[j];
            tops.push(j);
        } else {
            bottom_sum += totals[j];
            bottoms.push(j);
        }
    }

    bottoms.reverse();
    bottoms.extend(tops);
    bottoms
}

// ── Plot struct ───────────────────────────────────────────────────────────────

/// A streamgraph — a flowing stacked area with a displaced baseline.
///
/// Streamgraphs are ideal for showing the evolution of multiple categories
/// over a continuous axis (typically time) when there are too many series for
/// a traditional stacked area chart to read cleanly.
///
/// **Import path:** `kuva::plot::StreamgraphPlot`
///
/// # Quick start
///
/// ```rust,no_run
/// use kuva::plot::StreamgraphPlot;
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let weeks: Vec<f64> = (1..=12).map(|w| w as f64).collect();
///
/// let sg = StreamgraphPlot::new()
///     .with_x(weeks)
///     .with_series([10.0, 14.0, 18.0, 22.0, 20.0, 16.0,
///                   12.0, 18.0, 24.0, 28.0, 22.0, 16.0])
///     .with_label("Alpha")
///     .with_series([ 5.0,  8.0, 12.0, 15.0, 14.0, 10.0,
///                    8.0, 11.0, 16.0, 18.0, 14.0,  9.0])
///     .with_label("Beta")
///     .with_legend("");
///
/// let plots = vec![Plot::Streamgraph(sg)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Weekly activity")
///     .with_x_label("Week");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("stream.svg", svg).unwrap();
/// ```
#[derive(Clone)]
pub struct StreamgraphPlot {
    /// Shared x-axis values for all series.
    pub x: Vec<f64>,
    /// `series[k][i]` — y value for series k at x index i.
    pub series: Vec<Vec<f64>>,
    /// Optional explicit color per series (parallel to `series`).
    pub colors: Vec<Option<String>>,
    /// Optional inline label per series (parallel to `series`).
    pub labels: Vec<Option<String>>,
    /// Baseline algorithm (default `Wiggle`).
    pub baseline: StreamBaseline,
    /// Layer ordering (default `InsideOut`).
    pub order: StreamOrder,
    /// Use Catmull-Rom spline smoothing (default `true`).
    pub smooth: bool,
    /// Fill opacity for each band (default `0.85`).
    pub fill_opacity: f64,
    /// Draw a thin stroke between adjacent streams (default `false`).
    pub stroke_between: bool,
    /// Width of the inter-stream stroke when enabled (default `0.8`).
    pub stroke_width: f64,
    /// Draw inline stream labels at each band's widest point (default `true`).
    pub show_labels: bool,
    /// Minimum pixel height a band must reach before its label is drawn (default `14.0`).
    pub min_label_height: f64,
    /// Normalise each column to sum to 100 % (default `false`).
    pub normalized: bool,
    /// Legend title; `Some("")` enables the legend with no title.
    pub legend_label: Option<String>,
    /// Legend position (default `OutsideRightTop`).
    pub legend_position: LegendPosition,
}

const STREAM_PALETTE: &[&str] = &[
    "steelblue",
    "tomato",
    "orange",
    "mediumseagreen",
    "mediumpurple",
    "goldenrod",
    "cornflowerblue",
    "coral",
    "mediumaquamarine",
    "orchid",
    "peru",
    "lightslategray",
];

impl Default for StreamgraphPlot {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamgraphPlot {
    /// Create a streamgraph with default settings.
    pub fn new() -> Self {
        Self {
            x: Vec::new(),
            series: Vec::new(),
            colors: Vec::new(),
            labels: Vec::new(),
            baseline: StreamBaseline::Wiggle,
            order: StreamOrder::InsideOut,
            smooth: true,
            fill_opacity: 0.85,
            stroke_between: false,
            stroke_width: 0.8,
            show_labels: true,
            min_label_height: 14.0,
            normalized: false,
            legend_label: None,
            legend_position: LegendPosition::OutsideRightTop,
        }
    }

    /// Set the shared x-axis values.
    pub fn with_x<T, I>(mut self, x: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<f64>,
    {
        self.x = x.into_iter().map(Into::into).collect();
        self
    }

    /// Append a new series.  Chain `.with_label()` and `.with_color()` to
    /// configure it.
    pub fn with_series<T, I>(mut self, y: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<f64>,
    {
        self.series.push(y.into_iter().map(Into::into).collect());
        self.colors.push(None);
        self.labels.push(None);
        self
    }

    /// Set the fill color of the most recently added series.
    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        if let Some(last) = self.colors.last_mut() {
            *last = Some(color.into());
        }
        self
    }

    /// Set the inline label of the most recently added series.
    pub fn with_label<S: Into<String>>(mut self, label: S) -> Self {
        if let Some(last) = self.labels.last_mut() {
            *last = Some(label.into());
        }
        self
    }

    /// Set the baseline algorithm.
    pub fn with_baseline(mut self, b: StreamBaseline) -> Self {
        self.baseline = b;
        self
    }

    /// Set the layer ordering.
    pub fn with_order(mut self, o: StreamOrder) -> Self {
        self.order = o;
        self
    }

    /// Disable Catmull-Rom smoothing (use straight line segments).
    pub fn with_linear(mut self) -> Self {
        self.smooth = false;
        self
    }

    /// Set fill opacity (default `0.85`).
    pub fn with_fill_opacity(mut self, opacity: f64) -> Self {
        self.fill_opacity = opacity;
        self
    }

    /// Draw a thin stroke between adjacent streams.
    pub fn with_stroke(mut self) -> Self {
        self.stroke_between = true;
        self
    }

    /// Set the stroke width when `.with_stroke()` is enabled.
    pub fn with_stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    /// Show or hide inline stream labels (default `true`).
    pub fn with_stream_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    /// Minimum pixel height a band must reach to display its inline label
    /// (default `14.0`).
    pub fn with_min_label_height(mut self, h: f64) -> Self {
        self.min_label_height = h;
        self
    }

    /// Normalise each column to sum to 100 %.
    pub fn with_normalized(mut self) -> Self {
        self.normalized = true;
        self
    }

    /// Enable the legend box.  Pass `""` for no title.
    pub fn with_legend<S: Into<String>>(mut self, title: S) -> Self {
        self.legend_label = Some(title.into());
        self
    }

    /// Set the legend position.
    pub fn with_legend_position(mut self, pos: LegendPosition) -> Self {
        self.legend_position = pos;
        self
    }

    /// Resolve the display color for series `k`.
    pub fn resolve_color(&self, k: usize) -> &str {
        if let Some(Some(ref c)) = self.colors.get(k) {
            c.as_str()
        } else {
            STREAM_PALETTE[k % STREAM_PALETTE.len()]
        }
    }

    /// Compute the full stream geometry (baseline + per-stream lower/upper
    /// edges).  Used by both `bounds()` and the renderer.
    pub fn compute_geometry(&self) -> Option<StreamGeometry> {
        if self.x.is_empty() || self.series.is_empty() {
            return None;
        }
        let n_pts = self.x.len();
        let n_series = self.series.len();

        // Normalise if requested
        let values: Vec<Vec<f64>> = if self.normalized {
            let totals: Vec<f64> = (0..n_pts)
                .map(|i| {
                    self.series
                        .iter()
                        .map(|s| s.get(i).copied().unwrap_or(0.0))
                        .sum::<f64>()
                })
                .collect();
            self.series
                .iter()
                .map(|s| {
                    (0..n_pts)
                        .map(|i| {
                            let t = totals[i].max(f64::EPSILON);
                            s.get(i).copied().unwrap_or(0.0) / t * 100.0
                        })
                        .collect()
                })
                .collect()
        } else {
            self.series
                .iter()
                .map(|s| {
                    (0..n_pts)
                        .map(|i| s.get(i).copied().unwrap_or(0.0))
                        .collect()
                })
                .collect()
        };

        // Column totals
        let totals: Vec<f64> = (0..n_pts)
            .map(|i| values.iter().map(|s| s[i]).sum::<f64>())
            .collect();

        // Render order
        let series_totals: Vec<f64> = values.iter().map(|s| s.iter().sum::<f64>()).collect();
        let render_order: Vec<usize> = match self.order {
            StreamOrder::InsideOut => inside_out_order(&series_totals),
            StreamOrder::ByTotal => {
                let mut o: Vec<usize> = (0..n_series).collect();
                o.sort_by(|&a, &b| series_totals[b].nan_cmp(&series_totals[a]));
                o
            }
            StreamOrder::Original => (0..n_series).collect(),
        };

        // Baseline
        let mut baseline: Vec<f64> = match self.baseline {
            StreamBaseline::Zero => vec![0.0; n_pts],
            StreamBaseline::Symmetric => (0..n_pts).map(|i| -0.5 * totals[i]).collect(),
            StreamBaseline::Wiggle => {
                let n = n_series as f64;
                let mut b: Vec<f64> = (0..n_pts)
                    .map(|i| {
                        let sum: f64 = render_order
                            .iter()
                            .enumerate()
                            .map(|(k, &j)| (n - k as f64) * values[j][i])
                            .sum();
                        -sum / (n + 1.0)
                    })
                    .collect();
                // Shift so the mean stream centre sits at y = 0
                let mean_centre: f64 = b
                    .iter()
                    .zip(totals.iter())
                    .map(|(&bi, &ti)| bi + 0.5 * ti)
                    .sum::<f64>()
                    / n_pts as f64;
                for bi in &mut b {
                    *bi -= mean_centre;
                }
                b
            }
        };

        // Clamp baseline so the minimum is always at most 0 for Zero mode
        if self.baseline == StreamBaseline::Zero {
            for bi in &mut baseline {
                *bi = bi.max(0.0);
            }
        }

        // Build per-stream lower/upper edges
        let mut lowers: Vec<Vec<f64>> = Vec::with_capacity(n_series);
        let mut uppers: Vec<Vec<f64>> = Vec::with_capacity(n_series);
        let mut current_lower: Vec<f64> = baseline.clone();

        for &j in &render_order {
            let upper: Vec<f64> = (0..n_pts)
                .map(|i| current_lower[i] + values[j][i])
                .collect();
            lowers.push(current_lower.clone());
            uppers.push(upper.clone());
            current_lower = upper;
        }

        Some(StreamGeometry {
            baseline,
            lowers,
            uppers,
            render_order,
        })
    }
}
