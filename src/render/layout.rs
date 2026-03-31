use std::sync::Arc;
use crate::render::render_utils;
use crate::render::plots::Plot;
use crate::render::annotations::{TextAnnotation, ReferenceLine, ShadedRegion};
use crate::render::theme::Theme;
use crate::render::palette::Palette;
use crate::plot::legend::{LegendEntry, LegendGroup, LegendPosition};
use crate::render::datetime::DateTimeAxis;

/// Default font-family stack applied when the user has not specified a font
/// and no theme font is set.  Prefers DejaVu Sans (pre-installed on most Linux
/// systems including HPC clusters), falls back through common sans-serif fonts.
pub(crate) const DEFAULT_FONT_FAMILY: &str =
    "DejaVu Sans, Liberation Sans, Arial, sans-serif";

/// Controls how tick labels are formatted on an axis.
pub enum TickFormat {
    /// Smart default: integers as "5", minimal decimals, scientific notation for extremes.
    Auto,
    /// Exactly n decimal places: `Fixed(2)` → `"3.14"`.
    Fixed(usize),
    /// Round to nearest integer: `"5"`.
    Integer,
    /// ASCII scientific notation: `"1.23e4"`, `"3.5e-2"`.
    Sci,
    /// Multiply by 100 and append `%`: `0.45` → `"45.0%"`.
    Percent,
    /// Theta degree for polar plots: `0.0` → `"0°"`, `90.0` → `"90°"`.
    Degree,
    /// Custom formatter function.
    Custom(Arc<dyn Fn(f64) -> String + Send + Sync>),
}

impl std::fmt::Debug for TickFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto      => write!(f, "TickFormat::Auto"),
            Self::Fixed(n)  => write!(f, "TickFormat::Fixed({n})"),
            Self::Integer   => write!(f, "TickFormat::Integer"),
            Self::Sci       => write!(f, "TickFormat::Sci"),
            Self::Percent   => write!(f, "TickFormat::Percent"),
            Self::Degree    => write!(f, "TickFormat::Degree"),
            Self::Custom(_) => write!(f, "TickFormat::Custom(<fn>)"),
        }
    }
}

impl Clone for TickFormat {
    fn clone(&self) -> Self {
        match self {
            Self::Auto      => Self::Auto,
            Self::Fixed(n)  => Self::Fixed(*n),
            Self::Integer   => Self::Integer,
            Self::Sci       => Self::Sci,
            Self::Percent   => Self::Percent,
            Self::Degree    => Self::Degree,
            Self::Custom(f) => Self::Custom(Arc::clone(f)),
        }
    }
}

impl TickFormat {
    pub fn format(&self, v: f64) -> String {
        // IEEE 754 negative zero (-0.0 == 0.0 but formats as "-0"). Normalise
        // it to positive zero so no formatter can produce "-0" on a tick label.
        let v = if v == 0.0 { 0.0 } else { v };
        match self {
            Self::Auto      => tick_format_auto(v),
            Self::Fixed(n)  => format!("{:.*}", n, v),
            Self::Integer   => format!("{:.0}", v),
            Self::Sci       => tick_format_sci(v),
            Self::Percent   => format!("{:.1}%", v * 100.0),
            Self::Degree    => tick_format_degree(v),
            Self::Custom(f) => f(v),
        }
    }
}

fn tick_format_degree(v: f64) -> String {
    if v == 0.0 {
        "0°".to_string()
    } else {
        format!("{}°", v as i64)
    }
}

