use crate::plot::scatter::ScatterPlot;
use crate::plot::line::LinePlot;
use crate::plot::bar::BarPlot;
use crate::plot::histogram::Histogram;
use crate::plot::boxplot::BoxPlot;
use crate::plot::violin::ViolinPlot;

use crate::plot::{PiePlot, SeriesPlot};
use crate::render::render_utils;


pub enum Plot {
    Scatter(ScatterPlot),
    Line(LinePlot),
    Bar(BarPlot),
    Histogram(Histogram),
    Box(BoxPlot),
    Violin(ViolinPlot),
    Series(SeriesPlot),
    Pie(PiePlot),
}

fn bounds_from_2d(points: &[(f64, f64)]) -> Option<((f64, f64), (f64, f64))> {
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
            
            Plot::Scatter(s) => bounds_from_2d(&s.data),
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
        }
    }
}