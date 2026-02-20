use crate::render::render_utils::{self, percentile, simple_kde, linear_regression, pearson_corr};
use crate::render::layout::{Layout, ComputedLayout};
use crate::render::plots::Plot;
use crate::render::axis::{add_axes_and_grid, add_labels_and_title};
use crate::render::annotations::{add_shaded_regions, add_reference_lines, add_text_annotations};

use crate::plot::scatter::{ScatterPlot, TrendLine};
use crate::plot::line::LinePlot;
use crate::plot::bar::BarPlot;
use crate::plot::histogram::Histogram;
use crate::plot::band::BandPlot;
use crate::plot::{BoxPlot, BrickPlot, Heatmap, Histogram2D, PiePlot, SeriesPlot, SeriesStyle, ViolinPlot};
use crate::plot::pie::PieLabelPosition;


use crate::plot::Legend;
use crate::plot::legend::{ColorBarInfo, LegendEntry, LegendPosition, LegendShape};

// TODO: make setters/builders for these primitives
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
        stroke_dasharray: Option<String>,
    },
    Path {
        d: String,
        fill: Option<String>,
        stroke: String,
        stroke_width: f64,
        opacity: Option<f64>,
    },
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        fill: String,
        stroke: Option<String>,
        stroke_width: Option<f64>,
        opacity: Option<f64>,
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

/// Build an SVG path string from a sequence of (x, y) screen-coordinate points.
pub fn build_path(points: &[(f64, f64)]) -> String {
    let mut path = String::new();
    for (i, &(x, y)) in points.iter().enumerate() {
        if i == 0 {
            path += &format!("M {x} {y} ");
        } else {
            path += &format!("L {x} {y} ");
        }
    }
    path
}

fn add_band(band: &BandPlot, scene: &mut Scene, computed: &ComputedLayout) {
    if band.x.len() < 2 { return; }
    let mut path = String::new();
    // Forward along upper curve
    for (i, (&x, &y)) in band.x.iter().zip(band.y_upper.iter()).enumerate() {
        let sx = computed.map_x(x);
        let sy = computed.map_y(y);
        if i == 0 {
            path += &format!("M {sx} {sy} ");
        } else {
            path += &format!("L {sx} {sy} ");
        }
    }
    // Backward along lower curve
    for (&x, &y) in band.x.iter().zip(band.y_lower.iter()).rev() {
        let sx = computed.map_x(x);
        let sy = computed.map_y(y);
        path += &format!("L {sx} {sy} ");
    }
    path += "Z";
    scene.add(Primitive::Path {
        d: path,
        fill: Some(band.color.clone()),
        stroke: "none".into(),
        stroke_width: 0.0,
        opacity: Some(band.opacity * 100.0),
    });
}

