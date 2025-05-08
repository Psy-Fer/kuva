use crate::plot::scatter::ScatterPlot;
use crate::plot::line::LinePlot;
use crate::plot::bar::BarPlot;
use crate::plot::histogram::Histogram;
use crate::plot::BoxPlot;


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
        stroke_width: f64,
    },
    Path {
        d: String,
        stroke: String,
        stroke_width: f64,
    },
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        fill: String,
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
    pub x_categories: Option<Vec<String>>,
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

            if let Plot::Bar(bp) = plot {
                let labels = bp.groups.iter().map(|g| g.label.clone()).collect::<Vec<_>>();
                x_labels = Some(labels);
            }
        }

        let mut layout = Self::new((x_min, x_max), (y_min, y_max));
        if let Some(labels) = x_labels {
            layout = layout.with_x_categories(labels);
        }

        layout
    }
    
    // pub fn auto_from_plots(plots: &[Plot]) -> Self {
    //     let mut x_min = f64::INFINITY;
    //     let mut x_max = f64::NEG_INFINITY;
    //     let mut y_min = f64::INFINITY;
    //     let mut y_max = f64::NEG_INFINITY;

    //     for plot in plots {
    //         if let Some(((xmin, xmax), (ymin, ymax))) = plot.bounds() {
    //             x_min = x_min.min(xmin);
    //             x_max = x_max.max(xmax);
    //             y_min = y_min.min(ymin);
    //             y_max = y_max.max(ymax);
    //         }
    //     }

    //     // Add margin
    //     let x_margin = (x_max - x_min) * 0.05;
    //     let y_margin = (y_max - y_min) * 0.05;

    //     Layout::new(
    //         (x_min - x_margin, x_max + x_margin),
    //         (y_min - y_margin, y_max + y_margin),
    //     )
    // }

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
        stroke_width: 1.0,
    });

    // Y axis
    scene.add(Primitive::Line {
        x1: computed.margin_left,
        y1: computed.margin_top,
        x2: computed.margin_left,
        y2: computed.height - computed.margin_bottom,
        stroke: "green".into(),
        stroke_width: 1.0,
    });

    if let Some(categories) = &layout.x_categories {
        // draw x axis category labels and ticks
        for (i, label) in categories.iter().enumerate() {
            let x_val = i as f64 + 1.0; // match x-positioning
            let x_pos = computed.map_x(x_val);
    
            // Draw label
            scene.add(Primitive::Text {
                x: x_pos,
                y: computed.height - computed.margin_bottom + 15.0,
                content: label.clone(),
                size: 10,
                anchor: TextAnchor::Middle,
                rotate: None,
            });
    
            // Optional: draw a small tick
            scene.add(Primitive::Line {
                x1: x_pos,
                y1: computed.height - computed.margin_bottom,
                x2: x_pos,
                y2: computed.height - computed.margin_bottom + 5.0,
                stroke: "black".into(),
                stroke_width: 1.0,
            });
        }
        for i in 0..=computed.ticks {
            // let tx = computed.x_range.0 + (i as f64) * (computed.x_range.1 - computed.x_range.0) / computed.ticks as f64;
            let ty = computed.y_range.0 + (i as f64) * (computed.y_range.1 - computed.y_range.0) / computed.ticks as f64;

            // let x = map_x(tx);
            let y = map_y(ty);
            // Y ticks
            scene.add(Primitive::Line {
                x1: computed.margin_left - 5.0,
                y1: y,
                x2: computed.margin_left,
                y2: y,
                stroke: "black".into(),
                stroke_width: 1.0,
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
        }
    }
    else {
        // Draw value ticks and labels
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
                stroke_width: 1.0,
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
                stroke_width: 1.0,
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
                        stroke_width: 1.0,
                    });
            
                    // Horizontal grid
                    scene.add(Primitive::Line {
                        x1: computed.margin_left,
                        y1: y,
                        x2: computed.width - computed.margin_right,
                        y2: y,
                        stroke: "#ccc".to_string(),
                        stroke_width: 1.0,
                    });
                }
            }
        }
    }
}

// fn add_axes_and_grid_categories(scene: &mut Scene, computed: &ComputedLayout, layout: &Layout) {

//     let map_x = |x| computed.map_x(x);
//     let map_y = |y| computed.map_y(y);

