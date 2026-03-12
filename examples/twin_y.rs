//! Twin-Y axis documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example twin_y
//! ```
//!
//! SVGs are written to `docs/src/assets/twin_y/`.

use kuva::plot::{LinePlot, LegendPosition};
use kuva::plot::scatter::ScatterPlot;
use kuva::plot::histogram::Histogram;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_twin_y;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::Palette;

const OUT: &str = "docs/src/assets/twin_y";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/twin_y");

    basic();
    labels();
    log_y2();
    gc_bias();

    println!("Twin-Y SVGs written to {OUT}/");
}

/// Minimal twin-y — two line series on independent y-axes.
fn basic() {
    let temp: Vec<(f64, f64)> = vec![
        (1.0, 5.0), (2.0, 8.0), (3.0, 14.0), (4.0, 20.0), (5.0, 24.0), (6.0, 22.0),
    ];
    let rain: Vec<(f64, f64)> = vec![
        (1.0, 80.0), (2.0, 60.0), (3.0, 45.0), (4.0, 30.0), (5.0, 20.0), (6.0, 35.0),
    ];

    let primary   = vec![Plot::Line(LinePlot::new().with_data(temp).with_legend("Temperature (°C)"))];
    let secondary = vec![Plot::Line(LinePlot::new().with_data(rain).with_legend("Rainfall (mm)"))];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary)
        .with_title("Temperature & Rainfall")
        .with_x_label("Month")
        .with_y_label("Temperature (°C)")
        .with_y2_label("Rainfall (mm)")
        .with_palette(Palette::wong());

    let svg = SvgBackend.render_scene(&render_twin_y(primary, secondary, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Twin-y with axis labels and a legend positioned outside the plot.
fn labels() {
    let temp: Vec<(f64, f64)> = vec![
        (1.0, 5.0), (2.0, 8.0), (3.0, 14.0), (4.0, 20.0), (5.0, 24.0), (6.0, 22.0),
    ];
    let rain: Vec<(f64, f64)> = vec![
        (1.0, 80.0), (2.0, 60.0), (3.0, 45.0), (4.0, 30.0), (5.0, 20.0), (6.0, 35.0),
    ];

    let primary   = vec![Plot::Line(LinePlot::new().with_data(temp).with_color("#e69f00").with_legend("Temperature (°C)"))];
    let secondary = vec![Plot::Line(LinePlot::new().with_data(rain).with_color("#0072b2").with_legend("Rainfall (mm)"))];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary)
        .with_title("Temperature & Rainfall — Labeled Axes")
        .with_x_label("Month")
        .with_y_label("Temperature (°C)")
        .with_y2_label("Rainfall (mm)")
        .with_legend_position(LegendPosition::OutsideRightTop);

    let svg = SvgBackend.render_scene(&render_twin_y(primary, secondary, layout));
    std::fs::write(format!("{OUT}/labels.svg"), svg).unwrap();
}

/// Log scale on the secondary y-axis.
fn log_y2() {
    let linear: Vec<(f64, f64)> = (1..=5).map(|x| (x as f64, x as f64 * 10.0)).collect();
    let exponential: Vec<(f64, f64)> = vec![
        (1.0, 1.0), (2.0, 10.0), (3.0, 100.0), (4.0, 1000.0), (5.0, 10000.0),
    ];

    let primary   = vec![Plot::Line(LinePlot::new().with_data(linear).with_color("#e69f00").with_legend("Linear"))];
    let secondary = vec![Plot::Line(LinePlot::new().with_data(exponential).with_color("#cc79a7").with_legend("Exponential"))];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary)
        .with_title("Linear vs. Exponential — Log Y2")
        .with_x_label("X")
        .with_y_label("Linear value")
        .with_y2_label("Exponential value (log scale)")
        .with_log_y2()
        .with_legend_position(LegendPosition::InsideTopLeft);

    let svg = SvgBackend.render_scene(&render_twin_y(primary, secondary, layout));
    std::fs::write(format!("{OUT}/log_y2.svg"), svg).unwrap();
}

