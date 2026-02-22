use crate::render::render_utils::{self, percentile, linear_regression, pearson_corr};
use crate::render::layout::{Layout, ComputedLayout};
use crate::render::plots::Plot;
use crate::render::axis::{add_axes_and_grid, add_labels_and_title, add_y2_axis};
use crate::render::annotations::{add_shaded_regions, add_reference_lines, add_text_annotations};
use crate::render::theme::Theme;

use crate::plot::scatter::{ScatterPlot, TrendLine, MarkerShape};
use crate::plot::line::LinePlot;
use crate::plot::bar::BarPlot;
use crate::plot::histogram::Histogram;
use crate::plot::band::BandPlot;
use crate::plot::{BoxPlot, BrickPlot, Heatmap, Histogram2D, PiePlot, SeriesPlot, SeriesStyle, ViolinPlot};
use crate::plot::pie::PieLabelPosition;
use crate::plot::waterfall::{WaterfallPlot, WaterfallKind};
use crate::plot::strip::{StripPlot, StripStyle};
use crate::plot::volcano::{VolcanoPlot, LabelStyle};


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
        bold: bool,
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
        stroke_dasharray: Option<String>,
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
    GroupStart {
        transform: Option<String>,
    },
    GroupEnd,
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
    pub text_color: Option<String>,
    pub font_family: Option<String>,
    pub elements: Vec<Primitive>,
}

impl Scene {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width,
               height,
               background_color: Some("white".to_string()),
               text_color: None,
               font_family: None,
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

fn apply_theme(scene: &mut Scene, theme: &Theme) {
    scene.background_color = Some(theme.background.clone());
    scene.text_color = Some(theme.text_color.clone());
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

/// Build an SVG step-path (staircase) from a sequence of (x, y) screen-coordinate points.
/// For each pair of consecutive points, inserts a horizontal segment to x1 before moving to y1.
pub fn build_step_path(points: &[(f64, f64)]) -> String {
    let mut path = String::new();
    for (i, &(x, y)) in points.iter().enumerate() {
        if i == 0 {
            path += &format!("M {x} {y} ");
        } else {
            let prev_y = points[i - 1].1;
            path += &format!("L {x} {prev_y} ");
            path += &format!("L {x} {y} ");
        }
    }
    path
}

fn draw_marker(scene: &mut Scene, marker: MarkerShape, cx: f64, cy: f64, size: f64, fill: &str) {
    match marker {
        MarkerShape::Circle => {
            scene.add(Primitive::Circle { cx, cy, r: size, fill: fill.into() });
        }
        MarkerShape::Square => {
            scene.add(Primitive::Rect {
                x: cx - size,
                y: cy - size,
                width: size * 2.0,
                height: size * 2.0,
                fill: fill.into(),
                stroke: None,
                stroke_width: None,
                opacity: None,
            });
        }
        MarkerShape::Triangle => {
            let h = size * 1.7; // equilateral-ish
            let d = format!(
                "M{},{} L{},{} L{},{} Z",
                cx, cy - h * 0.6,
                cx - size, cy + h * 0.4,
                cx + size, cy + h * 0.4,
            );
            scene.add(Primitive::Path {
                d,
                fill: Some(fill.into()),
                stroke: fill.into(),
                stroke_width: 0.5,
                opacity: None,
                stroke_dasharray: None,
            });
        }
        MarkerShape::Diamond => {
            let s = size * 1.3;
            let d = format!(
                "M{},{} L{},{} L{},{} L{},{} Z",
                cx, cy - s,
                cx + s, cy,
                cx, cy + s,
                cx - s, cy,
            );
            scene.add(Primitive::Path {
                d,
                fill: Some(fill.into()),
                stroke: fill.into(),
                stroke_width: 0.5,
                opacity: None,
                stroke_dasharray: None,
            });
        }
        MarkerShape::Cross => {
            let s = size * 0.9;
            scene.add(Primitive::Line {
                x1: cx - s, y1: cy - s, x2: cx + s, y2: cy + s,
                stroke: fill.into(), stroke_width: 1.5, stroke_dasharray: None,
            });
            scene.add(Primitive::Line {
                x1: cx - s, y1: cy + s, x2: cx + s, y2: cy - s,
                stroke: fill.into(), stroke_width: 1.5, stroke_dasharray: None,
            });
        }
        MarkerShape::Plus => {
            let s = size * 0.9;
            scene.add(Primitive::Line {
                x1: cx - s, y1: cy, x2: cx + s, y2: cy,
                stroke: fill.into(), stroke_width: 1.5, stroke_dasharray: None,
            });
            scene.add(Primitive::Line {
                x1: cx, y1: cy - s, x2: cx, y2: cy + s,
                stroke: fill.into(), stroke_width: 1.5, stroke_dasharray: None,
            });
        }
    }
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
        opacity: Some(band.opacity),
        stroke_dasharray: None,
    });
}