//     // Draw axes
//     // X axis
//     scene.add(Primitive::Line {
//         x1: computed.margin_left,
//         y1: computed.height - computed.margin_bottom,
//         x2: computed.width - computed.margin_right,
//         y2: computed.height - computed.margin_bottom,
//         stroke: "red".into(),
//         stroke_width: 1.0,
//     });
//     // Y axis
//     scene.add(Primitive::Line {
//         x1: computed.margin_left,
//         y1: computed.margin_top,
//         x2: computed.margin_left,
//         y2: computed.height - computed.margin_bottom,
//         stroke: "green".into(),
//         stroke_width: 1.0,
//     });

//     // Draw ticks and labels
//     for i in 0..=computed.ticks {
//         let tx = computed.x_range.0 + (i as f64) * (computed.x_range.1 - computed.x_range.0) / computed.ticks as f64;
//         let ty = computed.y_range.0 + (i as f64) * (computed.y_range.1 - computed.y_range.0) / computed.ticks as f64;

//         let x = map_x(tx);
//         let y = map_y(ty);

//         // X ticks
//         scene.add(Primitive::Line {
//             x1: x,
//             y1: computed.height - computed.margin_bottom,
//             x2: x,
//             y2: computed.height - computed.margin_bottom + 5.0,
//             stroke: "black".into(),
//             stroke_width: 1.0,
//         });
//         // X tick labels
//         // scene.add(Primitive::Text {
//         //     x,
//         //     y: computed.height - computed.margin_bottom + 15.0,
//         //     content: format!("{:.1}", tx),
//         //     size: 10,
//         //     anchor: TextAnchor::Middle,
//         //     rotate: None,
//         // });

//         // Y ticks
//         scene.add(Primitive::Line {
//             x1: computed.margin_left - 5.0,
//             y1: y,
//             x2: computed.margin_left,
//             y2: y,
//             stroke: "black".into(),
//             stroke_width: 1.0,
//         });

//         // Y tick labels
//         scene.add(Primitive::Text {
//             x: computed.margin_left - 15.0,
//             y,
//             content: format!("{:.1}", ty),
//             size: 10,
//             anchor: TextAnchor::Middle,
//             rotate: None,
//         });

//         // Grid lines
//         if layout.show_grid {
//             if i != 0 {
//                 // Vertical grid
//                 scene.add(Primitive::Line {
//                     x1: x,
//                     y1: computed.margin_top,
//                     x2: x,
//                     y2: computed.height - computed.margin_bottom,
//                     stroke: "#ccc".to_string(),
//                     stroke_width: 1.0,
//                 });
        
//                 // Horizontal grid
//                 scene.add(Primitive::Line {
//                     x1: computed.margin_left,
//                     y1: y,
//                     x2: computed.width - computed.margin_right,
//                     y2: y,
//                     stroke: "#ccc".to_string(),
//                     stroke_width: 1.0,
//                 });
//             }
//         }
//     }
// }

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
            cx:  computed.map_x(point.0),
            cy: computed.map_y(point.1),
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
            let sx = computed.map_x(coords.0);
            let sy = computed.map_y(coords.1);
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


fn add_bar(bar: &BarPlot, scene: &mut Scene, computed: &ComputedLayout) {
    // for each tick, make a group, then within the groups do the bars
    for (i, group) in bar.groups.iter().enumerate() {
        let group_x = i as f64 + 1.0; // make bar groups at 1, 2, etc...
        let n = group.bars.len();
        let total_width = bar.width;
        let single_width = total_width / n as f64; // each individual bar as fraction
    
        for (j, bar) in group.bars.iter().enumerate() {
            let x = group_x - total_width / 2.0 + single_width * (j as f64 + 0.5);
            let x0 = computed.map_x(x - single_width / 2.0);
            let x1 = computed.map_x(x + single_width / 2.0);
            let y0 = computed.map_y(0.0); // all bars start at y=0
            let y1 = computed.map_y(bar.value);
    
            scene.add(Primitive::Rect {
                x: x0,
                y: y1.min(y0),
                width: (x1 - x0).abs(),
                height: (y0 - y1).abs(),
                fill: bar.color.clone(),
            });
        }
    }
}