fn add_scatter(scatter: &ScatterPlot, scene: &mut Scene, computed: &ComputedLayout) {
    // Draw band behind points if present
    if let Some(ref band) = scatter.band {
        add_band(band, scene, computed);
    }

    // Draw points
    for point in &scatter.data {
        scene.add(Primitive::Circle {
            cx:  computed.map_x(point.x),
            cy: computed.map_y(point.y),
            r: scatter.size,
            fill: scatter.color.clone(),
        });

        // x error
        if let Some((neg, pos)) = point.x_err {
            let cy = computed.map_y(point.y);
            let cx_low = computed.map_x(point.x - neg);
            let cx_high = computed.map_x(point.x + pos);
        
            scene.add(Primitive::Line {
                x1: cx_low,
                y1: cy,
                x2: cx_high,
                y2: cy,
                stroke: scatter.color.clone(),
                stroke_width: 1.0,
                stroke_dasharray: None,
            });

            // Add caps
            scene.add(Primitive::Line {
                x1: cx_low,
                y1: cy - 5.0,
                x2: cx_low,
                y2: cy + 5.0,
                stroke: scatter.color.clone(),
                stroke_width: 1.0,
                stroke_dasharray: None,
            });

            scene.add(Primitive::Line {
                x1: cx_high,
                y1: cy - 5.0,
                x2: cx_high,
                y2: cy + 5.0,
                stroke: scatter.color.clone(),
                stroke_width: 1.0,
                stroke_dasharray: None,
            });
        }

        // y error
        if let Some((neg, pos)) = point.y_err {
            let cx = computed.map_x(point.x);
            let cy_low = computed.map_y(point.y - neg);
            let cy_high = computed.map_y(point.y + pos);
        
            scene.add(Primitive::Line {
                x1: cx,
                y1: cy_low,
                x2: cx,
                y2: cy_high,
                stroke: scatter.color.clone(),
                stroke_width: 1.0,
                stroke_dasharray: None,
            });

            // Add caps
            scene.add(Primitive::Line {
                x1: cx - 5.0,
                y1: cy_low,
                x2: cx + 5.0,
                y2: cy_low,
                stroke: scatter.color.clone(),
                stroke_width: 1.0,
                stroke_dasharray: None,
            });

            scene.add(Primitive::Line {
                x1: cx - 5.0,
                y1: cy_high,
                x2: cx + 5.0,
                y2: cy_high,
                stroke: scatter.color.clone(),
                stroke_width: 1.0,
                stroke_dasharray: None,
            });
        }
    }
    
    // if trend, draw the line
    if let Some(trend) = scatter.trend {
        match trend {
            TrendLine::Linear => {
                
                if let Some((slope, intercept, r)) = linear_regression(&scatter.data) {
                    // get line start and end co-ords
                    let x1 = computed.x_range.0;
                    let x2 = computed.x_range.1;
                    let y1 = slope * x1 + intercept;
                    let y2 = slope * x2 + intercept;
                    
                    // draw the line
                    scene.add(Primitive::Line {
                        x1: computed.map_x(x1),
                        y1: computed.map_y(y1),
                        x2: computed.map_x(x2),
                        y2: computed.map_y(y2),
                        stroke: scatter.trend_color.clone(),
                        stroke_width: scatter.trend_width,
                        stroke_dasharray: None,
                    });
    
                    // display equation and correlation
                    if scatter.show_equation || scatter.show_correlation {
                        let mut label = String::new();
                        if scatter.show_equation {
                            label.push_str(&format!("y = {:.2}x + {:.2}", slope, intercept));
                        }
                        if scatter.show_correlation {
                            if !label.is_empty() {
                                label.push_str("  ");
                            }
                            label.push_str(&format!("r = {:.2}", r));
                        }
    
                        scene.add(Primitive::Text {
                            x: computed.margin_left + 10.0,
                            y: computed.margin_top + 20.0,
                            content: label,
                            size: 12,
                            anchor: TextAnchor::Start,
                            rotate: None,
                        });
                    }
                }
            }
            // _ => {}
        }
    }
}

fn add_line(line: &LinePlot, scene: &mut Scene, computed: &ComputedLayout) {
    // Draw band behind line if present
    if let Some(ref band) = line.band {
        add_band(band, scene, computed);
    }

    if line.data.len() >= 2 {
        let points: Vec<(f64, f64)> = line.data.iter()
            .map(|c| (computed.map_x(c.x), computed.map_y(c.y)))
            .collect();
        scene.add(Primitive::Path {
            d: build_path(&points),
            fill: None,
            stroke: line.color.clone(),
            stroke_width: line.stroke_width,
            opacity: None,
        });
    }
}