fn add_scatter(scatter: &ScatterPlot, scene: &mut Scene, computed: &ComputedLayout) {
    // Draw band behind points if present
    if let Some(ref band) = scatter.band {
        add_band(band, scene, computed);
    }

    // Draw points
    for (i, point) in scatter.data.iter().enumerate() {
        let size = scatter.sizes.as_ref()
            .and_then(|s| s.get(i).copied())
            .unwrap_or(scatter.size);
        draw_marker(
            scene,
            scatter.marker,
            computed.map_x(point.x),
            computed.map_y(point.y),
            size,
            &scatter.color,
        );

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
                            size: computed.body_size,
                            anchor: TextAnchor::Start,
                            rotate: None,
                            bold: false,
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

        let stroke_d = if line.step {
            build_step_path(&points)
        } else {
            build_path(&points)
        };

        // Draw fill area behind the stroke line
        if line.fill {
            let baseline_y = computed.map_y(computed.y_range.0.max(0.0));
            let first_x = points.first().unwrap().0;
            let last_x = points.last().unwrap().0;
            let fill_d = format!(
                "{}L {last_x} {baseline_y} L {first_x} {baseline_y} Z",
                stroke_d
            );
            scene.add(Primitive::Path {
                d: fill_d,
                fill: Some(line.color.clone()),
                stroke: "none".into(),
                stroke_width: 0.0,
                opacity: Some(line.fill_opacity),
                stroke_dasharray: None,
            });
        }

        scene.add(Primitive::Path {
            d: stroke_d,
            fill: None,
            stroke: line.color.clone(),
            stroke_width: line.stroke_width,
            opacity: None,
            stroke_dasharray: line.line_style.dasharray(),
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
                        stroke_dasharray: None,
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
                        stroke_dasharray: None,
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
    for (i, group) in bar.groups.iter().enumerate() {
        let group_x = i as f64 + 1.0;
        let total_width = bar.width;

        if bar.stacked {
            let mut y_accum = 0.0;
            for bar_val in &group.bars {
                let x0 = computed.map_x(group_x - total_width / 2.0);
                let x1 = computed.map_x(group_x + total_width / 2.0);
                let y0 = computed.map_y(y_accum);
                let y1 = computed.map_y(y_accum + bar_val.value);

                scene.add(Primitive::Rect {
                    x: x0,
                    y: y1.min(y0),
                    width: (x1 - x0).abs(),
                    height: (y0 - y1).abs(),
                    fill: bar_val.color.clone(),
                    stroke: None,
                    stroke_width: None,
                    opacity: None,
                });

                y_accum += bar_val.value;
            }
        } else {
            let n = group.bars.len();
            let single_width = total_width / n as f64;

            for (j, bar_val) in group.bars.iter().enumerate() {
                let x = group_x - total_width / 2.0 + single_width * (j as f64 + 0.5);
                let x0 = computed.map_x(x - single_width / 2.0);
                let x1 = computed.map_x(x + single_width / 2.0);
                let y0 = computed.map_y(0.0);
                let y1 = computed.map_y(bar_val.value);

                scene.add(Primitive::Rect {
                    x: x0,
                    y: y1.min(y0),
                    width: (x1 - x0).abs(),
                    height: (y0 - y1).abs(),
                    fill: bar_val.color.clone(),
                    stroke: None,
                    stroke_width: None,
                    opacity: None,
                });
            }
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
            size: computed.body_size,
            anchor: TextAnchor::End,
            rotate: None,
            bold: false,
        });
    }
}


fn add_boxplot(boxplot: &BoxPlot, scene: &mut Scene, computed: &ComputedLayout) {
    let theme = &computed.theme;
    
    
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
            stroke: theme.box_median.clone(),
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

    // Overlay strip/swarm points after boxes
    if let Some(ref style) = boxplot.overlay {
        for (i, group) in boxplot.groups.iter().enumerate() {
            add_strip_points(
                &group.values,
                (i + 1) as f64,
                style,
                &boxplot.overlay_color,
                boxplot.overlay_size,
                boxplot.overlay_seed.wrapping_add(i as u64),
                scene,
                computed,
            );
        }
    }
}

fn add_violin(violin: &ViolinPlot, scene: &mut Scene, computed: &ComputedLayout) {
    let theme = &computed.theme;

    for (i, group) in violin.groups.iter().enumerate() {
        if group.values.is_empty() { continue; }
        let x_center = computed.map_x((i + 1) as f64);

        // Compute KDE with auto or manual bandwidth
        let h = violin.bandwidth
            .unwrap_or_else(|| render_utils::silverman_bandwidth(&group.values));
        let kde = render_utils::simple_kde(&group.values, h, violin.kde_samples);
        if kde.is_empty() { continue; }

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
            stroke: theme.violin_border.clone(),
            stroke_width: 0.5,
            opacity: None,
            stroke_dasharray: None,
        });
    }

    // Overlay strip/swarm points after violin shapes
    if let Some(ref style) = violin.overlay {
        for (i, group) in violin.groups.iter().enumerate() {
            add_strip_points(
                &group.values,
                (i + 1) as f64,
                style,
                &violin.overlay_color,
                violin.overlay_size,
                violin.overlay_seed.wrapping_add(i as u64),
                scene,
                computed,
            );
        }
    }
}

