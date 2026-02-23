use std::sync::Arc;
use crate::render::render_utils;
use crate::render::plots::Plot;
use crate::render::annotations::{TextAnnotation, ReferenceLine, ShadedRegion};
use crate::render::theme::Theme;
use crate::render::palette::Palette;
use crate::plot::legend::LegendPosition;
use crate::render::datetime::DateTimeAxis;

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
    /// Custom formatter function.
    Custom(Arc<dyn Fn(f64) -> String + Send + Sync>),
}

impl Clone for TickFormat {
    fn clone(&self) -> Self {
        match self {
            Self::Auto      => Self::Auto,
            Self::Fixed(n)  => Self::Fixed(*n),
            Self::Integer   => Self::Integer,
            Self::Sci       => Self::Sci,
            Self::Percent   => Self::Percent,
            Self::Custom(f) => Self::Custom(Arc::clone(f)),
        }
    }
}

impl TickFormat {
    pub fn format(&self, v: f64) -> String {
        match self {
            Self::Auto      => tick_format_auto(v),
            Self::Fixed(n)  => format!("{:.*}", n, v),
            Self::Integer   => format!("{:.0}", v),
            Self::Sci       => tick_format_sci(v),
            Self::Percent   => format!("{:.1}%", v * 100.0),
            Self::Custom(f) => f(v),
        }
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
    pub theme: Theme,
    pub palette: Option<Palette>,
    pub x_tick_format: TickFormat,
    pub y_tick_format: TickFormat,
    pub y2_range: Option<(f64, f64)>,
    pub data_y2_range: Option<(f64, f64)>,
    pub y2_label: Option<String>,
    pub log_y2: bool,
    pub y2_tick_format: TickFormat,
    pub suppress_y2_ticks: bool,
    pub x_datetime: Option<DateTimeAxis>,
    pub y_datetime: Option<DateTimeAxis>,
    pub x_tick_rotate: Option<f64>,
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
            legend_position: LegendPosition::TopRight,
            legend_width: 120.0,
            log_x: false,
            log_y: false,
            annotations: Vec::new(),
            reference_lines: Vec::new(),
            shaded_regions: Vec::new(),
            suppress_x_ticks: false,
            suppress_y_ticks: false,
            font_family: None,
            title_size: 16,
            label_size: 14,
            tick_size: 10,
            body_size: 12,
            theme: Theme::default(),
            palette: None,
            x_tick_format: TickFormat::Auto,
            y_tick_format: TickFormat::Auto,
            y2_range: None,
            data_y2_range: None,
            y2_label: None,
            log_y2: false,
            y2_tick_format: TickFormat::Auto,
            suppress_y2_ticks: false,
            x_datetime: None,
            y_datetime: None,
            x_tick_rotate: None,
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
                let labels = bp.names.iter().map(|g| g.clone()).collect::<Vec<_>>();
                y_labels = Some(labels);
                has_legend = true;
                if let Some(ref motifs) = bp.motifs {
                    for (_k, v) in motifs {
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
        }

        // Save raw data range before padding (log scale needs it)
        let raw_x = (x_min, x_max);
        let raw_y = (y_min, y_max);

        // Add a small margin so data points don't land exactly on axis edges.
        // Category-based plots (bar, box, violin, brick) already have built-in
        // padding in their bounds(), so only pad continuous-axis plots.
        // Grid-based plots (heatmap, histogram2d) also skip padding.
        let has_x_cats = x_labels.is_some();
        let has_y_cats = y_labels.is_some();
        if !has_x_cats && !has_colorbar && x_max > x_min {
            let x_range = x_max - x_min;
            if x_min > 0.0 && x_min > x_range {
                // Large positive offset (e.g. years, genomic positions): padding
                // relative to the absolute value would push the axis to start at 0.
                // Instead pad by a fraction of the data range.
                let pad = x_range * 0.05;
                x_min -= pad;
                x_max += pad;
            } else {
                x_max = pad_max(x_max);
                x_min = pad_min(x_min);
            }
        }
        if !has_y_cats && !has_colorbar && y_max > y_min {
            y_max = pad_max(y_max);
            y_min = pad_min(y_min);
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
            let dynamic_width = max_label_len as f64 * 7.0 + 35.0;
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

        // UpSet plots manage their own axes; disable the standard grid.
        if plots.iter().any(|p| matches!(p, Plot::UpSet(_))) {
            layout.show_grid = false;
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

    pub fn with_y2_range(mut self, min: f64, max: f64) -> Self {
        self.y2_range = Some((min, max));
        self
    }

    pub fn with_y2_label<S: Into<String>>(mut self, label: S) -> Self {
        self.y2_label = Some(label.into());
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

    /// Auto-compute y2_range from secondary plots, also expanding x_range to cover them.
    pub fn with_y2_auto(mut self, secondary: &[Plot]) -> Self {
        let mut x_min = self.x_range.0;
        let mut x_max = self.x_range.1;
        let mut y2_min = f64::INFINITY;
        let mut y2_max = f64::NEG_INFINITY;
        for plot in secondary {
            if let Some(((xlo, xhi), (ylo, yhi))) = plot.bounds() {
                x_min = x_min.min(xlo);
                x_max = x_max.max(xhi);
                y2_min = y2_min.min(ylo);
                y2_max = y2_max.max(yhi);
            }
        }
        self.x_range = (x_min, x_max);
        let raw = (y2_min, y2_max);
        self.data_y2_range = Some(raw);
        if y2_max > y2_min {
            y2_max = pad_max(y2_max);
            y2_min = pad_min(y2_min);
        }
        self.y2_range = Some((y2_min, y2_max));
        self
    }

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
    pub y2_range: Option<(f64, f64)>,
    pub log_y2: bool,
    pub y2_tick_format: TickFormat,
    /// Pixel width consumed by the y2 axis (ticks + labels). 0.0 when no y2 axis.
    pub y2_axis_width: f64,
}

impl ComputedLayout {
    pub fn from_layout(layout: &Layout) -> Self {
        let title_size = layout.title_size as f64;
        let label_size = layout.label_size as f64;
        let tick_size = layout.tick_size as f64;

        // Top: title height + padding, or small padding if no title
        let margin_top = if layout.title.is_some() {
            title_size + label_size + 12.0
        } else {
            10.0
        };
        // Bottom: tick mark (5) + gap (5) + tick label + gap (5) + axis label + padding
        // When ticks are suppressed, still keep room for custom labels (e.g., chromosome names).
        let margin_bottom = if layout.suppress_x_ticks {
            tick_size + 15.0
        } else if layout.x_tick_rotate.is_some() {
            tick_size + label_size + 45.0
        } else {
            tick_size + label_size + 25.0
        };
        // Left: axis label + gap + tick label width + gap to axis
        let margin_left = if layout.suppress_y_ticks {
            10.0
        } else {
            label_size + tick_size * 3.0 + 15.0
        };
        let mut margin_right = label_size;

        let y2_axis_width = if layout.y2_range.is_some() && !layout.suppress_y2_ticks {
            label_size + tick_size * 3.0 + 15.0
        } else {
            0.0
        };
        margin_right += y2_axis_width;
        if layout.show_legend {
            margin_right += layout.legend_width;
        }
        if layout.show_colorbar {
            margin_right += 85.0; // 20px bar + 50px labels + 15px gap
        }
        let plot_width = 600.0;
        let plot_height = 450.0;

        let width = layout.width.unwrap_or(margin_left + plot_width + margin_right);
        let height = layout.height.unwrap_or(margin_top + plot_height + margin_bottom);

        let x_ticks = render_utils::auto_tick_count(width);
        let y_ticks = render_utils::auto_tick_count(height);

        // For log scale, prefer the raw data range (before pad_min clamped to 0)
        let (x_min, x_max) = if layout.log_x {
            let (xlo, xhi) = layout.data_x_range.unwrap_or(layout.x_range);
            render_utils::auto_nice_range_log(xlo, xhi)
        } else {
            render_utils::auto_nice_range(layout.x_range.0, layout.x_range.1, x_ticks)
        };
        let (y_min, y_max) = if layout.log_y {
            let (ylo, yhi) = layout.data_y_range.unwrap_or(layout.y_range);
            render_utils::auto_nice_range_log(ylo, yhi)
        } else {
            render_utils::auto_nice_range(layout.y_range.0, layout.y_range.1, y_ticks)
        };

        let y2_range = if let Some((ylo, yhi)) = layout.y2_range {
            if layout.log_y2 {
                let (ylo, yhi) = layout.data_y2_range.unwrap_or((ylo, yhi));
                Some(render_utils::auto_nice_range_log(ylo, yhi))
            } else {
                Some(render_utils::auto_nice_range(ylo, yhi, y_ticks))
            }
        } else {
            None
        };

        Self {
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
            legend_width: layout.legend_width,
            log_x: layout.log_x,
            log_y: layout.log_y,
            font_family: layout.font_family.clone().or(layout.theme.font_family.clone()),
            title_size: layout.title_size,
            label_size: layout.label_size,
            tick_size: layout.tick_size,
            body_size: layout.body_size,
            theme: layout.theme.clone(),
            x_tick_format: layout.x_tick_format.clone(),
            y_tick_format: layout.y_tick_format.clone(),
            y2_range,
            log_y2: layout.log_y2,
            y2_tick_format: layout.y2_tick_format.clone(),
            y2_axis_width,
        }
    }

    pub fn plot_width(&self) -> f64 {
        self.width - self.margin_left - self.margin_right
    }

    pub fn plot_height(&self) -> f64 {
        self.height - self.margin_top - self.margin_bottom
    }

    pub fn map_x(&self, x: f64) -> f64 {
        if self.log_x {
            let x = x.max(1e-10);
            let log_min = self.x_range.0.log10();
            let log_max = self.x_range.1.log10();
            self.margin_left + (x.log10() - log_min) / (log_max - log_min) * self.plot_width()
        } else {
            self.margin_left + (x - self.x_range.0) / (self.x_range.1 - self.x_range.0) * self.plot_width()
        }
    }

    pub fn map_y(&self, y: f64) -> f64 {
        if self.log_y {
            let y = y.max(1e-10);
            let log_min = self.y_range.0.log10();
            let log_max = self.y_range.1.log10();
            self.height - self.margin_bottom - (y.log10() - log_min) / (log_max - log_min) * self.plot_height()
        } else {
            self.height - self.margin_bottom - (y - self.y_range.0) / (self.y_range.1 - self.y_range.0) * self.plot_height()
        }
    }

    pub fn map_y2(&self, y: f64) -> f64 {
        if let Some((y2_min, y2_max)) = self.y2_range {
            if self.log_y2 {
                let y = y.max(1e-10);
                let log_min = y2_min.log10();
                let log_max = y2_max.log10();
                self.height - self.margin_bottom
                    - (y.log10() - log_min) / (log_max - log_min) * self.plot_height()
            } else {
                self.height - self.margin_bottom
                    - (y - y2_min) / (y2_max - y2_min) * self.plot_height()
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
        c
    }
}

/// Pad the maximum axis value so data doesn't sit on the edge.
/// Values below 10 get +1, values >= 10 get *1.05.
fn pad_max(v: f64) -> f64 {
    if v.abs() < 10.0 {
        v + 1.0
    } else {
        v * 1.05
    }
}

/// Pad the minimum axis value.
/// Non-negative values clamp to 0. Values between -10 and 0 get -1.
/// Values <= -10 get *1.05 (which makes them more negative).
fn pad_min(v: f64) -> f64 {
    if v >= 0.0 {
        0.0
    } else if v > -10.0 {
        v - 1.0
    } else {
        v * 1.05
    }
}