fn tick_format_auto(v: f64) -> String {
    if v.fract().abs() < 1e-9 {
        format!("{:.0}", v)
    } else if v.abs() >= 10_000.0 || (v != 0.0 && v.abs() < 0.01) {
        tick_format_sci(v)
    } else {
        let s = format!("{:.3}", v);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}

fn tick_format_sci(v: f64) -> String {
    let raw = format!("{:e}", v);
    // raw looks like "1.23e4" or "1e0" or "3.5e-3"
    if let Some(e_pos) = raw.find('e') {
        let mantissa = &raw[..e_pos];
        let exponent = &raw[e_pos + 1..];
        // Strip trailing zeros from mantissa
        let mantissa = if mantissa.contains('.') {
            let m = mantissa.trim_end_matches('0').trim_end_matches('.');
            m
        } else {
            mantissa
        };
        if exponent == "0" {
            mantissa.to_string()
        } else {
            format!("{}e{}", mantissa, exponent)
        }
    } else {
        raw
    }
}

/// Defines the layout of the plot
pub struct Layout {
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub x_range: (f64, f64),
    pub y_range: (f64, f64),
    /// Raw data range before padding (used by log scale to avoid pad_min issues)
    pub data_x_range: Option<(f64, f64)>,
    pub data_y_range: Option<(f64, f64)>,
    pub ticks: usize,
    pub show_grid: bool,
    pub x_label: Option<String>,
    pub y_label: Option<String>,
    pub title: Option<String>,
    pub x_categories: Option<Vec<String>>,
    pub y_categories: Option<Vec<String>>,
    pub show_legend: bool,
    pub show_colorbar: bool,
    pub legend_position: LegendPosition,
    pub legend_width: f64,
    /// Manual legend entries. When `Some`, replaces auto-collection from plot data.
    pub legend_entries: Option<Vec<LegendEntry>>,
    /// Optional title rendered as a bold header above legend entries.
    pub legend_title: Option<String>,
    /// Grouped legend sections. When `Some`, takes priority over `legend_entries`.
    pub legend_groups: Option<Vec<LegendGroup>>,
    /// Draw background + border rects around the legend. Default: true.
    pub legend_box: bool,
    /// Override the computed legend height. When `None`, height is auto-computed from
    /// the number of entries/groups. Set explicitly via `with_legend_height(px)`.
    pub legend_height: Option<f64>,
    pub log_x: bool,
    pub log_y: bool,
    pub annotations: Vec<TextAnnotation>,
    pub reference_lines: Vec<ReferenceLine>,
    pub shaded_regions: Vec<ShadedRegion>,
    pub suppress_x_ticks: bool,
    pub suppress_y_ticks: bool,
    pub font_family: Option<String>,
    pub title_size: u32,
    pub label_size: u32,
    pub tick_size: u32,
    pub body_size: u32,
    /// Override axis line stroke width (px at scale=1). `None` = use scale default (1.0).
    pub axis_line_width: Option<f64>,
    /// Override tick mark stroke width (px at scale=1). `None` = use scale default (1.0).
    pub tick_width: Option<f64>,
    /// Override major tick mark length (px at scale=1). `None` = use scale default (5.0).
    /// Minor tick length scales proportionally (60% of major).
    pub tick_length: Option<f64>,
    /// Override grid line stroke width (px at scale=1). `None` = use scale default (1.0).
    pub grid_line_width: Option<f64>,
    pub theme: Theme,
    pub palette: Option<Palette>,
    pub x_tick_format: TickFormat,
    pub y_tick_format: TickFormat,
    pub colorbar_tick_format: TickFormat,
    pub y2_range: Option<(f64, f64)>,
    pub data_y2_range: Option<(f64, f64)>,
    pub y2_label: Option<String>,
    pub log_y2: bool,
    pub y2_tick_format: TickFormat,
    pub suppress_y2_ticks: bool,
    pub x_datetime: Option<DateTimeAxis>,
    pub y_datetime: Option<DateTimeAxis>,
    pub x_tick_rotate: Option<f64>,
    /// When true, the computed axis range snaps to the tick boundary that just
    /// contains the data — no extra breathing-room step is added.  Useful for
    /// cases like `TickFormat::Percent` where you want the axis to stop exactly
    /// at 100 % rather than extending to 110 % or 120 %.
    pub clamp_axis: bool,
    /// Like `clamp_axis` but only for the y-axis.  Set automatically by
    /// `auto_from_plots` when all histograms in the plot list are normalized
    /// (so that the y-axis tops out at exactly 1.0, not 1.1).
    pub clamp_y_axis: bool,
    /// Bin width detected from histogram data by `auto_from_plots`.  When set,
    /// the x-axis range is taken from the raw data range (no rounding outward)
    /// and ticks are generated as integer multiples of this width so they fall
    /// exactly on bar edges.  `None` when no histograms are present or when
    /// multiple overlapping histograms have differing bin widths.
    pub x_bin_width: Option<f64>,
    /// Number of character rows in the terminal target.  When set, legend
    /// `line_height` is quantised to an integer multiple of the cell height so
    /// that every legend entry lands on its own terminal row with no gaps.
    pub term_rows: Option<u32>,
    /// Override the lower bound of the x-axis after auto-ranging.
    pub x_axis_min: Option<f64>,
    /// Override the upper bound of the x-axis after auto-ranging.
    pub x_axis_max: Option<f64>,
    /// Override the lower bound of the y-axis after auto-ranging.
    pub y_axis_min: Option<f64>,
    /// Override the upper bound of the y-axis after auto-ranging.
    pub y_axis_max: Option<f64>,
    /// Explicit major tick step for the x-axis.  Skips auto computation when set.
    pub x_tick_step: Option<f64>,
    /// Explicit major tick step for the y-axis.  Skips auto computation when set.
    pub y_tick_step: Option<f64>,
    /// Sub-intervals between major ticks (e.g. 5 → 4 minor marks per gap).
    pub minor_ticks: Option<u32>,
    /// Draw faint gridlines at minor tick positions (requires `minor_ticks`).
    pub show_minor_grid: bool,
    /// Pixel offset applied to the x-axis label after auto-positioning: `(dx, dy)`.
    /// Positive dx shifts right; positive dy shifts down.
    pub x_label_offset: (f64, f64),
    /// Pixel offset applied to the y-axis label after auto-positioning: `(dx, dy)`.
    /// Positive dx shifts right (away from the left edge); positive dy shifts down.
    pub y_label_offset: (f64, f64),
    /// Pixel offset applied to the y2-axis label after auto-positioning: `(dx, dy)`.
    /// Positive dx shifts right (further from the right axis); positive dy shifts down.
    pub y2_label_offset: (f64, f64),
    /// Uniform scale factor for all plot chrome (font sizes, margins, tick marks,
    /// legend geometry, arrow sizes). Canvas `width`/`height` are not affected.
    /// Default: 1.0. Set via `with_scale(f)`.
    pub scale: f64,
    /// Angular position (in degrees) at which r-axis (ring) labels are drawn on
    /// polar plots. Default: midpoint between the 0° spoke and the first clockwise
    /// spoke (`360 / (theta_divisions * 2)`). Override to avoid overlap with
    /// custom theta tick labels.
    pub polar_r_label_angle: Option<f64>,
    /// When `true`, the SVG backend injects interactive CSS, JavaScript, and
    /// `data-*` attributes so the chart responds to hover, click, and search.
    pub interactive: bool,
}

impl Layout {
    pub fn new(x_range: (f64, f64), y_range: (f64, f64)) -> Self {
        Self {
            width: None,
            height: None,
            x_range,
            y_range,
            data_x_range: None,
            data_y_range: None,
            ticks: 5,
            show_grid: true,
            x_label: None,
            y_label: None,
            title: None,
            x_categories: None,
            y_categories: None,
            show_legend: false,
            show_colorbar: false,
            legend_position: LegendPosition::OutsideRightTop,
            legend_width: 120.0,
            legend_entries: None,
            legend_title: None,
            legend_groups: None,
            legend_box: true,
            legend_height: None,
            log_x: false,
            log_y: false,
            annotations: Vec::new(),
            reference_lines: Vec::new(),
            shaded_regions: Vec::new(),
            suppress_x_ticks: false,
            suppress_y_ticks: false,
            font_family: None,
            title_size: 18,
            label_size: 14,
            tick_size: 12,
            body_size: 12,
            axis_line_width: None,
            tick_width: None,
            tick_length: None,
            grid_line_width: None,
            theme: Theme::default(),
            palette: None,
            x_tick_format: TickFormat::Auto,
            y_tick_format: TickFormat::Auto,
            colorbar_tick_format: TickFormat::Auto,
            y2_range: None,
            data_y2_range: None,
            y2_label: None,
            log_y2: false,
            y2_tick_format: TickFormat::Auto,
            suppress_y2_ticks: false,
            x_datetime: None,
            y_datetime: None,
            x_tick_rotate: None,
            clamp_axis: false,
            clamp_y_axis: false,
            x_bin_width: None,
            term_rows: None,
            x_axis_min: None,
            x_axis_max: None,
            y_axis_min: None,
            y_axis_max: None,
            x_tick_step: None,
            y_tick_step: None,
            minor_ticks: None,
            show_minor_grid: false,
            x_label_offset: (0.0, 0.0),
            y_label_offset: (0.0, 0.0),
            y2_label_offset: (0.0, 0.0),
            scale: 1.0,
            polar_r_label_angle: None,
            interactive: false,
        }
    }

    pub fn auto_from_data(data: &[f64], x_range: std::ops::Range<f64>) -> Self {
        let y_min = 0.0;
        let y_max = data.iter().cloned().fold(0.0, f64::max);

        Layout::new((x_range.start, x_range.end), (y_min, y_max * 1.05))
    }

    pub fn auto_from_plots(plots: &[Plot]) -> Self {
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;

        let mut x_labels = None;
        let mut y_labels = None;

        let mut has_legend: bool = false;
        let mut has_colorbar: bool = false;
        let mut has_manhattan: bool = false;
        let mut has_polar: bool = false;
        let mut max_label_len: usize = 0;

        for plot in plots {
            if let Some(((xmin, xmax), (ymin, ymax))) = plot.bounds() {
                x_min = x_min.min(xmin);
                x_max = x_max.max(xmax);
                y_min = y_min.min(ymin);
                y_max = y_max.max(ymax);
            }

            if let Plot::Strip(sp) = plot {
                let labels = sp.groups.iter().map(|g| g.label.clone()).collect();
                x_labels = Some(labels);
                if let Some(ref label) = sp.legend_label {
                    has_legend = true;
                    max_label_len = max_label_len.max(label.len());
                }
            }

            if let Plot::Box(bp) = plot {
                let labels = bp.groups.iter().map(|g| g.label.clone()).collect::<Vec<_>>();
                x_labels = Some(labels);
            }

            if let Plot::Violin(vp) = plot {
                let labels = vp.groups.iter().map(|g| g.label.clone()).collect::<Vec<_>>();
                x_labels = Some(labels);
            }

            if let Plot::Waterfall(wp) = plot {
                let labels = wp.bars.iter().map(|b| b.label.clone()).collect::<Vec<_>>();
                x_labels = Some(labels);
            }

            if let Plot::Bar(bp) = plot {
                let labels = bp.groups.iter().map(|g| g.label.clone()).collect::<Vec<_>>();
                x_labels = Some(labels);
                if let Some(ref ll) = bp.legend_label {
                    has_legend = true;
                    for l in ll {
                        max_label_len = max_label_len.max(l.len());
                    }
                }
            }

            if let Plot::Scatter(sp) = plot {
                if let Some(ref label) = sp.legend_label {
                    has_legend = true;
                    max_label_len = max_label_len.max(label.len());
                }
            }

            if let Plot::Line(lp) = plot {
                if let Some(ref label) = lp.legend_label {
                    has_legend = true;
                    max_label_len = max_label_len.max(label.len());
                }
            }

            if let Plot::Series(sp) = plot {
                if let Some(ref label) = sp.legend_label {
                    has_legend = true;
                    max_label_len = max_label_len.max(label.len());
                }
            }
            if let Plot::Brick(bp) = plot {
                let labels = bp.names.to_vec();
                y_labels = Some(labels);
                has_legend = true;
                if let Some(ref motifs) = bp.motifs {
                    for v in motifs.values() {
                        max_label_len = max_label_len.max(v.len());
                    }
                }
            }

            if let Plot::Pie(pp) = plot {
                if let Some(ref _label) = pp.legend_label {
                    has_legend = true;
                    let total: f64 = pp.slices.iter().map(|s| s.value).sum();
                    for slice in &pp.slices {
                        let entry_label = if pp.show_percent {
                            let pct = slice.value / total * 100.0;
                            format!("{} ({:.1}%)", slice.label, pct)
                        } else {
                            slice.label.clone()
                        };
                        max_label_len = max_label_len.max(entry_label.len());
                    }
                }
            }

            if matches!(plot, Plot::Heatmap(_) | Plot::Histogram2d(_)) {
                has_colorbar = true;
            }

            if let Plot::Volcano(vp) = plot {
                if vp.legend_label.is_some() {
                    has_legend = true;
                    max_label_len = max_label_len.max(4); // "Down"
                }
            }

            if let Plot::Manhattan(mp) = plot {
                if mp.legend_label.is_some() {
                    has_legend = true;
                    max_label_len = max_label_len.max(12); // "Genome-wide"
                }
                has_manhattan = true;
            }

            if let Plot::DotPlot(dp) = plot {
                x_labels = Some(dp.x_categories.clone());
                // Reverse so y_cat[0] appears at the TOP (map_y maps larger values to top)
                y_labels = Some(dp.y_categories.iter().rev().cloned().collect());
                let dot_has_both = dp.size_label.is_some() && dp.color_legend_label.is_some();
                // Colorbar handled by stacked renderer when both are present
                if dp.color_legend_label.is_some() && !dot_has_both {
                    has_colorbar = true;
                }
                if dp.size_label.is_some() {
                    has_legend = true;
                    // Entry labels are short numbers like "100.0" (5 chars)
                    max_label_len = max_label_len.max(5);
                }
            }

            if let Plot::StackedArea(sa) = plot {
                for label in sa.labels.iter().flatten() {
                    has_legend = true;
                    max_label_len = max_label_len.max(label.len());
                }
            }

            if let Plot::Candlestick(cp) = plot {
                let continuous = cp.candles.iter().any(|c| c.x.is_some());
                if !continuous {
                    let labels = cp.candles.iter().map(|c| c.label.clone()).collect();
                    x_labels = Some(labels);
                }
                if let Some(ref label) = cp.legend_label {
                    has_legend = true;
                    max_label_len = max_label_len.max(label.len());
                }
            }

            if let Plot::Contour(cp) = plot {
                if cp.filled {
                    has_colorbar = true;
                }
                if let Some(ref label) = cp.legend_label {
                    if !cp.filled {
                        has_legend = true;
                        max_label_len = max_label_len.max(label.len());
                    }
                }
            }

            if let Plot::Chord(cp) = plot {
                if cp.legend_label.is_some() {
                    has_legend = true;
                    for label in &cp.labels {
                        max_label_len = max_label_len.max(label.len());
                    }
                }
            }

            if let Plot::Sankey(sp) = plot {
                if sp.legend_label.is_some() {
                    has_legend = true;
                    for node in &sp.nodes {
                        max_label_len = max_label_len.max(node.label.len());
                    }
                }
            }

            if let Plot::PhyloTree(t) = plot {
                if t.legend_label.is_some() {
                    has_legend = true;
                    for (node_id, _) in &t.clade_colors {
                        let llen = t.nodes[*node_id].label.as_deref().unwrap_or("").len();
                        max_label_len = max_label_len.max(llen);
                    }
                }
            }

            if let Plot::Synteny(sp) = plot {
                if sp.legend_label.is_some() {
                    has_legend = true;
                    for seq in &sp.sequences {
                        max_label_len = max_label_len.max(seq.label.len());
                    }
                }
            }

            if let Plot::Density(dp) = plot {
                if let Some(ref label) = dp.legend_label {
                    has_legend = true;
                    max_label_len = max_label_len.max(label.len());
                }
            }

            if let Plot::Forest(fp) = plot {
                // Reversed: row[0] at top, map_y maps larger values to top
                y_labels = Some(fp.rows.iter().rev().map(|r| r.label.clone()).collect());
                if let Some(ref label) = fp.legend_label {
                    has_legend = true;
                    max_label_len = max_label_len.max(label.len());
                }
            }

            if let Plot::Ridgeline(rp) = plot {
                // Reversed: group[0] at top, map_y maps larger values to top
                y_labels = Some(rp.groups.iter().rev().map(|g| g.label.clone()).collect());
                if rp.show_legend {
                    has_legend = true;
                    for g in &rp.groups {
                        max_label_len = max_label_len.max(g.label.len());
                    }
                }
            }

            if let Plot::Polar(pp) = plot {
                has_polar = true;
                if pp.show_legend {
                    has_legend = true;
                    for s in &pp.series {
                        if let Some(ref lbl) = s.label {
                            max_label_len = max_label_len.max(lbl.len());
                        }
                    }
                }
            }

            if let Plot::Ternary(tp) = plot {
                if tp.show_legend {
                    has_legend = true;
                    for g in tp.unique_groups() {
                        max_label_len = max_label_len.max(g.len());
                    }
                }
            }
        }

        // Save raw data range before padding (log scale needs it)
        let raw_x = (x_min, x_max);
        let raw_y = (y_min, y_max);

        // Add a small margin so data points don't land exactly on axis edges.
        // Category-based plots (bar, box, violin, brick) already have built-in
        // padding in their bounds(), so only pad continuous-axis plots.
        // Grid-based plots (heatmap, histogram2d) also skip padding.
        //
        // Strategy: add 1% of the data span to max (and symmetrically to negative
        // min). This is just enough to push an exact tick-boundary value above the
        // boundary so that auto_nice_range's ceil moves it up by exactly one step,
        // avoiding the old flat "+1" which could expand a 0-1 range to 0-2.
        let has_x_cats = x_labels.is_some();
        let has_y_cats = y_labels.is_some();
        if !has_x_cats && !has_colorbar && x_max > x_min {
            let x_span = x_max - x_min;
            if x_min > 0.0 && x_min > x_span {
                // Large positive offset (e.g. years, genomic positions): padding
                // relative to the absolute value would push the axis to start at 0.
                // Instead pad by a fraction of the data range.
                let pad = x_span * 0.05;
                x_min -= pad;
                x_max += pad;
            } else {
                x_max += x_span * 0.01;
                if x_min >= 0.0 {
                    x_min = 0.0;
                } else {
                    x_min -= x_span * 0.01;
                }
            }
        }
        if !has_y_cats && !has_colorbar && y_max > y_min {
            let y_span = y_max - y_min;
            y_max += y_span * 0.01;
            if y_min >= 0.0 {
                y_min = 0.0;
            } else {
                y_min -= y_span * 0.01;
            }
        }

        let mut layout = Self::new((x_min, x_max), (y_min, y_max));
        layout.data_x_range = Some(raw_x);
        layout.data_y_range = Some(raw_y);
        if let Some(labels) = x_labels {
            layout = layout.with_x_categories(labels);
        }

        if let Some(labels) = y_labels {
            layout = layout.with_y_categories(labels);
        }

        // DotPlot with both size legend + colorbar uses a single stacked column
        let has_dot_stacked = plots.iter().any(|p| {
            if let Plot::DotPlot(dp) = p {
                dp.size_label.is_some() && dp.color_legend_label.is_some()
            } else { false }
        });

        if has_legend {
            layout = layout.with_show_legend();
            let dynamic_width = max_label_len as f64 * 8.5 + 35.0;
            layout.legend_width = dynamic_width.max(80.0);
        }

        if has_dot_stacked {
            // Single column wide enough for the stacked colorbar + size-legend
            layout.legend_width = 75.0;
        }

        if has_colorbar {
            layout.show_colorbar = true;
        }

        if has_manhattan {
            // Suppress numeric x tick labels and tick marks; chromosome names are drawn by add_manhattan.
            layout.x_tick_format = TickFormat::Custom(Arc::new(|_| String::new()));
            layout.suppress_x_ticks = true;
            // Disable horizontal grid lines so threshold lines pop out clearly.
            layout.show_grid = false;
        }

        if has_polar {
            // Use degrees as default tick format for polar plots.
            layout.x_tick_format = TickFormat::Degree;
        }

        // UpSet plots manage their own axes; disable the standard grid.
        if plots.iter().any(|p| matches!(p, Plot::UpSet(_))) {
            layout.show_grid = false;
        }

        // For normalized histograms the y range is always [0, 1].  Clamp the
        // y-axis so it stops at exactly 1.0 rather than rounding up to 1.1.
        // Only activate when every histogram in the list is normalized (mixing
        // normalized with un-normalized histograms produces a y_max that is a
        // count, not 1.0, so clamping is unnecessary there).
        let any_hist = plots.iter().any(|p| matches!(p, Plot::Histogram(_)));
        let all_normalized = plots.iter().all(|p| match p {
            Plot::Histogram(h) => h.normalize,
            _ => true, // non-histogram plots don't vote
        });
        if any_hist && all_normalized {
            layout.clamp_y_axis = true;
        }

        // Collect bin widths from all histograms.  When every histogram shares
        // the same bin width (the common case, including overlapping histograms
        // with a shared range), store it so the axis code can generate ticks
        // that fall exactly on bar edges.
        if any_hist {
            let bin_widths: Vec<f64> = plots.iter().filter_map(|p| {
                if let Plot::Histogram(h) = p {
                    if let Some((edges, _)) = &h.precomputed {
                        if edges.len() >= 2 {
                            let bw = edges[1] - edges[0];
                            let uniform = edges.windows(2).all(|w| (w[1] - w[0] - bw).abs() < 1e-9 * bw.abs().max(1e-10));
                            if uniform { return Some(bw); }
                        }
                        return None;
                    }
                    h.range.map(|r| (r.1 - r.0) / h.bins as f64)
                } else {
                    None
                }
            }).collect();
            if !bin_widths.is_empty() {
                let first = bin_widths[0];
                if bin_widths.iter().all(|&bw| (bw - first).abs() < 1e-9 * first.abs().max(1e-10)) {
                    layout.x_bin_width = Some(first);
                }
            }
        }

        layout
    }


    pub fn with_x_categories(mut self, labels: Vec<String>) -> Self {
        self.x_categories = Some(labels);
        self
    }

    pub fn with_y_categories(mut self, labels: Vec<String>) -> Self {
        self.y_categories = Some(labels);
        self
    }

    pub fn with_width(mut self, width: f64) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: f64) -> Self {
        self.height = Some(height);
        self
    }

    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_x_label<S: Into<String>>(mut self, label: S) -> Self {
        self.x_label = Some(label.into());
        self
    }

    pub fn with_y_label<S: Into<String>>(mut self, label: S) -> Self {
        self.y_label = Some(label.into());
        self
    }

    /// Shift the x-axis label by `(dx, dy)` pixels from its auto-computed position.
    /// Positive `dx` moves right; positive `dy` moves down.
    pub fn with_x_label_offset(mut self, dx: f64, dy: f64) -> Self {
        self.x_label_offset = (dx, dy);
        self
    }

    /// Shift the y-axis label by `(dx, dy)` pixels from its auto-computed position.
    /// Positive `dx` moves right (away from the left edge); positive `dy` moves down.
    pub fn with_y_label_offset(mut self, dx: f64, dy: f64) -> Self {
        self.y_label_offset = (dx, dy);
        self
    }

    pub fn with_ticks(mut self, ticks: usize) -> Self {
        self.ticks = ticks;
        self
    }

    pub fn with_show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    fn with_show_legend(mut self) -> Self {
        self.show_legend = true;
        self
    }

    pub fn with_legend_position(mut self, pos: LegendPosition) -> Self {
        self.legend_position = pos;
        self
    }

    /// Supply `Vec<LegendEntry>` directly, bypassing auto-collection from plot data.
    /// Auto-sizes `legend_width` from the longest label.
    pub fn with_legend_entries(mut self, entries: Vec<LegendEntry>) -> Self {
        let max_chars = entries.iter().map(|e| e.label.len()).max().unwrap_or(4);
        self.legend_width = (max_chars as f64 * 8.5 + 35.0).max(80.0);
        self.show_legend = true;
        self.legend_entries = Some(entries);
        self
    }

    /// Place legend at absolute SVG canvas pixel coordinates; no right-margin reserved.
    pub fn with_legend_at(mut self, x: f64, y: f64) -> Self {
        self.legend_position = LegendPosition::Custom(x, y);
        self.show_legend = true;
        self
    }

    /// Place the legend at data-space coordinates, mapped through `map_x`/`map_y` at render time.
    pub fn with_legend_at_data(mut self, x: f64, y: f64) -> Self {
        self.legend_position = LegendPosition::DataCoords(x, y);
        self.show_legend = true;
        self
    }

    /// Show or hide the legend background and border box (default: `true`).
    pub fn with_legend_box(mut self, show: bool) -> Self {
        self.legend_box = show;
        self
    }

    /// Set a bold title row above legend entries.
    /// Also widens `legend_width` if the title text is wider than the current box.
    pub fn with_legend_title<S: Into<String>>(mut self, title: S) -> Self {
        let t = title.into();
        // Title is centre-anchored; needs legend_width >= title_px + 10 to stay inside the box.
        let needed = (t.len() as f64 * 8.5 + 10.0).max(80.0);
        if needed > self.legend_width {
            self.legend_width = needed;
        }
        self.legend_title = Some(t);
        self
    }

    /// Add a labelled group of legend entries. Multiple calls stack; takes priority over
    /// `with_legend_entries`.
    /// Also widens `legend_width` to accommodate the group title and entry labels.
    pub fn with_legend_group<S: Into<String>>(mut self, title: S, entries: Vec<LegendEntry>) -> Self {
        let t = title.into();
        // Group title is start-anchored at legend_x+5; needs legend_width >= title_px + 10.
        let needed_title = (t.len() as f64 * 8.5 + 10.0).max(80.0);
        // Entry labels start at legend_x+25 (after swatch); same formula as with_legend_entries.
        let max_entry_chars = entries.iter().map(|e| e.label.len()).max().unwrap_or(0);
        let needed_entries = (max_entry_chars as f64 * 8.5 + 35.0).max(80.0);
        self.legend_width = self.legend_width.max(needed_title).max(needed_entries);
        self.legend_groups.get_or_insert_with(Vec::new).push(LegendGroup {
            title: t,
            entries,
        });
        self.show_legend = true;
        self
    }

    /// Override the auto-computed legend width. Use when labels overflow the default box.
    pub fn with_legend_width(mut self, px: f64) -> Self {
        self.legend_width = px;
        self
    }

    /// Override the auto-computed legend height. Use when content overflows the default box.
    pub fn with_legend_height(mut self, px: f64) -> Self {
        self.legend_height = Some(px);
        self
    }

    /// Set a uniform scale factor for all plot chrome.
    ///
    /// Multiplies font sizes, margins, tick mark lengths, legend padding and swatch
    /// geometry, and annotation arrow sizes.  Canvas `width`/`height` are **not**
    /// scaled — the user controls those independently (or relies on auto-sizing).
    ///
    /// Useful for producing large SVG exports without manually adjusting every size
    /// parameter.  For raster PNG output at higher DPI, use `PngBackend`'s DPI scale
    /// instead.
    ///
    /// `TextAnnotation::font_size` and `ReferenceLine::stroke_width` are user-set
    /// and are **not** auto-scaled; set them explicitly if needed.
    ///
    /// Clamped to a minimum of 0.1 to prevent degenerate sub-pixel rendering.
    pub fn with_scale(mut self, f: f64) -> Self {
        self.scale = f.max(0.1);
        self
    }

    /// Override the angle (degrees) at which r-axis labels are drawn on polar plots.
    ///
    /// By default, labels sit at the midpoint between the 0° spoke and the first
    /// clockwise spoke (`360 / (theta_divisions * 2)`). Use this to nudge them when
    /// a custom theta tick label would overlap.
    ///
    /// ```rust,no_run
    /// use kuva::render::layout::Layout;
    /// use kuva::plot::polar::{PolarPlot, PolarMode};
    /// use kuva::render::plots::Plot;
    ///
    /// let plot = PolarPlot::new().with_series(vec![1.0_f64], vec![0.0_f64]);
    /// let plots = vec![Plot::Polar(plot)];
    /// let layout = Layout::auto_from_plots(&plots)
    ///     .with_polar_r_label_angle(30.0); // labels at 30° from north
    /// ```
    pub fn with_polar_r_label_angle(mut self, deg: f64) -> Self {
        self.polar_r_label_angle = Some(deg);
        self
    }

    /// Enable SVG interactivity: hover highlighting, click-to-pin, search box,
    /// coordinate readout, and legend-driven dim/highlight.
    pub fn with_interactive(mut self) -> Self {
        self.interactive = true;
        self
    }

    pub fn with_log_x(mut self) -> Self {
        self.log_x = true;
        self
    }

    pub fn with_log_y(mut self) -> Self {
        self.log_y = true;
        self
    }

    pub fn with_log_scale(mut self) -> Self {
        self.log_x = true;
        self.log_y = true;
        self
    }

    pub fn with_annotation(mut self, annotation: TextAnnotation) -> Self {
        self.annotations.push(annotation);
        self
    }

    pub fn with_reference_line(mut self, line: ReferenceLine) -> Self {
        self.reference_lines.push(line);
        self
    }

    pub fn with_shaded_region(mut self, region: ShadedRegion) -> Self {
        self.shaded_regions.push(region);
        self
    }

    pub fn with_font_family<S: Into<String>>(mut self, family: S) -> Self {
        self.font_family = Some(family.into());
        self
    }

    pub fn with_title_size(mut self, size: u32) -> Self {
        self.title_size = size;
        self
    }

    pub fn with_label_size(mut self, size: u32) -> Self {
        self.label_size = size;
        self
    }

    pub fn with_tick_size(mut self, size: u32) -> Self {
        self.tick_size = size;
        self
    }

    pub fn with_body_size(mut self, size: u32) -> Self {
        self.body_size = size;
        self
    }

    /// Set the axis line stroke width in logical pixels (at scale 1.0).
    /// Affects the X and Y axis border lines only, not ticks or grid.
    pub fn with_axis_line_width(mut self, width: f64) -> Self {
        self.axis_line_width = Some(width);
        self
    }

    /// Set the tick mark stroke width in logical pixels (at scale 1.0).
    pub fn with_tick_width(mut self, width: f64) -> Self {
        self.tick_width = Some(width);
        self
    }

    /// Set the major tick mark length in logical pixels (at scale 1.0).
    /// Minor tick length is scaled proportionally (60% of major).
    pub fn with_tick_length(mut self, length: f64) -> Self {
        self.tick_length = Some(length);
        self
    }

    /// Set the grid line stroke width in logical pixels (at scale 1.0).
    pub fn with_grid_line_width(mut self, width: f64) -> Self {
        self.grid_line_width = Some(width);
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.show_grid = theme.show_grid;
        if let Some(ref font) = theme.font_family {
            self.font_family = Some(font.clone());
        }
        self.theme = theme;
        self
    }

    pub fn with_palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set the same tick format for both axes.
    pub fn with_tick_format(mut self, fmt: TickFormat) -> Self {
        self.x_tick_format = fmt.clone();
        self.y_tick_format = fmt;
        self
    }

    /// Set the tick format for the x-axis only.
    pub fn with_x_tick_format(mut self, fmt: TickFormat) -> Self {
        self.x_tick_format = fmt;
        self
    }

    /// Set the tick format for the y-axis only.
    pub fn with_y_tick_format(mut self, fmt: TickFormat) -> Self {
        self.y_tick_format = fmt;
        self
    }

    /// Set the tick format for colorbar labels. Default: [`TickFormat::Auto`]
    /// (switches to scientific notation for values ≥ 10 000 or ≤ 0.01).
    pub fn with_colorbar_tick_format(mut self, fmt: TickFormat) -> Self {
        self.colorbar_tick_format = fmt;
        self
    }

    pub fn with_y2_range(mut self, min: f64, max: f64) -> Self {
        self.y2_range = Some((min, max));
        self
    }

    pub fn with_y2_label<S: Into<String>>(mut self, label: S) -> Self {
        self.y2_label = Some(label.into());
        self
    }

    /// Shift the y2-axis label by `(dx, dy)` pixels from its auto-computed position.
    /// Positive `dx` moves right (further from the right axis); positive `dy` moves down.
    pub fn with_y2_label_offset(mut self, dx: f64, dy: f64) -> Self {
        self.y2_label_offset = (dx, dy);
        self
    }

    pub fn with_log_y2(mut self) -> Self {
        self.log_y2 = true;
        self
    }

    pub fn with_y2_tick_format(mut self, fmt: TickFormat) -> Self {
        self.y2_tick_format = fmt;
        self
    }

    pub fn with_x_datetime(mut self, axis: DateTimeAxis) -> Self {
        self.x_datetime = Some(axis);
        self
    }

    pub fn with_y_datetime(mut self, axis: DateTimeAxis) -> Self {
        self.y_datetime = Some(axis);
        self
    }

    pub fn with_x_tick_rotate(mut self, angle: f64) -> Self {
        self.x_tick_rotate = Some(angle);
        self
    }

    /// Snap both axes to the tick boundary that just contains the data,
    /// with no extra breathing-room step.  Useful for `TickFormat::Percent`
    /// (so the axis stops at 100 % instead of 110 %) or any domain where the
    /// data naturally fills the full scale.
    pub fn with_clamp_axis(mut self) -> Self {
        self.clamp_axis = true;
        self
    }

    /// Like `with_clamp_axis` but only for the y-axis.  Set automatically by
    /// `auto_from_plots` for normalized histograms; can also be used manually.
    pub fn with_clamp_y_axis(mut self) -> Self {
        self.clamp_y_axis = true;
        self
    }

    /// Auto-compute y2_range from secondary plots, also expanding x_range to cover them.
    pub fn with_y2_auto(mut self, secondary: &[Plot]) -> Self {
        let mut x_min = self.x_range.0;
        let mut x_max = self.x_range.1;
        let mut y2_min = f64::INFINITY;
        let mut y2_max = f64::NEG_INFINITY;
        let mut max_secondary_label: usize = 0;
        for plot in secondary {
            if let Some(((xlo, xhi), (ylo, yhi))) = plot.bounds() {
                x_min = x_min.min(xlo);
                x_max = x_max.max(xhi);
                y2_min = y2_min.min(ylo);
                y2_max = y2_max.max(yhi);
            }
            // Collect legend label lengths so legend_width covers secondary labels too.
            match plot {
                Plot::Scatter(p)     => if let Some(l) = &p.legend_label { max_secondary_label = max_secondary_label.max(l.len()); }
                Plot::Line(p)        => if let Some(l) = &p.legend_label { max_secondary_label = max_secondary_label.max(l.len()); }
                Plot::Series(p)      => if let Some(l) = &p.legend_label { max_secondary_label = max_secondary_label.max(l.len()); }
                Plot::Band(p)        => if let Some(l) = &p.legend_label { max_secondary_label = max_secondary_label.max(l.len()); }
                Plot::Histogram(p)   => if let Some(l) = &p.legend_label { max_secondary_label = max_secondary_label.max(l.len()); }
                Plot::Box(p)         => if let Some(l) = &p.legend_label { max_secondary_label = max_secondary_label.max(l.len()); }
                Plot::Violin(p)      => if let Some(l) = &p.legend_label { max_secondary_label = max_secondary_label.max(l.len()); }
                Plot::Strip(p)       => if let Some(l) = &p.legend_label { max_secondary_label = max_secondary_label.max(l.len()); }
                Plot::Waterfall(p)   => if let Some(l) = &p.legend_label { max_secondary_label = max_secondary_label.max(l.len()); }
                Plot::Candlestick(p) => if let Some(l) = &p.legend_label { max_secondary_label = max_secondary_label.max(l.len()); }
                Plot::StackedArea(p) => for l in p.labels.iter().flatten() { max_secondary_label = max_secondary_label.max(l.len()); }
                Plot::Bar(p)         => if let Some(ll) = &p.legend_label { for l in ll { max_secondary_label = max_secondary_label.max(l.len()); } }
                _ => {}
            }
        }
        if max_secondary_label > 0 {
            let needed = max_secondary_label as f64 * 8.5 + 35.0;
            if needed > self.legend_width {
                self.legend_width = needed;
                self.show_legend = true;
            }
        }
        self.x_range = (x_min, x_max);
        let raw = (y2_min, y2_max);
        self.data_y2_range = Some(raw);
        if y2_max > y2_min {
            let y2_span = y2_max - y2_min;
            y2_max += y2_span * 0.01;
            if y2_min >= 0.0 {
                y2_min = 0.0;
            } else {
                y2_min -= y2_span * 0.01;
            }
        }
        self.y2_range = Some((y2_min, y2_max));
        self
    }

    pub fn with_term_rows(mut self, rows: u32) -> Self {
        self.term_rows = Some(rows);
        self
    }

    pub fn with_x_axis_min(mut self, v: f64) -> Self { self.x_axis_min = Some(v); self }
    pub fn with_x_axis_max(mut self, v: f64) -> Self { self.x_axis_max = Some(v); self }
    pub fn with_y_axis_min(mut self, v: f64) -> Self { self.y_axis_min = Some(v); self }
    pub fn with_y_axis_max(mut self, v: f64) -> Self { self.y_axis_max = Some(v); self }
    pub fn with_x_tick_step(mut self, s: f64) -> Self { self.x_tick_step = Some(s); self }
    pub fn with_y_tick_step(mut self, s: f64) -> Self { self.y_tick_step = Some(s); self }
    pub fn with_minor_ticks(mut self, n: u32) -> Self { self.minor_ticks = Some(n); self }
    pub fn with_show_minor_grid(mut self, v: bool) -> Self { self.show_minor_grid = v; self }

    /// Convenience: auto-range both axes from separate plot lists.
    pub fn auto_from_twin_y_plots(primary: &[Plot], secondary: &[Plot]) -> Self {
        Layout::auto_from_plots(primary).with_y2_auto(secondary)
    }
}