fn add_pie(pie: &PiePlot, scene: &mut Scene, computed: &ComputedLayout) {
    let theme = &computed.theme;

    let total: f64 = pie.slices.iter().map(|s| s.value).sum();

    let has_outside = matches!(pie.label_position, PieLabelPosition::Outside | PieLabelPosition::Auto);

    let leader_gap = 30.0;
    let pad = 5.0;

    // Size radius from vertical space; canvas was already widened in render_pie
    let radius = if has_outside {
        computed.plot_height() / 2.0 - pad
    } else {
        computed.plot_width().min(computed.plot_height()) / 2.0 - 10.0
    };

    // Center pie in the plot area (width may have been adjusted by render_pie)
    let cx = computed.margin_left + computed.plot_width() / 2.0;
    let cy = computed.margin_top + computed.plot_height() / 2.0;
    let inner_radius = pie.inner_radius;
    let inside_label_radius = (radius + inner_radius) / 2.0;
    let mut angle = 0.0;

    // Collect outside labels for anti-overlap pass
    struct OutsideLabel {
        content: String,
        right_side: bool,
        // Fixed: radial segment from edge to elbow
        edge_x: f64,
        edge_y: f64,
        elbow_x: f64,
        elbow_y: f64,
        // Text position (y will be nudged)
        text_x: f64,
        text_y: f64,
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
            stroke_dasharray: None,
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
                size: computed.body_size,
                anchor: TextAnchor::Middle,
                rotate: None,
                bold: false,
            });
        } else {
            let right_side = mid_angle.cos() >= 0.0;
            let edge_x = cx + (radius + 5.0) * mid_angle.cos();
            let edge_y = cy + (radius + 5.0) * mid_angle.sin();
            let elbow_x = cx + (radius + 20.0) * mid_angle.cos();
            let elbow_y = cy + (radius + 20.0) * mid_angle.sin();
            // Text extends horizontally from the elbow
            let text_x = if right_side { cx + radius + leader_gap } else { cx - radius - leader_gap };
            let text_y = elbow_y;

            outside_labels.push(OutsideLabel {
                content: label_text,
                right_side,
                edge_x, edge_y,
                elbow_x, elbow_y,
                text_x, text_y,
            });
        }

        angle = end_angle;
    }

    // Anti-overlap: process right and left sides independently
    let min_gap = computed.body_size as f64 + 2.0;
    for side in [true, false] {
        let mut indices: Vec<usize> = outside_labels.iter().enumerate()
            .filter(|(_, l)| l.right_side == side)
            .map(|(i, _)| i)
            .collect();
        indices.sort_by(|a, b| outside_labels[*a].text_y.partial_cmp(&outside_labels[*b].text_y).unwrap());
        for j in 1..indices.len() {
            let prev_y = outside_labels[indices[j - 1]].text_y;
            if outside_labels[indices[j]].text_y - prev_y < min_gap {
                outside_labels[indices[j]].text_y = prev_y + min_gap;
            }
        }
    }

    // Render outside labels with two-segment leader lines
    for label in &outside_labels {
        // Segment 1: radial line from pie edge to elbow
        scene.add(Primitive::Line {
            x1: label.edge_x,
            y1: label.edge_y,
            x2: label.elbow_x,
            y2: label.elbow_y,
            stroke: theme.pie_leader.clone(),
            stroke_width: 1.0,
            stroke_dasharray: None,
        });
        // Segment 2: connector from elbow to text position (tracks nudged y)
        scene.add(Primitive::Line {
            x1: label.elbow_x,
            y1: label.elbow_y,
            x2: label.text_x,
            y2: label.text_y,
            stroke: theme.pie_leader.clone(),
            stroke_width: 1.0,
            stroke_dasharray: None,
        });
        let anchor = if label.right_side { TextAnchor::Start } else { TextAnchor::End };
        scene.add(Primitive::Text {
            x: label.text_x,
            y: label.text_y,
            content: label.content.clone(),
            size: computed.body_size,
            anchor,
            rotate: None,
            bold: false,
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
                    size: computed.body_size,
                    anchor: TextAnchor::Middle,
                    rotate: None,
                    bold: false,
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
    // Resolve the offset for a given row index.
    // Strigar mode always uses 0; DNA mode uses the per-row value if available,
    // otherwise falls back to the global x_offset.
    let row_offset = |i: usize| -> f64 {
        if brickplot.strigar_exp.is_some() {
            0.0
        } else if let Some(ref offsets) = brickplot.x_offsets {
            offsets.get(i).copied().flatten().unwrap_or(brickplot.x_offset)
        } else {
            brickplot.x_offset
        }
    };

    for (i, row) in rows.iter().enumerate() {
        let x_offset = row_offset(i);
        let mut x_pos: f64 = 0.0;
        for (j, value) in row.chars().enumerate() {
            let width = if let Some(ref ml) = brickplot.motif_lengths {
                *ml.get(&value).unwrap_or(&1) as f64
            } else {
                1.0
            };
            let x_start = if has_variable_width { x_pos } else { j as f64 };

            let color = brickplot.template.as_ref().unwrap().get(&value).unwrap();

            let x0 = computed.map_x(x_start - x_offset);
            let x1 = computed.map_x(x_start + width - x_offset);
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
            let x_offset = row_offset(i);
            let mut x_pos: f64 = 0.0;
            for (j, value) in row.chars().enumerate() {
                let width = if let Some(ref ml) = brickplot.motif_lengths {
                    *ml.get(&value).unwrap_or(&1) as f64
                } else {
                    1.0
                };
                let x_start = if has_variable_width { x_pos } else { j as f64 };

                let x0 = computed.map_x(x_start - x_offset);
                let x1 = computed.map_x(x_start + width - x_offset);
                let y0 = computed.map_y(i as f64 + 1.0);
                let y1 = computed.map_y(i as f64);
                scene.add(Primitive::Text {
                    x: x0 + ((x1-x0).abs() / 2.0),
                    y: y0 + ((y1-y0).abs() / 2.0),
                    content: format!("{}", value),
                    size: computed.body_size,
                    anchor: TextAnchor::Middle,
                    rotate: None,
                    bold: false,
                });

                x_pos += width;
            }
        }
    }
}

fn add_strip_points(
    values: &[f64],
    x_center_data: f64,
    style: &StripStyle,
    color: &str,
    point_size: f64,
    seed: u64,
    scene: &mut Scene,
    computed: &ComputedLayout,
) {
    use rand::SeedableRng;
    use rand::Rng;
    use rand::rngs::SmallRng;

    match style {
        StripStyle::Center => {
            let cx = computed.map_x(x_center_data);
            for &v in values {
                let cy = computed.map_y(v);
                scene.add(Primitive::Circle { cx, cy, r: point_size, fill: color.into() });
            }
        }
        StripStyle::Strip { jitter } => {
            let mut rng = SmallRng::seed_from_u64(seed);
            for &v in values {
                let offset: f64 = (rng.random::<f64>() - 0.5) * jitter;
                let cx = computed.map_x(x_center_data + offset);
                let cy = computed.map_y(v);
                scene.add(Primitive::Circle { cx, cy, r: point_size, fill: color.into() });
            }
        }
        StripStyle::Swarm => {
            let y_screen: Vec<f64> = values.iter().map(|&v| computed.map_y(v)).collect();
            let x_offsets = render_utils::beeswarm_positions(&y_screen, point_size);
            let cx_center = computed.map_x(x_center_data);
            for (i, &v) in values.iter().enumerate() {
                let cx = cx_center + x_offsets[i];
                let cy = computed.map_y(v);
                scene.add(Primitive::Circle { cx, cy, r: point_size, fill: color.into() });
            }
        }
    }
}

fn add_strip(strip: &StripPlot, scene: &mut Scene, computed: &ComputedLayout) {
    for (i, group) in strip.groups.iter().enumerate() {
        add_strip_points(
            &group.values,
            (i + 1) as f64,
            &strip.style,
            &strip.color,
            strip.point_size,
            strip.seed.wrapping_add(i as u64),
            scene,
            computed,
        );
    }
}

fn add_waterfall(waterfall: &WaterfallPlot, scene: &mut Scene, computed: &ComputedLayout) {
    let mut running = 0.0_f64;
    let mut prev_connection_y: Option<f64> = None;
    let half = waterfall.bar_width / 2.0;

    for (i, bar) in waterfall.bars.iter().enumerate() {
        let x_center = (i + 1) as f64;
        let x0 = computed.map_x(x_center - half);
        let x1 = computed.map_x(x_center + half);

        let (base, top, color) = match bar.kind {
            WaterfallKind::Delta => {
                let base = running;
                running += bar.value;
                let color = if bar.value >= 0.0 {
                    waterfall.color_positive.clone()
                } else {
                    waterfall.color_negative.clone()
                };
                (base, running, color)
            }
            WaterfallKind::Total => {
                (0.0, running, waterfall.color_total.clone())
            }
        };

        // Connector: dashed horizontal line from previous bar's right edge to this bar's left edge
        if waterfall.show_connectors {
            if let Some(py) = prev_connection_y {
                let prev_x_right = computed.map_x(i as f64 + half);
                scene.add(Primitive::Line {
                    x1: prev_x_right,
                    y1: py,
                    x2: x0,
                    y2: py,
                    stroke: "gray".into(),
                    stroke_width: 1.0,
                    stroke_dasharray: Some("4,3".into()),
                });
            }
        }

        // Bar rect
        let y_screen_lo = computed.map_y(base.min(top));
        let y_screen_hi = computed.map_y(base.max(top));
        let bar_height = (y_screen_lo - y_screen_hi).abs();
        if bar_height > 0.0 {
            scene.add(Primitive::Rect {
                x: x0,
                y: y_screen_hi,
                width: (x1 - x0).abs(),
                height: bar_height,
                fill: color,
                stroke: None,
                stroke_width: None,
                opacity: None,
            });
        }

        // Value label
        if waterfall.show_values {
            let (display, label_y) = match bar.kind {
                WaterfallKind::Delta => {
                    let s = if bar.value >= 0.0 {
                        format!("+{:.2}", bar.value)
                    } else {
                        format!("{:.2}", bar.value)
                    };
                    let ly = if bar.value >= 0.0 {
                        y_screen_hi - 5.0
                    } else {
                        y_screen_lo + 15.0
                    };
                    (s, ly)
                }
                WaterfallKind::Total => {
                    let s = format!("{:.2}", running);
                    let ly = if running >= 0.0 {
                        y_screen_hi - 5.0
                    } else {
                        y_screen_lo + 15.0
                    };
                    (s, ly)
                }
            };
            scene.add(Primitive::Text {
                x: (x0 + x1) / 2.0,
                y: label_y,
                content: display,
                size: computed.body_size,
                anchor: TextAnchor::Middle,
                rotate: None,
                bold: false,
            });
        }

        prev_connection_y = Some(computed.map_y(running));
    }
}

fn add_legend(legend: &Legend, scene: &mut Scene, computed: &ComputedLayout) {
    let theme = &computed.theme;

    let legend_width = computed.legend_width;
    let legend_padding = 10.0;
    let line_height = 18.0;
    let legend_height = legend.entries.len() as f64 * line_height + legend_padding * 2.0;

    let (legend_x, mut legend_y) = match computed.legend_position {
        LegendPosition::TopRight => {
            (computed.width - computed.margin_right + computed.y2_axis_width + 10.0, computed.margin_top)
        }
        LegendPosition::BottomRight => {
            (computed.width - computed.margin_right + computed.y2_axis_width + 10.0,
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
        fill: theme.legend_bg.clone(),
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
        stroke: Some(theme.legend_border.clone()),
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
            size: computed.body_size,
            rotate: None,
            bold: false,
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
                stroke_dasharray: entry.dasharray.clone(),
            }),
            LegendShape::Circle => scene.add(Primitive::Circle {
                cx: legend_x + 5.0 + 6.0,
                cy: legend_y + 1.0,
                r: 5.0,
                fill: entry.color.clone(),
            }),
            LegendShape::Marker(marker) => {
                draw_marker(scene, marker, legend_x + 5.0 + 6.0, legend_y + 1.0, 5.0, &entry.color);
            }
        }

        legend_y += line_height;
    }
}

fn add_colorbar(info: &ColorBarInfo, scene: &mut Scene, computed: &ComputedLayout) {
    let theme = &computed.theme;
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

    // Border around the bar
    scene.add(Primitive::Rect {
        x: bar_x,
        y: bar_y,
        width: bar_width,
        height: bar_height,
        fill: "none".into(),
        stroke: Some(theme.colorbar_border.clone()),
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
            stroke: theme.colorbar_border.clone(),
            stroke_width: 1.0,
            stroke_dasharray: None,
        });

        // tick label
        scene.add(Primitive::Text {
            x: bar_x + bar_width + 6.0,
            y: y + 4.0,
            content: format!("{:.1}", tick),
            size: computed.tick_size,
            anchor: TextAnchor::Start,
            rotate: None,
            bold: false,
        });
    }

    // Optional label above the bar
    if let Some(ref label) = info.label {
        scene.add(Primitive::Text {
            x: bar_x + bar_width / 2.0,
            y: bar_y - 6.0,
            content: label.clone(),
            size: computed.tick_size,
            anchor: TextAnchor::Middle,
            rotate: None,
            bold: false,
        });
    }
}


