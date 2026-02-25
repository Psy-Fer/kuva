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
use crate::plot::manhattan::ManhattanPlot;
use crate::plot::dotplot::DotPlot;
use crate::plot::upset::UpSetPlot;
use crate::plot::stacked_area::StackedAreaPlot;
use crate::plot::candlestick::{CandlestickPlot, CandleDataPoint};
use crate::plot::contour::ContourPlot;
use crate::plot::chord::ChordPlot;
use crate::plot::sankey::{SankeyPlot, SankeyLinkColor};
use crate::plot::phylo::{PhyloTree, TreeBranchStyle, TreeOrientation};
use crate::plot::synteny::{SyntenyPlot, Strand};


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
    /// Raw SVG strings to emit inside a `<defs>` block (e.g. linearGradients).
    pub defs: Vec<String>,
}

impl Scene {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width,
               height,
               background_color: Some("white".to_string()),
               text_color: None,
               font_family: None,
               elements: vec![],
               defs: vec![] }
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

    // Draw error bars
    for point in &line.data {
        // x error
        if let Some((neg, pos)) = point.x_err {
            let cy = computed.map_y(point.y);
            let cx_low  = computed.map_x(point.x - neg);
            let cx_high = computed.map_x(point.x + pos);

            scene.add(Primitive::Line {
                x1: cx_low, y1: cy, x2: cx_high, y2: cy,
                stroke: line.color.clone(), stroke_width: 1.0, stroke_dasharray: None,
            });
            scene.add(Primitive::Line {
                x1: cx_low, y1: cy - 5.0, x2: cx_low, y2: cy + 5.0,
                stroke: line.color.clone(), stroke_width: 1.0, stroke_dasharray: None,
            });
            scene.add(Primitive::Line {
                x1: cx_high, y1: cy - 5.0, x2: cx_high, y2: cy + 5.0,
                stroke: line.color.clone(), stroke_width: 1.0, stroke_dasharray: None,
            });
        }

        // y error
        if let Some((neg, pos)) = point.y_err {
            let cx = computed.map_x(point.x);
            let cy_low  = computed.map_y(point.y - neg);
            let cy_high = computed.map_y(point.y + pos);

            scene.add(Primitive::Line {
                x1: cx, y1: cy_low, x2: cx, y2: cy_high,
                stroke: line.color.clone(), stroke_width: 1.0, stroke_dasharray: None,
            });
            scene.add(Primitive::Line {
                x1: cx - 5.0, y1: cy_low, x2: cx + 5.0, y2: cy_low,
                stroke: line.color.clone(), stroke_width: 1.0, stroke_dasharray: None,
            });
            scene.add(Primitive::Line {
                x1: cx - 5.0, y1: cy_high, x2: cx + 5.0, y2: cy_high,
                stroke: line.color.clone(), stroke_width: 1.0, stroke_dasharray: None,
            });
        }
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
    let y_bin_height = (hist2d.y_range.1 - hist2d.y_range.0) / hist2d.bins_y as f64;

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

            // Bounds are (0.5, cols+0.5) / (0.5, rows+0.5) so cell j spans
            // [j+0.5, j+1.5] in x and cell i spans [i+0.5, i+1.5] in y.
            let x0 = computed.map_x(j as f64 + 0.5);
            let x1 = computed.map_x(j as f64 + 1.5);
            let y0 = computed.map_y(i as f64 + 1.5);
            let y1 = computed.map_y(i as f64 + 0.5);
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
                let x0 = computed.map_x(j as f64 + 0.5);
                let x1 = computed.map_x(j as f64 + 1.5);
                let y0 = computed.map_y(i as f64 + 1.5);
                let y1 = computed.map_y(i as f64 + 0.5);
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
            WaterfallKind::Difference { from, to } => {
                let color = if to >= from {
                    waterfall.color_positive.clone()
                } else {
                    waterfall.color_negative.clone()
                };
                // Running total is intentionally unchanged.
                (from, to, color)
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
                WaterfallKind::Difference { from, to } => {
                    let diff = to - from;
                    let s = if diff >= 0.0 {
                        format!("+{:.2}", diff)
                    } else {
                        format!("{:.2}", diff)
                    };
                    let ly = if diff >= 0.0 {
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

        // For Difference bars the running total didn't change, so the next
        // connector should come from the `to` edge of this bar.
        prev_connection_y = Some(match bar.kind {
            WaterfallKind::Difference { to, .. } => computed.map_y(to),
            _ => computed.map_y(running),
        });
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
            LegendShape::CircleSize(r) => {
                let swatch_half = 8.0;
                let draw_r = r.min(swatch_half);
                scene.add(Primitive::Circle {
                    cx: legend_x + 5.0 + 6.0,
                    cy: legend_y + 1.0,
                    r: draw_r,
                    fill: entry.color.clone(),
                });
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

fn add_manhattan(mp: &ManhattanPlot, scene: &mut Scene, computed: &ComputedLayout) {
    let floor = mp.floor();
    let plot_left  = computed.margin_left;
    let plot_right = computed.width - computed.margin_right;
    let plot_top   = computed.margin_top;
    let plot_bottom = computed.height - computed.margin_bottom;

    // 1. Threshold lines (genome-wide and suggestive)
    let gw_y = mp.genome_wide;
    if gw_y >= computed.y_range.0 && gw_y <= computed.y_range.1 {
        let sy = computed.map_y(gw_y);
        scene.add(Primitive::Line {
            x1: plot_left, y1: sy, x2: plot_right, y2: sy,
            stroke: "#cc3333".into(),
            stroke_width: 1.0,
            stroke_dasharray: Some("4 4".into()),
        });
    }
    let sg_y = mp.suggestive;
    if sg_y >= computed.y_range.0 && sg_y <= computed.y_range.1 {
        let sy = computed.map_y(sg_y);
        scene.add(Primitive::Line {
            x1: plot_left, y1: sy, x2: plot_right, y2: sy,
            stroke: "#888888".into(),
            stroke_width: 1.0,
            stroke_dasharray: Some("4 4".into()),
        });
    }

    // 2. Chromosome divider lines (faint verticals between chromosomes)
    for span in mp.spans.iter().skip(1) {
        let sx = computed.map_x(span.x_start);
        if sx >= plot_left && sx <= plot_right {
            scene.add(Primitive::Line {
                x1: sx, y1: plot_top, x2: sx, y2: plot_bottom,
                stroke: computed.theme.grid_color.clone(),
                stroke_width: 0.5,
                stroke_dasharray: None,
            });
        }
    }

    // 3. Draw points, one chromosome at a time for correct coloring.
    // Points are clamped to their chromosome's pixel band so they don't bleed over dividers.
    for (span_idx, span) in mp.spans.iter().enumerate() {
        let color = if let Some(ref pal) = mp.palette {
            pal[span_idx].to_string()
        } else if span_idx % 2 == 0 {
            mp.color_a.clone()
        } else {
            mp.color_b.clone()
        };
        let band_left  = computed.map_x(span.x_start).max(plot_left);
        let band_right = computed.map_x(span.x_end).min(plot_right);
        for p in mp.points.iter().filter(|p| p.chromosome == span.name) {
            let y_val = -(p.pvalue.max(floor)).log10();
            let cx = computed.map_x(p.x).clamp(band_left, band_right);
            let cy = computed.map_y(y_val);
            scene.add(Primitive::Circle { cx, cy, r: mp.point_size, fill: color.clone() });
        }
    }

    // 4. Chromosome labels in the bottom margin.
    // Skip bands that are too narrow to fit a label (e.g., MT at genome scale is ~0 px).
    let label_y = computed.height - computed.margin_bottom + 5.0 + computed.tick_size as f64;
    let min_label_px = 6.0_f64; // below this the band is invisible; anything above gets a label
    for span in &mp.spans {
        let band_px = (computed.map_x(span.x_end) - computed.map_x(span.x_start)).abs();
        let mid_x = computed.map_x((span.x_start + span.x_end) / 2.0);
        if mid_x >= plot_left && mid_x <= plot_right && band_px >= min_label_px {
            scene.add(Primitive::Text {
                x: mid_x,
                y: label_y,
                content: span.name.clone(),
                size: computed.tick_size,
                anchor: TextAnchor::Middle,
                rotate: None,
                bold: false,
            });
        }
    }

    // 5. Top-hit labels
    if mp.label_top == 0 {
        return;
    }

    // Collect points above genome-wide threshold, sort by screen y ascending (most significant first)
    let mut sig_points: Vec<(f64, f64, String)> = mp.points.iter()
        .filter(|p| -(p.pvalue.max(floor)).log10() >= mp.genome_wide)
        .map(|p| {
            let y_val = -(p.pvalue.max(floor)).log10();
            let label = p.label.clone().unwrap_or_else(|| p.chromosome.clone());
            (computed.map_x(p.x), computed.map_y(y_val), label)
        })
        .collect();
    sig_points.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    sig_points.truncate(mp.label_top);

    match mp.label_style {
        LabelStyle::Exact => {
            for (cx, cy, name) in &sig_points {
                scene.add(Primitive::Text {
                    x: *cx,
                    y: cy - mp.point_size - 2.0,
                    content: name.clone(),
                    size: computed.body_size,
                    anchor: TextAnchor::Middle,
                    rotate: None,
                    bold: false,
                });
            }
        }
        LabelStyle::Nudge => {
            let mut labels: Vec<(f64, f64, String)> = sig_points.iter()
                .map(|(cx, cy, name)| (*cx, cy - mp.point_size - 2.0, name.clone()))
                .collect();
            labels.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            let min_gap = computed.body_size as f64 + 2.0;
            for j in 1..labels.len() {
                let prev_y = labels[j - 1].1;
                if (prev_y - labels[j].1).abs() < min_gap {
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
                let dx = cx - text_x;
                let dy = cy - text_y;
                let len = (dx * dx + dy * dy).sqrt();
                if len > mp.point_size + 3.0 {
                    let scale = (len - mp.point_size - 3.0) / len;
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
                    content: name.clone(),
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

pub fn render_manhattan(mp: &ManhattanPlot, layout: &Layout) -> Scene {
    let computed = ComputedLayout::from_layout(layout);
    let mut scene = Scene::new(computed.width, computed.height);
    scene.font_family = computed.font_family.clone();
    apply_theme(&mut scene, &computed.theme);
    add_axes_and_grid(&mut scene, &computed, layout);
    add_labels_and_title(&mut scene, &computed, layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);
    add_manhattan(mp, &mut scene, &computed);
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

fn add_dot_plot(dp: &DotPlot, scene: &mut Scene, computed: &ComputedLayout) {
    const EPSILON: f64 = f64::EPSILON;

    let (size_min, size_max) = dp.size_range.unwrap_or_else(|| dp.size_extent());
    let (color_min, color_max) = dp.color_range.unwrap_or_else(|| dp.color_extent());

    let n_x = dp.x_categories.len() as f64;
    let n_y = dp.y_categories.len() as f64;
    let n_y_usize = dp.y_categories.len();

    let cell_w = if n_x > 0.0 { computed.plot_width() / n_x } else { 1.0 };
    let cell_h = if n_y > 0.0 { computed.plot_height() / n_y } else { 1.0 };
    // Cap effective max radius so circles never bleed outside their grid cell
    let effective_max_r = dp.max_radius.min((cell_w.min(cell_h) / 2.0) * 0.9);

    for pt in &dp.points {
        let xi = dp.x_categories.iter().position(|c| c == &pt.x_cat);
        let yi = dp.y_categories.iter().position(|c| c == &pt.y_cat);

        let (xi, yi) = match (xi, yi) {
            (Some(xi), Some(yi)) => (xi, yi),
            _ => continue,
        };

        // y is REVERSED: y_cat[0] is rendered at the top (map_y maps larger values to top)
        let cx = computed.map_x(xi as f64 + 1.0);
        let cy = computed.map_y((n_y_usize - yi) as f64);

        let norm_size  = (pt.size  - size_min)  / (size_max  - size_min  + EPSILON);
        let norm_color = (pt.color - color_min) / (color_max - color_min + EPSILON);

        let r    = dp.min_radius + norm_size.clamp(0.0, 1.0) * (effective_max_r - dp.min_radius);
        let fill = dp.color_map.map(norm_color.clamp(0.0, 1.0));

        scene.add(Primitive::Circle { cx, cy, r, fill });
    }
}

/// Draw DotPlot size legend (top) and colorbar (bottom) stacked in the same right-margin column.
fn add_dot_stacked_legends(
    size_title: &str,
    size_entries: &[LegendEntry],
    info: &ColorBarInfo,
    scene: &mut Scene,
    computed: &ComputedLayout,
) {
    let theme = &computed.theme;
    let legend_x = computed.width - computed.margin_right + computed.y2_axis_width + 10.0;
    let legend_width = computed.legend_width;
    let line_height = 18.0;
    let legend_padding = 10.0;

    // --- Size legend (top) ---
    // Title text sits above the box; the box starts below the title baseline
    // so the background rect doesn't paint over the text.
    let title_y = computed.margin_top + computed.tick_size as f64;
    // box visual top = box_top - legend_padding; we want this >= title_y + 4
    let box_top = title_y + legend_padding + 4.0;
    let size_legend_height = size_entries.len() as f64 * line_height + legend_padding * 2.0;

    // Background (drawn before title so title text sits on top)
    scene.add(Primitive::Rect {
        x: legend_x - legend_padding + 5.0,
        y: box_top - legend_padding,
        width: legend_width,
        height: size_legend_height,
        fill: theme.legend_bg.clone(),
        stroke: None,
        stroke_width: None,
        opacity: None,
    });
    // Border
    scene.add(Primitive::Rect {
        x: legend_x - legend_padding + 5.0,
        y: box_top - legend_padding,
        width: legend_width,
        height: size_legend_height,
        fill: "none".into(),
        stroke: Some(theme.legend_border.clone()),
        stroke_width: Some(1.0),
        opacity: None,
    });
    // Title drawn after the rects so it paints on top of them
    scene.add(Primitive::Text {
        x: legend_x + legend_width * 0.5 - legend_padding,
        y: title_y,
        content: size_title.to_string(),
        size: computed.tick_size,
        anchor: TextAnchor::Middle,
        rotate: None,
        bold: false,
    });

    let mut legend_y = box_top;
    for entry in size_entries {
        scene.add(Primitive::Text {
            x: legend_x + 25.0,
            y: legend_y + 5.0,
            content: entry.label.clone(),
            size: computed.body_size,
            anchor: TextAnchor::Start,
            rotate: None,
            bold: false,
        });
        if let LegendShape::CircleSize(r) = entry.shape {
            scene.add(Primitive::Circle {
                cx: legend_x + 5.0 + 6.0,
                cy: legend_y + 1.0,
                r: r.min(8.0),
                fill: entry.color.clone(),
            });
        }
        legend_y += line_height;
    }

    // --- Colorbar (bottom) ---
    let gap = 15.0;
    let bar_x = legend_x;
    let bar_width = 20.0;
    let colorbar_top = box_top - legend_padding + size_legend_height + gap;

    // Colorbar label/title
    if let Some(ref label) = info.label {
        scene.add(Primitive::Text {
            x: bar_x + bar_width * 0.5,
            y: colorbar_top - 6.0,
            content: label.clone(),
            size: computed.tick_size,
            anchor: TextAnchor::Middle,
            rotate: None,
            bold: false,
        });
    }

    let bar_y = colorbar_top;
    let bar_height = (computed.height - computed.margin_bottom - bar_y - gap).max(50.0);

    // Gradient slices
    let num_slices = 50;
    let slice_height = bar_height / num_slices as f64;
    for i in 0..num_slices {
        let t = 1.0 - (i as f64 / (num_slices - 1) as f64);
        let value = info.min_value + t * (info.max_value - info.min_value);
        let color = (info.map_fn)(value);
        let y = bar_y + i as f64 * slice_height;
        scene.add(Primitive::Rect {
            x: bar_x,
            y,
            width: bar_width,
            height: slice_height + 0.5,
            fill: color,
            stroke: None,
            stroke_width: None,
            opacity: None,
        });
    }
    // Border
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
        if *tick < info.min_value || *tick > info.max_value { continue; }
        let frac = (tick - info.min_value) / range;
        let y = bar_y + bar_height - frac * bar_height;
        scene.add(Primitive::Line {
            x1: bar_x + bar_width,
            y1: y,
            x2: bar_x + bar_width + 4.0,
            y2: y,
            stroke: theme.colorbar_border.clone(),
            stroke_width: 1.0,
            stroke_dasharray: None,
        });
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
            Plot::Manhattan(mp) => {
                if mp.legend_label.is_some() {
                    entries.push(LegendEntry {
                        label: "Genome-wide".into(),
                        color: "#cc3333".into(),
                        shape: LegendShape::Line,
                        dasharray: Some("4 4".into()),
                    });
                    entries.push(LegendEntry {
                        label: "Suggestive".into(),
                        color: "#888888".into(),
                        shape: LegendShape::Line,
                        dasharray: Some("4 4".into()),
                    });
                }
            }
            Plot::DotPlot(dp) => {
                if dp.size_label.is_some() {
                    let (size_min, size_max) = dp.size_range.unwrap_or_else(|| dp.size_extent());
                    for &pct in &[0.25_f64, 0.50, 0.75, 1.0] {
                        let value_at_pct = size_min + pct * (size_max - size_min);
                        let radius_at_pct = dp.max_radius * pct;
                        entries.push(LegendEntry {
                            label: format!("{:.1}", value_at_pct),
                            color: "#444444".into(),
                            shape: LegendShape::CircleSize(radius_at_pct),
                            dasharray: None,
                        });
                    }
                }
            }
            Plot::StackedArea(sa) => {
                for k in 0..sa.series.len() {
                    if let Some(Some(ref label)) = sa.labels.get(k) {
                        entries.push(LegendEntry {
                            label: label.clone(),
                            color: sa.resolve_color(k).to_string(),
                            shape: LegendShape::Rect,
                            dasharray: None,
                        });
                    }
                }
            }
            Plot::Candlestick(cp) => {
                if let Some(ref label) = cp.legend_label {
                    entries.push(LegendEntry {
                        label: label.clone(),
                        color: cp.color_up.clone(),
                        shape: LegendShape::Rect,
                        dasharray: None,
                    });
                }
            }
            Plot::Chord(chord) => {
                if chord.legend_label.is_some() {
                    use crate::render::palette::Palette;
                    let fallback = Palette::category10();
                    let n = chord.n_nodes();
                    for i in 0..n {
                        let color = if let Some(c) = chord.colors.get(i) {
                            if !c.is_empty() { c.clone() } else { fallback[i % fallback.len()].to_string() }
                        } else {
                            fallback[i % fallback.len()].to_string()
                        };
                        let label = if let Some(l) = chord.labels.get(i) { l.clone() } else { format!("{i}") };
                        entries.push(LegendEntry {
                            label,
                            color,
                            shape: LegendShape::Rect,
                            dasharray: None,
                        });
                    }
                }
            }
            Plot::Sankey(s) => {
                if s.legend_label.is_some() {
                    use crate::render::palette::Palette;
                    let fallback = Palette::category10();
                    for (i, node) in s.nodes.iter().enumerate() {
                        let color = node.color.clone()
                            .unwrap_or_else(|| fallback[i % fallback.len()].to_string());
                        entries.push(LegendEntry {
                            label: node.label.clone(),
                            color,
                            shape: LegendShape::Rect,
                            dasharray: None,
                        });
                    }
                }
            }
            Plot::Contour(cp) => {
                if let Some(ref label) = cp.legend_label {
                    if !cp.filled {
                        let line_color = cp.line_color.clone()
                            .unwrap_or_else(|| cp.color_map.map(0.5));
                        entries.push(LegendEntry {
                            label: label.clone(),
                            color: line_color,
                            shape: LegendShape::Line,
                            dasharray: None,
                        });
                    }
                }
            }
            Plot::PhyloTree(t) => {
                if t.legend_label.is_some() {
                    for (node_id, color) in &t.clade_colors {
                        let label = t.nodes[*node_id].label.clone()
                            .unwrap_or_else(|| format!("Node {}", node_id));
                        entries.push(LegendEntry {
                            label,
                            color: color.clone(),
                            shape: LegendShape::Line,
                            dasharray: None,
                        });
                    }
                }
            }
            Plot::Synteny(sp) => {
                if sp.legend_label.is_some() {
                    use crate::render::palette::Palette;
                    let fallback = Palette::category10();
                    for (i, seq) in sp.sequences.iter().enumerate() {
                        let color = seq.color.clone()
                            .unwrap_or_else(|| fallback[i % fallback.len()].to_string());
                        entries.push(LegendEntry {
                            label: seq.label.clone(),
                            color,
                            shape: LegendShape::Rect,
                            dasharray: None,
                        });
                    }
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
            LegendShape::CircleSize(r) => {
                let swatch_half = 8.0;
                let draw_r = r.min(swatch_half);
                scene.add(Primitive::Circle {
                    cx: x + 5.0 + 6.0,
                    cy: legend_y + 1.0,
                    r: draw_r,
                    fill: entry.color.clone(),
                });
            }
        }
        legend_y += line_height;
    }
}

fn add_upset(up: &UpSetPlot, scene: &mut Scene, computed: &ComputedLayout) {
    if up.set_names.is_empty() {
        return;
    }
    let sorted = up.sorted_intersections();
    if sorted.is_empty() {
        return;
    }

    let n_sets = up.set_names.len();
    let n_cols = sorted.len();

    let theme = &computed.theme;
    let pl = computed.margin_left;
    let pr = computed.width - computed.margin_right;
    let pt = computed.margin_top;
    let pb = computed.height - computed.margin_bottom;
    let pw = pr - pl;
    let ph = pb - pt;

    let tick_size = computed.tick_size as f64;
    let label_size = computed.label_size as f64;

    // Left panel layout (left → right): [bar_area][count_gap][name_area]
    let max_name_len = up.set_names.iter().map(|n| n.len()).max().unwrap_or(0);
    let name_area = (max_name_len as f64 * tick_size * 0.6 + 10.0).max(40.0).min(120.0);
    let bar_area = if up.show_set_sizes {
        (pw * 0.18).max(50.0).min(150.0)
    } else {
        0.0
    };
    // Reserve a fixed zone for count labels so they never overlap the set names.
    let count_gap = if up.show_counts && up.show_set_sizes { 28.0 } else { 0.0 };
    let left_panel_w = bar_area + count_gap + name_area;

    // Top panel: intersection size bars (55 % of plot height).
    let inter_bar_h = ph * 0.55;

    // Dot-matrix region.
    let mat_l = pl + left_panel_w;
    let mat_t = pt + inter_bar_h;
    let mat_r = pr;
    let mat_b = pb;

    let dot_col_w = if n_cols > 0 { (mat_r - mat_l) / n_cols as f64 } else { 1.0 };
    let dot_row_h = if n_sets > 0 { (mat_b - mat_t) / n_sets as f64 } else { 1.0 };
    let dot_r = (dot_col_w.min(dot_row_h) * 0.35).max(3.0).min(12.0);
    let bar_half_w = (dot_col_w * 0.3).max(3.0);

    let max_inter = sorted.iter().map(|i| i.count).max().unwrap_or(1) as f64;
    let max_set = up.set_sizes.iter().copied().max().unwrap_or(1) as f64;

    // ── Set-size bars (left panel, left portion) ────────────────────────────
    if up.show_set_sizes {
        let bar_x_start = pl;
        let bar_x_end = pl + bar_area;
        let bar_half_h = (dot_row_h * 0.25).max(3.0).min(12.0);

        // Axis line (right edge of bar area).
        scene.add(Primitive::Line {
            x1: bar_x_end, y1: mat_t,
            x2: bar_x_end, y2: mat_b,
            stroke: theme.axis_color.clone(), stroke_width: 1.0, stroke_dasharray: None,
        });
        // Baseline.
        scene.add(Primitive::Line {
            x1: bar_x_start, y1: mat_b,
            x2: bar_x_end, y2: mat_b,
            stroke: theme.axis_color.clone(), stroke_width: 1.0, stroke_dasharray: None,
        });

        for (j, &size) in up.set_sizes.iter().enumerate() {
            let cy = mat_t + (j as f64 + 0.5) * dot_row_h;
            let bar_w = size as f64 / max_set * bar_area;

            // Bars grow leftward from the right edge (zero baseline at bar_x_end).
            scene.add(Primitive::Rect {
                x: bar_x_end - bar_w,
                y: cy - bar_half_h,
                width: bar_w,
                height: bar_half_h * 2.0,
                fill: up.bar_color.clone(),
                stroke: None, stroke_width: None, opacity: None,
            });

            if up.show_counts {
                // Fixed position in the count_gap zone — never encroaches on name_area.
                scene.add(Primitive::Text {
                    x: pl + bar_area + 3.0,
                    y: cy + tick_size * 0.35,
                    content: format!("{}", size),
                    size: computed.tick_size,
                    anchor: TextAnchor::Start, rotate: None, bold: false,
                });
            }
        }

        // "Set size" axis label.
        scene.add(Primitive::Text {
            x: bar_x_start + bar_area / 2.0,
            y: mat_t - tick_size - 4.0,
            content: "Set size".to_string(),
            size: computed.label_size,
            anchor: TextAnchor::Middle, rotate: None, bold: false,
        });
    }

    // ── Set names (right portion of left panel) ──────────────────────────────
    let name_x = mat_l - 5.0; // right-aligned just before dot matrix
    for (j, name) in up.set_names.iter().enumerate() {
        let cy = mat_t + (j as f64 + 0.5) * dot_row_h;
        scene.add(Primitive::Text {
            x: name_x,
            y: cy + tick_size * 0.35,
            content: name.clone(),
            size: computed.tick_size,
            anchor: TextAnchor::End, rotate: None, bold: false,
        });
    }

    // ── Intersection-size bars (top panel) ───────────────────────────────────
    let bar_y_max = pt + inter_bar_h - 5.0; // baseline
    let bar_y_min = pt + tick_size + 2.0;   // top of tallest bar
    let bar_h_range = (bar_y_max - bar_y_min).max(1.0);

    // Left axis line for intersection bars.
    scene.add(Primitive::Line {
        x1: mat_l, y1: bar_y_min,
        x2: mat_l, y2: bar_y_max,
        stroke: theme.axis_color.clone(), stroke_width: 1.0, stroke_dasharray: None,
    });
    // Baseline.
    scene.add(Primitive::Line {
        x1: mat_l, y1: bar_y_max,
        x2: mat_r, y2: bar_y_max,
        stroke: theme.axis_color.clone(), stroke_width: 1.0, stroke_dasharray: None,
    });

    // Y-axis ticks for intersection bars.
    let n_yticks = 4;
    for ti in 0..=n_yticks {
        let frac = ti as f64 / n_yticks as f64;
        let val = (max_inter * frac).round() as usize;
        let y = bar_y_max - frac * bar_h_range;

        scene.add(Primitive::Line {
            x1: mat_l - 4.0, y1: y,
            x2: mat_l, y2: y,
            stroke: theme.tick_color.clone(), stroke_width: 1.0, stroke_dasharray: None,
        });
        scene.add(Primitive::Text {
            x: mat_l - 7.0,
            y: y + tick_size * 0.35,
            content: format!("{}", val),
            size: computed.tick_size,
            anchor: TextAnchor::End, rotate: None, bold: false,
        });
    }

    // "Intersection size" Y-axis label (rotated, left of the tick labels).
    scene.add(Primitive::Text {
        x: mat_l - 7.0 - tick_size * 2.5 - label_size * 0.5,
        y: (bar_y_min + bar_y_max) / 2.0,
        content: "Intersection size".to_string(),
        size: computed.label_size,
        anchor: TextAnchor::Middle,
        rotate: Some(-90.0),
        bold: false,
    });

    // Intersection bars.
    for (i, inter) in sorted.iter().enumerate() {
        let cx = mat_l + (i as f64 + 0.5) * dot_col_w;
        let bar_h = (inter.count as f64 / max_inter * bar_h_range).max(0.0);
        let bar_x = cx - bar_half_w;
        let bar_y = bar_y_max - bar_h;

        scene.add(Primitive::Rect {
            x: bar_x, y: bar_y,
            width: bar_half_w * 2.0, height: bar_h,
            fill: up.bar_color.clone(),
            stroke: None, stroke_width: None, opacity: None,
        });

        if up.show_counts && bar_h > 0.0 {
            scene.add(Primitive::Text {
                x: cx,
                y: bar_y - 2.0,
                content: format!("{}", inter.count),
                size: computed.tick_size,
                anchor: TextAnchor::Middle, rotate: None, bold: false,
            });
        }
    }

    // ── Dot matrix ───────────────────────────────────────────────────────────
    // Light horizontal separator lines.
    for j in 0..=n_sets {
        let y = mat_t + j as f64 * dot_row_h;
        scene.add(Primitive::Line {
            x1: mat_l, y1: y,
            x2: mat_r, y2: y,
            stroke: theme.grid_color.clone(), stroke_width: 0.5, stroke_dasharray: None,
        });
    }

    for (i, inter) in sorted.iter().enumerate() {
        let cx = mat_l + (i as f64 + 0.5) * dot_col_w;

        let filled_rows: Vec<usize> = (0..n_sets)
            .filter(|&j| inter.mask & (1u64 << j) != 0)
            .collect();

        // Connector line between the topmost and bottommost filled dots.
        if filled_rows.len() >= 2 {
            let top_j = *filled_rows.first().unwrap();
            let bot_j = *filled_rows.last().unwrap();
            let top_cy = mat_t + (top_j as f64 + 0.5) * dot_row_h;
            let bot_cy = mat_t + (bot_j as f64 + 0.5) * dot_row_h;
            scene.add(Primitive::Line {
                x1: cx, y1: top_cy,
                x2: cx, y2: bot_cy,
                stroke: up.dot_color.clone(),
                stroke_width: (dot_r * 0.5).max(2.0),
                stroke_dasharray: None,
            });
        }

        for j in 0..n_sets {
            let cy = mat_t + (j as f64 + 0.5) * dot_row_h;
            let filled = inter.mask & (1u64 << j) != 0;
            let fill = if filled {
                up.dot_color.clone()
            } else {
                up.dot_empty_color.clone()
            };
            scene.add(Primitive::Circle { cx, cy, r: dot_r, fill });
        }
    }
}

fn add_stacked_area(sa: &StackedAreaPlot, scene: &mut Scene, computed: &ComputedLayout) {
    if sa.x.is_empty() || sa.series.is_empty() { return; }
    let n = sa.x.len();

    // Precompute per-column totals for normalisation
    let totals: Vec<f64> = if sa.normalized {
        (0..n).map(|i| {
            sa.series.iter().map(|s| s.get(i).copied().unwrap_or(0.0)).sum::<f64>()
        }).collect()
    } else {
        vec![1.0; n]
    };

    let scale = if sa.normalized { 100.0 } else { 1.0 };

    // cumulative baseline per column (grows as we draw each series)
    let mut lower: Vec<f64> = vec![0.0; n];

    for k in 0..sa.series.len() {
        let series = &sa.series[k];
        let color = sa.resolve_color(k).to_string();

        // Compute upper edge for this series
        let upper: Vec<f64> = (0..n).map(|i| {
            let raw = series.get(i).copied().unwrap_or(0.0);
            let t = totals[i].max(f64::EPSILON);
            lower[i] + raw / t * scale
        }).collect();

        // Build closed SVG path: forward along upper, backward along lower
        let mut path = String::new();
        for (i, &x) in sa.x.iter().enumerate() {
            let sx = computed.map_x(x);
            let sy = computed.map_y(upper[i]);
            if i == 0 {
                path += &format!("M {sx} {sy} ");
            } else {
                path += &format!("L {sx} {sy} ");
            }
        }
        for i in (0..n).rev() {
            let sx = computed.map_x(sa.x[i]);
            let sy = computed.map_y(lower[i]);
            path += &format!("L {sx} {sy} ");
        }
        path += "Z";

        scene.add(Primitive::Path {
            d: path,
            fill: Some(color.clone()),
            stroke: "none".into(),
            stroke_width: 0.0,
            opacity: Some(sa.fill_opacity),
            stroke_dasharray: None,
        });

        // Optional stroke along the top edge
        if sa.show_strokes {
            let mut stroke_path = String::new();
            for (i, &x) in sa.x.iter().enumerate() {
                let sx = computed.map_x(x);
                let sy = computed.map_y(upper[i]);
                if i == 0 {
                    stroke_path += &format!("M {sx} {sy} ");
                } else {
                    stroke_path += &format!("L {sx} {sy} ");
                }
            }
            scene.add(Primitive::Path {
                d: stroke_path,
                fill: None,
                stroke: color,
                stroke_width: sa.stroke_width,
                opacity: None,
                stroke_dasharray: None,
            });
        }

        // Advance lower to current upper for the next series
        for i in 0..n {
            lower[i] = upper[i];
        }
    }
}

fn add_candlestick(cp: &CandlestickPlot, scene: &mut Scene, computed: &ComputedLayout) {
    if cp.candles.is_empty() { return; }

    let continuous = cp.candles.iter().any(|c| c.x.is_some());
    let n = cp.candles.len();

    // Compute slot pixel width for body sizing
    let slot_px = if continuous {
        if n > 1 {
            // Use average spacing between consecutive candles
            let xs: Vec<f64> = cp.candles.iter().filter_map(|c| c.x).collect();
            if xs.len() > 1 {
                let span = xs[xs.len() - 1] - xs[0];
                let avg_spacing = span / (xs.len() - 1) as f64;
                computed.map_x(computed.x_range.0 + avg_spacing) - computed.map_x(computed.x_range.0)
            } else {
                computed.plot_width()
            }
        } else {
            computed.plot_width()
        }
    } else {
        // categorical: width of one slot = map_x(1.5) - map_x(0.5)
        computed.map_x(1.5) - computed.map_x(0.5)
    };
    let body_w = slot_px * cp.candle_width;

    // Price panel bottom pixel coordinate
    let price_bottom_px = if cp.show_volume {
        computed.margin_top + computed.plot_height() * (1.0 - cp.volume_ratio) - 4.0
    } else {
        computed.margin_top + computed.plot_height()
    };

    let y_min = computed.y_range.0;
    let y_max = computed.y_range.1;
    let map_y_price = |v: f64| -> f64 {
        let t = (y_max - v) / (y_max - y_min);
        computed.margin_top + t * (price_bottom_px - computed.margin_top)
    };

    let candle_color = |c: &CandleDataPoint| -> &str {
        if c.close > c.open { &cp.color_up }
        else if c.close < c.open { &cp.color_down }
        else { &cp.color_doji }
    };

    for (i, candle) in cp.candles.iter().enumerate() {
        let x_val = if continuous {
            candle.x.unwrap_or(i as f64 + 1.0)
        } else {
            i as f64 + 1.0
        };
        let x_center = computed.map_x(x_val);
        let color = candle_color(candle).to_string();

        // Wick
        scene.add(Primitive::Line {
            x1: x_center,
            y1: map_y_price(candle.high),
            x2: x_center,
            y2: map_y_price(candle.low),
            stroke: color.clone(),
            stroke_width: cp.wick_width,
            stroke_dasharray: None,
        });

        // Body
        let body_top    = map_y_price(candle.open.max(candle.close));
        let body_bottom = map_y_price(candle.open.min(candle.close));
        let body_h = (body_bottom - body_top).max(1.0);
        scene.add(Primitive::Rect {
            x: x_center - body_w / 2.0,
            y: body_top,
            width: body_w,
            height: body_h,
            fill: color.clone(),
            stroke: Some(color.clone()),
            stroke_width: Some(0.5),
            opacity: None,
        });
    }

    // Volume panel
    if cp.show_volume {
        let vol_panel_top    = price_bottom_px + 4.0;
        let vol_panel_bottom = computed.margin_top + computed.plot_height() - 2.0;
        let vol_panel_h      = vol_panel_bottom - vol_panel_top;

        let vol_max = cp.candles.iter()
            .filter_map(|c| c.volume)
            .fold(0.0_f64, f64::max);

        if vol_max > 0.0 {
            for (i, candle) in cp.candles.iter().enumerate() {
                if let Some(vol) = candle.volume {
                    let x_val = if continuous {
                        candle.x.unwrap_or(i as f64 + 1.0)
                    } else {
                        i as f64 + 1.0
                    };
                    let x_center = computed.map_x(x_val);
                    let color = candle_color(candle).to_string();
                    let bar_h = (vol / vol_max) * vol_panel_h;
                    scene.add(Primitive::Rect {
                        x: x_center - body_w / 2.0,
                        y: vol_panel_bottom - bar_h,
                        width: body_w,
                        height: bar_h,
                        fill: color,
                        stroke: None,
                        stroke_width: None,
                        opacity: Some(0.5),
                    });
                }
            }
        }
    }
}

// ── Contour / Marching-Squares ─────────────────────────────────────────────

/// Build the SVG path string for one iso-level using marching squares.
/// Emits individual "M x1 y1 L x2 y2" segments — no stitching needed.
///
/// Grid convention: z[row][col], x_coords[col], y_coords[row].
/// Cell (col, row) has corners TL=(col,row) TR=(col+1,row) BR=(col+1,row+1) BL=(col,row+1).
fn contour_path(
    z: &[Vec<f64>],
    x_coords: &[f64],
    y_coords: &[f64],
    t: f64,
    computed: &ComputedLayout,
) -> String {
    let rows = z.len();
    if rows < 2 { return String::new(); }
    let cols = z[0].len();
    if cols < 2 { return String::new(); }

    let mut d = String::new();

    // Crossing on the horizontal edge between corner (col, row) and (col+1, row).
    // x interpolates between x_coords[col] and x_coords[col+1]; y is fixed at y_coords[row].
    let h = |col: usize, row: usize| -> (f64, f64) {
        let va = z[row][col];
        let vb = z[row][col + 1];
        let frac = if (vb - va).abs() < 1e-12 { 0.5 } else { ((t - va) / (vb - va)).clamp(0.0, 1.0) };
        let wx = x_coords[col] + frac * (x_coords[col + 1] - x_coords[col]);
        (computed.map_x(wx), computed.map_y(y_coords[row]))
    };

    // Crossing on the vertical edge between corner (col, row) and (col, row+1).
    // x is fixed at x_coords[col]; y interpolates between y_coords[row] and y_coords[row+1].
    let v = |col: usize, row: usize| -> (f64, f64) {
        let va = z[row][col];
        let vb = z[row + 1][col];
        let frac = if (vb - va).abs() < 1e-12 { 0.5 } else { ((t - va) / (vb - va)).clamp(0.0, 1.0) };
        let wy = y_coords[row] + frac * (y_coords[row + 1] - y_coords[row]);
        (computed.map_x(x_coords[col]), computed.map_y(wy))
    };

    // Append one line segment to the path string.
    let mut seg = |p1: (f64, f64), p2: (f64, f64)| {
        d += &format!("M{:.2} {:.2} L{:.2} {:.2} ", p1.0, p1.1, p2.0, p2.1);
    };

    for row in 0..rows - 1 {
        for col in 0..cols - 1 {
            let tl = z[row][col];
            let tr = z[row][col + 1];
            let br = z[row + 1][col + 1];
            let bl = z[row + 1][col];

            // Marching squares case: TL=8, TR=4, BR=2, BL=1
            let case = ((tl >= t) as u8) * 8
                     + ((tr >= t) as u8) * 4
                     + ((br >= t) as u8) * 2
                     + ((bl >= t) as u8);

            // Canonical edge names for this cell:
            //   top    = h(col, row)       between TL and TR
            //   bottom = h(col, row+1)     between BL and BR
            //   left   = v(col, row)       between TL and BL
            //   right  = v(col+1, row)     between TR and BR
            let avg = (tl + tr + br + bl) / 4.0;

            match case {
                0 | 15 => {}
                1  => seg(v(col,   row), h(col, row + 1)),           // left → bottom
                2  => seg(h(col, row + 1), v(col + 1, row)),         // bottom → right
                3  => seg(v(col,   row), v(col + 1, row)),           // left → right
                4  => seg(h(col,   row), v(col + 1, row)),           // top → right
                5  => if avg >= t {
                        seg(h(col, row),     v(col,     row));        // top → left
                        seg(h(col, row + 1), v(col + 1, row));        // bottom → right
                    } else {
                        seg(h(col, row),     v(col + 1, row));        // top → right
                        seg(h(col, row + 1), v(col,     row));        // bottom → left
                    }
                6  => seg(h(col,   row), h(col, row + 1)),           // top → bottom
                7  => seg(h(col,   row), v(col,   row)),             // top → left
                8  => seg(h(col,   row), v(col,   row)),             // top → left
                9  => seg(h(col,   row), h(col, row + 1)),           // top → bottom
                10 => if avg >= t {
                        seg(h(col, row),     v(col + 1, row));        // top → right
                        seg(h(col, row + 1), v(col,     row));        // bottom → left
                    } else {
                        seg(h(col, row),     v(col,     row));        // top → left
                        seg(h(col, row + 1), v(col + 1, row));        // bottom → right
                    }
                11 => seg(h(col,   row), v(col + 1, row)),           // top → right
                12 => seg(v(col,   row), v(col + 1, row)),           // left → right
                13 => seg(v(col + 1, row), h(col, row + 1)),         // right → bottom
                14 => seg(v(col,   row), h(col, row + 1)),           // left → bottom
                _  => {}
            }
        }
    }

    d
}

/// Build the SVG path string for the filled region {z >= t}, one closed polygon per cell piece.
#[allow(non_snake_case)]
/// Uses the per-cell marching-squares polygon table so boundaries follow the iso-line exactly.
fn contour_fill_path(
    z: &[Vec<f64>],
    x_coords: &[f64],
    y_coords: &[f64],
    t: f64,
    computed: &ComputedLayout,
) -> String {
    let rows = z.len();
    if rows < 2 { return String::new(); }
    let cols = z[0].len();
    if cols < 2 { return String::new(); }

    let mut d = String::new();

    // Horizontal edge crossing between corner (col, row) and (col+1, row).
    let h = |col: usize, row: usize| -> (f64, f64) {
        let va = z[row][col];
        let vb = z[row][col + 1];
        let frac = if (vb - va).abs() < 1e-12 { 0.5 } else { ((t - va) / (vb - va)).clamp(0.0, 1.0) };
        let wx = x_coords[col] + frac * (x_coords[col + 1] - x_coords[col]);
        (computed.map_x(wx), computed.map_y(y_coords[row]))
    };

    // Vertical edge crossing between corner (col, row) and (col, row+1).
    let v = |col: usize, row: usize| -> (f64, f64) {
        let va = z[row][col];
        let vb = z[row + 1][col];
        let frac = if (vb - va).abs() < 1e-12 { 0.5 } else { ((t - va) / (vb - va)).clamp(0.0, 1.0) };
        let wy = y_coords[row] + frac * (y_coords[row + 1] - y_coords[row]);
        (computed.map_x(x_coords[col]), computed.map_y(wy))
    };

    // Append one closed polygon subpath.
    let mut poly = |verts: &[(f64, f64)]| {
        if verts.len() < 3 { return; }
        d += &format!("M{:.2} {:.2}", verts[0].0, verts[0].1);
        for &(x, y) in &verts[1..] {
            d += &format!(" L{:.2} {:.2}", x, y);
        }
        d += " Z ";
    };

    for row in 0..rows - 1 {
        for col in 0..cols - 1 {
            let tl = z[row][col];
            let tr = z[row][col + 1];
            let br = z[row + 1][col + 1];
            let bl = z[row + 1][col];

            // Marching squares case: TL=8, TR=4, BR=2, BL=1.
            let case = ((tl >= t) as u8) * 8
                     + ((tr >= t) as u8) * 4
                     + ((br >= t) as u8) * 2
                     + ((bl >= t) as u8);

            // Pixel corners.
            let tl_p = (computed.map_x(x_coords[col]),     computed.map_y(y_coords[row]));
            let tr_p = (computed.map_x(x_coords[col + 1]), computed.map_y(y_coords[row]));
            let br_p = (computed.map_x(x_coords[col + 1]), computed.map_y(y_coords[row + 1]));
            let bl_p = (computed.map_x(x_coords[col]),     computed.map_y(y_coords[row + 1]));

            // Edge crossings.
            //   T = top edge   h(col,   row)   between TL and TR
            //   B = bot edge   h(col,   row+1) between BL and BR
            //   L = left edge  v(col,   row)   between TL and BL
            //   R = right edge v(col+1, row)   between TR and BR
            let avg = (tl + tr + br + bl) / 4.0;

            match case {
                0  => {}
                // One corner above — triangle.
                1  => { let (L, B) = (v(col, row),   h(col, row + 1)); poly(&[bl_p, L, B]); }
                2  => { let (B, R) = (h(col, row+1), v(col+1, row));   poly(&[br_p, B, R]); }
                4  => { let (T, R) = (h(col, row),   v(col+1, row));   poly(&[tr_p, T, R]); }
                8  => { let (T, L) = (h(col, row),   v(col, row));     poly(&[tl_p, T, L]); }
                // Two adjacent corners above — trapezoid/quad.
                3  => { let (L, R) = (v(col, row),   v(col+1, row));   poly(&[bl_p, br_p, R, L]); }
                6  => { let (T, B) = (h(col, row),   h(col, row+1));   poly(&[T, tr_p, br_p, B]); }
                9  => { let (T, B) = (h(col, row),   h(col, row+1));   poly(&[tl_p, T, B, bl_p]); }
                12 => { let (L, R) = (v(col, row),   v(col+1, row));   poly(&[tl_p, tr_p, R, L]); }
                // Two diagonal corners above — saddle: resolve with cell average.
                5  => {
                    let (T, B, L, R) = (h(col, row), h(col, row+1), v(col, row), v(col+1, row));
                    if avg >= t {
                        // Connected region (hourglass through centre): one hexagon.
                        poly(&[bl_p, L, T, tr_p, R, B]);
                    } else {
                        // Two separate triangles.
                        poly(&[bl_p, L, B]);
                        poly(&[tr_p, T, R]);
                    }
                }
                10 => {
                    let (T, B, L, R) = (h(col, row), h(col, row+1), v(col, row), v(col+1, row));
                    if avg >= t {
                        poly(&[tl_p, T, R, br_p, B, L]);
                    } else {
                        poly(&[tl_p, T, L]);
                        poly(&[br_p, B, R]);
                    }
                }
                // Three corners above — pentagon wrapping the single below corner.
                7  => { let (T, L) = (h(col, row),   v(col, row));     poly(&[T, tr_p, br_p, bl_p, L]); }
                11 => { let (T, R) = (h(col, row),   v(col+1, row));   poly(&[T, tl_p, bl_p, br_p, R]); }
                13 => { let (R, B) = (v(col+1, row), h(col, row+1));   poly(&[tl_p, tr_p, R, B, bl_p]); }
                14 => { let (L, B) = (v(col, row),   h(col, row+1));   poly(&[L, tl_p, tr_p, br_p, B]); }
                // All four above — full cell.
                15 => { poly(&[tl_p, tr_p, br_p, bl_p]); }
                _  => {}
            }
        }
    }

    d
}

fn add_contour(cp: &ContourPlot, scene: &mut Scene, computed: &ComputedLayout) {
    if cp.z.is_empty() || cp.x_coords.len() < 2 || cp.y_coords.len() < 2 { return; }

    let levels = cp.effective_levels();
    if levels.is_empty() { return; }

    let (z_min, z_max) = cp.z_range();
    let z_span = z_max - z_min + f64::EPSILON;

    let level_color = |level: f64| -> String {
        let norm = (level - z_min) / z_span;
        cp.color_map.map(norm.clamp(0.0, 1.0))
    };

    if cp.filled {
        // Painter's algorithm with per-cell marching-squares polygons.
        // 1. Fill the entire grid extent with the minimum colormap colour (base layer).
        let x0_d = cp.x_coords.iter().cloned().fold(f64::INFINITY, f64::min);
        let x1_d = cp.x_coords.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let y0_d = cp.y_coords.iter().cloned().fold(f64::INFINITY, f64::min);
        let y1_d = cp.y_coords.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let px0 = computed.map_x(x0_d).min(computed.map_x(x1_d));
        let px1 = computed.map_x(x0_d).max(computed.map_x(x1_d));
        let py0 = computed.map_y(y0_d).min(computed.map_y(y1_d));
        let py1 = computed.map_y(y0_d).max(computed.map_y(y1_d));
        scene.add(Primitive::Rect {
            x: px0, y: py0, width: px1 - px0, height: py1 - py0,
            fill: cp.color_map.map(0.0),
            stroke: None, stroke_width: None, opacity: None,
        });

        // 2. For each level (lowest → highest), fill the {z ≥ level} region.
        //    Higher levels overdraw lower ones, leaving the correct colour per band.
        for &lvl in &levels {
            let color = level_color(lvl);
            let d = contour_fill_path(&cp.z, &cp.x_coords, &cp.y_coords, lvl, computed);
            if d.is_empty() { continue; }
            scene.add(Primitive::Path {
                d,
                fill: Some(color),
                stroke: "none".into(),
                stroke_width: 0.0,
                opacity: None,
                stroke_dasharray: None,
            });
        }

        // 3. Draw iso-lines on top.
        for &lvl in &levels {
            let stroke = cp.line_color.clone().unwrap_or_else(|| "black".to_string());
            let d = contour_path(&cp.z, &cp.x_coords, &cp.y_coords, lvl, computed);
            if d.is_empty() { continue; }
            scene.add(Primitive::Path {
                d,
                fill: None,
                stroke,
                stroke_width: cp.line_width,
                opacity: None,
                stroke_dasharray: None,
            });
        }
    } else {
        // Lines-only mode: one path per level, colored by the colormap (or fixed line_color).
        for &lvl in &levels {
            let stroke = cp.line_color.clone().unwrap_or_else(|| level_color(lvl));
            let d = contour_path(&cp.z, &cp.x_coords, &cp.y_coords, lvl, computed);
            if d.is_empty() { continue; }
            scene.add(Primitive::Path {
                d,
                fill: None,
                stroke,
                stroke_width: cp.line_width,
                opacity: None,
                stroke_dasharray: None,
            });
        }
    }
}

fn add_chord(chord: &ChordPlot, scene: &mut Scene, computed: &ComputedLayout) {
    use std::f64::consts::TAU;
    use crate::render::palette::Palette;

    let n = chord.n_nodes();
    if n == 0 { return; }

    // Fallback palette
    let fallback = Palette::category10();
    let node_color = |i: usize| -> String {
        if let Some(c) = chord.colors.get(i) {
            if !c.is_empty() { return c.clone(); }
        }
        fallback[i % fallback.len()].to_string()
    };

    // Geometry
    let label_margin = computed.body_size as f64 * 2.5;
    let outer_r = (computed.plot_width().min(computed.plot_height()) / 2.0 - label_margin).max(10.0);
    let inner_r = outer_r * chord.pad_fraction;
    let cx = computed.margin_left + computed.plot_width() / 2.0;
    let cy = computed.margin_top + computed.plot_height() / 2.0;

    // Row totals and grand total
    let row_total: Vec<f64> = chord.matrix.iter().map(|row| row.iter().sum()).collect();
    let grand_total: f64 = row_total.iter().sum();
    if grand_total <= 0.0 { return; }

    let gap_rad = chord.gap_degrees.to_radians();
    let usable = TAU - n as f64 * gap_rad;

    // Start angle for each node arc
    let mut node_start = Vec::with_capacity(n);
    let mut node_span = Vec::with_capacity(n);
    let mut angle = -std::f64::consts::FRAC_PI_2; // start at top
    for i in 0..n {
        node_start.push(angle);
        let span = if grand_total > 0.0 { (row_total[i] / grand_total) * usable } else { usable / n as f64 };
        node_span.push(span);
        angle += span + gap_rad;
    }

    // Helper: SVG arc flag
    let la = |sweep: f64| if sweep > std::f64::consts::PI { 1 } else { 0 };

    // ── Draw outer arc segments (donut slices) ──
    for i in 0..n {
        let a0 = node_start[i];
        let a1 = a0 + node_span[i];
        let x1o = cx + outer_r * a0.cos();
        let y1o = cy + outer_r * a0.sin();
        let x2o = cx + outer_r * a1.cos();
        let y2o = cy + outer_r * a1.sin();
        let x2i = cx + inner_r * a1.cos();
        let y2i = cy + inner_r * a1.sin();
        let x1i = cx + inner_r * a0.cos();
        let y1i = cy + inner_r * a0.sin();
        let laf = la(node_span[i]);

        let d = format!(
            "M {x1o} {y1o} A {outer_r} {outer_r} 0 {laf} 1 {x2o} {y2o} \
             L {x2i} {y2i} A {inner_r} {inner_r} 0 {laf} 0 {x1i} {y1i} Z"
        );
        let color = node_color(i);
        scene.add(Primitive::Path {
            d,
            fill: Some(color),
            stroke: "none".into(),
            stroke_width: 0.0,
            opacity: None,
            stroke_dasharray: None,
        });
    }

    // ── Draw labels ──
    // Place labels just outside the arc. Anchor to Start (right side) or End
    // (left side) so text always extends away from the arc regardless of length.
    let label_gap = outer_r + computed.body_size as f64 * 1.6;
    for i in 0..n {
        let mid = node_start[i] + node_span[i] / 2.0;
        let lx = cx + label_gap * mid.cos();
        let ly = cy + label_gap * mid.sin() + computed.body_size as f64 * 0.35;

        let label = if let Some(l) = chord.labels.get(i) { l.clone() } else { format!("{i}") };
        let anchor = if mid.cos() >= 0.0 { TextAnchor::Start } else { TextAnchor::End };

        scene.add(Primitive::Text {
            x: lx,
            y: ly,
            content: label,
            size: computed.body_size,
            anchor,
            rotate: None,
            bold: false,
        });
    }

    // ── Draw ribbons ──
    // Sub-arc cursor: for each node, track how much inner arc has been consumed.
    // Allocate sub-arc widths for each cell [i][j]
    // We need full forward pass to build all ribbon endpoints before drawing.
    // Store as: sub_start[i][j] = angle where node i's sub-arc for ribbon (i,j) starts
    let mut sub_start = vec![vec![0.0f64; n]; n];
    {
        // Reset cursors
        let mut cursors = node_start.clone();
        for i in 0..n {
            for j in 0..n {
                sub_start[i][j] = cursors[i];
                let flow = chord.matrix.get(i).and_then(|r| r.get(j)).copied().unwrap_or(0.0);
                if grand_total > 0.0 {
                    cursors[i] += (flow / grand_total) * usable;
                }
            }
        }
    }

    for i in 0..n {
        for j in 0..=i {
            let flow_ij = chord.matrix.get(i).and_then(|r| r.get(j)).copied().unwrap_or(0.0);
            let flow_ji = chord.matrix.get(j).and_then(|r| r.get(i)).copied().unwrap_or(0.0);

            // Self-loops
            if i == j {
                if flow_ij <= 0.0 { continue; }
                let a0 = sub_start[i][j];
                let span = if grand_total > 0.0 { (flow_ij / grand_total) * usable } else { 0.0 };
                if span <= 0.0 { continue; }
                let a1 = a0 + span;
                let laf = la(span);
                let x1 = cx + inner_r * a0.cos();
                let y1 = cy + inner_r * a0.sin();
                let x2 = cx + inner_r * a1.cos();
                let y2 = cy + inner_r * a1.sin();
                let d = format!(
                    "M {x1} {y1} A {inner_r} {inner_r} 0 {laf} 1 {x2} {y2} \
                     C {cx} {cy} {cx} {cy} {x1} {y1} Z"
                );
                scene.add(Primitive::Path {
                    d,
                    fill: Some(node_color(i)),
                    stroke: "none".into(),
                    stroke_width: 0.0,
                    opacity: Some(chord.ribbon_opacity),
                    stroke_dasharray: None,
                });
                continue;
            }

            if flow_ij <= 0.0 && flow_ji <= 0.0 { continue; }

            // Node i sub-arc for ribbon (i→j)
            let a_i0 = sub_start[i][j];
            let span_i = if grand_total > 0.0 { (flow_ij / grand_total) * usable } else { 0.0 };
            let a_i1 = a_i0 + span_i;

            // Node j sub-arc for ribbon (j→i)
            let a_j0 = sub_start[j][i];
            let span_j = if grand_total > 0.0 { (flow_ji / grand_total) * usable } else { 0.0 };
            let a_j1 = a_j0 + span_j;

            let xi1 = cx + inner_r * a_i0.cos();
            let yi1 = cy + inner_r * a_i0.sin();
            let xi2 = cx + inner_r * a_i1.cos();
            let yi2 = cy + inner_r * a_i1.sin();
            let xj1 = cx + inner_r * a_j0.cos();
            let yj1 = cy + inner_r * a_j0.sin();
            let xj2 = cx + inner_r * a_j1.cos();
            let yj2 = cy + inner_r * a_j1.sin();

            let laf_i = la(span_i);
            let laf_j = la(span_j);

            let d = format!(
                "M {xi1} {yi1} \
                 A {inner_r} {inner_r} 0 {laf_i} 1 {xi2} {yi2} \
                 C {cx} {cy} {cx} {cy} {xj1} {yj1} \
                 A {inner_r} {inner_r} 0 {laf_j} 1 {xj2} {yj2} \
                 C {cx} {cy} {cx} {cy} {xi1} {yi1} Z"
            );

            scene.add(Primitive::Path {
                d,
                fill: Some(node_color(i)),
                stroke: "none".into(),
                stroke_width: 0.0,
                opacity: Some(chord.ribbon_opacity),
                stroke_dasharray: None,
            });
        }
    }

}
// Legend for ChordPlot is handled by collect_legend_entries + render_multiple.

pub fn render_chord(chord: &ChordPlot, layout: &Layout) -> Scene {
    let computed = ComputedLayout::from_layout(layout);
    let mut scene = Scene::new(computed.width, computed.height);
    scene.font_family = computed.font_family.clone();
    apply_theme(&mut scene, &computed.theme);

    add_labels_and_title(&mut scene, &computed, layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);

    add_chord(chord, &mut scene, &computed);

    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);

    scene
}

fn add_sankey(sankey: &SankeyPlot, scene: &mut Scene, computed: &ComputedLayout) {
    use crate::render::palette::Palette;

    if sankey.nodes.is_empty() || sankey.links.is_empty() { return; }

    let n = sankey.nodes.len();
    let fallback = Palette::category10();
    let node_color = |i: usize| -> String {
        sankey.nodes[i].color.clone()
            .unwrap_or_else(|| fallback[i % fallback.len()].to_string())
    };

    // ── Step 1: Column assignment ──
    let mut col: Vec<Option<usize>> = sankey.nodes.iter().map(|nd| nd.column).collect();

    // Seed column 0 for nodes with no incoming links (pure sources).
    let mut has_incoming = vec![false; n];
    for link in &sankey.links {
        has_incoming[link.target] = true;
    }
    for i in 0..n {
        if col[i].is_none() && !has_incoming[i] {
            col[i] = Some(0);
        }
    }

    // Iteratively propagate: col[tgt] = max(col[tgt], col[src] + 1)
    let mut changed = true;
    while changed {
        changed = false;
        for link in &sankey.links {
            let src = link.source;
            let tgt = link.target;
            if let Some(sc) = col[src] {
                let new_tgt_col = sc + 1;
                if col[tgt].map_or(true, |tc| tc < new_tgt_col) {
                    col[tgt] = Some(new_tgt_col);
                    changed = true;
                }
            }
        }
    }

    // Assign any remaining unresolved nodes (cycle members) to max_col + 1.
    let max_assigned = col.iter().flatten().copied().max().unwrap_or(0);
    for c in col.iter_mut() {
        if c.is_none() {
            *c = Some(max_assigned + 1);
        }
    }
    let col: Vec<usize> = col.into_iter().map(|c| c.unwrap()).collect();
    let n_cols = col.iter().copied().max().unwrap_or(0) + 1;

    // ── Step 2: Node flow totals ──
    let mut out_flow = vec![0.0_f64; n];
    let mut in_flow = vec![0.0_f64; n];
    for link in &sankey.links {
        out_flow[link.source] += link.value;
        in_flow[link.target] += link.value;
    }
    let node_flow: Vec<f64> = (0..n)
        .map(|i| out_flow[i].max(in_flow[i]))
        .collect();

    // ── Step 3: Vertical layout per column ──
    let plot_h = computed.plot_height();
    let plot_w = computed.plot_width();

    // nodes_in_col[c] = ordered list of node indices in column c
    let mut nodes_in_col: Vec<Vec<usize>> = vec![vec![]; n_cols];
    for i in 0..n {
        nodes_in_col[col[i]].push(i);
    }

    let mut node_y = vec![0.0_f64; n];
    let mut node_h = vec![0.0_f64; n];

    for c in 0..n_cols {
        let members = &nodes_in_col[c];
        if members.is_empty() { continue; }
        let m = members.len();
        let total_gap = (m - 1) as f64 * sankey.node_gap;
        let usable_h = (plot_h - total_gap).max(1.0);
        let total_col_flow: f64 = members.iter().map(|&i| node_flow[i]).sum();
        if total_col_flow <= 0.0 { continue; }

        let total_h: f64 = members.iter().map(|&i| {
            (node_flow[i] / total_col_flow) * usable_h
        }).sum();
        // Center the column vertically
        let start_y = computed.margin_top + (plot_h - total_h - total_gap) / 2.0;

        let mut cursor_y = start_y;
        for &i in members {
            let h = (node_flow[i] / total_col_flow) * usable_h;
            node_h[i] = h;
            node_y[i] = cursor_y;
            cursor_y += h + sankey.node_gap;
        }
    }

    // ── Step 4: Horizontal layout ──
    let col_w = plot_w / n_cols as f64;
    let node_x: Vec<f64> = (0..n)
        .map(|i| computed.margin_left + col[i] as f64 * col_w + (col_w - sankey.node_width) / 2.0)
        .collect();

    // ── Step 5: Draw node rectangles and labels ──
    let max_col = col.iter().copied().max().unwrap_or(0);
    for i in 0..n {
        // Rectangle
        scene.add(Primitive::Rect {
            x: node_x[i],
            y: node_y[i],
            width: sankey.node_width,
            height: node_h[i].max(1.0),
            fill: node_color(i),
            stroke: None,
            stroke_width: None,
            opacity: None,
        });

        // Label: left of leftmost col → End anchor; right of rightmost → Start; inner → Start
        let (lx, anchor) = if col[i] == 0 {
            (node_x[i] - 6.0, TextAnchor::End)
        } else {
            (node_x[i] + sankey.node_width + 6.0, TextAnchor::Start)
        };
        let _ = max_col; // both sides get Start except col 0
        scene.add(Primitive::Text {
            x: lx,
            y: node_y[i] + node_h[i] / 2.0 + computed.body_size as f64 * 0.35,
            content: sankey.nodes[i].label.clone(),
            size: computed.body_size,
            anchor,
            rotate: None,
            bold: false,
        });
    }

    // ── Step 6: Draw ribbons ──
    // Cursors tracking how far down each node's in/out flow has been consumed.
    let mut out_cursor = node_y.clone();
    let mut in_cursor = node_y.clone();

    // Sort links by (target_col, target_node_idx_in_col) for stable ordering.
    let mut link_order: Vec<usize> = (0..sankey.links.len()).collect();
    link_order.sort_by_key(|&li| {
        let tgt = sankey.links[li].target;
        (col[tgt], nodes_in_col[col[tgt]].iter().position(|&x| x == tgt).unwrap_or(0))
    });

    for (link_i, &li) in link_order.iter().enumerate() {
        let link = &sankey.links[li];
        let src = link.source;
        let tgt = link.target;
        if link.value <= 0.0 { continue; }
        if out_flow[src] <= 0.0 || in_flow[tgt] <= 0.0 { continue; }

        let link_h_out = (link.value / out_flow[src]) * node_h[src];
        let link_h_in  = (link.value / in_flow[tgt])  * node_h[tgt];

        let x_src = node_x[src] + sankey.node_width;
        let x_tgt = node_x[tgt];
        let cx_mid = (x_src + x_tgt) / 2.0;

        let y_src_top = out_cursor[src];
        let y_src_bot = y_src_top + link_h_out;
        let y_tgt_top = in_cursor[tgt];
        let y_tgt_bot = y_tgt_top + link_h_in;

        out_cursor[src] += link_h_out;
        in_cursor[tgt]  += link_h_in;

        let d = format!(
            "M {x_src} {y_src_top} \
             C {cx_mid} {y_src_top} {cx_mid} {y_tgt_top} {x_tgt} {y_tgt_top} \
             L {x_tgt} {y_tgt_bot} \
             C {cx_mid} {y_tgt_bot} {cx_mid} {y_src_bot} {x_src} {y_src_bot} Z"
        );

        let fill = match &sankey.link_color {
            SankeyLinkColor::Source => node_color(src),
            SankeyLinkColor::PerLink => link.color.clone().unwrap_or_else(|| node_color(src)),
            SankeyLinkColor::Gradient => {
                let grad_id = format!("grad_{link_i}");
                let src_color = node_color(src);
                let tgt_color = node_color(tgt);
                scene.defs.push(format!(
                    r#"<linearGradient id="{grad_id}" x1="0%" y1="0%" x2="100%" y2="0%"><stop offset="0%" stop-color="{src_color}"/><stop offset="100%" stop-color="{tgt_color}"/></linearGradient>"#
                ));
                format!("url(#{grad_id})")
            }
        };

        scene.add(Primitive::Path {
            d,
            fill: Some(fill),
            stroke: "none".into(),
            stroke_width: 0.0,
            opacity: Some(sankey.link_opacity),
            stroke_dasharray: None,
        });
    }
}

pub fn render_sankey(sankey: &SankeyPlot, layout: &Layout) -> Scene {
    let computed = ComputedLayout::from_layout(layout);
    let mut scene = Scene::new(computed.width, computed.height);
    scene.font_family = computed.font_family.clone();
    apply_theme(&mut scene, &computed.theme);

    add_labels_and_title(&mut scene, &computed, layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);

    add_sankey(sankey, &mut scene, &computed);

    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);

    scene
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
                // Manhattan uses per-chromosome coloring; skip palette auto-cycling.
                _ => {}
            }
        }
    }

    let computed = ComputedLayout::from_layout(&layout);
    let mut scene = Scene::new(computed.width, computed.height);
    scene.font_family = computed.font_family.clone();
    apply_theme(&mut scene, &computed.theme);

    let skip_axes = plots.iter().all(|p| matches!(p, Plot::Pie(_) | Plot::UpSet(_) | Plot::Chord(_) | Plot::Sankey(_) | Plot::PhyloTree(_) | Plot::Synteny(_)));
    if !skip_axes {
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
            Plot::Manhattan(m) => {
                add_manhattan(m, &mut scene, &computed);
            }
            Plot::DotPlot(d) => {
                add_dot_plot(d, &mut scene, &computed);
            }
            Plot::UpSet(u) => {
                add_upset(u, &mut scene, &computed);
            }
            Plot::StackedArea(sa) => {
                add_stacked_area(sa, &mut scene, &computed);
            }
            Plot::Candlestick(cp) => {
                add_candlestick(cp, &mut scene, &computed);
            }
            Plot::Contour(cp) => {
                add_contour(cp, &mut scene, &computed);
            }
            Plot::Chord(c) => {
                add_chord(c, &mut scene, &computed);
            }
            Plot::Sankey(s) => {
                add_sankey(s, &mut scene, &computed);
            }
            Plot::PhyloTree(t) => {
                add_phylo_tree(t, &mut scene, &computed);
            }
            Plot::Synteny(s) => {
                add_synteny(s, &mut scene, &computed);
            }
        }
    }

    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);

    // Check for DotPlot stacked legend (size legend + colorbar in one column)
    let dot_stacked = plots.iter().find_map(|p| {
        if let Plot::DotPlot(dp) = p {
            if dp.size_label.is_some() && dp.color_legend_label.is_some() {
                p.colorbar_info().map(|info| (dp, info))
            } else { None }
        } else { None }
    });

    if let Some((dp, info)) = dot_stacked {
        // Build size entries inline to avoid needing Clone on LegendEntry
        let (size_min, size_max) = dp.size_range.unwrap_or_else(|| dp.size_extent());
        let mut size_entries = Vec::new();
        for &pct in &[0.25_f64, 0.50, 0.75, 1.0] {
            let value_at_pct = size_min + pct * (size_max - size_min);
            let radius_at_pct = dp.max_radius * pct;
            size_entries.push(LegendEntry {
                label: format!("{:.1}", value_at_pct),
                color: "#444444".into(),
                shape: LegendShape::CircleSize(radius_at_pct),
                dasharray: None,
            });
        }
        let title = dp.size_label.as_deref().unwrap_or("");
        add_dot_stacked_legends(title, &size_entries, &info, &mut scene, &computed);
    } else {
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

// ── Phylogenetic tree ─────────────────────────────────────────────────────────

fn add_phylo_tree(tree: &PhyloTree, scene: &mut Scene, computed: &ComputedLayout) {
    use crate::plot::phylo::post_order_dfs;
    use std::f64::consts::PI;

    let n_nodes = tree.nodes.len();
    if n_nodes == 0 { return; }

    // ── Step 1: post-order DFS to assign leaf positions ──────────────────────
    let post_order = post_order_dfs(tree.root, &tree.nodes);

    let mut pos: Vec<f64> = vec![0.0; n_nodes];
    let mut leaf_counter = 0usize;
    for &id in &post_order {
        if tree.nodes[id].children.is_empty() {
            pos[id] = leaf_counter as f64;
            leaf_counter += 1;
        } else {
            // mean of children positions
            let sum: f64 = tree.nodes[id].children.iter().map(|&c| pos[c]).sum();
            pos[id] = sum / tree.nodes[id].children.len() as f64;
        }
    }
    let n_leaves = leaf_counter;
    if n_leaves == 0 { return; }

    // Collect ordered leaves for label rendering
    let leaves: Vec<usize> = post_order.iter()
        .copied()
        .filter(|&id| tree.nodes[id].children.is_empty())
        .collect();

    // ── Step 2: depth (x) positions ──────────────────────────────────────────
    let depth: Vec<f64>;
    let max_depth_f: f64;

    if tree.phylogram {
        // Phylogram: accumulate branch lengths (BFS from root)
        let mut acc = vec![0.0f64; n_nodes];
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(tree.root);
        while let Some(id) = queue.pop_front() {
            for &child in &tree.nodes[id].children {
                acc[child] = acc[id] + tree.nodes[child].branch_length;
                queue.push_back(child);
            }
        }
        let max_len = leaves.iter().map(|&l| acc[l]).fold(0.0_f64, f64::max);
        max_depth_f = if max_len > 0.0 { max_len } else { 1.0 };
        depth = acc;
    } else {
        // Cladogram: leaves align at max depth
        let mut subtree_depth = vec![0usize; n_nodes];
        for &id in &post_order {
            if tree.nodes[id].children.is_empty() {
                subtree_depth[id] = 0;
            } else {
                subtree_depth[id] = tree.nodes[id].children.iter()
                    .map(|&c| subtree_depth[c] + 1)
                    .max()
                    .unwrap_or(0);
            }
        }
        let max_depth = subtree_depth[tree.root];
        max_depth_f = max_depth as f64;
        depth = (0..n_nodes).map(|i| (max_depth - subtree_depth[i]) as f64).collect();
    }

    // ── Step 3: pixel mapping ─────────────────────────────────────────────────
    let pw = computed.plot_width();
    let ph = computed.plot_height();
    let ml = computed.margin_left;
    let mt = computed.margin_top;

    // Reserve pixels for leaf labels and general padding so labels don't clip.
    let max_label_chars = tree.nodes.iter()
        .filter(|n| n.children.is_empty())
        .filter_map(|n| n.label.as_ref())
        .map(|l| l.len())
        .max()
        .unwrap_or(6);
    let label_pad = ((max_label_chars as f64) * 7.0 + 20.0).clamp(70.0, 200.0);
    let edge_pad  = 25.0_f64;

    // Effective rendering area — leaves land inside this box; labels overflow into the reserved strip.
    let (eff_ml, eff_mt, eff_pw, eff_ph) = match tree.branch_style {
        TreeBranchStyle::Circular => {
            let pad = edge_pad + label_pad * 0.55;
            (ml + pad, mt + pad, (pw - 2.0 * pad).max(50.0), (ph - 2.0 * pad).max(50.0))
        }
        _ => match tree.orientation {
            TreeOrientation::Left => (
                ml + edge_pad,
                mt + edge_pad,
                (pw - label_pad - edge_pad).max(50.0),
                (ph - 2.0 * edge_pad).max(50.0),
            ),
            TreeOrientation::Right => (
                ml + label_pad,
                mt + edge_pad,
                (pw - label_pad - edge_pad).max(50.0),
                (ph - 2.0 * edge_pad).max(50.0),
            ),
            TreeOrientation::Top => (
                ml + edge_pad,
                mt + edge_pad,
                (pw - 2.0 * edge_pad).max(50.0),
                (ph - label_pad - edge_pad).max(50.0),
            ),
            TreeOrientation::Bottom => (
                ml + edge_pad,
                mt + label_pad,
                (pw - 2.0 * edge_pad).max(50.0),
                (ph - label_pad - edge_pad).max(50.0),
            ),
        },
    };

    let d_frac = |i: usize| -> f64 {
        if max_depth_f > 0.0 { depth[i] / max_depth_f } else { 0.0 }
    };
    let p_frac = |i: usize| -> f64 {
        (pos[i] + 0.5) / n_leaves as f64
    };

    let (px, py, r_arr, theta_arr): (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) =
        if tree.branch_style == TreeBranchStyle::Circular {
            let cx = eff_ml + eff_pw / 2.0;
            let cy = eff_mt + eff_ph / 2.0;
            let max_r = eff_pw.min(eff_ph) * 0.5;
            let mut pxv = vec![0.0; n_nodes];
            let mut pyv = vec![0.0; n_nodes];
            let mut rv   = vec![0.0; n_nodes];
            let mut tv   = vec![0.0; n_nodes];
            for i in 0..n_nodes {
                let r     = d_frac(i) * max_r;
                let theta = p_frac(i) * 2.0 * PI;
                rv[i] = r;
                tv[i] = theta;
                pxv[i] = cx + r * theta.cos();
                pyv[i] = cy + r * theta.sin();
            }
            (pxv, pyv, rv, tv)
        } else {
            let mut pxv = vec![0.0; n_nodes];
            let mut pyv = vec![0.0; n_nodes];
            for i in 0..n_nodes {
                let df = d_frac(i);
                let pf = p_frac(i);
                let (x, y) = match tree.orientation {
                    TreeOrientation::Left => (
                        eff_ml + df * eff_pw,
                        eff_mt + pf * eff_ph,
                    ),
                    TreeOrientation::Right => (
                        eff_ml + (1.0 - df) * eff_pw,
                        eff_mt + pf * eff_ph,
                    ),
                    TreeOrientation::Top => (
                        eff_ml + pf * eff_pw,
                        eff_mt + df * eff_ph,
                    ),
                    TreeOrientation::Bottom => (
                        eff_ml + pf * eff_pw,
                        eff_mt + (1.0 - df) * eff_ph,
                    ),
                };
                pxv[i] = x;
                pyv[i] = y;
            }
            (pxv, pyv, vec![0.0; n_nodes], vec![0.0; n_nodes])
        };

    // ── Step 4: clade color lookup ────────────────────────────────────────────
    let mut node_color: Vec<String> = vec![tree.branch_color.clone(); n_nodes];
    for &(clade_root, ref color) in &tree.clade_colors {
        let mut stack = vec![clade_root];
        while let Some(id) = stack.pop() {
            if id < n_nodes {
                node_color[id] = color.clone();
                for &child in &tree.nodes[id].children {
                    stack.push(child);
                }
            }
        }
    }

    // ── Step 5: draw branches ─────────────────────────────────────────────────
    let sw = 1.5_f64;

    match tree.branch_style {
        TreeBranchStyle::Slanted => {
            for i in 0..n_nodes {
                if let Some(p) = tree.nodes[i].parent {
                    scene.elements.push(Primitive::Line {
                        x1: px[p], y1: py[p],
                        x2: px[i], y2: py[i],
                        stroke: node_color[i].clone(),
                        stroke_width: sw,
                        stroke_dasharray: None,
                    });
                }
            }
        }
        TreeBranchStyle::Rectangular => {
            let horiz = matches!(tree.orientation, TreeOrientation::Left | TreeOrientation::Right);
            // Arms
            for i in 0..n_nodes {
                if let Some(p) = tree.nodes[i].parent {
                    if horiz {
                        scene.elements.push(Primitive::Line {
                            x1: px[p], y1: py[i],
                            x2: px[i], y2: py[i],
                            stroke: node_color[i].clone(),
                            stroke_width: sw, stroke_dasharray: None,
                        });
                    } else {
                        scene.elements.push(Primitive::Line {
                            x1: px[i], y1: py[p],
                            x2: px[i], y2: py[i],
                            stroke: node_color[i].clone(),
                            stroke_width: sw, stroke_dasharray: None,
                        });
                    }
                }
            }
            // Spines
            for i in 0..n_nodes {
                let children = &tree.nodes[i].children;
                if children.is_empty() { continue; }
                if horiz {
                    let y_min = children.iter().map(|&c| py[c]).fold(f64::INFINITY, f64::min);
                    let y_max = children.iter().map(|&c| py[c]).fold(f64::NEG_INFINITY, f64::max);
                    scene.elements.push(Primitive::Line {
                        x1: px[i], y1: y_min, x2: px[i], y2: y_max,
                        stroke: node_color[i].clone(),
                        stroke_width: sw, stroke_dasharray: None,
                    });
                } else {
                    let x_min = children.iter().map(|&c| px[c]).fold(f64::INFINITY, f64::min);
                    let x_max = children.iter().map(|&c| px[c]).fold(f64::NEG_INFINITY, f64::max);
                    scene.elements.push(Primitive::Line {
                        x1: x_min, y1: py[i], x2: x_max, y2: py[i],
                        stroke: node_color[i].clone(),
                        stroke_width: sw, stroke_dasharray: None,
                    });
                }
            }
        }
        TreeBranchStyle::Circular => {
            let cx = eff_ml + eff_pw / 2.0;
            let cy = eff_mt + eff_ph / 2.0;

            // Radial arms: at child's theta, from parent radius to child radius
            for i in 0..n_nodes {
                if let Some(p) = tree.nodes[i].parent {
                    let theta_c = theta_arr[i];
                    let r_p     = r_arr[p];
                    let x1 = cx + r_p * theta_c.cos();
                    let y1 = cy + r_p * theta_c.sin();
                    scene.elements.push(Primitive::Line {
                        x1, y1, x2: px[i], y2: py[i],
                        stroke: node_color[i].clone(),
                        stroke_width: sw, stroke_dasharray: None,
                    });
                }
            }
            // Arc spines at each internal node's radius
            for i in 0..n_nodes {
                let children = &tree.nodes[i].children;
                if children.is_empty() { continue; }
                let r_i = r_arr[i];
                if r_i < 1.0 { continue; }

                let thetas: Vec<f64> = children.iter().map(|&c| theta_arr[c]).collect();
                let theta_min = thetas.iter().cloned().fold(f64::INFINITY, f64::min);
                let theta_max = thetas.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

                let x_start = cx + r_i * theta_min.cos();
                let y_start = cy + r_i * theta_min.sin();
                let x_end   = cx + r_i * theta_max.cos();
                let y_end   = cy + r_i * theta_max.sin();

                let arc_span  = theta_max - theta_min;
                let large_arc = if arc_span > PI { 1 } else { 0 };

                let d = format!(
                    "M {:.3} {:.3} A {:.3} {:.3} 0 {} 1 {:.3} {:.3}",
                    x_start, y_start, r_i, r_i, large_arc, x_end, y_end
                );
                scene.elements.push(Primitive::Path {
                    d,
                    fill: None,
                    stroke: node_color[i].clone(),
                    stroke_width: sw,
                    opacity: None,
                    stroke_dasharray: None,
                });
            }
        }
    }

    // ── Step 6: root marker ───────────────────────────────────────────────────
    scene.elements.push(Primitive::Circle {
        cx: px[tree.root],
        cy: py[tree.root],
        r:  3.0,
        fill: tree.branch_color.clone(),
    });

    // ── Step 7: leaf labels ───────────────────────────────────────────────────
    for &leaf in &leaves {
        if let Some(ref label) = tree.nodes[leaf].label {
            let (lx, ly, anchor, rotate) = match tree.branch_style {
                TreeBranchStyle::Circular => {
                    let theta  = theta_arr[leaf];
                    let offset = 8.0;
                    let lx = px[leaf] + offset * theta.cos();
                    let ly = py[leaf] + offset * theta.sin() + 4.0;
                    let anc = if theta.cos() >= 0.0 { TextAnchor::Start } else { TextAnchor::End };
                    (lx, ly, anc, None)
                }
                _ => match tree.orientation {
                    TreeOrientation::Left =>
                        (px[leaf] + 6.0, py[leaf] + 4.0, TextAnchor::Start, None),
                    TreeOrientation::Right =>
                        (px[leaf] - 6.0, py[leaf] + 4.0, TextAnchor::End, None),
                    TreeOrientation::Top =>
                        (px[leaf] + 4.0, py[leaf] + 14.0, TextAnchor::Start, Some(90.0)),
                    TreeOrientation::Bottom =>
                        (px[leaf] + 4.0, py[leaf] - 5.0, TextAnchor::Start, Some(90.0)),
                },
            };
            scene.elements.push(Primitive::Text {
                x: lx, y: ly,
                content: label.clone(),
                size: 11,
                anchor,
                rotate,
                bold: false,
            });
        }
    }

    // ── Step 8: support values ────────────────────────────────────────────────
    if let Some(threshold) = tree.support_threshold {
        for i in 0..n_nodes {
            if tree.nodes[i].children.is_empty() { continue; }
            if let Some(support) = tree.nodes[i].support {
                if support >= threshold {
                    scene.elements.push(Primitive::Text {
                        x: px[i] + 2.0,
                        y: py[i] - 2.0,
                        content: format!("{}", support as u32),
                        size: 10,
                        anchor: TextAnchor::Start,
                        rotate: None,
                        bold: false,
                    });
                }
            }
        }
    }
}

pub fn render_phylo_tree(tree: &PhyloTree, layout: &Layout) -> Scene {
    let computed = ComputedLayout::from_layout(layout);
    let mut scene = Scene::new(computed.width, computed.height);
    scene.font_family = computed.font_family.clone();
    apply_theme(&mut scene, &computed.theme);

    add_labels_and_title(&mut scene, &computed, layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);

    add_phylo_tree(tree, &mut scene, &computed);

    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);

    scene
}

fn add_synteny(synteny: &SyntenyPlot, scene: &mut Scene, computed: &ComputedLayout) {
    use crate::render::palette::Palette;

    if synteny.sequences.is_empty() { return; }

    let pw = computed.plot_width();
    let ph = computed.plot_height();
    let ml = computed.margin_left;
    let mt = computed.margin_top;
    let n = synteny.sequences.len();

    // Reserve left margin for bar labels
    let max_label_chars = synteny.sequences.iter().map(|s| s.label.len()).max().unwrap_or(0);
    let label_pad = (max_label_chars as f64 * 7.0 + 15.0).clamp(60.0, 160.0);
    let edge_pad = 20.0;

    let bar_x_left  = ml + label_pad + edge_pad;
    let bar_x_right = ml + pw - edge_pad;
    let bar_px_width = bar_x_right - bar_x_left;
    let bar_h = synteny.bar_height;

    // Global max for shared scale
    let global_max = synteny.sequences.iter()
        .map(|s| s.length)
        .fold(0.0_f64, f64::max);

    // Evenly distribute bars top-to-bottom
    let gap = if n > 1 {
        ((ph - 2.0 * edge_pad - n as f64 * bar_h) / (n - 1) as f64).max(bar_h * 1.5)
    } else {
        0.0
    };
    let bar_top: Vec<f64> = (0..n)
        .map(|i| mt + edge_pad + i as f64 * (bar_h + gap))
        .collect();

    // X-coordinate mapping
    let x_of = |seq_idx: usize, pos: f64| -> f64 {
        let raw = if synteny.shared_scale && global_max > 0.0 {
            bar_x_left + (pos / global_max) * bar_px_width
        } else {
            let len = synteny.sequences[seq_idx].length;
            bar_x_left + if len > 0.0 { (pos / len) * bar_px_width } else { 0.0 }
        };
        raw.clamp(bar_x_left, bar_x_right)
    };

    let fallback = Palette::category10();

    // Step 1 — Draw ribbons (before bars, so bars overlay them)
    for (block_idx, block) in synteny.blocks.iter().enumerate() {
        // Ensure r1 = upper row, r2 = lower row
        let (r1, s1_lo, s1_hi, r2, s2_lo, s2_hi) = if block.seq1 <= block.seq2 {
            (block.seq1, block.start1, block.end1, block.seq2, block.start2, block.end2)
        } else {
            (block.seq2, block.start2, block.end2, block.seq1, block.start1, block.end1)
        };

        if r1 >= n || r2 >= n { continue; }

        let x1_s = x_of(r1, s1_lo);
        let x1_e = x_of(r1, s1_hi);
        let x2_s = x_of(r2, s2_lo);
        let x2_e = x_of(r2, s2_hi);

        let y1_bot = bar_top[r1] + bar_h;
        let y2_top = bar_top[r2];
        let y_mid  = (y1_bot + y2_top) / 2.0;

        // Determine strand: if seq1 > seq2 and strand is Reverse, the swap above
        // inverts meaning, so we preserve the original strand on the (possibly swapped) pair.
        let is_inverted = block.strand == Strand::Reverse;

        let color = block.color.clone()
            .unwrap_or_else(|| fallback[block_idx % fallback.len()].to_string());

        let d = if !is_inverted {
            // Forward: parallel Bézier sides
            format!(
                "M {x1_s} {y1_bot} C {x1_s} {y_mid} {x2_s} {y_mid} {x2_s} {y2_top} \
                 L {x2_e} {y2_top} C {x2_e} {y_mid} {x1_e} {y_mid} {x1_e} {y1_bot} Z",
                x1_s=x1_s, y1_bot=y1_bot, y_mid=y_mid, x2_s=x2_s, y2_top=y2_top,
                x2_e=x2_e, x1_e=x1_e
            )
        } else {
            // Inverted: self-intersecting path — SVG nonzero fill rule fills the crossing
            format!(
                "M {x1_s} {y1_bot} C {x1_s} {y_mid} {x2_e} {y_mid} {x2_e} {y2_top} \
                 L {x2_s} {y2_top} C {x2_s} {y_mid} {x1_e} {y_mid} {x1_e} {y1_bot} Z",
                x1_s=x1_s, y1_bot=y1_bot, y_mid=y_mid, x2_e=x2_e, y2_top=y2_top,
                x2_s=x2_s, x1_e=x1_e
            )
        };

        scene.elements.push(Primitive::Path {
            d,
            fill: Some(color.clone()),
            stroke: color,
            stroke_width: 0.3,
            opacity: Some(synteny.block_opacity),
            stroke_dasharray: None,
        });
    }

    // Step 2 — Draw sequence bars (on top of ribbons)
    for (i, seq) in synteny.sequences.iter().enumerate() {
        let bar_color = seq.color.clone().unwrap_or_else(|| "#555555".to_string());
        let x_right = if synteny.shared_scale && global_max > 0.0 {
            bar_x_left + (seq.length / global_max) * bar_px_width
        } else {
            bar_x_right
        };
        scene.elements.push(Primitive::Rect {
            x: bar_x_left,
            y: bar_top[i],
            width: (x_right - bar_x_left).max(0.0),
            height: bar_h,
            fill: bar_color,
            stroke: None,
            stroke_width: None,
            opacity: None,
        });
    }

    // Step 3 — Bar labels (right-anchored, flush to left of bars)
    for (i, seq) in synteny.sequences.iter().enumerate() {
        scene.elements.push(Primitive::Text {
            x: bar_x_left - 6.0,
            y: bar_top[i] + bar_h / 2.0 + 4.0,
            content: seq.label.clone(),
            size: computed.body_size,
            anchor: TextAnchor::End,
            rotate: None,
            bold: false,
        });
    }
}

pub fn render_synteny(synteny: &SyntenyPlot, layout: &Layout) -> Scene {
    let computed = ComputedLayout::from_layout(layout);
    let mut scene = Scene::new(computed.width, computed.height);
    scene.font_family = computed.font_family.clone();
    apply_theme(&mut scene, &computed.theme);

    add_labels_and_title(&mut scene, &computed, layout);
    add_shaded_regions(&layout.shaded_regions, &mut scene, &computed);

    add_synteny(synteny, &mut scene, &computed);

    add_reference_lines(&layout.reference_lines, &mut scene, &computed);
    add_text_annotations(&layout.annotations, &mut scene, &computed);

    scene
}

