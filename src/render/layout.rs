use crate::render::render_utils;
use crate::render::plots::Plot;
use crate::plot::legend::LegendPosition;

/// Defines the layout of the plot
pub struct Layout {
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub x_range: (f64, f64),
    pub y_range: (f64, f64),
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
}

impl Layout {
    pub fn new(x_range: (f64, f64), y_range: (f64, f64)) -> Self {
        Self {
            width: None,
            height: None,
            x_range,
            y_range,
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

            if matches!(plot, Plot::Heatmap(_) | Plot::Histogram2d(_)) {
                has_colorbar = true;
            }
        }

        let mut layout = Self::new((x_min, x_max), (y_min, y_max));
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
    pub ticks: usize,
    pub legend_position: LegendPosition,
    pub legend_width: f64,
    pub log_x: bool,
    pub log_y: bool,
}

impl ComputedLayout {
    pub fn from_layout(layout: &Layout) -> Self {
        let font_size = 14.0;
        let tick_space = 20.0;

        let margin_top = if layout.title.is_some() { font_size * 2.0 } else { font_size * 0.5 };
        let margin_bottom = font_size * 2.0 + tick_space;
        let margin_left = font_size * 2.0 + tick_space;
        let mut margin_right = font_size;

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

        let (x_min, x_max) = if layout.log_x {
            render_utils::auto_nice_range_log(layout.x_range.0, layout.x_range.1)
        } else {
            render_utils::auto_nice_range(layout.x_range.0, layout.x_range.1, x_ticks)
        };
        let (y_min, y_max) = if layout.log_y {
            render_utils::auto_nice_range_log(layout.y_range.0, layout.y_range.1)
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
            ticks: layout.ticks,
            legend_position: layout.legend_position,
            legend_width: layout.legend_width,
            log_x: layout.log_x,
            log_y: layout.log_y,
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