#[derive(Clone)]
pub struct ComputedLayout {
    pub width: f64,
    pub height: f64,
    pub margin_top: f64,
    pub margin_bottom: f64,
    pub margin_left: f64,
    pub margin_right: f64,

    pub x_range: (f64, f64),
    pub y_range: (f64, f64),
    pub x_ticks: usize,
    pub y_ticks: usize,
    pub legend_position: LegendPosition,
    pub legend_width: f64,
    /// Optional explicit legend height override from `Layout::with_legend_height`.
    pub legend_height_override: Option<f64>,
    /// Pixel width of the widest y-axis tick label, computed from actual tick strings.
    /// Used in `axis.rs` to position the Y axis label flush with the tick labels.
    pub y_tick_label_px: f64,
    pub log_x: bool,
    pub log_y: bool,
    pub font_family: Option<String>,
    pub title_size: u32,
    pub label_size: u32,
    pub tick_size: u32,
    pub body_size: u32,
    pub theme: Theme,
    pub x_tick_format: TickFormat,
    pub y_tick_format: TickFormat,
    pub colorbar_tick_format: TickFormat,
    pub y2_range: Option<(f64, f64)>,
    pub log_y2: bool,
    pub y2_tick_format: TickFormat,
    /// Pixel width consumed by the y2 axis (ticks + labels). 0.0 when no y2 axis.
    pub y2_axis_width: f64,
    /// Rotation angle for x-axis tick labels (degrees, typically -45.0). None = no rotation.
    pub x_tick_rotate: Option<f64>,
    /// Pixel spacing between legend entries, quantised to a whole terminal-row
    /// multiple when `term_rows` is set.  Always >= 18.0 (the SVG default).
    pub legend_line_height: f64,
    /// Explicit major tick step for the x-axis (None = auto).
    pub x_tick_step: Option<f64>,
    /// Explicit major tick step for the y-axis (None = auto).
    pub y_tick_step: Option<f64>,
    /// Sub-intervals between major ticks for minor tick marks.
    pub minor_ticks: Option<u32>,
    /// Draw faint gridlines at minor tick positions.
    pub show_minor_grid: bool,
    /// Common bin width when all histograms share the same bin size.
    /// When set, x-axis ticks are generated to fall exactly on bin edges.
    pub x_bin_width: Option<f64>,
    /// Angular position (degrees) at which r-axis labels are drawn on polar plots.
    /// `None` means auto (midpoint between 0° spoke and first clockwise spoke).
    pub polar_r_label_angle: Option<f64>,
    /// Scaled pixel constants for rendering, derived from `layout.scale`.
    /// Avoids threading the scale factor through every render function.
    pub tick_mark_major: f64,       // 5.0 * scale (or layout.tick_length * scale)
    pub tick_mark_minor: f64,       // 3.0 * scale (60% of major)
    pub tick_label_margin: f64,     // 8.0 * scale — gap from axis line to tick label text
    pub axis_stroke_width: f64,     // 1.0 * scale — base stroke width (annotations, plot shapes)
    pub axis_line_width: f64,       // axis border lines (overridable via Layout::with_axis_line_width)
    pub tick_stroke_width: f64,     // tick mark strokes (overridable via Layout::with_tick_width)
    pub grid_stroke_width: f64,     // grid line strokes (overridable via Layout::with_grid_line_width)
    pub legend_padding: f64,        // 10.0 * scale — legend box internal padding
    pub legend_inset: f64,          // 8.0 * scale — Inside legend inset from plot edge
    pub legend_swatch_size: f64,    // 12.0 * scale — Rect/Line swatch length and height
    pub legend_swatch_x: f64,       // 5.0 * scale — swatch left inset within legend box
    pub legend_text_x: f64,         // 25.0 * scale — label text left inset within legend box
    pub legend_swatch_r: f64,       // 5.0 * scale — Circle swatch radius
    pub legend_swatch_half: f64,    // 8.0 * scale — CircleSize cap radius
    pub annotation_arrow_len: f64,  // 8.0 * scale — annotation arrowhead length
    pub annotation_arrow_half_w: f64, // 4.0 * scale — annotation arrowhead half-width
    pub colorbar_bar_width: f64,    // 20.0 * scale — colorbar bar rect width
    pub colorbar_x_inset: f64,      // 70.0 * scale — colorbar position from canvas right