fn add_volcano(vp: &VolcanoPlot, scene: &mut Scene, computed: &ComputedLayout) {
    let floor = vp.floor();

    // Draw threshold lines (behind points)
    let threshold_color = "#888888";
    let plot_left = computed.margin_left;
    let plot_right = computed.width - computed.margin_right;
    let plot_top = computed.margin_top;
    let plot_bottom = computed.height - computed.margin_bottom;

    // Horizontal significance line at -log10(p_cutoff)
    let y_sig = -vp.p_cutoff.log10();
    if y_sig >= computed.y_range.0 && y_sig <= computed.y_range.1 {
        let sy = computed.map_y(y_sig);
        scene.add(Primitive::Line {
            x1: plot_left, y1: sy, x2: plot_right, y2: sy,
            stroke: threshold_color.into(),
            stroke_width: 1.0,
            stroke_dasharray: Some("4 4".into()),
        });
    }

    // Vertical fc cutoff lines at ±fc_cutoff
    for &fc_val in &[-vp.fc_cutoff, vp.fc_cutoff] {
        if fc_val >= computed.x_range.0 && fc_val <= computed.x_range.1 {
            let sx = computed.map_x(fc_val);
            scene.add(Primitive::Line {
                x1: sx, y1: plot_top, x2: sx, y2: plot_bottom,
                stroke: threshold_color.into(),
                stroke_width: 1.0,
                stroke_dasharray: Some("4 4".into()),
            });
        }
    }

    // Draw points: NS first, then Down, then Up
    for pass in 0..3u8 {
        for p in &vp.points {
            let is_up = p.log2fc >= vp.fc_cutoff && p.pvalue <= vp.p_cutoff;
            let is_down = p.log2fc <= -vp.fc_cutoff && p.pvalue <= vp.p_cutoff;
            let color = match (pass, is_up, is_down) {
                (0, false, false) => &vp.color_ns,
                (1, false, true)  => &vp.color_down,
                (2, true, false)  => &vp.color_up,
                _ => continue,
            };
            let y_val = -(p.pvalue.max(floor)).log10();
            let cx = computed.map_x(p.log2fc);
            let cy = computed.map_y(y_val);
            scene.add(Primitive::Circle { cx, cy, r: vp.point_size, fill: color.clone() });
        }
    }

    // Draw labels if label_top > 0
    if vp.label_top == 0 {
        return;
    }

    // Collect significant points, sort by pvalue ascending, take top N
    let mut sig_points: Vec<(f64, f64, &str)> = vp.points.iter()
        .filter(|p| p.pvalue <= vp.p_cutoff)
        .map(|p| {
            let y_val = -(p.pvalue.max(floor)).log10();
            (computed.map_x(p.log2fc), computed.map_y(y_val), p.name.as_str())
        })
        .collect();
    // Sort by pvalue ascending = highest -log10(p) = smallest cy
    sig_points.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    sig_points.truncate(vp.label_top);

    match vp.label_style {
        LabelStyle::Exact => {
            for (cx, cy, name) in &sig_points {
                scene.add(Primitive::Text {
                    x: *cx,
                    y: cy - vp.point_size - 2.0,
                    content: name.to_string(),
                    size: computed.body_size,
                    anchor: TextAnchor::Middle,
                    rotate: None,
                    bold: false,
                });
            }
        }
        LabelStyle::Nudge => {
            // Build label positions: initially just above each point
            let mut labels: Vec<(f64, f64, String)> = sig_points.iter()
                .map(|(cx, cy, name)| (*cx, cy - vp.point_size - 2.0, name.to_string()))
                .collect();

            // Sort by cx (x screen position, left to right)
            labels.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            // Greedy vertical nudge: push y up when adjacent labels are too close
            let min_gap = computed.body_size as f64 + 2.0;
            for j in 1..labels.len() {
                let prev_y = labels[j - 1].1;
                let curr_y = labels[j].1;
                if (prev_y - curr_y).abs() < min_gap {
                    labels[j].1 = prev_y - min_gap;
                }
            }

            for (cx, label_y, name) in &labels {
                scene.add(Primitive::Text {
                    x: *cx,
                    y: *label_y,
                    content: name.clone(),
                    size: computed.body_size,
                    anchor: TextAnchor::Middle,
                    rotate: None,
                    bold: false,
                });
            }
        }
        LabelStyle::Arrow { offset_x, offset_y } => {
            for (cx, cy, name) in &sig_points {
                let text_x = cx + offset_x;
                let text_y = cy - offset_y;

                // Leader line from text toward point, stopping short
                let dx = cx - text_x;
                let dy = cy - text_y;
                let len = (dx * dx + dy * dy).sqrt();
                if len > vp.point_size + 3.0 {
                    let scale = (len - vp.point_size - 3.0) / len;
                    let end_x = text_x + dx * scale;
                    let end_y = text_y + dy * scale;
                    scene.add(Primitive::Line {
                        x1: text_x, y1: text_y, x2: end_x, y2: end_y,
                        stroke: "#666666".into(),
                        stroke_width: 0.8,
                        stroke_dasharray: None,
                    });
                }

                let anchor = if offset_x >= 0.0 { TextAnchor::Start } else { TextAnchor::End };
                scene.add(Primitive::Text {
                    x: text_x,
                    y: text_y,
                    content: name.to_string(),
                    size: computed.body_size,
                    anchor,
                    rotate: None,
                    bold: false,
                });
            }
        }
    }
}

