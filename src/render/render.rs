use crate::render::render_utils::{percentile, simple_kde};
use crate::render::layout::{Layout, ComputedLayout};
use crate::render::plots::Plot;
use crate::render::axis::{add_axes_and_grid, add_labels_and_title};

use crate::plot::scatter::ScatterPlot;
use crate::plot::line::LinePlot;
use crate::plot::bar::BarPlot;
use crate::plot::histogram::Histogram;
use crate::plot::{BoxPlot, PiePlot, SeriesPlot, SeriesStyle, ViolinPlot};

use crate::plot::Legend;
use crate::plot::legend::{LegendEntry, LegendShape};


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
        fill: Option<String>,
        stroke: String,
        stroke_width: f64,
    },
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        fill: String,
        stroke: Option<String>,
        stroke_width: Option<f64>,
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
    //TODO: export this function
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
            fill: None,
            stroke: line.color.clone(),
            stroke_width: line.stroke_width,
        });
    }
}

fn add_series(series: &SeriesPlot, scene: &mut Scene, computed: &ComputedLayout) {

    let points: Vec<(f64, f64)> = series.values.iter().enumerate()
        .map(|(i, &y)| (computed.map_x(i as f64), computed.map_y(y)))
        .collect();

    // TODO: abstract the path/circle builders
    match series.style {
        SeriesStyle::Line => {
            if points.len() >= 2 {
                let mut pathstr = String::new();
                for (i, &coords) in points.iter().enumerate() {
                    let sx = coords.0;
                    let sy = coords.1;
                    if i == 0 {
                        pathstr += &format!("M {sx} {sy} ");
                    } else {
                        pathstr += &format!("L {sx} {sy} ");
                    }
                }
                scene.add(Primitive::Path {
                        d: pathstr,
                        fill: None,
                        stroke: series.color.clone(),
                        stroke_width: 2.0, // TODO: add interface
                });
            }
        }
        SeriesStyle::Point => {
            for (x, y) in points {
                scene.add(Primitive::Circle {
                    cx:  x,
                    cy: y,
                    r: 3.0, // TODO: add interface
                    fill: series.color.clone()
                });
            }
        }
        SeriesStyle::Both => {
            if points.len() >= 2 {
                let mut pathstr = String::new();
                for (i, &coords) in points.iter().enumerate() {
                    let sx = coords.0;
                    let sy = coords.1;
                    if i == 0 {
                        pathstr += &format!("M {sx} {sy} ");
                    } else {
                        pathstr += &format!("L {sx} {sy} ");
                    }
                }
                scene.add(Primitive::Path {
                        d: pathstr,
                        fill: None,
                        stroke: series.color.clone(),
                        stroke_width: 2.0, // TODO: add interface
                });
            }
            for (x, y) in points {
                scene.add(Primitive::Circle {
                    cx:  x,
                    cy: y,
                    r: 3.0, // TODO: add interface
                    fill: series.color.clone()
                });
            }
        }
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
                stroke: None,
                stroke_width: None,
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
            stroke: None,
            stroke_width: None,
        });
    }
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
            stroke: None,
            stroke_width: None,
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

fn add_violin(violin: &ViolinPlot, scene: &mut Scene, computed: &ComputedLayout) {

    for (i, group) in violin.groups.iter().enumerate() {
        let x_center = computed.map_x((i + 1) as f64);

        // Compute KDE
        let kde = simple_kde(&group.values, 0.3, 50); // 0.3 bandwidth, 50 steps

        // Normalize
        let max_density = kde.iter().map(|(_, y)| *y).fold(f64::NEG_INFINITY, f64::max);
        let scale = violin.width / max_density;

        // Map KDE to plot coordinates
        let mut path_data = String::new();

        //from top, left to right
        for (j, (y, d)) in kde.iter().enumerate() {
            let dy = computed.map_y(*y);
            let dx = x_center - d * scale;
            if j == 0 {
                path_data += &format!("M {dx} {dy} ");
            } else {
                path_data += &format!("L {dx} {dy} ");
            }
        }

        // from bottom, right to left
        for (y, d) in kde.iter().rev() {
            let dy = computed.map_y(*y);
            let dx = x_center + d * scale;
            path_data += &format!("L {dx} {dy} ");
        }

        path_data += "Z";

        scene.add(Primitive::Path {
            d: path_data,
            fill: Some(violin.color.clone()),
            stroke: "black".into(),
            stroke_width: 0.5,
        });
    }
}