fn add_series(series: &SeriesPlot, scene: &mut Scene, computed: &ComputedLayout) {

    let points: Vec<(f64, f64)> = series.values.iter().enumerate()
        .map(|(i, &y)| (computed.map_x(i as f64), computed.map_y(y)))
        .collect();

    match series.style {
        SeriesStyle::Line => {
            if points.len() >= 2 {
                scene.add(Primitive::Path {
                        d: build_path(&points),
                        fill: None,
                        stroke: series.color.clone(),
                        stroke_width: series.stroke_width,
                        opacity: None,
                });
            }
        }
        SeriesStyle::Point => {
            for (x, y) in points {
                scene.add(Primitive::Circle {
                    cx:  x,
                    cy: y,
                    r: series.point_radius,
                    fill: series.color.clone()
                });
            }
        }
        SeriesStyle::Both => {
            if points.len() >= 2 {
                scene.add(Primitive::Path {
                        d: build_path(&points),
                        fill: None,
                        stroke: series.color.clone(),
                        stroke_width: series.stroke_width,
                        opacity: None,
                });
            }
            for (x, y) in points {
                scene.add(Primitive::Circle {
                    cx:  x,
                    cy: y,
                    r: series.point_radius,
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
                opacity: None,
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
            opacity: None,
        });
    }
}

fn add_histogram2d(hist2d: &Histogram2D, scene: &mut Scene, computed: &ComputedLayout) {
    let max_count = hist2d.bins.iter().flatten().copied().max().unwrap_or(1) as f64;

    let x_bin_width = (hist2d.x_range.1 - hist2d.x_range.0) / hist2d.bins_x as f64;
    let y_bin_height = (hist2d.y_range.1 - hist2d.y_range.0) / hist2d.bins_x as f64;

    // let cmap = hist2d.color_map.clone();
    // for (i, row) in hist2d.data.iter().enumerate() {
    //     for (j, &value) in row.iter().enumerate() {
    //         let color = cmap.map(norm(value));

    //         // let x = computed.map_x(j as f64);
    //         let x0 = computed.map_x(j as f64);
    //         let x1 = computed.map_x(j as f64 + 1.0);
    //         let y0 = computed.map_y(i as f64 + 1.0);
    //         let y1 = computed.map_y(i as f64);
    //         scene.add(Primitive::Rect {
    //             x: x0,
    //             y: y0,
    //             width: (x1-x0).abs()*0.99,
    //             height: (y1-y0).abs()*0.99,
    //             fill: color,
    //             stroke: None,
    //             stroke_width: None,
    //         });

    let cmap = hist2d.color_map.clone();
    for (row_idx, row) in hist2d.bins.iter().enumerate() {
        for (col_idx, &count) in row.iter().enumerate() {
            if count == 0 { continue; }

            let x0 = hist2d.x_range.0 + col_idx as f64 * x_bin_width;
            let y0 = hist2d.y_range.0 + row_idx as f64 * y_bin_height;
            let x1 = x0 + x_bin_width;
            let y1 = y0 + y_bin_height;
            let norm = (count as f64 / max_count).clamp(0.0, 1.0);
            let color = cmap.map(norm);

            scene.add(Primitive::Rect {
                x: computed.map_x(x0),
                y: computed.map_y(y1), // y1 is the bottom, SVG coords go down
                width: computed.map_x(x1) - computed.map_x(x0),
                height: computed.map_y(y0) - computed.map_y(y1),
                fill: color,
                stroke: None,
                stroke_width: None,
                opacity: None,
            });
        }
    }

    if hist2d.show_correlation {
        let corr = pearson_corr(&hist2d.data).unwrap();
        scene.add(Primitive::Text {
            x: computed.width - 120.0,
            y: computed.margin_top + 20.0,
            content: format!("r = {:.2}", corr),
            size: 14,
            anchor: TextAnchor::End,
            rotate: None,
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
            opacity: None,
        });

        // Median line
        scene.add(Primitive::Line {
            x1: x0,
            y1: ymed,
            x2: x1,
            y2: ymed,
            stroke: "white".into(),
            stroke_width: 1.5,
            stroke_dasharray: None,
        });

        // Whiskers
        scene.add(Primitive::Line {
            x1: xmid,
            y1: ylow,
            x2: xmid,
            y2: yq1,
            stroke: boxplot.color.clone(),
            stroke_width: 1.0,
            stroke_dasharray: None,
        });
        scene.add(Primitive::Line {
            x1: xmid,
            y1: yq3,
            x2: xmid,
            y2: yhigh,
            stroke: boxplot.color.clone(),
            stroke_width: 1.0,
            stroke_dasharray: None,
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
                stroke_dasharray: None,
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
            opacity: None,
        });
    }
}

fn add_pie(pie: &PiePlot, scene: &mut Scene, computed: &ComputedLayout) {

    let cx = computed.width / 2.0;
    let cy = computed.height / 2.0;
    let radius = computed.plot_width().min(computed.plot_height()) / 2.0 - 10.0;
    let inner_radius = pie.inner_radius;
    let inside_label_radius = (radius + inner_radius) / 2.0;

    let total: f64 = pie.slices.iter().map(|s| s.value).sum();
    let mut angle = 0.0;

    // Collect outside labels for anti-overlap pass
    struct OutsideLabel {
        x: f64,
        y: f64,
        content: String,
        anchor: TextAnchor,
        leader_x1: f64,
        leader_y1: f64,
        leader_x2: f64,
        leader_y2: f64,
    }
    let mut outside_labels: Vec<OutsideLabel> = Vec::new();

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
            stroke_width: 1.0,
            opacity: None,
        });

        // Build label text
        let label_text = if pie.show_percent {
            let pct = frac * 100.0;
            if slice.label.is_empty() {
                format!("{:.1}%", pct)
            } else {
                format!("{} ({:.1}%)", slice.label, pct)
            }
        } else {
            slice.label.clone()
        };

        // Determine placement
        let place_inside = match pie.label_position {
            PieLabelPosition::None => { angle = end_angle; continue; }
            PieLabelPosition::Inside => true,
            PieLabelPosition::Outside => false,
            PieLabelPosition::Auto => frac >= pie.min_label_fraction,
        };

        let mid_angle = angle + sweep / 2.0;

        if place_inside {
            let label_x = cx + inside_label_radius * mid_angle.cos();
            let label_y = cy + inside_label_radius * mid_angle.sin();
            scene.add(Primitive::Text {
                x: label_x,
                y: label_y,
                content: label_text,
                size: 12,
                anchor: TextAnchor::Middle,
                rotate: None,
            });
        } else {
            // Outside label with leader line
            let leader_x1 = cx + (radius + 5.0) * mid_angle.cos();
            let leader_y1 = cy + (radius + 5.0) * mid_angle.sin();
            let leader_x2 = cx + (radius + 20.0) * mid_angle.cos();
            let leader_y2 = cy + (radius + 20.0) * mid_angle.sin();
            let label_x = cx + (radius + 25.0) * mid_angle.cos();
            let label_y = cy + (radius + 25.0) * mid_angle.sin();
            let anchor = if mid_angle.cos() >= 0.0 { TextAnchor::Start } else { TextAnchor::End };

            outside_labels.push(OutsideLabel {
                x: label_x,
                y: label_y,
                content: label_text,
                anchor,
                leader_x1,
                leader_y1,
                leader_x2,
                leader_y2,
            });
        }

        angle = end_angle;
    }

    // Anti-overlap pass for outside labels: sort by y, nudge adjacent labels apart
    outside_labels.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
    let min_gap = 14.0;
    for i in 1..outside_labels.len() {
        let prev_y = outside_labels[i - 1].y;
        let curr_y = outside_labels[i].y;
        if curr_y - prev_y < min_gap {
            outside_labels[i].y = prev_y + min_gap;
        }
    }

    // Render outside labels and leader lines
    for label in &outside_labels {
        scene.add(Primitive::Line {
            x1: label.leader_x1,
            y1: label.leader_y1,
            x2: label.leader_x2,
            y2: label.leader_y2,
            stroke: "#666".into(),
            stroke_width: 1.0,
            stroke_dasharray: None,
        });
        scene.add(Primitive::Text {
            x: label.x,
            y: label.y,
            content: label.content.clone(),
            size: 12,
            anchor: match label.anchor { TextAnchor::Start => TextAnchor::Start, TextAnchor::End => TextAnchor::End, TextAnchor::Middle => TextAnchor::Middle },
            rotate: None,
        });
    }
}