pub fn render_volcano(vp: &VolcanoPlot, layout: &Layout) -> Scene {
    let computed = ComputedLayout::from_layout(layout);
    let mut scene = Scene::new(computed.width, computed.height);
    scene.font_family = computed.font_family.clone();
    apply_theme(&mut scene, &computed.theme);
    add_axes_and_grid(&mut scene, &computed, layout);
    add_labels_and_title(&mut scene, &computed, layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);
    add_volcano(vp, &mut scene, &computed);
    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);
    scene
}

/// render_scatter
pub fn render_scatter(scatter: &ScatterPlot, layout: Layout) -> Scene {

    let computed = ComputedLayout::from_layout(&layout);

    let mut scene = Scene::new(computed.width, computed.height);
    scene.font_family = computed.font_family.clone();
    apply_theme(&mut scene, &computed.theme);
    
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
    scene.font_family = computed.font_family.clone();
    apply_theme(&mut scene, &computed.theme);

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
    scene.font_family = computed.font_family.clone();
    apply_theme(&mut scene, &computed.theme);

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
    scene.font_family = computed.font_family.clone();
    apply_theme(&mut scene, &computed.theme);

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
    scene.font_family = computed.font_family.clone();
    apply_theme(&mut scene, &computed.theme);

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
    scene.font_family = computed.font_family.clone();
    apply_theme(&mut scene, &computed.theme);

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
    scene.font_family = computed.font_family.clone();
    apply_theme(&mut scene, &computed.theme);

    add_axes_and_grid(&mut scene, &computed, &layout);
    add_labels_and_title(&mut scene, &computed, &layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);

    add_violin(violin, &mut scene, &computed);

    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);

    scene
}

