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
use crate::plot::legend_plot::LegendPlot;
use crate::render::color::Color;
use crate::render::layout::Layout;
use crate::render::plots::Plot;
use crate::render::render::{
    collect_legend_entries, render_multiple, Primitive, Scene, TextAnchor,
};
use crate::render::text_metrics::{center_offset, measure_text_width, FontStyle};
use crate::render::theme::Theme;
use crate::render::track_stack::merge_translated;

const LABEL_SIZE: f64 = 12.0;
const HEADER_SIZE: u32 = 11;
const AXIS_LABEL_SIZE: u32 = 10;
const TITLE_BAND_PX: f64 = 30.0;
const TITLE_SIZE: u32 = 18;
const HEADER_BAND_PX: f64 = 26.0;
/// Bottom band reserved for the frequency axis and the embedded brick's x-axis.
const AXIS_BAND_PX: f64 = 42.0;
const GUTTER_PAD: f64 = 6.0;
const PANEL_GAP: f64 = 8.0;
const FREQ_PANEL_W: f64 = 110.0;
const HEAT_COL_W: f64 = 22.0;
const HEAT_CELL_GAP: f64 = 3.0;
const DEFAULT_ROW_HEIGHT: f64 = 18.0;
const DEFAULT_FREQ_COLOR: &str = "#4c78a8";
const DEFAULT_NA_COLOR: &str = "#eeeeee";
/// Bottom band: the motif legend, rendered as an embedded `LegendPlot` grid (kept out of
/// the way of the right-hand colourbars, since motif lists get large for complex loci).
const LEGEND_ROW_H: f64 = 18.0;
/// Uniform legend column width = swatch + widest label + padding (mirrors `LegendPlot`).
const LEGEND_COL_PAD: f64 = 38.0;
/// Right-hand region: one metric colourbar per column, drawn side by side.
const COLORBAR_W: f64 = 14.0;
const COLORBAR_H: f64 = 64.0;
const COLORBAR_STEPS: usize = 24;
/// Width budget for one colourbar unit: [vertical title | bar | value labels].
const COLORBAR_UNIT_W: f64 = 76.0;