fn add_heatmap(heatmap: &Heatmap, scene: &mut Scene, computed: &ComputedLayout) {

    let rows = heatmap.data.len();
    let cols = heatmap.data.first().map_or(0, |row| row.len());
    if rows == 0 || cols == 0 {
        return;
    }

    let flat: Vec<f64> = heatmap.data.iter().flatten().cloned().collect();
    let min = flat.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = flat.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let norm = |v: f64| (v - min) / (max - min + f64::EPSILON);

    let cmap = heatmap.color_map.clone();
    for (i, row) in heatmap.data.iter().enumerate() {
        for (j, &value) in row.iter().enumerate() {
            let color = cmap.map(norm(value));

            // let x = computed.map_x(j as f64);
            let x0 = computed.map_x(j as f64);
            let x1 = computed.map_x(j as f64 + 1.0);
            let y0 = computed.map_y(i as f64 + 1.0);
            let y1 = computed.map_y(i as f64);
            scene.add(Primitive::Rect {
                x: x0,
                y: y0,
                width: (x1-x0).abs()*0.99,
                height: (y1-y0).abs()*0.99,
                fill: color,
                stroke: None,
                stroke_width: None,
                opacity: None,
            });

        }
    }
    for (i, row) in heatmap.data.iter().enumerate() {
        for (j, &value) in row.iter().enumerate() {
            if heatmap.show_values {
                let x0 = computed.map_x(j as f64);
                let x1 = computed.map_x(j as f64 + 1.0);
                let y0 = computed.map_y(i as f64 + 1.0);
                let y1 = computed.map_y(i as f64);
                scene.add(Primitive::Text {
                    x: x0 + ((x1-x0).abs() / 2.0),
                    y: y0 + ((y1-y0).abs() / 2.0),
                    content: format!("{:.2}", value),
                    size: 12,
                    anchor: TextAnchor::Middle,
                    rotate: None,
                });
            }
        }
    }
}