    // Pre-computed linear transform coefficients for map_x / map_y.
    // map_x(x) = x_offset + x * x_scale  (linear)
    // map_x(x) = x_offset + log10(x) * x_scale  (log)
    x_scale: f64,
    x_offset: f64,
    y_scale: f64,
    y_offset: f64,
    /// Mirror of `Layout::interactive` — propagated so renderers can access it.
    pub interactive: bool,
}

impl ComputedLayout {
    pub fn from_layout(layout: &Layout) -> Self {
        let s = layout.scale.max(0.1);
        let title_size = layout.title_size as f64 * s;
        let label_size = layout.label_size as f64 * s;
        let tick_size = layout.tick_size as f64 * s;
        // Compute tick mark length early — needed for margin_left and tick_label_margin.
        let tick_mark_major_px = layout.tick_length.map(|l| l * s).unwrap_or(5.0 * s);

        // Top: title height + padding, or small padding if no title
        let mut margin_top = if layout.title.is_some() {
            title_size + label_size + 12.0 * s
        } else {
            10.0 * s
        };
        // Bottom: tick_mark + gap(5) + tick_label + gap(5) + axis_label + padding(10)
        // When ticks are suppressed AND no rotation is requested (e.g. pure numeric axes),
        // keep only minimal space. When rotation IS set (e.g. Manhattan chromosome labels drawn
        // by the renderer itself), compute space for the rotated custom labels.
        let mut margin_bottom = if layout.suppress_x_ticks && layout.x_tick_rotate.is_none() {
            tick_size + 15.0 * s
        } else if let Some(angle) = layout.x_tick_rotate {
            // Rotated labels extend below their anchor point by label_px * sin(|angle|).
            let char_w = tick_size * 0.6;
            let max_chars = layout.x_categories.as_ref()
                .and_then(|cats| cats.iter().map(|s| s.len()).max())
                .unwrap_or(10) as f64;
            let label_px = max_chars * char_w;
            let angle_rad = angle.abs() * std::f64::consts::PI / 180.0;
            let needed = label_px * angle_rad.sin() + tick_size + tick_mark_major_px + 10.0 * s;
            needed.max(tick_size + label_size + tick_mark_major_px + 20.0 * s)
        } else {
            tick_size + label_size + tick_mark_major_px + 20.0 * s
        };
        // Left: axis label + y tick label text width + gaps.
        // Compute the actual maximum tick label pixel width from real tick strings so the
        // left margin is exactly as wide as needed and the Y axis label snugs up against
        // the tick labels rather than sitting at a fixed canvas-edge offset.
        //
        // Layout (left→right):  [3px edge] [Y-label] [5px gap] [tick labels] [8px gap] [axis]
        //   → margin_left = label_size + y_tick_label_px + 16
        let y_tick_label_px: f64 = if layout.suppress_y_ticks {
            0.0
        } else if let Some(ref cats) = layout.y_categories {
            let max_chars = cats.iter().map(|s| s.len()).max().unwrap_or(4) as f64;
            (max_chars * tick_size * 0.6).max(tick_size * 2.0)
        } else if layout.log_y {
            let ticks_log = render_utils::generate_ticks_log(
                layout.y_range.0.max(1e-300), layout.y_range.1.max(1e-300),
            );
            let max_chars = ticks_log.iter()
                .map(|&v| render_utils::format_log_tick(v).len())
                .max().unwrap_or(3) as f64;
            (max_chars * tick_size * 0.6).max(tick_size * 2.0)
        } else if layout.y_datetime.is_some() {
            tick_size * 5.0 // datetime labels vary; ~5 char-widths is a reasonable default
        } else {
            // Generate a preliminary set of tick values from the raw y_range (no auto-ranging
            // yet) and format them to find the widest label string.  Using layout.y_range
            // rather than the final auto-ranged range is fine here — the formatted width
            // changes very little after nice-rounding.
            let n = if layout.ticks > 0 { layout.ticks } else { 5 };
            let tick_vals = if let Some(step) = layout.y_tick_step {
                render_utils::generate_ticks_with_step(layout.y_range.0, layout.y_range.1, step)
            } else {
                render_utils::generate_ticks(layout.y_range.0, layout.y_range.1, n)
            };
            let max_chars = tick_vals.iter()
                .map(|&v| layout.y_tick_format.format(v).len())
                .max().unwrap_or(3) as f64;
            (max_chars * tick_size * 0.6).max(tick_size * 2.0)
        };
        let mut margin_left = if layout.suppress_y_ticks {
            10.0 * s
        } else {
            // 16px = 3 edge + 5 label-to-ticklabels gap + 8 tick_label_margin base;
            // tick_mark_major_px is added separately so the margin grows with tick length.
            label_size + y_tick_label_px + 16.0 * s + tick_mark_major_px
        };
        // Estimate the overhang of the rightmost numeric x-tick label.
        // Tick labels are centred on their tick position (TextAnchor::Middle), so the
        // last tick (at x_max) extends half its pixel width to the right of the plot edge.
        // Without this, labels like "15000" or "100.5" clip against the SVG boundary.
        // Uses layout.x_range.1 / x_axis_max as a proxy — nice-rounding rarely changes
        // the label length, mirroring how y_tick_label_px uses layout.y_range before
        // auto-ranging (lines ~1174-1187 above).
        let x_last_tick_half_w: f64 = if layout.suppress_x_ticks
            || layout.x_categories.is_some()
            || layout.x_tick_rotate.is_some()
            || layout.log_x
        {
            0.0 // handled elsewhere or not applicable
        } else {
            let val = layout.x_axis_max.unwrap_or(layout.x_range.1);
            let label = layout.x_tick_format.format(val);
            label.len() as f64 * tick_size * 0.6 * 0.5
        };
        let mut margin_right = label_size.max(x_last_tick_half_w);

        // For rotated x-axis category labels the text extends horizontally from its anchor.
        // Negative angle → TextAnchor::End → extends left  → first label can clip left edge.
        // Positive angle → TextAnchor::Start → extends right → last label can clip right edge.
        if let Some(angle) = layout.x_tick_rotate {
            if !layout.suppress_x_ticks {
                if let Some(ref cats) = layout.x_categories {
                    let char_w = tick_size * 0.6;
                    let angle_rad = angle.abs() * std::f64::consts::PI / 180.0;
                    let cos_a = angle_rad.cos();
                    if angle < 0.0 {
                        if let Some(first) = cats.first() {
                            let needed = first.len() as f64 * char_w * cos_a;
                            if needed > margin_left { margin_left = needed; }
                        }
                    } else if let Some(last) = cats.last() {
                        let needed = last.len() as f64 * char_w * cos_a;
                        if needed > margin_right { margin_right = needed; }
                    }
                }
            }
        }

        let y2_axis_width = if layout.y2_range.is_some() && !layout.suppress_y2_ticks {
            label_size + tick_size * 3.0 + 15.0 * s
        } else {
            0.0
        };
        margin_right += y2_axis_width;

        if layout.show_legend {
            // Estimate legend height for OutsideTop/Bottom margin adjustments.
            let legend_line_h = 18.0 * s;
            let legend_h_estimate = if let Some(ref groups) = layout.legend_groups {
                let n = groups.iter().map(|g| g.entries.len() + 1).sum::<usize>();
                n as f64 * legend_line_h + 20.0 * s
            } else if let Some(ref entries) = layout.legend_entries {
                entries.len() as f64 * legend_line_h + 20.0 * s
            } else {
                80.0 * s // conservative default for auto-collected entries
            };
            match layout.legend_position {
                LegendPosition::OutsideRightTop
                | LegendPosition::OutsideRightMiddle
                | LegendPosition::OutsideRightBottom => {
                    margin_right += layout.legend_width * s;
                }
                LegendPosition::OutsideLeftTop
                | LegendPosition::OutsideLeftMiddle
                | LegendPosition::OutsideLeftBottom => {
                    margin_left += layout.legend_width * s;
                }
                LegendPosition::OutsideTopLeft
                | LegendPosition::OutsideTopCenter
                | LegendPosition::OutsideTopRight => {
                    margin_top += legend_h_estimate;
                }
                LegendPosition::OutsideBottomLeft
                | LegendPosition::OutsideBottomCenter
                | LegendPosition::OutsideBottomRight => {
                    margin_bottom += legend_h_estimate;
                }
                // Inside*, Custom, DataCoords: overlay or user controls — no margin change
                _ => {}
            }
        }
        if layout.show_colorbar {
            margin_right += 90.0 * s; // 20px label-gap + 20px bar + 5px tick-mark + 30px tick labels + 15px gap
        }
        let plot_width = 600.0;
        let plot_height = 450.0;

        // Reserve space below the plot for the interactive UI strip.
        // Only applies when height is auto-computed; user-fixed heights are left unchanged.
        if layout.interactive && layout.height.is_none() {
            margin_bottom += 32.0;
        }

        let width = layout.width.unwrap_or(margin_left + plot_width + margin_right);
        let height = layout.height.unwrap_or(margin_top + plot_height + margin_bottom);

        let x_ticks = if layout.ticks > 0 { layout.ticks } else { render_utils::auto_tick_count(width) };
        let y_ticks = if layout.ticks > 0 { layout.ticks } else { render_utils::auto_tick_count(height) };

        // For log scale, prefer the raw data range (before proportional padding).
        // For clamp_axis, also use the raw range so the boundary lands on the
        // tick that just contains the data with no extra step.
        let (x_min, x_max) = if layout.log_x {
            let (xlo, xhi) = layout.data_x_range.unwrap_or(layout.x_range);
            render_utils::auto_nice_range_log(xlo, xhi)
        } else if layout.clamp_axis {
            let (xlo, xhi) = layout.data_x_range.unwrap_or(layout.x_range);
            render_utils::auto_nice_range(xlo, xhi, x_ticks)
        } else if layout.x_bin_width.is_some() {
            // Histogram: use the exact data range so ticks start and end on bin
            // boundaries rather than being rounded outward by auto_nice_range.
            let (xlo, xhi) = layout.data_x_range.unwrap_or(layout.x_range);
            (xlo, xhi)
        } else {
            render_utils::auto_nice_range(layout.x_range.0, layout.x_range.1, x_ticks)
        };
        let (y_min, y_max) = if layout.log_y {
            let (ylo, yhi) = layout.data_y_range.unwrap_or(layout.y_range);
            render_utils::auto_nice_range_log(ylo, yhi)
        } else if layout.clamp_axis || layout.clamp_y_axis {
            let (ylo, yhi) = layout.data_y_range.unwrap_or(layout.y_range);
            render_utils::auto_nice_range(ylo, yhi, y_ticks)
        } else {
            render_utils::auto_nice_range(layout.y_range.0, layout.y_range.1, y_ticks)
        };

        // Apply explicit axis-range overrides (after auto-ranging).
        let x_min = layout.x_axis_min.unwrap_or(x_min);
        let x_max = layout.x_axis_max.unwrap_or(x_max);
        let y_min = layout.y_axis_min.unwrap_or(y_min);
        let y_max = layout.y_axis_max.unwrap_or(y_max);

        let y2_range = if let Some((ylo, yhi)) = layout.y2_range {
            if layout.log_y2 {
                let (ylo, yhi) = layout.data_y2_range.unwrap_or((ylo, yhi));
                Some(render_utils::auto_nice_range_log(ylo, yhi))
            } else if layout.clamp_axis {
                let (ylo, yhi) = layout.data_y2_range.unwrap_or((ylo, yhi));
                Some(render_utils::auto_nice_range(ylo, yhi, y_ticks))
            } else {
                Some(render_utils::auto_nice_range(ylo, yhi, y_ticks))
            }
        } else {
            None
        };

        // Quantise legend line-height to a whole number of terminal rows so that
        // every legend entry maps to a distinct row without gaps.
        let legend_line_height = if let Some(tr) = layout.term_rows {
            let cell_h = height / tr as f64;
            let rows_per_entry = ((18.0 * s) / cell_h).round().max(1.0);
            rows_per_entry * cell_h
        } else {
            18.0 * s
        };

        let mut s = Self {
            width,
            height,
            margin_top,
            margin_bottom,
            margin_left,
            margin_right,
            x_range: (x_min, x_max),
            y_range: (y_min, y_max),
            x_ticks,
            y_ticks,
            legend_position: layout.legend_position,
            legend_width: layout.legend_width * s,
            legend_height_override: layout.legend_height.map(|h| h * s),
            y_tick_label_px,
            log_x: layout.log_x,
            log_y: layout.log_y,
            font_family: layout.font_family.clone()
                .or(layout.theme.font_family.clone())
                .or(Some(DEFAULT_FONT_FAMILY.to_string())),
            title_size: (layout.title_size as f64 * s).round().max(1.0) as u32,
            label_size: (layout.label_size as f64 * s).round().max(1.0) as u32,
            tick_size:  (layout.tick_size  as f64 * s).round().max(1.0) as u32,
            body_size:  (layout.body_size  as f64 * s).round().max(1.0) as u32,
            theme: layout.theme.clone(),
            x_tick_format: layout.x_tick_format.clone(),
            y_tick_format: layout.y_tick_format.clone(),
            colorbar_tick_format: layout.colorbar_tick_format.clone(),
            y2_range,
            log_y2: layout.log_y2,
            y2_tick_format: layout.y2_tick_format.clone(),
            y2_axis_width,
            x_tick_rotate: layout.x_tick_rotate,
            legend_line_height,
            x_tick_step: layout.x_tick_step,
            y_tick_step: layout.y_tick_step,
            minor_ticks: layout.minor_ticks,
            show_minor_grid: layout.show_minor_grid,
            x_bin_width: layout.x_bin_width,
            polar_r_label_angle: layout.polar_r_label_angle,
            tick_mark_major: tick_mark_major_px,
            tick_mark_minor: layout.tick_length.map(|l| l * s * 0.6).unwrap_or(3.0 * s),
            tick_label_margin: tick_mark_major_px + 3.0 * s,
            axis_stroke_width: s,
            axis_line_width: layout.axis_line_width.map(|w| w * s).unwrap_or(s),
            tick_stroke_width: layout.tick_width.map(|w| w * s).unwrap_or(s),
            grid_stroke_width: layout.grid_line_width.map(|w| w * s).unwrap_or(s),
            legend_padding: 10.0 * s,
            legend_inset: 8.0 * s,
            legend_swatch_size: 12.0 * s,
            legend_swatch_x: 5.0 * s,
            legend_text_x: 25.0 * s,
            legend_swatch_r: 5.0 * s,
            legend_swatch_half: 8.0 * s,
            annotation_arrow_len: 8.0 * s,
            annotation_arrow_half_w: 4.0 * s,
            colorbar_bar_width: 20.0 * s,
            colorbar_x_inset: 65.0 * s,
            x_scale: 0.0,
            x_offset: 0.0,
            y_scale: 0.0,
            y_offset: 0.0,
            interactive: layout.interactive,
        };
        s.recompute_transforms();
        s
    }

