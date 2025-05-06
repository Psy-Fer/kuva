use crate::plot::scatter::ScatterPlot;
use crate::plot::line::LinePlot;


#[derive(Debug)]
pub enum Primitive {
    Circle {
        cx: f64,
        cy: f64,
        r: f64,
        fill: String,
    },
    Text {
        x: f64,
        y: f64,
        content: String,
        size: u32,
        anchor: TextAnchor,
        rotate: Option<f64>,
    },
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        stroke: String,
    },
    Path {
        d: String,
        stroke: String,
        stroke_width: f64,
    }
}

#[derive(Debug)]
pub enum TextAnchor {
    Start,
    Middle,
    End,
}

#[derive(Debug)]
pub struct Scene {
    pub width: f64,
    pub height: f64,
    pub background_color: Option<String>,
    pub elements: Vec<Primitive>,
}

impl Scene {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width,
               height,
               background_color: Some("white".to_string()),
               elements: vec![] }
    }

    pub fn with_background(mut self, color: Option<&str>) -> Self {
        self.background_color = color.map(|c| c.to_string());
        self

    }

    pub fn add(&mut self, p: Primitive) {
        self.elements.push(p);
    }
}


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
        }
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
        let margin_right = font_size;

        let plot_width = 400.0;
        let plot_height = 300.0;

        let width = layout.width.unwrap_or(margin_left + plot_width + margin_right);
        let height = layout.height.unwrap_or(margin_top + plot_height + margin_bottom);

        Self {
            width,
            height,
            margin_top,
            margin_bottom,
            margin_left,
            margin_right,
            x_range: layout.x_range,
            y_range: layout.y_range,
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


fn add_axes_and_grid(scene: &mut Scene, computed: &ComputedLayout, layout: &Layout) {

    let map_x = |x| computed.map_x(x);
    let map_y = |y| computed.map_y(y);

    // Draw axes
    // X axis
    scene.add(Primitive::Line {
        x1: computed.margin_left,
        y1: computed.height - computed.margin_bottom,
        x2: computed.width - computed.margin_right,
        y2: computed.height - computed.margin_bottom,
        stroke: "red".into(),
    });
    // Y axis
    scene.add(Primitive::Line {
        x1: computed.margin_left,
        y1: computed.margin_top,
        x2: computed.margin_left,
        y2: computed.height - computed.margin_bottom,
        stroke: "green".into(),
    });

    // Draw ticks and labels
    for i in 0..=computed.ticks {
        let tx = computed.x_range.0 + (i as f64) * (computed.x_range.1 - computed.x_range.0) / computed.ticks as f64;
        let ty = computed.y_range.0 + (i as f64) * (computed.y_range.1 - computed.y_range.0) / computed.ticks as f64;

        let x = map_x(tx);
        let y = map_y(ty);

        // X ticks
        scene.add(Primitive::Line {
            x1: x,
            y1: computed.height - computed.margin_bottom,
            x2: x,
            y2: computed.height - computed.margin_bottom + 5.0,
            stroke: "black".into(),
        });
        // X tick labels
        scene.add(Primitive::Text {
            x,
            y: computed.height - computed.margin_bottom + 15.0,
            content: format!("{:.1}", tx),
            size: 10,
            anchor: TextAnchor::Middle,
            rotate: None,
        });

        // Y ticks
        scene.add(Primitive::Line {
            x1: computed.margin_left - 5.0,
            y1: y,
            x2: computed.margin_left,
            y2: y,
            stroke: "black".into(),
        });
        // Y tick labels
        scene.add(Primitive::Text {
            x: computed.margin_left - 15.0,
            y,
            content: format!("{:.1}", ty),
            size: 10,
            anchor: TextAnchor::Middle,
            rotate: None,
        });

        // Grid lines
        if layout.show_grid {
            if i != 0 {
                // Vertical grid
                scene.add(Primitive::Line {
                    x1: x,
                    y1: computed.margin_top,
                    x2: x,
                    y2: computed.height - computed.margin_bottom,
                    stroke: "#ccc".to_string(),
                });
        
                // Horizontal grid
                scene.add(Primitive::Line {
                    x1: computed.margin_left,
                    y1: y,
                    x2: computed.width - computed.margin_right,
                    y2: y,
                    stroke: "#ccc".to_string(),
                });
            }
        }
    }
}

fn add_labels_and_title(scene: &mut Scene, computed: &ComputedLayout, layout: &Layout) {
    // X Axis Label
    if let Some(label) = &layout.x_label {
        scene.add(Primitive::Text {
            x: computed.width / 2.0,
            y: computed.height - computed.margin_bottom / 4.0,
            content: label.clone(),
            size: 14,
            anchor: TextAnchor::Middle,
            rotate: None,
        });
    }

    // Y Axis Label
    if let Some(label) = &layout.y_label {
        scene.add(Primitive::Text {
            x: 20.0,
            y: computed.height / 2.0,
            content: label.clone(),
            size: 14,
            anchor: TextAnchor::Middle,
            rotate: Some(-90.0),
        });
    }

    // Title
    if let Some(title) = &layout.title {
        scene.add(Primitive::Text {
            x: computed.width / 2.0,
            y: computed.margin_top / 2.0,
            content: title.clone(),
            size: 16,
            anchor: TextAnchor::Middle,
            rotate: None,
        });
    }
}

fn add_scatter(scatter: &ScatterPlot, scene: &mut Scene, computed: &ComputedLayout) {
    // Draw points
    for point in &scatter.data {
        scene.add(Primitive::Circle {
            cx:  computed.map_x(point.x),
            cy: computed.map_y(point.y),
            r: scatter.size,
            fill: scatter.color.clone(),
        });
    }
}

fn add_line(line: &LinePlot, scene: &mut Scene, computed: &ComputedLayout) {
    // Add the line path
    if line.data.len() >= 2 {
        let mut path = String::new();
        for (i, &coords) in line.data.iter().enumerate() {
            let sx = computed.map_x(coords.x);
            let sy = computed.map_y(coords.y);
            if i == 0 {
                path += &format!("M {sx} {sy} ");
            } else {
                path += &format!("L {sx} {sy} ");
            }
        }

        scene.add(Primitive::Path {
            d: path,
            stroke: line.color.clone(),
            stroke_width: line.stroke_width,
        });
    }
}



pub enum Plot {
    Scatter(ScatterPlot),
    Line(LinePlot),
    // Bar,
    // Histogram,
    // boxplot,
}




/// render_scatter
pub fn render_scatter(scatter: &ScatterPlot, layout: Layout) -> Scene {
    
    let computed = ComputedLayout::from_layout(&layout);
        
    let mut scene = Scene::new(computed.width, computed.height);
    
    add_axes_and_grid(&mut scene, &computed, &layout);

    add_labels_and_title(&mut scene, &computed, &layout);

    add_scatter(scatter, &mut scene, &computed);

    scene
}


// render_line
pub fn render_line(line: &LinePlot, layout: Layout) -> Scene {

    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);

    // Add grid and axes
    add_axes_and_grid(&mut scene, &computed, &layout);

    add_labels_and_title(&mut scene, &computed, &layout);

    add_line(line, &mut scene, &computed);

    scene
}


pub fn render_multiple(plots: Vec<Plot>, layout: Layout) -> Scene {
    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);

    add_axes_and_grid(&mut scene, &computed, &layout);
    add_labels_and_title(&mut scene, &computed, &layout);

    // for each plot, plot it
    for plot in plots {
        match plot {
            Plot::Scatter(s) => {
                add_scatter(&s, &mut scene, &computed);
            }
            Plot::Line(l) => {
               add_line(&l, &mut scene, &computed);
            }
        }
    }

    scene
}