fn add_histogram(hist: &Histogram, scene: &mut Scene, computed: &ComputedLayout) {

    // fold is basically a fancy for loop
    let range: (f64, f64) = hist.range.unwrap_or_else(|| {
        let min: f64 = hist.data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max: f64 = hist.data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        (min, max)
    });

    let bin_width: f64 = (range.1 - range.0) / hist.bins as f64;
    let mut counts: Vec<usize> = vec![0; hist.bins];

    for &value in &hist.data {
        if value < range.0 || value > range.1 {
            continue;
        }
        let bin: usize = ((value - range.0) / bin_width).floor() as usize;
        let bin: usize = if bin == hist.bins { bin - 1 } else { bin };
        counts[bin] += 1;
    }

    let max_count: f64 = *counts.iter().max().unwrap_or(&1) as f64;
    let norm: f64 = if hist.normalize { 1.0 / max_count } else { 1.0 };

    for (i, count) in counts.iter().enumerate() {
        let x = range.0 + i as f64 * bin_width;
        let height = *count as f64 * norm;

        let x0 = computed.map_x(x);
        let x1 = computed.map_x(x + bin_width);
        let y0 = computed.map_y(0.0);
        let y1 = computed.map_y(height);

        let rect_width = (x1 - x0).abs();
        let rect_height = (y0 - y1).abs();

        scene.add(Primitive::Rect {
            x: x0,
            y: y1.min(y0),
            width: rect_width,
            height: rect_height,
            fill: hist.color.clone(),
        });
    }
}

// TODO: move helper
fn percentile(sorted: &[f64], p: f64) -> f64 {
    let rank = p / 100.0 * (sorted.len() - 1) as f64;
    let low = rank.floor() as usize;
    let high = rank.ceil() as usize;
    let weight = rank - low as f64;
    sorted[low] * (1.0 - weight) + sorted[high] * weight
}

fn add_boxplot(boxplot: &BoxPlot, scene: &mut Scene, computed: &ComputedLayout) {
    
    
    for (i, group) in boxplot.groups.iter().enumerate() {
        if group.values.is_empty() { continue; }

        let mut sorted = group.values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1 = percentile(&sorted, 25.0); // Q1
        let q2 = percentile(&sorted, 50.0); // median
        let q3 = percentile(&sorted, 75.0); // Q3
        let iqr = q3 - q1;
        let lower_whisker = sorted.iter().cloned().filter(|v| *v >= q1 - 1.5 * iqr).fold(f64::INFINITY, f64::min);
        let upper_whisker = sorted.iter().cloned().filter(|v| *v <= q3 + 1.5 * iqr).fold(f64::NEG_INFINITY, f64::max);

        let x = i as f64 + 1.0;
        let w = boxplot.width / 2.0;

        let x0 = computed.map_x(x - w);
        let x1 = computed.map_x(x + w);
        let yq1 = computed.map_y(q1);
        let yq3 = computed.map_y(q3);
        let ymed = computed.map_y(q2);
        let ylow = computed.map_y(lower_whisker);
        let yhigh = computed.map_y(upper_whisker);
        let xmid = computed.map_x(x);

        // Box
        scene.add(Primitive::Rect {
            x: x0,
            y: yq3.min(yq1),
            width: (x1 - x0).abs(),
            height: (yq1 - yq3).abs(),
            fill: boxplot.color.clone(),
        });

        // Median line
        scene.add(Primitive::Line {
            x1: x0,
            y1: ymed,
            x2: x1,
            y2: ymed,
            stroke: "white".into(),
            stroke_width: 1.5,
        });

        // Whiskers
        scene.add(Primitive::Line {
            x1: xmid,
            y1: ylow,
            x2: xmid,
            y2: yq1,
            stroke: boxplot.color.clone(),
            stroke_width: 1.0,
        });
        scene.add(Primitive::Line {
            x1: xmid,
            y1: yq3,
            x2: xmid,
            y2: yhigh,
            stroke: boxplot.color.clone(),
            stroke_width: 1.0,
        });

        // Whisker caps
        for &y in &[ylow, yhigh] {
            scene.add(Primitive::Line {
                x1: computed.map_x(x - w / 2.0),
                x2: computed.map_x(x + w / 2.0),
                y1: y,
                y2: y,
                stroke: boxplot.color.clone(),
                stroke_width: 1.0,
            });
        }
    }
}


fn bounds_from_xy(points: &[(f64, f64)]) -> Option<((f64, f64), (f64, f64))> {
    if points.is_empty() {
        return None;
    }
    let (mut x_min, mut x_max) = (points[0].0, points[0].0);
    let (mut y_min, mut y_max) = (points[0].1, points[0].1);
    for &(x, y) in points {
        x_min = x_min.min(x);
        x_max = x_max.max(x);
        y_min = y_min.min(y);
        y_max = y_max.max(y);
    }
    Some(((x_min, x_max), (y_min, y_max)))
}