pub fn render_pie(pie: &PiePlot, layout: &Layout) -> Scene {

    let mut computed = ComputedLayout::from_layout(&layout);

    // Widen canvas for outside pie labels before rendering title/labels
    let has_outside = matches!(pie.label_position, PieLabelPosition::Outside | PieLabelPosition::Auto);
    if has_outside {
        let total: f64 = pie.slices.iter().map(|s| s.value).sum();
        let char_width = computed.body_size as f64 * 0.6;
        let max_label_px = pie.slices.iter().map(|slice| {
            let frac = slice.value / total;
            let place_inside = match pie.label_position {
                PieLabelPosition::None | PieLabelPosition::Inside => true,
                PieLabelPosition::Outside => false,
                PieLabelPosition::Auto => frac >= pie.min_label_fraction,
            };
            if place_inside { return 0.0; }
            let label_text = if pie.show_percent {
                let pct = frac * 100.0;
                if slice.label.is_empty() { format!("{:.1}%", pct) }
                else { format!("{} ({:.1}%)", slice.label, pct) }
            } else {
                slice.label.clone()
            };
            label_text.len() as f64 * char_width
        }).fold(0.0f64, f64::max);

        let leader_gap = 30.0;
        let pad = 5.0;
        let radius = computed.plot_height() / 2.0 - pad;
        let needed_half = radius + leader_gap + max_label_px + pad;
        let needed_plot_width = needed_half * 2.0;
        if needed_plot_width > computed.plot_width() {
            computed.width = needed_plot_width + computed.margin_left + computed.margin_right;
        }
    }

    let mut scene = Scene::new(computed.width, computed.height);
    scene.font_family = computed.font_family.clone();
    apply_theme(&mut scene, &computed.theme);

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
    scene.font_family = computed.font_family.clone();
    apply_theme(&mut scene, &computed.theme);

    // add_axes_and_grid(&mut scene, &computed, &layout);
    add_labels_and_title(&mut scene, &computed, &layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);

    add_brickplot(brickplot, &mut scene, &computed);

    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);

    scene
}



pub fn render_waterfall(waterfall: &WaterfallPlot, layout: &Layout) -> Scene {
    let computed = ComputedLayout::from_layout(layout);
    let mut scene = Scene::new(computed.width, computed.height);
    scene.font_family = computed.font_family.clone();
    apply_theme(&mut scene, &computed.theme);
    add_axes_and_grid(&mut scene, &computed, layout);
    add_labels_and_title(&mut scene, &computed, layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);
    add_waterfall(waterfall, &mut scene, &computed);
    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);
    scene
}