fn add_pie(pie: &PiePlot, scene: &mut Scene, computed: &ComputedLayout) {

    let cx = computed.width / 2.0;
    let cy = computed.height / 2.0;
    let radius = computed.plot_width().min(computed.plot_height()) / 2.0 - 10.0;
    let inner_radius = pie.inner_radius;
    let label_radius = (radius + inner_radius) / 2.0;
    // let label_radius = radius * 1.15;

    let total: f64 = pie.slices.iter().map(|s| s.value).sum();
    let mut angle = 0.0;

    // slice maths in radians - IN REAL LIFE!!! - my school teachers would be proud
    for slice in &pie.slices {
        let frac = slice.value / total;
        let sweep = frac * std::f64::consts::TAU;
        let end_angle = angle + sweep;

        let x1 = cx + radius * angle.cos();
        let y1 = cy + radius * angle.sin();
        let x2 = cx + radius * end_angle.cos();
        let y2 = cy + radius * end_angle.sin();

        let large_arc = if sweep > std::f64::consts::PI { 1 } else { 0 };

        let path_data = if inner_radius == 0.0 {
            format!(
                "M{cx},{cy} L{x1},{y1} A{r},{r} 0 {large_arc},1 {x2},{y2} Z",
                r = radius
            )
        } else {
            let ix1 = cx + inner_radius * end_angle.cos();
            let iy1 = cy + inner_radius * end_angle.sin();
            let ix2 = cx + inner_radius * angle.cos();
            let iy2 = cy + inner_radius * angle.sin();
            format!(
                "M{x1},{y1} A{r},{r} 0 {large_arc},1 {x2},{y2} L{ix1},{iy1} A{ir},{ir} 0 {large_arc},0 {ix2},{iy2} Z",
                r = radius,
                ir = inner_radius
            )
        };

        scene.add(Primitive::Path {
            d: path_data,
            fill: Some(slice.color.clone()),
            stroke: slice.color.clone(),
            stroke_width: 1.0 }
        );

        // SLICE LABEL
        let mid_angle = angle + sweep / 2.0;
        let label_x = cx + label_radius * mid_angle.cos();
        let label_y = cy + label_radius * mid_angle.sin();

        scene.add(Primitive::Text {
            x: label_x,
            y: label_y,
            content: slice.label.clone(),
            size: 12,
            anchor: TextAnchor::Middle,
            rotate: None,
        });


        angle = end_angle;
    }
}


fn add_legend(legend: &Legend, scene: &mut Scene, computed: &ComputedLayout) {

    // TODO: make this the length of the text
    let legend_width = 110.0;
    let legend_padding = 10.0;
    let legend_x = computed.width - computed.margin_right + 10.0;
    let mut legend_y = computed.margin_top;
    let line_height = 18.0;

    let legend_height = legend.entries.len() as f64 * line_height + legend_padding * 2.0;

    scene.add(Primitive::Rect {
        x: legend_x - legend_padding + 5.0,
        y: legend_y - legend_padding,
        width: legend_width,
        height: legend_height,
        fill: "white".into(),
        stroke: None,
        stroke_width: None,
    });
    
    scene.add(Primitive::Rect {
        x: legend_x - legend_padding +5.0,
        y: legend_y - legend_padding,
        width: legend_width,
        height: legend_height,
        fill: "none".into(),
        stroke: Some("black".into()),
        stroke_width: Some(1.0),
    });

    for entry in &legend.entries {
        // add label
        scene.add(Primitive::Text {
            x: legend_x + 25.0,
            y: legend_y + 5.0,
            content: entry.label.clone(),
            anchor: TextAnchor::Start,
            size: 12,
            rotate: None,
        });
        // add shape with colour
        match entry.shape {
            LegendShape::Rect => scene.add(Primitive::Rect {
                x: legend_x+5.0,
                y: legend_y,
                width: 12.0,
                height: 12.0,
                fill: entry.color.clone(),
                stroke: None,
                stroke_width: None,
            }),
            LegendShape::Line => scene.add(Primitive::Line {
                x1: legend_x + 5.0,
                y1: legend_y + 2.0,
                x2: legend_x + 5.0 + 12.0,
                y2: legend_y + 2.0,
                stroke: entry.color.clone(),
                stroke_width: 2.0,
            }),
            LegendShape::Circle => scene.add(Primitive::Circle {
                cx: legend_x +5.0 + 6.0,
                cy: legend_y + 1.0,
                r: 5.0,
                fill: entry.color.clone(),
            }),
        }

        legend_y += line_height;
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
    add_axes_and_grid(&mut scene, &computed, &layout);

    add_labels_and_title(&mut scene, &computed, &layout);

    add_histogram(hist, &mut scene, &computed);

    scene
}

// render_boxplot
pub fn render_boxplot(boxplot: &BoxPlot, layout: &Layout) -> Scene {

    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);

    // Add grid and axes
    add_axes_and_grid(&mut scene, &computed, &layout);

    add_labels_and_title(&mut scene, &computed, &layout);

    add_boxplot(boxplot, &mut scene, &computed);

    scene
}