pub enum Plot {
    Scatter(ScatterPlot),
    Line(LinePlot),
    Bar(BarPlot),
    Histogram(Histogram),
    Box(BoxPlot),
}

impl Plot {
    pub fn bounds(&self) -> Option<((f64, f64), (f64, f64))> {
        match self {
            Plot::Scatter(p) => bounds_from_xy(&p.data),
            Plot::Line(p) => bounds_from_xy(&p.data),
            Plot::Bar(bp) => {

                if bp.groups.is_empty() {
                    None
                } 
                else {
                    let x_min = 0.5;
                    let x_max = bp.groups.len() as f64 + 0.5;
                    let y_min = 0.0;
    
                    let mut y_max = f64::NEG_INFINITY;
                    for group in &bp.groups {
                        for bar in &group.bars {
                            y_max = y_max.max(bar.value);
                        }
                    }

                    Some(((x_min, x_max), (y_min, y_max)))
                }
            }
            Plot::Histogram(h) => {
                let range = h.range?;
                let bins = h.bins;
                let bin_width = (range.1 - range.0) / bins as f64;

                let mut counts = vec![0usize; bins];
                for &value in &h.data {
                    if value < range.0 || value > range.1 {
                        continue;
                    }
                    let bin = ((value - range.0) / bin_width).floor() as usize;
                    let bin = if bin == bins { bin - 1 } else { bin };
                    counts[bin] += 1;
                }

                let max_y = *counts.iter().max().unwrap_or(&1) as f64;

                Some((range, (0.0, max_y)))
            }
            Plot::Box(bp) => {
                if bp.groups.is_empty() {
                    None
                }
                else {
                    let x_min = 0.5;
                    let x_max = bp.groups.len() as f64 + 0.5;
            
                    let mut y_min = f64::INFINITY;
                    let mut y_max = f64::NEG_INFINITY;
                    for g in &bp.groups {
                        if g.values.is_empty() { continue; }
                        let mut vals = g.values.clone();
                        vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
                        let q1 = percentile(&vals, 25.0);
                        let q3 = percentile(&vals, 75.0);
                        let iqr = q3 - q1;
                        let lo = q1 - 1.5 * iqr;
                        let hi = q3 + 1.5 * iqr;
                        y_min = y_min.min(lo);
                        y_max = y_max.max(hi);
                    }
            
                    Some(((x_min, x_max), (y_min, y_max)))
                }
            }
        }
    }
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

// render_bar
pub fn render_bar(bar: &BarPlot, layout: Layout) -> Scene {

    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);

    // Add grid and axes
    add_axes_and_grid(&mut scene, &computed, &layout);

    add_labels_and_title(&mut scene, &computed, &layout);

    add_bar(bar, &mut scene, &computed);

    scene
}

// render_bar_categories
pub fn render_bar_categories(bar: &BarPlot, layout: Layout) -> Scene {

    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);

    // Add grid and axes
    // add_axes_and_grid(&mut scene, &computed, &layout);
    add_axes_and_grid(&mut scene, &computed, &layout);

    add_labels_and_title(&mut scene, &computed, &layout);

    add_bar(bar, &mut scene, &computed);

    scene
}

// render_histogram
pub fn render_histogram(hist: &Histogram, layout: &Layout) -> Scene {

    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);

    // Add grid and axes
    // add_axes_and_grid(&mut scene, &computed, &layout);
    add_axes_and_grid(&mut scene, &computed, &layout);

    add_labels_and_title(&mut scene, &computed, &layout);

    add_histogram(hist, &mut scene, &computed);

    scene
}

// render_histogram
pub fn render_boxplot(boxplot: &BoxPlot, layout: &Layout) -> Scene {

    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);

    // Add grid and axes
    // add_axes_and_grid(&mut scene, &computed, &layout);
    add_axes_and_grid(&mut scene, &computed, &layout);

    add_labels_and_title(&mut scene, &computed, &layout);

    add_boxplot(boxplot, &mut scene, &computed);

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
            Plot::Bar(b) => {
               add_bar(&b, &mut scene, &computed);
            }
            Plot::Histogram(h) => {
                add_histogram(&h, &mut scene, &computed);
            }
            Plot::Box(b) => {
                add_boxplot(&b, &mut scene, &computed);
            }
        }
    }

    scene
}