    /// Recompute cached linear-transform coefficients after changing
    /// width, height, margins, or axis ranges.
    pub fn recompute_transforms(&mut self) {
        let pw = self.plot_width();
        let ph = self.plot_height();
        if self.log_x {
            let log_min = self.x_range.0.max(1e-10).log10();
            let log_max = self.x_range.1.max(1e-10).log10();
            let span = log_max - log_min;
            self.x_scale = if span.abs() > f64::EPSILON { pw / span } else { 0.0 };
            self.x_offset = self.margin_left - log_min * self.x_scale;
        } else {
            let span = self.x_range.1 - self.x_range.0;
            self.x_scale = if span.abs() > f64::EPSILON { pw / span } else { 0.0 };
            self.x_offset = self.margin_left - self.x_range.0 * self.x_scale;
        }
        if self.log_y {
            let log_min = self.y_range.0.max(1e-10).log10();
            let log_max = self.y_range.1.max(1e-10).log10();
            let span = log_max - log_min;
            self.y_scale = if span.abs() > f64::EPSILON { ph / span } else { 0.0 };
            self.y_offset = self.height - self.margin_bottom + log_min * self.y_scale;
        } else {
            let span = self.y_range.1 - self.y_range.0;
            self.y_scale = if span.abs() > f64::EPSILON { ph / span } else { 0.0 };
            self.y_offset = self.height - self.margin_bottom + self.y_range.0 * self.y_scale;
        }
    }