fn add_brickplot(brickplot: &BrickPlot, scene: &mut Scene, computed: &ComputedLayout) {

    // Use expanded strigars when available, otherwise raw sequences
    let rows: &Vec<String> = if let Some(ref exp) = brickplot.strigar_exp {
        exp
    } else {
        &brickplot.sequences
    };

    let num_rows = rows.len();
    if num_rows == 0 {
        return;
    }

    let has_variable_width = brickplot.motif_lengths.is_some();

    for (i, row) in rows.iter().enumerate() {
        let mut x_pos: f64 = 0.0;
        for (j, value) in row.chars().enumerate() {
            let width = if let Some(ref ml) = brickplot.motif_lengths {
                *ml.get(&value).unwrap_or(&1) as f64
            } else {
                1.0
            };
            let x_start = if has_variable_width { x_pos } else { j as f64 };

            let color = brickplot.template.as_ref().unwrap().get(&value).unwrap();

            let x0 = computed.map_x(x_start - brickplot.x_offset);
            let x1 = computed.map_x(x_start + width - brickplot.x_offset);
            let y0 = computed.map_y(i as f64 + 1.0);
            let y1 = computed.map_y(i as f64);
            scene.add(Primitive::Rect {
                x: x0,
                y: y0,
                width: (x1-x0).abs()*0.95,
                height: (y1-y0).abs()*0.95,
                fill: color.clone(),
                stroke: None,
                stroke_width: None,
                opacity: None,
            });

            x_pos += width;
        }
    }
    if brickplot.show_values {
        for (i, row) in rows.iter().enumerate() {
            let mut x_pos: f64 = 0.0;
            for (j, value) in row.chars().enumerate() {
                let width = if let Some(ref ml) = brickplot.motif_lengths {
                    *ml.get(&value).unwrap_or(&1) as f64
                } else {
                    1.0
                };
                let x_start = if has_variable_width { x_pos } else { j as f64 };

                let x0 = computed.map_x(x_start - brickplot.x_offset);
                let x1 = computed.map_x(x_start + width - brickplot.x_offset);
                let y0 = computed.map_y(i as f64 + 1.0);
                let y1 = computed.map_y(i as f64);
                scene.add(Primitive::Text {
                    x: x0 + ((x1-x0).abs() / 2.0),
                    y: y0 + ((y1-y0).abs() / 2.0),
                    content: format!("{}", value),
                    size: 12,
                    anchor: TextAnchor::Middle,
                    rotate: None,
                });

                x_pos += width;
            }
        }
    }
}

