use crate::render::render_utils;
use crate::render::plots::Plot;
use crate::render::annotations::{TextAnnotation, ReferenceLine, ShadedRegion};
use crate::plot::legend::LegendPosition;

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
        let mut max_label_len: usize = 0;

        for plot in plots {
            if let Some(((xmin, xmax), (ymin, ymax))) = plot.bounds() {
                x_min = x_min.min(xmin);
                x_max = x_max.max(xmax);
                y_min = y_min.min(ymin);
                y_max = y_max.max(ymax);
            }

            if let Plot::Box(bp) = plot {
                let labels = bp.groups.iter().map(|g| g.label.clone()).collect::<Vec<_>>();
                x_labels = Some(labels);
            }

            if let Plot::Violin(vp) = plot {
                let labels = vp.groups.iter().map(|g| g.label.clone()).collect::<Vec<_>>();
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
            x_max = pad_max(x_max);
            x_min = pad_min(x_min);
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

        if has_legend {
            layout = layout.with_show_legend();
            let dynamic_width = max_label_len as f64 * 7.0 + 35.0;
            layout.legend_width = dynamic_width.max(80.0);
        }

        if has_colorbar {
            layout.show_colorbar = true;
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
}


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
        let margin_bottom = if layout.suppress_x_ticks {
            10.0
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

        if layout.show_legend {
            margin_right += layout.legend_width;
        }
        if layout.show_colorbar {
            margin_right += 85.0; // 20px bar + 50px labels + 15px gap
        }
        let plot_width = 400.0;
        let plot_height = 300.0;

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
            font_family: layout.font_family.clone(),
            title_size: layout.title_size,
            label_size: layout.label_size,
            tick_size: layout.tick_size,
            body_size: layout.body_size,
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