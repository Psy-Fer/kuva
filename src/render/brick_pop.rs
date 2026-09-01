//! `BrickPopPlot` — a population allele-frequency brick composite.
//!
//! For a single STR locus, this arranges one row per (frequency-sorted) unique allele,
//! each row laid out left-to-right as:
//!
//! ```text
//! [ allele name | frequency bar | metric heatboxes | motif bricks ]
//! ```
//!
//! It is a standalone composite (like [`Figure`](crate::render::figure) and
//! [`TrackStack`](crate::render::track_stack)), not a [`Plot`] enum variant: it owns
//! several panels with independent scales (a frequency bar, one colour scale per metric)
//! plus its own gutter, and it emits a single [`Scene`].
//!
//! The brick column embeds a [`BrickPlot`] rendered via [`render_multiple`]; the frequency
//! bar and heatboxes are drawn in the left gutter, aligned to the *same* per-row y-centres.
//! Alignment is guaranteed by pinning the embedded plot's row band with
//! [`Layout::with_force_margins_y`], so all three panels agree on `num_rows` and row height.
//!
//! Rows are rendered in the order the caller supplies them (row 0 = top); sort your alleles
//! by frequency before building. The `frequencies` and each metric's `values` are indexed
//! by that same row order.

use crate::plot::brick::BrickPlot;
use crate::plot::colormap::ColorMap;
use crate::render::color::Color;
use crate::render::layout::Layout;
use crate::render::plots::Plot;
use crate::render::render::{render_multiple, Primitive, Scene, TextAnchor};
use crate::render::text_metrics::{center_offset, measure_text_width, FontStyle};
use crate::render::theme::Theme;
use crate::render::track_stack::merge_translated;

const LABEL_SIZE: f64 = 12.0;
const HEADER_SIZE: u32 = 11;
const TITLE_BAND_PX: f64 = 30.0;
const TITLE_SIZE: u32 = 18;
const HEADER_BAND_PX: f64 = 26.0;
const BOTTOM_PAD: f64 = 10.0;
const GUTTER_PAD: f64 = 6.0;
const PANEL_GAP: f64 = 8.0;
const FREQ_PANEL_W: f64 = 110.0;
const HEAT_COL_W: f64 = 22.0;
const HEAT_CELL_GAP: f64 = 3.0;
const DEFAULT_ROW_HEIGHT: f64 = 18.0;
const DEFAULT_FREQ_COLOR: &str = "#4c78a8";
const DEFAULT_NA_COLOR: &str = "#eeeeee";

/// One column of per-allele metric heatboxes (e.g. methylation, motif entropy, longest
/// pure chain). Each row's value is mapped through `colormap` after normalising to the
/// column's value range (auto from the finite values unless pinned with
/// [`with_range`](MetricColumn::with_range)). Missing values (`None`) render as the NA
/// colour set on the plot.
#[derive(Debug, Clone)]
pub struct MetricColumn {
    /// Column header label.
    pub label: String,
    /// One value per allele row (same order as the plot's rows); `None` = missing.
    pub values: Vec<Option<f64>>,
    /// Colour scale applied across this column's normalised range.
    pub colormap: ColorMap,
    /// Explicit lower bound of the colour scale. `None` = min of the finite values.
    pub vmin: Option<f64>,
    /// Explicit upper bound of the colour scale. `None` = max of the finite values.
    pub vmax: Option<f64>,
}

impl MetricColumn {
    /// Create a metric column with an auto-computed value range.
    pub fn new(label: impl Into<String>, values: Vec<Option<f64>>, colormap: ColorMap) -> Self {
        Self {
            label: label.into(),
            values,
            colormap,
            vmin: None,
            vmax: None,
        }
    }

    /// Pin the colour-scale range instead of auto-computing it from the data.
    pub fn with_range(mut self, vmin: f64, vmax: f64) -> Self {
        self.vmin = Some(vmin);
        self.vmax = Some(vmax);
        self
    }

