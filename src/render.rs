use crate::plot::scatter::ScatterPlot;


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

    pub fn map_x(&self, x: f64) -> f64 {
        self.margin_left + (x - self.x_range.0) / (self.x_range.1 - self.x_range.0) * self.width
    }

    pub fn map_y(&self, y: f64) -> f64 {
        self.height - self.margin_bottom - (y - self.y_range.0) / (self.y_range.1 - self.y_range.0) * self.height
    }
}






/// render_scatter
pub fn render_scatter(scatter: &ScatterPlot, input_layout: Layout) -> Scene {
    
    let layout = ComputedLayout::from_layout(&input_layout);
    
    let map_x = |x| layout.map_x(x);
    let map_y = |y| layout.map_y(y);
    
    let mut scene = Scene::new(layout.width, layout.height);
    // Draw axes
    // X axis
    scene.add(Primitive::Line {
        x1: layout.margin_left,
        y1: layout.height - layout.margin_bottom,
        x2: layout.width - layout.margin_right,
        y2: layout.height - layout.margin_bottom,
        stroke: "red".into(),
    });
    // Y axis
    scene.add(Primitive::Line {
        x1: layout.margin_left,
        y1: layout.margin_top,
        x2: layout.margin_left,
        y2: layout.height - layout.margin_bottom,
        stroke: "green".into(),
    });

    // Draw ticks and labels
    for i in 0..=layout.ticks {
        let tx = layout.x_range.0 + (i as f64) * (layout.x_range.1 - layout.x_range.0) / layout.ticks as f64;
        let ty = layout.y_range.0 + (i as f64) * (layout.y_range.1 - layout.y_range.0) / layout.ticks as f64;

        let x = map_x(tx);
        let y = map_y(ty);

        // X ticks
        scene.add(Primitive::Line {
            x1: x,
            y1: layout.height - layout.margin_bottom,
            x2: x,
            y2: layout.height - layout.margin_bottom + 5.0,
            stroke: "black".into(),
        });
        // X tick labels
        scene.add(Primitive::Text {
            x,
            y: layout.height - layout.margin_bottom + 15.0,
            content: format!("{:.1}", tx),
            size: 10,
            anchor: TextAnchor::Middle,
            rotate: None,
        });

        // Y ticks
        scene.add(Primitive::Line {
            x1: layout.margin_left - 5.0,
            y1: y,
            x2: layout.margin_left,
            y2: y,
            stroke: "black".into(),
        });
        // Y tick labels
        scene.add(Primitive::Text {
            x: layout.margin_left - 15.0,
            y,
            content: format!("{:.1}", ty),
            size: 10,
            anchor: TextAnchor::Middle,
            rotate: None,
        });

        // Grid lines
        if input_layout.show_grid {
            if i != 0 {
                // Vertical grid
                scene.add(Primitive::Line {
                    x1: x,
                    y1: layout.margin_top,
                    x2: x,
                    y2: layout.height - layout.margin_bottom,
                    stroke: "#ccc".to_string(),
                });
        
                // Horizontal grid
                scene.add(Primitive::Line {
                    x1: layout.margin_left,
                    y1: y,
                    x2: layout.width - layout.margin_right,
                    y2: y,
                    stroke: "#ccc".to_string(),
                });
            }
        }
    }

    // X Axis Label
    if let Some(label) = &input_layout.x_label {
        scene.add(Primitive::Text {
            x: layout.width / 2.0,
            y: layout.height - layout.margin_bottom / 4.0,
            content: label.clone(),
            size: 14,
            anchor: TextAnchor::Middle,
            rotate: None,
        });
    }

    // Y Axis Label
    if let Some(label) = &input_layout.y_label {
        scene.add(Primitive::Text {
            x: 20.0,
            y: layout.height / 2.0,
            content: label.clone(),
            size: 14,
            anchor: TextAnchor::Middle,
            rotate: Some(-90.0),
        });
    }

    // Title
    if let Some(title) = &input_layout.title {
        scene.add(Primitive::Text {
            x: layout.width / 2.0,
            y: layout.margin_top / 2.0,
            content: title.clone(),
            size: 16,
            anchor: TextAnchor::Middle,
            rotate: None,
        });
    }


    // Draw points
    for point in &scatter.points {
        scene.add(Primitive::Circle {
            cx: map_x(point.x),
            cy: map_y(point.y),
            r: 3.0,
            fill: "blue".into(),
        });
    }

    scene
}