// render_violinplot
pub fn render_violin(violin: &ViolinPlot, layout: &Layout) -> Scene {

    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);

    // Add grid and axes
    add_axes_and_grid(&mut scene, &computed, &layout);

    add_labels_and_title(&mut scene, &computed, &layout);

    add_violin(violin, &mut scene, &computed);

    scene
}

pub fn render_pie(pie: &PiePlot, layout: &Layout) -> Scene {

    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);

    // Add grid and axes
    // add_axes_and_grid(&mut scene, &computed, &layout);

    add_labels_and_title(&mut scene, &computed, &layout);

    add_pie(pie, &mut scene, &computed);

    scene
}



/// this should be the default renderer.
/// TODO: make an alias of this for single plots, that vectorises
pub fn render_multiple(plots: Vec<Plot>, layout: Layout) -> Scene {
    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);

    add_axes_and_grid(&mut scene, &computed, &layout);
    add_labels_and_title(&mut scene, &computed, &layout);

    // for each plot, plot it
    for plot in plots.iter() {
        match plot {
            Plot::Scatter(s) => {
                add_scatter(&s, &mut scene, &computed);
            }
            Plot::Line(l) => {
                add_line(&l, &mut scene, &computed);
            }
            Plot::Series(s) => {
                add_series(&s, &mut scene, &computed);
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
            Plot::Violin(v) => {
                add_violin(&v, &mut scene, &computed);
            }
            Plot::Pie(p) => {
                add_pie(&p, &mut scene, &computed);
            }
        }
    }
    
    // create legend
    // only bar, line, and scatter have legends for now
    let mut legend = Legend::default();
    for plot in plots.iter() {
        match plot {
            Plot::Bar(barplot) => {
                if let Some(label) = barplot.legend_label.clone() {
                    for (i, barval) in barplot.groups.first().unwrap().bars.iter().enumerate() {
                        legend.entries.push(LegendEntry {
                            label: format!("{}", label.get(i).unwrap()),
                            color: barval.color.clone(),
                            shape: LegendShape::Rect,
                        });
                        eprintln!("label:{}", label.get(i).unwrap());
                    }
                }
            }
            Plot::Line(line) => {
                if let Some(label) = &line.legend_label {
                    legend.entries.push(LegendEntry {
                        label: label.clone(),
                        color: line.color.clone(),
                        shape: LegendShape::Line,
                    });
                }
            }
            Plot::Scatter(scatter) => {
                if let Some(label) = &scatter.legend_label {
                    legend.entries.push(LegendEntry {
                        label: label.clone(),
                        color: scatter.color.clone(),
                        shape: LegendShape::Circle,
                    });
                }
            }
            Plot::Series(series) => {
                if let Some(label) = &series.legend_label {
                    legend.entries.push(LegendEntry {
                        label: label.clone(),
                        color: series.color.clone(),
                        shape: LegendShape::Circle,
                    });
                }
            }
            _ => {}
        }
    }

    if layout.show_legend {
        add_legend(&legend, &mut scene, &computed);
    }

    scene
}