    pub fn plot_width(&self) -> f64 {
        self.width - self.margin_left - self.margin_right
    }

    pub fn plot_height(&self) -> f64 {
        self.height - self.margin_top - self.margin_bottom
    }

    #[inline(always)]
    pub fn map_x(&self, x: f64) -> f64 {
        if self.log_x {
            self.x_offset + x.max(1e-10).log10() * self.x_scale
        } else {
            self.x_offset + x * self.x_scale
        }
    }

    #[inline(always)]
    pub fn map_y(&self, y: f64) -> f64 {
        if self.log_y {
            self.y_offset - y.max(1e-10).log10() * self.y_scale
        } else {
            self.y_offset - y * self.y_scale
        }
    }

    pub fn map_y2(&self, y: f64) -> f64 {
        if let Some((y2_min, y2_max)) = self.y2_range {
            let ph = self.plot_height();
            if self.log_y2 {
                let y = y.max(1e-10);
                let log_min = y2_min.log10();
                let log_max = y2_max.log10();
                self.height - self.margin_bottom
                    - (y.log10() - log_min) / (log_max - log_min) * ph
            } else {
                self.height - self.margin_bottom
                    - (y - y2_min) / (y2_max - y2_min) * ph
            }
        } else {
            self.map_y(y)
        }
    }

    /// Clone self with y_range = y2_range, log_y = log_y2, y_tick_format = y2_tick_format.
    /// Used to render secondary-axis plots through existing add_* functions unchanged.
    pub fn for_y2(&self) -> ComputedLayout {
        let mut c = self.clone();
        if let Some(y2) = self.y2_range {
            c.y_range = y2;
        }
        c.log_y = self.log_y2;
        c.y_tick_format = self.y2_tick_format.clone();
        c.recompute_transforms();
        c
    }
}