fn add_legend(legend: &Legend, scene: &mut Scene, computed: &ComputedLayout) {

    let legend_width = computed.legend_width;
    let legend_padding = 10.0;
    let line_height = 18.0;
    let legend_height = legend.entries.len() as f64 * line_height + legend_padding * 2.0;

    let (legend_x, mut legend_y) = match computed.legend_position {
        LegendPosition::TopRight => {
            (computed.width - computed.margin_right + 10.0, computed.margin_top)
        }
        LegendPosition::BottomRight => {
            (computed.width - computed.margin_right + 10.0,
             computed.height - computed.margin_bottom - legend_height)
        }
        LegendPosition::TopLeft => {
            (legend_padding, computed.margin_top)
        }
        LegendPosition::BottomLeft => {
            (legend_padding,
             computed.height - computed.margin_bottom - legend_height)
        }
    };

    scene.add(Primitive::Rect {
        x: legend_x - legend_padding + 5.0,
        y: legend_y - legend_padding,
        width: legend_width,
        height: legend_height,
        fill: "white".into(),
        stroke: None,
        stroke_width: None,
        opacity: None,
    });

    scene.add(Primitive::Rect {
        x: legend_x - legend_padding + 5.0,
        y: legend_y - legend_padding,
        width: legend_width,
        height: legend_height,
        fill: "none".into(),
        stroke: Some("black".into()),
        stroke_width: Some(1.0),
        opacity: None,
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
                x: legend_x + 5.0,
                y: legend_y - 5.0,
                width: 12.0,
                height: 12.0,
                fill: entry.color.clone(),
                stroke: None,
                stroke_width: None,
                opacity: None,
            }),
            LegendShape::Line => scene.add(Primitive::Line {
                x1: legend_x + 5.0,
                y1: legend_y + 2.0,
                x2: legend_x + 5.0 + 12.0,
                y2: legend_y + 2.0,
                stroke: entry.color.clone(),
                stroke_width: 2.0,
                stroke_dasharray: None,
            }),
            LegendShape::Circle => scene.add(Primitive::Circle {
                cx: legend_x + 5.0 + 6.0,
                cy: legend_y + 1.0,
                r: 5.0,
                fill: entry.color.clone(),
            }),
        }

        legend_y += line_height;
    }
}