/// Format a scale value compactly (integers without a decimal, else 2 dp).
fn fmt_num(v: f64) -> String {
    if v.fract().abs() < 1e-9 && v.abs() < 1e6 {
        format!("{v:.0}")
    } else {
        format!("{v:.2}")
    }
}

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
    /// Explicit `(min, max)` for the frequency-bar scale. `None` = auto: `[0, 1]` when the
    /// values look like proportions (all in `[0, 1]`), else `[0, max]`.
    freq_range: Option<(f64, f64)>,
    /// When true, metric cells show the numeric value with the heat colour on the cell
    /// border (instead of a solid fill), where the cell is large enough for text.
    show_metric_values: bool,
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
            freq_range: None,
            show_metric_values: false,
            row_height_px: DEFAULT_ROW_HEIGHT,
            theme: Theme::default(),
        }
    }

    /// Set the per-row frequency values (one per allele row, same order as the rows).
    ///
    /// Everything is positional, so this must have exactly one entry per brick row, in the
    /// same order (a `debug_assert` enforces this in debug/test builds).
    pub fn with_frequencies<I: IntoIterator<Item = f64>>(mut self, freqs: I) -> Self {
        self.frequencies = freqs.into_iter().collect();
        debug_assert_eq!(
            self.frequencies.len(),
            self.brick.num_rows(),
            "frequencies must have one entry per allele row"
        );
        self
    }

    /// Pin the frequency-bar scale to `[vmin, vmax]` (mirrors [`MetricColumn::with_range`]).
    ///
    /// By default the bars auto-scale to `[0, 1]` when the values look like proportions,
    /// else to `[0, max]`. Pin an explicit range (e.g. `[0.0, 1.0]`) so bars are comparable
    /// across loci: a locus with one allele at 0.9 then looks obviously different from one
    /// with twelve singletons at 0.08, instead of every bar filling to the max.
    pub fn with_frequency_range(mut self, vmin: f64, vmax: f64) -> Self {
        self.freq_range = Some((vmin, vmax));
        self
    }

    /// Append a metric heatbox column with an auto-computed colour range.
    pub fn with_metric(
        mut self,
        label: impl Into<String>,
        values: Vec<Option<f64>>,
        colormap: ColorMap,
    ) -> Self {
        debug_assert_eq!(
            values.len(),
            self.brick.num_rows(),
            "metric values must have one entry per allele row"
        );
        self.metrics
            .push(MetricColumn::new(label, values, colormap));
        self
    }

    /// Append a fully-specified metric column (e.g. with a pinned range).
    pub fn with_metric_column(mut self, column: MetricColumn) -> Self {
        debug_assert_eq!(
            column.values.len(),
            self.brick.num_rows(),
            "metric values must have one entry per allele row"
        );
        self.metrics.push(column);
        self
    }

    /// Show each metric cell's numeric value as text, with the heat colour on the cell
    /// border rather than as a solid fill (where the cell is large enough for text; small
    /// cells fall back to a solid fill).
    pub fn with_metric_values(mut self, on: bool) -> Self {
        self.show_metric_values = on;
        self
    }

    /// Sort alleles by descending frequency, permuting the rows, the frequency values, and
    /// every metric column together so they stay aligned. Removes the need to hand-sort
    /// each vector in lockstep. No-op if no frequencies are set or lengths don't match.
    pub fn sorted_by_frequency(mut self) -> Self {
        let n = self.brick.num_rows();
        if n == 0 || self.frequencies.len() != n {
            return self;
        }
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&a, &b| {
            self.frequencies[b]
                .partial_cmp(&self.frequencies[a])
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.cmp(&b))
        });
        self.frequencies = order.iter().map(|&i| self.frequencies[i]).collect();
        for m in &mut self.metrics {
            if m.values.len() == n {
                m.values = order.iter().map(|&i| m.values[i]).collect();
            }
        }
        self.brick = self.brick.permute_rows(&order);
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

    /// Pixels reserved for the title band (0 when there is no title).
    fn title_band(&self) -> f64 {
        if self.title.is_some() {
            TITLE_BAND_PX
        } else {
            0.0
        }
    }

    /// Pixels reserved for the column-header band. Tall enough for the vertically-drawn
    /// metric labels; 0 when there is neither a frequency bar nor any metric column.
    fn header_band(&self) -> f64 {
        if self.frequencies.is_empty() && self.metrics.is_empty() {
            return 0.0;
        }
        let longest = self
            .metrics
            .iter()
            .map(|m| measure_text_width(&m.label, HEADER_SIZE as f64, FontStyle::Regular))
            .fold(0.0_f64, f64::max);
        (longest + 8.0).max(HEADER_BAND_PX)
    }

    /// Motif display labels (one per motif) for the bottom legend, in the same
    /// token-sorted order the drawn legend uses (so the wrap-based height estimate matches
    /// the drawing, and never depends on HashMap iteration order).
    fn motif_labels(&self) -> Vec<String> {
        if let Some(m) = &self.brick.motifs {
            let mut pairs: Vec<(&char, &String)> = m.iter().collect();
            pairs.sort_by_key(|(k, _)| **k);
            pairs.into_iter().map(|(_, v)| v.clone()).collect()
        } else if let Some(t) = &self.brick.template {
            let mut keys: Vec<char> = t.keys().copied().collect();
            keys.sort_unstable();
            keys.into_iter().map(|c| c.to_string()).collect()
        } else {
            Vec::new()
        }
    }

    /// Width of the right-hand colourbar region (0 when there are no metrics).
    fn colorbar_region_w(&self) -> f64 {
        self.metrics.len() as f64 * COLORBAR_UNIT_W
    }

    /// Resolved `(min, max)` for the frequency-bar scale. Uses the pinned range when set;
    /// otherwise `[0, 1]` when the values look like proportions (all finite and in `[0, 1]`),
    /// else `[0, max]`. Bars are thus comparable across loci by default when data are
    /// proportions, instead of every locus normalising to its own maximum.
    fn freq_scale(&self) -> (f64, f64) {
        if let Some(r) = self.freq_range {
            return r;
        }
        let mut max = f64::MIN;
        let mut looks_proportion = true;
        for &f in &self.frequencies {
            if !f.is_finite() {
                continue;
            }
            max = max.max(f);
            if !(0.0..=1.0).contains(&f) {
                looks_proportion = false;
            }
        }
        if max == f64::MIN || looks_proportion {
            // No finite values, or they look like proportions: use the [0, 1] scale.
            (0.0, 1.0)
        } else {
            (0.0, max.max(f64::EPSILON))
        }
    }

    /// Column count and pixel height of the bottom motif legend for a given legend width
    /// (returns `(1, 0.0)` when the brick has no legend). Mirrors `LegendPlot`'s uniform
    /// grid: a fixed column width from the widest label, as many columns as fit, then rows.
    fn motif_legend_layout(&self, legend_width: f64) -> (usize, f64) {
        let labels = self.motif_labels();
        if labels.is_empty() || legend_width <= 0.0 {
            return (1, 0.0);
        }
        let widest = labels
            .iter()
            .map(|l| measure_text_width(l, LABEL_SIZE, FontStyle::Regular))
            .fold(0.0_f64, f64::max);
        let col_w = widest + LEGEND_COL_PAD;
        let cols = ((legend_width / col_w).floor() as usize).max(1);
        let rows = labels.len().div_ceil(cols);
        (cols, rows as f64 * LEGEND_ROW_H + 24.0)
    }

    /// Render at the given width, auto-computing the height. The canvas is tall enough for
    /// whichever is larger: the allele rows + axis + bottom motif legend, or the top-right
    /// colourbars.
    pub fn render(self, width: f64) -> Scene {
        let n = self.num_rows().max(1) as f64;
        let force_top = self.title_band() + self.header_band();
        let legend_width = (width - self.colorbar_region_w()).max(1.0);
        let (_, legend_band_h) = self.motif_legend_layout(legend_width);
        let rows_stack = force_top + n * self.row_height_px + AXIS_BAND_PX + legend_band_h;
        let colorbar_stack = if self.metrics.is_empty() {
            0.0
        } else {
            force_top + COLORBAR_H + 26.0
        };
        self.render_sized(width, rows_stack.max(colorbar_stack))
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
        let text_color = Color::Css(self.theme.text_color.as_str().into());
        let axis_color = Color::Css(self.theme.axis_color.as_str().into());

        let has_freq = !self.frequencies.is_empty();
        let has_metrics = !self.metrics.is_empty();
        let brick_has_legend = self.brick.template.as_ref().is_some_and(|t| !t.is_empty());

        // Allele-name font shrinks to fit dense (many-allele) rows so labels don't overlap;
        // below ~4 px per row the names are dropped entirely (too dense to label).
        let name_font = (self.row_height_px * 0.9).clamp(4.0, LABEL_SIZE);
        let show_names = self.row_height_px >= 4.0;

        // ── Horizontal regions: [names | freq | heat | bricks | legend] ───────────────
        let names: Vec<String> = self.brick.names.clone();
        let name_w = if show_names {
            names
                .iter()
                .map(|s| measure_text_width(s, name_font, FontStyle::Regular))
                .fold(0.0_f64, f64::max)
        } else {
            0.0
        };
        let gutter_w = if name_w > 0.0 {
            name_w + 2.0 * GUTTER_PAD
        } else {
            0.0
        };

        let freq_x = gutter_w;
        let freq_w = if has_freq { FREQ_PANEL_W } else { 0.0 };
        let heat_x = freq_x + freq_w + if freq_w > 0.0 { PANEL_GAP } else { 0.0 };
        // Widen the metric columns in value-in-box mode so numbers fit inside the cells.
        let heat_col_w = if self.show_metric_values {
            HEAT_COL_W + 14.0
        } else {
            HEAT_COL_W
        };
        let heat_w = self.metrics.len() as f64 * heat_col_w;
        let brick_x = heat_x + heat_w + if heat_w > 0.0 { PANEL_GAP } else { 0.0 };
        // Colourbars occupy a right-hand region (side by side); bricks fill the space between
        // the heatboxes and that region.
        let colorbar_region_w = self.colorbar_region_w();
        let brick_w = (width - brick_x - colorbar_region_w).max(50.0);

        // The motif legend occupies a bottom band, spanning everything left of the
        // colourbar region, rendered as an embedded `LegendPlot` grid.
        let legend_width = width - colorbar_region_w;
        let (legend_cols, legend_band_h) = if brick_has_legend {
            self.motif_legend_layout(legend_width)
        } else {
            (1, 0.0)
        };

        // ── Vertical bands: [title][header][rows...][axis band] ───────────────────────
        // Rows keep their natural height (`row_height_px`) when the canvas is taller than
        // the rows need (e.g. sized up for a tall legend); the surplus becomes the bottom
        // band. When the canvas is tight, rows compress but the axis band is preserved.
        let force_top = self.title_band() + self.header_band();
        let force_bottom = (height - force_top - n as f64 * self.row_height_px).max(AXIS_BAND_PX);
        let band_h = (height - force_top - force_bottom).max(1.0);
        let row_h = band_h / n as f64;
        let rows_bottom = force_top + band_h;
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
                color: Some(text_color.clone()),
            });
        }

        // ── Column headers, drawn ABOVE the panels ────────────────────────────────────
        if has_freq {
            scene.add(Primitive::Text {
                x: freq_x + freq_w / 2.0,
                y: force_top - 6.0,
                content: self.freq_label.clone(),
                size: HEADER_SIZE,
                anchor: TextAnchor::Middle,
                rotate: None,
                bold: false,
                color: Some(text_color.clone()),
            });
        }
        // Metric labels read bottom-to-top (columns are narrow); anchored at the bottom of
        // the header band (just above the squares) so `rotate(-90)` extends them upward.
        for (c, metric) in self.metrics.iter().enumerate() {
            let cx = heat_x + c as f64 * heat_col_w + heat_col_w / 2.0;
            scene.add(Primitive::Text {
                x: cx + center_offset(HEADER_SIZE as f64, FontStyle::Regular),
                y: force_top - 4.0,
                content: metric.label.clone(),
                size: HEADER_SIZE,
                anchor: TextAnchor::Start,
                rotate: Some(-90.0),
                bold: false,
                color: Some(text_color.clone()),
            });
        }

        // ── Allele name gutter (font scaled to row height; dropped when rows too dense) ─
        if show_names {
            for (i, name) in names.iter().enumerate() {
                if name.is_empty() {
                    continue;
                }
                scene.add(Primitive::Text {
                    x: GUTTER_PAD,
                    y: row_center(i) + center_offset(name_font, FontStyle::Regular),
                    content: name.clone(),
                    size: name_font.round() as u32,
                    anchor: TextAnchor::Start,
                    rotate: None,
                    bold: false,
                    color: Some(text_color.clone()),
                });
            }
        }

        // ── Frequency bars + axis (baseline at panel right, bars grow left) ────────────
        if has_freq {
            let (fmin, fmax) = self.freq_scale();
            let span = (fmax - fmin).max(f64::EPSILON);
            let baseline_r = freq_x + freq_w - GUTTER_PAD;
            let avail = freq_w - GUTTER_PAD;
            let bar_h = (row_h * 0.7).clamp(2.0, 16.0);
            let fill = Color::Css(self.freq_bar_color.as_str().into());
            for i in 0..n {
                let f = self.frequencies.get(i).copied().unwrap_or(0.0);
                if !f.is_finite() {
                    continue;
                }
                let w = (((f - fmin) / span) * avail).clamp(0.0, avail);
                if w <= 0.0 {
                    continue;
                }
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
            // Axis line along the bottom band, with `fmin` at the (right) baseline and
            // `fmax` at the left edge, matching the leftward-growing bars.
            scene.add(Primitive::Line {
                x1: baseline_r - avail,
                y1: rows_bottom,
                x2: baseline_r,
                y2: rows_bottom,
                stroke: axis_color.clone(),
                stroke_width: 1.0,
                stroke_dasharray: None,
            });
            for k in 0..=2 {
                let frac = k as f64 / 2.0;
                let x = baseline_r - frac * avail;
                scene.add(Primitive::Line {
                    x1: x,
                    y1: rows_bottom,
                    x2: x,
                    y2: rows_bottom + 4.0,
                    stroke: axis_color.clone(),
                    stroke_width: 1.0,
                    stroke_dasharray: None,
                });
                scene.add(Primitive::Text {
                    x,
                    y: rows_bottom + 6.0 + AXIS_LABEL_SIZE as f64,
                    content: fmt_num(fmin + frac * span),
                    size: AXIS_LABEL_SIZE,
                    anchor: TextAnchor::Middle,
                    rotate: None,
                    bold: false,
                    color: Some(text_color.clone()),
                });
            }
        }

        // ── Metric heatboxes ──────────────────────────────────────────────────────────
        let cell_w = heat_col_w - HEAT_CELL_GAP;
        let cell_h = (row_h * 0.85).clamp(2.0, HEAT_COL_W);
        let na_fill = Color::Css(self.na_color.as_str().into());
        // Value-in-box mode: draw the number with the heat on the border, but only where a
        // cell is large enough for legible text; otherwise fall back to a solid fill.
        let value_mode = self.show_metric_values && cell_h >= 11.0 && cell_w >= 16.0;
        let value_size = (cell_h * 0.55).clamp(5.0, 9.0);
        let box_bg = Color::Css(self.theme.background.as_str().into());
        for (c, metric) in self.metrics.iter().enumerate() {
            let cx = heat_x + c as f64 * heat_col_w + HEAT_CELL_GAP / 2.0;
            for i in 0..n {
                let y = row_center(i) - cell_h / 2.0;
                match (metric.normalized(i), value_mode) {
                    (Some(t), true) => {
                        // Heat on the border, value in the box.
                        let heat = Color::Css(metric.colormap.map(t).into());
                        scene.add(Primitive::Rect {
                            x: cx,
                            y,
                            width: cell_w,
                            height: cell_h,
                            fill: box_bg.clone(),
                            stroke: Some(heat),
                            stroke_width: Some(2.0),
                            opacity: None,
                        });
                        if let Some(v) = metric.values.get(i).copied().flatten() {
                            scene.add(Primitive::Text {
                                x: cx + cell_w / 2.0,
                                y: row_center(i) + center_offset(value_size, FontStyle::Regular),
                                content: fmt_num(v),
                                size: value_size.round() as u32,
                                anchor: TextAnchor::Middle,
                                rotate: None,
                                bold: false,
                                color: Some(text_color.clone()),
                            });
                        }
                    }
                    (opt, _) => {
                        let fill = match opt {
                            Some(t) => Color::Css(metric.colormap.map(t).into()),
                            None => na_fill.clone(),
                        };
                        scene.add(Primitive::Rect {
                            x: cx,
                            y,
                            width: cell_w,
                            height: cell_h,
                            fill,
                            stroke: None,
                            stroke_width: None,
                            opacity: None,
                        });
                    }
                }
            }
        }

        // Build the brick plot now so its motif legend entries can be harvested for the
        // bottom legend; the brick itself is rendered last (below).
        let plots = vec![Plot::Brick(self.brick)];

        // ── Metric colourbars: side by side on the right, title vertical on each one's left
        if has_metrics {
            let region_x = width - colorbar_region_w;
            let cb_top = force_top + 8.0;
            let seg_h = COLORBAR_H / COLORBAR_STEPS as f64;
            for (c, metric) in self.metrics.iter().enumerate() {
                let unit_left = region_x + c as f64 * COLORBAR_UNIT_W;
                let bar_x = unit_left + 18.0;
                // Vertical title on the bar's left, reading bottom-to-top. Anchored at the
                // bar's bottom (left-justified) so titles of different lengths still line up
                // when the bars sit side by side.
                scene.add(Primitive::Text {
                    x: unit_left + 8.0 + center_offset(HEADER_SIZE as f64, FontStyle::Regular),
                    y: cb_top + COLORBAR_H,
                    content: metric.label.clone(),
                    size: HEADER_SIZE,
                    anchor: TextAnchor::Start,
                    rotate: Some(-90.0),
                    bold: false,
                    color: Some(text_color.clone()),
                });
                // Gradient bar (max at top, min at bottom).
                for s in 0..COLORBAR_STEPS {
                    let t = 1.0 - (s as f64 + 0.5) / COLORBAR_STEPS as f64;
                    scene.add(Primitive::Rect {
                        x: bar_x,
                        y: cb_top + s as f64 * seg_h,
                        width: COLORBAR_W,
                        height: seg_h + 0.5,
                        fill: Color::Css(metric.colormap.map(t).into()),
                        stroke: None,
                        stroke_width: None,
                        opacity: None,
                    });
                }
                if let Some((lo, hi)) = metric.range() {
                    let vx = bar_x + COLORBAR_W + 4.0;
                    scene.add(Primitive::Text {
                        x: vx,
                        y: cb_top + center_offset(AXIS_LABEL_SIZE as f64, FontStyle::Regular),
                        content: fmt_num(hi),
                        size: AXIS_LABEL_SIZE,
                        anchor: TextAnchor::Start,
                        rotate: None,
                        bold: false,
                        color: Some(text_color.clone()),
                    });
                    scene.add(Primitive::Text {
                        x: vx,
                        y: cb_top
                            + COLORBAR_H
                            + center_offset(AXIS_LABEL_SIZE as f64, FontStyle::Regular),
                        content: fmt_num(lo),
                        size: AXIS_LABEL_SIZE,
                        anchor: TextAnchor::Start,
                        rotate: None,
                        bold: false,
                        color: Some(text_color.clone()),
                    });
                }
            }
        }

        // ── Motif legend: an embedded LegendPlot grid along the bottom (like the standalone
        // brick / bladerunner). Motif lists get large for complex loci, so it wraps into as
        // many rows as needed across the full width left of the colourbars.
        if brick_has_legend && legend_band_h > 0.0 {
            let entries = collect_legend_entries(&plots);
            if !entries.is_empty() {
                // Pin both cols and max_cols so LegendPlot's height-driven bump-up can't
                // diverge from the band we reserved above.
                let lp = LegendPlot::from_entries(entries)
                    .with_cols(legend_cols)
                    .with_max_cols(legend_cols);
                let llayout = Layout::new((0.0, 1.0), (0.0, 1.0))
                    .with_width(legend_width)
                    .with_height(legend_band_h);
                let lscene = render_multiple(vec![lp.into()], llayout);
                merge_translated(&mut scene, lscene, 0.0, height - legend_band_h);
            }
        }

        // ── Brick column (embedded BrickPlot, row band pinned to align with the panels) ─
        let mut blayout = Layout::auto_from_plots(&plots)
            .with_width(brick_w)
            .with_height(height)
            .with_force_margins_y(force_top, force_bottom);
        // Allele names live in our gutter, and motif swatches in the shared legend, so drop
        // the brick's own y-ticks and legend.
        blayout.suppress_y_ticks = true;
        blayout.show_legend = false;
        let bscene = render_multiple(plots, blayout);
        merge_translated(&mut scene, bscene, brick_x, 0.0);

        scene
    }
}