    /// Resolve the effective (min, max) range. Returns `None` if there are no finite
    /// values and no explicit bounds (the column can't be coloured).
    fn range(&self) -> Option<(f64, f64)> {
        let auto = self
            .values
            .iter()
            .filter_map(|v| *v)
            .filter(|v| v.is_finite())
            .fold(None, |acc: Option<(f64, f64)>, v| {
                Some(acc.map_or((v, v), |(lo, hi)| (lo.min(v), hi.max(v))))
            });
        let lo = self.vmin.or(auto.map(|(lo, _)| lo));
        let hi = self.vmax.or(auto.map(|(_, hi)| hi));
        match (lo, hi) {
            (Some(lo), Some(hi)) => Some((lo, hi)),
            _ => None,
        }
    }

    /// Normalise row `i`'s value to `[0, 1]`. `None` if the value is missing/non-finite
    /// or the column has no range. A degenerate range (min == max) maps to `0.5`.
    fn normalized(&self, i: usize) -> Option<f64> {
        let v = (*self.values.get(i)?)?;
        if !v.is_finite() {
            return None;
        }
        let (lo, hi) = self.range()?;
        if hi <= lo {
            return Some(0.5);
        }
        Some(((v - lo) / (hi - lo)).clamp(0.0, 1.0))
    }
}

/// A population allele-frequency brick composite. See the [module docs](self).
pub struct BrickPopPlot {
    /// The embedded brick column (styling + per-row motif data); row 0 = top.
    brick: BrickPlot,
    /// Frequency-bar value per row (same order/length as the brick rows).
    frequencies: Vec<f64>,
    /// Metric heatbox columns, drawn left-to-right between the freq bar and the bricks.
    metrics: Vec<MetricColumn>,
    title: Option<String>,
    /// Header label for the frequency panel.
    freq_label: String,
    /// CSS colour for the frequency bars.
    freq_bar_color: String,
    /// Fill colour for missing (`None`) metric cells.
    na_color: String,
    /// Desired pixel height per allele row (used when auto-sizing the canvas height).
    row_height_px: f64,
    theme: Theme,
}

impl BrickPopPlot {
    /// Create a composite around a pre-built [`BrickPlot`]. The brick's rows define the
    /// allele rows (and their order); attach frequencies and metrics with the builders.
    pub fn new(brick: BrickPlot) -> Self {
        Self {
            brick,
            frequencies: Vec::new(),
            metrics: Vec::new(),
            title: None,
            freq_label: "Frequency".to_string(),
            freq_bar_color: DEFAULT_FREQ_COLOR.to_string(),
            na_color: DEFAULT_NA_COLOR.to_string(),
            row_height_px: DEFAULT_ROW_HEIGHT,
            theme: Theme::default(),
        }
    }

    /// Set the per-row frequency values (one per allele row, same order as the rows).
    pub fn with_frequencies<I: IntoIterator<Item = f64>>(mut self, freqs: I) -> Self {
        self.frequencies = freqs.into_iter().collect();
        self
    }

    /// Append a metric heatbox column with an auto-computed colour range.
    pub fn with_metric(
        mut self,
        label: impl Into<String>,
        values: Vec<Option<f64>>,
        colormap: ColorMap,
    ) -> Self {
        self.metrics
            .push(MetricColumn::new(label, values, colormap));
        self
    }

    /// Append a fully-specified metric column (e.g. with a pinned range).
    pub fn with_metric_column(mut self, column: MetricColumn) -> Self {
        self.metrics.push(column);
        self
    }

    /// Set the plot title (drawn in a band above the panels).
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the frequency panel's header label (default `"Frequency"`).
    pub fn with_frequency_label(mut self, label: impl Into<String>) -> Self {
        self.freq_label = label.into();
        self
    }

    /// Set the CSS colour of the frequency bars.
    pub fn with_freq_bar_color(mut self, color: impl Into<String>) -> Self {
        self.freq_bar_color = color.into();
        self
    }