fn add_colorbar(info: &ColorBarInfo, scene: &mut Scene, computed: &ComputedLayout) {
    let bar_width = 20.0;
    let bar_height = computed.plot_height() * 0.8;
    let bar_x = computed.width - 70.0; // rightmost area
    let bar_y = computed.margin_top + computed.plot_height() * 0.1; // vertically centered

    let num_slices = 50;
    let slice_height = bar_height / num_slices as f64;

    // Draw stacked rects (top = high value, bottom = low value)
    for i in 0..num_slices {
        let t = 1.0 - (i as f64 / (num_slices - 1) as f64); // top is high
        let value = info.min_value + t * (info.max_value - info.min_value);
        let color = (info.map_fn)(value);
        let y = bar_y + i as f64 * slice_height;

        scene.add(Primitive::Rect {
            x: bar_x,
            y,
            width: bar_width,
            height: slice_height + 0.5, // slight overlap to prevent gaps
            fill: color,
            stroke: None,
            stroke_width: None,
            opacity: None,
        });
    }

    // Black border around the bar
    scene.add(Primitive::Rect {
        x: bar_x,
        y: bar_y,
        width: bar_width,
        height: bar_height,
        fill: "none".into(),
        stroke: Some("black".into()),
        stroke_width: Some(1.0),
        opacity: None,
    });

    // Tick marks and labels
    let ticks = render_utils::generate_ticks(info.min_value, info.max_value, 5);
    let range = info.max_value - info.min_value;
    for tick in &ticks {
        if *tick < info.min_value || *tick > info.max_value {
            continue;
        }
        let frac = (tick - info.min_value) / range;
        let y = bar_y + bar_height - frac * bar_height; // invert: high values at top

        // tick mark
        scene.add(Primitive::Line {
            x1: bar_x + bar_width,
            y1: y,
            x2: bar_x + bar_width + 4.0,
            y2: y,
            stroke: "black".into(),
            stroke_width: 1.0,
            stroke_dasharray: None,
        });

        // tick label
        scene.add(Primitive::Text {
            x: bar_x + bar_width + 6.0,
            y: y + 4.0,
            content: format!("{:.1}", tick),
            size: 10,
            anchor: TextAnchor::Start,
            rotate: None,
        });
    }

    // Optional label above the bar
    if let Some(ref label) = info.label {
        scene.add(Primitive::Text {
            x: bar_x + bar_width / 2.0,
            y: bar_y - 6.0,
            content: label.clone(),
            size: 11,
            anchor: TextAnchor::Middle,
            rotate: None,
        });
    }
}


/// render_scatter
pub fn render_scatter(scatter: &ScatterPlot, layout: Layout) -> Scene {
    
    let computed = ComputedLayout::from_layout(&layout);
        
    let mut scene = Scene::new(computed.width, computed.height);
    
    add_axes_and_grid(&mut scene, &computed, &layout);
    add_labels_and_title(&mut scene, &computed, &layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);

    add_scatter(scatter, &mut scene, &computed);

    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);

    scene
}

// render_line
pub fn render_line(line: &LinePlot, layout: Layout) -> Scene {

    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);

    add_axes_and_grid(&mut scene, &computed, &layout);
    add_labels_and_title(&mut scene, &computed, &layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);

    add_line(line, &mut scene, &computed);

    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);

    scene
}

// render_bar
pub fn render_bar(bar: &BarPlot, layout: Layout) -> Scene {

    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);

    add_axes_and_grid(&mut scene, &computed, &layout);
    add_labels_and_title(&mut scene, &computed, &layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);

    add_bar(bar, &mut scene, &computed);

    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);

    scene
}

// render_bar_categories
pub fn render_bar_categories(bar: &BarPlot, layout: Layout) -> Scene {

    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);

    add_axes_and_grid(&mut scene, &computed, &layout);
    add_labels_and_title(&mut scene, &computed, &layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);

    add_bar(bar, &mut scene, &computed);

    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);

    scene
}

// render_histogram
pub fn render_histogram(hist: &Histogram, layout: &Layout) -> Scene {

    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);

    add_axes_and_grid(&mut scene, &computed, &layout);
    add_labels_and_title(&mut scene, &computed, &layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);

    add_histogram(hist, &mut scene, &computed);

    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);

    scene
}

// render_boxplot
pub fn render_boxplot(boxplot: &BoxPlot, layout: &Layout) -> Scene {

    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);

    add_axes_and_grid(&mut scene, &computed, &layout);
    add_labels_and_title(&mut scene, &computed, &layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);

    add_boxplot(boxplot, &mut scene, &computed);

    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);

    scene
}

// render_violinplot
pub fn render_violin(violin: &ViolinPlot, layout: &Layout) -> Scene {

    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);

    add_axes_and_grid(&mut scene, &computed, &layout);
    add_labels_and_title(&mut scene, &computed, &layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);

    add_violin(violin, &mut scene, &computed);

    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);

    scene
}