/// GC bias showcase — histogram + scatter on primary, two lines on secondary.
///
/// Mirrors the layout of a typical WGS GC bias QC chart:
///   Primary axis (Normalized Coverage): Genome GC histogram + Coverage scatter
///   Secondary axis (Base Quality 0–40): Reported BQ + Empirical BQ lines
fn gc_bias() {
    // Genome GC — precomputed bell-curve histogram, peak at ~38 % GC
    let edges: Vec<f64> = (0..=100).step_by(2).map(|x| x as f64).collect();
    let counts: Vec<f64> = (0..50_usize).map(|i| {
        let gc = i as f64 * 2.0 + 1.0;
        0.50 * (-0.5 * ((gc - 38.0) / 14.0).powi(2)).exp()
    }).collect();

    let genome_gc = Plot::Histogram(
        Histogram::from_bins(edges, counts)
            .with_color("#a8d8f0")
            .with_legend("Genome GC"),
    );

    // Coverage — U-shaped scatter, clamped at 2.0 at extreme GC
    let coverage_pts: Vec<(f64, f64)> = {
        let mut v = Vec::new();
        for gc in [2.0_f64, 4.0, 6.0, 8.0, 10.0] { v.push((gc, 2.0)); }
        for (gc, cov) in [(12.0,1.70),(14.0,1.45),(16.0,1.35),(18.0,1.25),(20.0,1.15)] { v.push((gc, cov)); }
        for i in 0..=24_u32 {
            let gc = 22.0 + i as f64 * 2.0;
            let cov = 0.90 + 0.35 * ((gc - 50.0) / 35.0).powi(2);
            v.push((gc, cov.min(2.0)));
        }
        for (gc, cov) in [(72.0,1.05),(74.0,1.10),(76.0,1.20),(78.0,1.30),(80.0,1.45)] { v.push((gc, cov)); }
        for (gc, cov) in [(82.0,1.60),(84.0,1.75),(86.0,1.80)] { v.push((gc, cov)); }
        for gc in [88.0_f64, 90.0, 92.0, 94.0, 96.0, 98.0] { v.push((gc, 2.0)); }
        v
    };

    let coverage = Plot::Scatter(
        ScatterPlot::new()
            .with_data(coverage_pts)
            .with_color("#4e90d9")
            .with_size(5.0)
            .with_legend("Coverage"),
    );

    // Reported BQ — broadly flat ~29, drops at extreme GC
    let reported_bq: Vec<(f64, f64)> = (1..=20_u32).map(|i| {
        let gc = i as f64 * 5.0;
        let bq = if gc < 15.0 || gc > 85.0 { 22.0 - (gc - 50.0).abs() * 0.3 }
                 else { 29.5 - (gc - 50.0).abs() * 0.025 };
        (gc, bq.clamp(8.0, 40.0))
    }).collect();

    // Empirical BQ — lower trace ~15, similar shape
    let empirical_bq: Vec<(f64, f64)> = (1..=20_u32).map(|i| {
        let gc = i as f64 * 5.0;
        let bq = if gc < 15.0 || gc > 85.0 { 10.0 - (gc - 50.0).abs() * 0.1 }
                 else { 15.0 - (gc - 50.0).abs() * 0.01 };
        (gc, bq.clamp(4.0, 40.0))
    }).collect();

    let primary   = vec![genome_gc, coverage];
    let secondary = vec![
        Plot::Line(LinePlot::new().with_data(reported_bq).with_color("#2ca02c").with_legend("Reported BQ")),
        Plot::Line(LinePlot::new().with_data(empirical_bq).with_color("#17becf").with_legend("Empirical BQ")),
    ];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary)
        .with_title("GC Bias")
        .with_x_label("GC%")
        .with_y_label("Normalized Coverage")
        .with_y2_label("Base Quality")
        .with_legend_position(LegendPosition::OutsideRightTop);

    let svg = SvgBackend.render_scene(&render_twin_y(primary, secondary, layout));
    std::fs::write(format!("{OUT}/gc_bias.svg"), svg).unwrap();
}
