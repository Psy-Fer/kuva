use crate::render::render_utils;
use crate::render::plots::Plot;

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
    pub show_legend: bool,
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
            show_legend: false,
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

        let mut has_legend: bool = false;
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
                if bp.legend_label.is_some() {
                    has_legend = true;
                }
            }

            if let Plot::Scatter(sp) = plot {
                if sp.legend_label.is_some() {
                    has_legend = true;
                }
            }

            if let Plot::Line(lp) = plot {
                if lp.legend_label.is_some() {
                    has_legend = true;
                }
            }

            if let Plot::Series(sp) = plot {
                if sp.legend_label.is_some() {
                    has_legend = true;
                }
            }
        }

        let mut layout = Self::new((x_min, x_max), (y_min, y_max));
        if let Some(labels) = x_labels {
            layout = layout.with_x_categories(labels);
        }

        if has_legend {
            layout = layout.with_show_legend();
        }


        layout
    }


    pub fn with_x_categories(mut self, labels: Vec<String>) -> Self {
        self.x_categories = Some(labels);
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
            margin_right += 120.0;
        }
        let plot_width = 400.0;
        let plot_height = 300.0;

        let width = layout.width.unwrap_or(margin_left + plot_width + margin_right);
        let height = layout.height.unwrap_or(margin_top + plot_height + margin_bottom);

        let x_ticks = render_utils::auto_tick_count(width);
        let y_ticks = render_utils::auto_tick_count(height);

        let (x_min, x_max) = render_utils::auto_nice_range(layout.x_range.0, layout.x_range.1, x_ticks);
        let (y_min, y_max) = render_utils::auto_nice_range(layout.y_range.0, layout.y_range.1, y_ticks);

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
        }
    }

    pub fn plot_width(&self) -> f64 {
        self.width - self.margin_left - self.margin_right
    }

    pub fn plot_height(&self) -> f64 {
        self.height - self.margin_top - self.margin_bottom
    }

    pub fn map_x(&self, x: f64) -> f64 {
        self.margin_left + (x - self.x_range.0) / (self.x_range.1 - self.x_range.0) * self.plot_width()
    }

    pub fn map_y(&self, y: f64) -> f64 {
        self.height - self.margin_bottom - (y - self.y_range.0) / (self.y_range.1 - self.y_range.0) * self.plot_height()
    }
}