pub fn render_pie(pie: &PiePlot, layout: &Layout) -> Scene {

    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);

    // add_axes_and_grid(&mut scene, &computed, &layout);
    add_labels_and_title(&mut scene, &computed, &layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);

    add_pie(pie, &mut scene, &computed);

    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);

    scene
}

// render_brickplot
pub fn render_brickplot(brickplot: &BrickPlot, layout: &Layout) -> Scene {

    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);

    // add_axes_and_grid(&mut scene, &computed, &layout);
    add_labels_and_title(&mut scene, &computed, &layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);

    add_brickplot(brickplot, &mut scene, &computed);

    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);

    scene
}



/// this should be the default renderer.
/// TODO: make an alias of this for single plots, that vectorises
pub fn render_multiple(plots: Vec<Plot>, layout: Layout) -> Scene {
    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);

    add_axes_and_grid(&mut scene, &computed, &layout);
    add_labels_and_title(&mut scene, &computed, &layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);

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
            Plot::Histogram2d(h) => {
                add_histogram2d(&h, &mut scene, &computed);
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
            Plot::Heatmap(h) => {
                add_heatmap(&h, &mut scene, &computed);
            }
            Plot::Brick(b) => {
                add_brickplot(&b, &mut scene, &computed);
            }
            Plot::Band(b) => {
                add_band(&b, &mut scene, &computed);
            }
        }
    }

    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);

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
                        shape: LegendShape::Rect,
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
            Plot::Brick(brickplot) => {
                let labels = brickplot.template.as_ref().unwrap();
                let motifs = brickplot.motifs.as_ref();
                for (letter, color) in labels {
                    let label = if let Some(m) = motifs {
                        m.get(letter).cloned().unwrap_or(letter.to_string())
                    } else {
                        letter.to_string()
                    };
                    legend.entries.push(LegendEntry {
                        label,
                        color: color.clone(),
                        shape: LegendShape::Rect,
                    })
                }
            }
            Plot::Box(boxplot) => {
                if let Some(label) = &boxplot.legend_label {
                    legend.entries.push(LegendEntry {
                        label: label.clone(),
                        color: boxplot.color.clone(),
                        shape: LegendShape::Rect,
                    });
                }
            }
            Plot::Violin(violin) => {
                if let Some(label) = &violin.legend_label {
                    legend.entries.push(LegendEntry {
                        label: label.clone(),
                        color: violin.color.clone(),
                        shape: LegendShape::Rect,
                    });
                }
            }
            Plot::Histogram(hist) => {
                if let Some(label) = &hist.legend_label {
                    legend.entries.push(LegendEntry {
                        label: label.clone(),
                        color: hist.color.clone(),
                        shape: LegendShape::Rect,
                    });
                }
            }
            Plot::Heatmap(heatmap) => {
                if let Some(label) = &heatmap.legend_label {
                    legend.entries.push(LegendEntry {
                        label: label.clone(),
                        color: "gray".into(),
                        shape: LegendShape::Rect,
                    });
                }
            }
            Plot::Pie(pie) => {
                if pie.legend_label.is_some() {
                    let total: f64 = pie.slices.iter().map(|s| s.value).sum();
                    for slice in &pie.slices {
                        let label = if pie.show_percent {
                            let pct = slice.value / total * 100.0;
                            if slice.label.is_empty() {
                                format!("{:.1}%", pct)
                            } else {
                                format!("{} ({:.1}%)", slice.label, pct)
                            }
                        } else {
                            slice.label.clone()
                        };
                        legend.entries.push(LegendEntry {
                            label,
                            color: slice.color.clone(),
                            shape: LegendShape::Rect,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    if layout.show_legend {
        add_legend(&legend, &mut scene, &computed);
    }

    if layout.show_colorbar {
        for plot in plots.iter() {
            if let Some(info) = plot.colorbar_info() {
                add_colorbar(&info, &mut scene, &computed);
                break; // one colorbar per figure
            }
        }
    }

    scene
}
