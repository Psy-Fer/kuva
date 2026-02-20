use std::sync::Arc;

use crate::plot::scatter::{ScatterPlot, TrendLine};
use crate::plot::line::LinePlot;
use crate::plot::bar::BarPlot;
use crate::plot::histogram::Histogram;
use crate::plot::boxplot::BoxPlot;
use crate::plot::violin::ViolinPlot;
use crate::plot::brick::BrickPlot;

use crate::plot::{Heatmap, Histogram2D, PiePlot, SeriesPlot};
use crate::plot::legend::ColorBarInfo;
use crate::render::render_utils;


pub enum Plot {
    Scatter(ScatterPlot),
    Line(LinePlot),
    Bar(BarPlot),
    Histogram(Histogram),
    Histogram2d(Histogram2D),
    Box(BoxPlot),
    Violin(ViolinPlot),
    Series(SeriesPlot),
    Pie(PiePlot),
    Heatmap(Heatmap),
    Brick(BrickPlot)
}

fn bounds_from_2d<I>(points: I) -> Option<((f64, f64), (f64, f64))> 
    where
        I: IntoIterator,
        I::Item: Into<(f64, f64)>,
    {

    // extract values
    let mut vals = Vec::new();

    for (x, y) in points.into_iter().map(Into::into) {
        vals.push((x, y));
    }

    if vals.is_empty() {
        return None;
    }
    let (mut x_min, mut x_max) = (vals[0].0, vals[0].0);
    let (mut y_min, mut y_max) = (vals[0].1, vals[0].1);
    for (x, y) in vals.into_iter() {
        x_min = x_min.min(x);
        x_max = x_max.max(x);
        y_min = y_min.min(y);
        y_max = y_max.max(y);
    }
    Some(((x_min, x_max), (y_min, y_max)))
}

fn _bounds_from_1d(points: &[f64]) -> Option<((f64, f64), (f64, f64))> {
    if points.is_empty() {
        return None;
    }
    let (mut min_val, mut max_val) = (points[0], points[0]);
    for i in points {
        min_val = min_val.min(*i);
        max_val = max_val.max(*i);
       
    }
    
    Some(((0.0f64, points.len() as f64), (min_val, max_val)))
}