    /// Set the fill colour used for missing (`None`) metric cells.
    pub fn with_na_color(mut self, color: impl Into<String>) -> Self {
        self.na_color = color.into();
        self
    }

    /// Set the desired pixel height per allele row (used by [`render`](Self::render)).
    pub fn with_row_height(mut self, px: f64) -> Self {
        self.row_height_px = px;
        self
    }

    /// Override the theme (default [`Theme::default`]).
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Number of allele rows (from the embedded brick plot).
    pub fn num_rows(&self) -> usize {
        self.brick.num_rows()
    }

    /// Render at the given width, auto-computing the height from the row count.
    pub fn render(self, width: f64) -> Scene {
        let n = self.num_rows().max(1) as f64;
        let title_h = if self.title.is_some() {
            TITLE_BAND_PX
        } else {
            0.0
        };
        let header_h = if self.frequencies.is_empty() && self.metrics.is_empty() {
            0.0
        } else {
            HEADER_BAND_PX
        };
        let height = title_h + header_h + n * self.row_height_px + BOTTOM_PAD;
        self.render_sized(width, height)
    }

    /// Render at an explicit width and height.
    pub fn render_sized(self, width: f64, height: f64) -> Scene {
        let mut scene = Scene::new(width, height);
        scene.background_color = Some(self.theme.background.clone());
        scene.font_family = self.theme.font_family.clone();

        let n = self.num_rows();
        if n == 0 {
            return scene;
        }
        let text = || Color::Css(self.theme.text_color.as_str().into());

        // ── Horizontal regions: [names | freq | heat | bricks] ────────────────────────
        let names: Vec<String> = self.brick.names.clone();
        let name_w = names
            .iter()
            .map(|s| measure_text_width(s, LABEL_SIZE, FontStyle::Regular))
            .fold(0.0_f64, f64::max);
        let gutter_w = if name_w > 0.0 {
            name_w + 2.0 * GUTTER_PAD
        } else {
            0.0
        };

        let has_freq = !self.frequencies.is_empty();
        let freq_x = gutter_w;
        let freq_w = if has_freq { FREQ_PANEL_W } else { 0.0 };

        let heat_x = freq_x + freq_w + if freq_w > 0.0 { PANEL_GAP } else { 0.0 };
        let heat_w = self.metrics.len() as f64 * HEAT_COL_W;

        let brick_x = heat_x + heat_w + if heat_w > 0.0 { PANEL_GAP } else { 0.0 };
        let brick_w = (width - brick_x).max(50.0);

        // ── Vertical bands: [title][header][rows...][bottom pad] ───────────────────────
        let title_h = if self.title.is_some() {
            TITLE_BAND_PX
        } else {
            0.0
        };
        let header_h = if has_freq || !self.metrics.is_empty() {
            HEADER_BAND_PX
        } else {
            0.0
        };
        let force_top = title_h + header_h;
        let force_bottom = BOTTOM_PAD;
        let band_h = (height - force_top - force_bottom).max(1.0);
        let row_h = band_h / n as f64;
        let row_center = |i: usize| force_top + (i as f64 + 0.5) * row_h;

        // ── Title ─────────────────────────────────────────────────────────────────────
        if let Some(title) = &self.title {
            scene.add(Primitive::Text {
                x: width / 2.0,
                y: TITLE_BAND_PX * 0.62,
                content: title.clone(),
                size: TITLE_SIZE,
                anchor: TextAnchor::Middle,
                rotate: None,
                bold: true,
                color: Some(text()),
            });
        }

        // ── Column headers ──────────────────────────────────────────────────────────
        let header_y = force_top - 5.0;
        if has_freq {
            scene.add(Primitive::Text {
                x: freq_x + freq_w / 2.0,
                y: header_y,
                content: self.freq_label.clone(),
                size: HEADER_SIZE,
                anchor: TextAnchor::Middle,
                rotate: None,
                bold: false,
                color: Some(text()),
            });
        }
        for (c, metric) in self.metrics.iter().enumerate() {
            // Column labels are drawn vertically (columns are narrow).
            let cx = heat_x + c as f64 * HEAT_COL_W + HEAT_COL_W / 2.0;
            scene.add(Primitive::Text {
                x: cx + center_offset(HEADER_SIZE as f64, FontStyle::Regular),
                y: force_top - 3.0,
                content: metric.label.clone(),
                size: HEADER_SIZE,
                anchor: TextAnchor::End,
                rotate: Some(-90.0),
                bold: false,
                color: Some(text()),
            });
        }

        // ── Allele name gutter ────────────────────────────────────────────────────────
        for (i, name) in names.iter().enumerate() {
            if name.is_empty() {
                continue;
            }
            scene.add(Primitive::Text {
                x: GUTTER_PAD,
                y: row_center(i) + center_offset(LABEL_SIZE, FontStyle::Regular),
                content: name.clone(),
                size: LABEL_SIZE as u32,
                anchor: TextAnchor::Start,
                rotate: None,
                bold: false,
                color: Some(text()),
            });
        }

        // ── Frequency bars (baseline at the right of the panel, growing left) ─────────
        if has_freq {
            let max_freq = self
                .frequencies
                .iter()
                .cloned()
                .filter(|v| v.is_finite())
                .fold(f64::MIN, f64::max);
            let max_freq = if max_freq > 0.0 { max_freq } else { 1.0 };
            let baseline_r = freq_x + freq_w - GUTTER_PAD;
            let avail = freq_w - GUTTER_PAD;
            let bar_h = (row_h * 0.7).clamp(2.0, 16.0);
            let fill = Color::Css(self.freq_bar_color.as_str().into());
            for i in 0..n {
                let f = self.frequencies.get(i).copied().unwrap_or(0.0);
                if !f.is_finite() || f <= 0.0 {
                    continue;
                }
                let w = (f / max_freq * avail).clamp(0.0, avail);
                scene.add(Primitive::Rect {
                    x: baseline_r - w,
                    y: row_center(i) - bar_h / 2.0,
                    width: w,
                    height: bar_h,
                    fill: fill.clone(),
                    stroke: None,
                    stroke_width: None,
                    opacity: None,
                });
            }
        }

        // ── Metric heatboxes ──────────────────────────────────────────────────────────
        let cell_w = HEAT_COL_W - HEAT_CELL_GAP;
        let cell_h = (row_h * 0.85).clamp(2.0, HEAT_COL_W);
        let na_fill = Color::Css(self.na_color.as_str().into());
        for (c, metric) in self.metrics.iter().enumerate() {
            let cx = heat_x + c as f64 * HEAT_COL_W + HEAT_CELL_GAP / 2.0;
            for i in 0..n {
                let fill = match metric.normalized(i) {
                    Some(t) => Color::Css(metric.colormap.map(t).into()),
                    None => na_fill.clone(),
                };
                scene.add(Primitive::Rect {
                    x: cx,
                    y: row_center(i) - cell_h / 2.0,
                    width: cell_w,
                    height: cell_h,
                    fill,
                    stroke: None,
                    stroke_width: None,
                    opacity: None,
                });
            }
        }

        // ── Brick column (embedded BrickPlot, row band pinned to align with the panels) ─
        let plots = vec![Plot::Brick(self.brick)];
        let mut blayout = Layout::auto_from_plots(&plots)
            .with_width(brick_w)
            .with_height(height)
            .with_force_margins_y(force_top, force_bottom);
        // The allele names live in our own gutter, so drop the brick's y-tick labels.
        blayout.suppress_y_ticks = true;
        let bscene = render_multiple(plots, blayout);
        merge_translated(&mut scene, bscene, brick_x, 0.0);

        scene
    }
}