// render_strip
pub fn render_strip(strip: &StripPlot, layout: &Layout) -> Scene {
    let computed = ComputedLayout::from_layout(layout);
    let mut scene = Scene::new(computed.width, computed.height);
    scene.font_family = computed.font_family.clone();
    apply_theme(&mut scene, &computed.theme);
    add_axes_and_grid(&mut scene, &computed, layout);
    add_labels_and_title(&mut scene, &computed, layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);
    add_strip(strip, &mut scene, &computed);
    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);
    scene
}

/// Collect legend entries from a slice of plots.
pub fn collect_legend_entries(plots: &[Plot]) -> Vec<LegendEntry> {
    let mut entries = Vec::new();
    for plot in plots {
        match plot {
            Plot::Bar(barplot) => {
                if let Some(label) = barplot.legend_label.clone() {
                    for (i, barval) in barplot.groups.first().unwrap().bars.iter().enumerate() {
                        entries.push(LegendEntry {
                            label: format!("{}", label.get(i).unwrap()),
                            color: barval.color.clone(),
                            shape: LegendShape::Rect,
                            dasharray: None,
                        });
                    }
                }
            }
            Plot::Line(line) => {
                if let Some(label) = &line.legend_label {
                    entries.push(LegendEntry {
                        label: label.clone(),
                        color: line.color.clone(),
                        shape: LegendShape::Line,
                        dasharray: line.line_style.dasharray(),
                    });
                }
            }
            Plot::Scatter(scatter) => {
                if let Some(label) = &scatter.legend_label {
                    entries.push(LegendEntry {
                        label: label.clone(),
                        color: scatter.color.clone(),
                        shape: LegendShape::Marker(scatter.marker),
                        dasharray: None,
                    });
                }
            }
            Plot::Series(series) => {
                if let Some(label) = &series.legend_label {
                    entries.push(LegendEntry {
                        label: label.clone(),
                        color: series.color.clone(),
                        shape: LegendShape::Circle,
                        dasharray: None,
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
                    entries.push(LegendEntry {
                        label,
                        color: color.clone(),
                        shape: LegendShape::Rect,
                        dasharray: None,
                    })
                }
            }
            Plot::Box(boxplot) => {
                if let Some(label) = &boxplot.legend_label {
                    entries.push(LegendEntry {
                        label: label.clone(),
                        color: boxplot.color.clone(),
                        shape: LegendShape::Rect,
                        dasharray: None,
                    });
                }
            }
            Plot::Violin(violin) => {
                if let Some(label) = &violin.legend_label {
                    entries.push(LegendEntry {
                        label: label.clone(),
                        color: violin.color.clone(),
                        shape: LegendShape::Rect,
                        dasharray: None,
                    });
                }
            }
            Plot::Histogram(hist) => {
                if let Some(label) = &hist.legend_label {
                    entries.push(LegendEntry {
                        label: label.clone(),
                        color: hist.color.clone(),
                        shape: LegendShape::Rect,
                        dasharray: None,
                    });
                }
            }
            Plot::Waterfall(wp) => {
                if let Some(ref label) = wp.legend_label {
                    entries.push(LegendEntry {
                        label: label.clone(),
                        color: wp.color_positive.clone(),
                        shape: LegendShape::Rect,
                        dasharray: None,
                    });
                }
            }
            Plot::Strip(sp) => {
                if let Some(ref label) = sp.legend_label {
                    entries.push(LegendEntry {
                        label: label.clone(),
                        color: sp.color.clone(),
                        shape: LegendShape::Circle,
                        dasharray: None,
                    });
                }
            }
            Plot::Heatmap(heatmap) => {
                if let Some(label) = &heatmap.legend_label {
                    entries.push(LegendEntry {
                        label: label.clone(),
                        color: "gray".into(),
                        shape: LegendShape::Rect,
                        dasharray: None,
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
                        entries.push(LegendEntry {
                            label,
                            color: slice.color.clone(),
                            shape: LegendShape::Rect,
                            dasharray: None,
                        });
                    }
                }
            }
            Plot::Volcano(vp) => {
                if vp.legend_label.is_some() {
                    entries.push(LegendEntry {
                        label: "Up".into(),
                        color: vp.color_up.clone(),
                        shape: LegendShape::Circle,
                        dasharray: None,
                    });
                    entries.push(LegendEntry {
                        label: "Down".into(),
                        color: vp.color_down.clone(),
                        shape: LegendShape::Circle,
                        dasharray: None,
                    });
                    entries.push(LegendEntry {
                        label: "NS".into(),
                        color: vp.color_ns.clone(),
                        shape: LegendShape::Circle,
                        dasharray: None,
                    });
                }
            }
            _ => {}
        }
    }
    entries
}

/// Render legend entries at an arbitrary (x, y) position on a scene.
pub fn render_legend_at(entries: &[LegendEntry], scene: &mut Scene, x: f64, y: f64, width: f64, body_size: u32, theme: &Theme) {
    let legend_padding = 10.0;
    let line_height = 18.0;
    let legend_height = entries.len() as f64 * line_height + legend_padding * 2.0;

    // Background
    scene.add(Primitive::Rect {
        x: x - legend_padding + 5.0,
        y: y - legend_padding,
        width,
        height: legend_height,
        fill: theme.legend_bg.clone(),
        stroke: None,
        stroke_width: None,
        opacity: None,
    });

    // Border
    scene.add(Primitive::Rect {
        x: x - legend_padding + 5.0,
        y: y - legend_padding,
        width,
        height: legend_height,
        fill: "none".into(),
        stroke: Some(theme.legend_border.clone()),
        stroke_width: Some(1.0),
        opacity: None,
    });

    let mut legend_y = y;
    for entry in entries {
        scene.add(Primitive::Text {
            x: x + 25.0,
            y: legend_y + 5.0,
            content: entry.label.clone(),
            anchor: TextAnchor::Start,
            size: body_size,
            rotate: None,
            bold: false,
        });
        match entry.shape {
            LegendShape::Rect => scene.add(Primitive::Rect {
                x: x + 5.0,
                y: legend_y - 5.0,
                width: 12.0,
                height: 12.0,
                fill: entry.color.clone(),
                stroke: None,
                stroke_width: None,
                opacity: None,
            }),
            LegendShape::Line => scene.add(Primitive::Line {
                x1: x + 5.0,
                y1: legend_y + 2.0,
                x2: x + 5.0 + 12.0,
                y2: legend_y + 2.0,
                stroke: entry.color.clone(),
                stroke_width: 2.0,
                stroke_dasharray: entry.dasharray.clone(),
            }),
            LegendShape::Circle => scene.add(Primitive::Circle {
                cx: x + 5.0 + 6.0,
                cy: legend_y + 1.0,
                r: 5.0,
                fill: entry.color.clone(),
            }),
            LegendShape::Marker(marker) => {
                draw_marker(scene, marker, x + 5.0 + 6.0, legend_y + 1.0, 5.0, &entry.color);
            }
        }
        legend_y += line_height;
    }
}

/// this should be the default renderer.
/// TODO: make an alias of this for single plots, that vectorises
pub fn render_multiple(plots: Vec<Plot>, layout: Layout) -> Scene {
    // Auto-assign palette colors to single-color plot types
    let mut plots = plots;
    if let Some(ref palette) = layout.palette {
        let mut color_idx = 0;
        for plot in plots.iter_mut() {
            match plot {
                Plot::Scatter(_) | Plot::Line(_) | Plot::Series(_) |
                Plot::Histogram(_) | Plot::Box(_) | Plot::Violin(_) |
                Plot::Band(_) | Plot::Strip(_) => {
                    plot.set_color(&palette[color_idx]);
                    color_idx += 1;
                }
                _ => {}
            }
        }
    }

    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);
    scene.font_family = computed.font_family.clone();
    apply_theme(&mut scene, &computed.theme);

    let all_pies = plots.iter().all(|p| matches!(p, Plot::Pie(_)));
    if !all_pies {
        add_axes_and_grid(&mut scene, &computed, &layout);
    }
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
            Plot::Waterfall(w) => {
                add_waterfall(&w, &mut scene, &computed);
            }
            Plot::Strip(s) => {
                add_strip(&s, &mut scene, &computed);
            }
            Plot::Volcano(v) => {
                add_volcano(v, &mut scene, &computed);
            }
        }
    }

    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);

    // create legend
    let entries = collect_legend_entries(&plots);
    if layout.show_legend && !entries.is_empty() {
        let legend = Legend { entries, position: layout.legend_position };
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

/// Render two groups of plots on a shared x-axis with independent left (primary) and right (secondary) y-axes.
pub fn render_twin_y(primary: Vec<Plot>, secondary: Vec<Plot>, layout: Layout) -> Scene {
    let mut primary = primary;
    let mut secondary = secondary;
    if let Some(ref palette) = layout.palette {
        let mut color_idx = 0;
        for plot in primary.iter_mut().chain(secondary.iter_mut()) {
            match plot {
                Plot::Scatter(_) | Plot::Line(_) | Plot::Series(_) |
                Plot::Histogram(_) | Plot::Box(_) | Plot::Violin(_) | Plot::Band(_) => {
                    plot.set_color(&palette[color_idx]);
                    color_idx += 1;
                }
                _ => {}
            }
        }
    }

    let computed = ComputedLayout::from_layout(&layout);
    let computed_y2 = computed.for_y2();
    let mut scene = Scene::new(computed.width, computed.height);
    scene.font_family = computed.font_family.clone();
    apply_theme(&mut scene, &computed.theme);

    add_axes_and_grid(&mut scene, &computed, &layout);
    add_y2_axis(&mut scene, &computed, &layout);
    add_labels_and_title(&mut scene, &computed, &layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);

    for plot in primary.iter() {
        match plot {
            Plot::Scatter(s) => add_scatter(s, &mut scene, &computed),
            Plot::Line(l)    => add_line(l, &mut scene, &computed),
            Plot::Series(s)  => add_series(s, &mut scene, &computed),
            Plot::Band(b)    => add_band(b, &mut scene, &computed),
            _ => {}
        }
    }
    for plot in secondary.iter() {
        match plot {
            Plot::Scatter(s) => add_scatter(s, &mut scene, &computed_y2),
            Plot::Line(l)    => add_line(l, &mut scene, &computed_y2),
            Plot::Series(s)  => add_series(s, &mut scene, &computed_y2),
            Plot::Band(b)    => add_band(b, &mut scene, &computed_y2),
            _ => {}
        }
    }

    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);

    let mut all_plots_for_legend: Vec<Plot> = primary;
    all_plots_for_legend.extend(secondary);
    let entries = collect_legend_entries(&all_plots_for_legend);
    if layout.show_legend && !entries.is_empty() {
        let legend = Legend { entries, position: layout.legend_position };
        add_legend(&legend, &mut scene, &computed);
    }

    scene
}