impl Plot {
    pub fn bounds(&self) -> Option<((f64, f64), (f64, f64))> {
        match self {
            
            Plot::Scatter(s) => {
                let ((mut x_min, mut x_max), (mut y_min, mut y_max)) = bounds_from_2d(&s.data).unwrap();

                // Expand with error bars
                for point in &s.data {
                    let x_lo = point.x - point.x_err.map_or(0.0, |e| e.0);
                    let x_hi = point.x + point.x_err.map_or(0.0, |e| e.1);
                    let y_lo = point.y - point.y_err.map_or(0.0, |e| e.0);
                    let y_hi = point.y + point.y_err.map_or(0.0, |e| e.1);

                    x_min = x_min.min(x_lo);
                    x_max = x_max.max(x_hi);
                    y_min = y_min.min(y_lo);
                    y_max = y_max.max(y_hi);
                }

                // Expand for trend line
                if let Some(trend) = s.trend {
                    let TrendLine::Linear = trend;
                        if let Some((slope, intercept, _)) = render_utils::linear_regression(&s.data) {
                            let y_start = slope * x_min + intercept;
                            let y_end = slope * x_max + intercept;

                            y_min = y_min.min(y_start).min(y_end);
                            y_max = y_max.max(y_start).max(y_end);
                        }
                }

                Some(((x_min, x_max), (y_min, y_max)))
            },
            Plot::Line(p) => bounds_from_2d(&p.data),
            // Plot::Series(s) => bounds_from_1d(&s.values),
            Plot::Series(sp) => {
                if sp.values.is_empty() {
                    None
                } else {
                    let x_min = 0.0;
                    let x_max = sp.values.len() as f64 - 1.0;
            
                    let mut y_min = f64::INFINITY;
                    let mut y_max = f64::NEG_INFINITY;
            
                    for &v in &sp.values {
                        y_min = y_min.min(v);
                        y_max = y_max.max(v);
                    }
            
                    Some(((x_min, x_max), (y_min, y_max)))
                }
            }
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
                        let q1 = render_utils::percentile(&vals, 25.0);
                        let q3 = render_utils::percentile(&vals, 75.0);
                        let iqr = q3 - q1;
                        let lo = q1 - 1.5 * iqr;
                        let hi = q3 + 1.5 * iqr;
                        y_min = y_min.min(lo);
                        y_max = y_max.max(hi);
                    }
            
                    Some(((x_min, x_max), (y_min, y_max)))
                }
            }
            Plot::Violin(vp) => {
                if vp.groups.is_empty() {
                    None
                } else {
                    let x_min = 0.5;
                    let x_max = vp.groups.len() as f64 + 0.5;
            
                    let mut y_min = f64::INFINITY;
                    let mut y_max = f64::NEG_INFINITY;
            
                    for group in &vp.groups {
                        if group.values.is_empty() { continue; }
            
                        for &v in &group.values {
                            y_min = y_min.min(v);
                            y_max = y_max.max(v);
                        }
                    }
                    // let padding = 0.05 * (y_max - y_min);
                    // y_min -= padding;
                    // y_max += padding;
            
                    Some(((x_min, x_max), (y_min, y_max)))
                }
            }
            Plot::Pie(_) => {
                // Centered at (0.0, 0.0) and rendered to fit the layout box
                Some(((-1.0, 1.0), (-1.0, 1.0))) 
            }
            Plot::Heatmap(hm) => {
                let rows = hm.data.len();
                let cols = hm.data.first().map_or(0, |row| row.len());
                Some(((0.0, cols as f64), (0.0, rows as f64)))
            }
            Plot::Histogram2d(h2d) => {
                let rows = h2d.bins.len();
                let cols = h2d.bins.first().map_or(0, |row| row.len());
                Some(((0.0, cols as f64), (0.0, rows as f64)))
            }
            Plot::Brick(bp) => {
                let rows = if let Some(ref exp) = bp.strigar_exp {
                    exp.len()
                } else {
                    bp.sequences.len()
                };

                let max_width = if let Some(ref exp) = bp.strigar_exp {
                    if let Some(ref ml) = bp.motif_lengths {
                        // Variable-width: sum motif lengths per row
                        exp.iter().map(|s| {
                            s.chars().map(|c| *ml.get(&c).unwrap_or(&1) as f64).sum::<f64>()
                        }).fold(0.0f64, f64::max)
                    } else {
                        exp.iter().map(|s| s.len()).max().unwrap_or(0) as f64
                    }
                } else {
                    bp.sequences.iter().map(|s| s.len()).max().unwrap_or(0) as f64
                };

                Some(((0.0 - bp.x_offset, max_width - bp.x_offset), (0.0, rows as f64)))
            }
        }
    }

    pub fn colorbar_info(&self) -> Option<ColorBarInfo> {
        match self {
            Plot::Heatmap(hm) => {
                let flat: Vec<f64> = hm.data.iter().flatten().cloned().collect();
                let min = flat.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = flat.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let cmap = hm.color_map.clone();
                Some(ColorBarInfo {
                    map_fn: Arc::new(move |t| {
                        let norm = (t - min) / (max - min + f64::EPSILON);
                        cmap.map(norm.clamp(0.0, 1.0))
                    }),
                    min_value: min,
                    max_value: max,
                    label: None,
                })
            }
            Plot::Histogram2d(h2d) => {
                let max_count = h2d.bins.iter().flatten().copied().max().unwrap_or(1) as f64;
                let cmap = h2d.color_map.clone();
                Some(ColorBarInfo {
                    map_fn: Arc::new(move |t| {
                        let norm = t / max_count;
                        cmap.map(norm.clamp(0.0, 1.0))
                    }),
                    min_value: 0.0,
                    max_value: max_count,
                    label: Some("Count".to_string()),
                })
            }
            _ => None,
        }
    }
}