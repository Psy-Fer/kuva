//! Band plot documentation examples.
//!
//! Generates canonical SVG outputs used in the visus documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example band
//! ```
//!
//! SVGs are written to `docs/src/assets/band/`.

use visus::plot::{BandPlot, LinePlot, ScatterPlot};
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

const OUT: &str = "docs/src/assets/band";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/band");

    standalone();
    attached_line();
    attached_scatter();
    multi_band();

    println!("Band SVGs written to {OUT}/");
}

/// Standalone band paired with a line.
///
/// `BandPlot::new(x, y_lower, y_upper)` fills the region between two curves.
/// Placing both `Plot::Band` and `Plot::Line` in the same `plots` vec renders
/// the band behind the line on shared axes.
fn standalone() {
    let x: Vec<f64> = (0..60).map(|i| i as f64 * 0.2).collect();
    let y: Vec<f64> = x.iter().map(|&v| v.sin()).collect();
    let lower: Vec<f64> = y.iter().map(|&v| v - 0.35).collect();
    let upper: Vec<f64> = y.iter().map(|&v| v + 0.35).collect();

    let band = BandPlot::new(x.clone(), lower, upper)
        .with_color("steelblue")
        .with_opacity(0.25);

    let line = LinePlot::new()
        .with_data(x.iter().copied().zip(y.iter().copied()))
        .with_color("steelblue");

    let plots = vec![Plot::Band(band), Plot::Line(line)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Band Plot — Standalone")
        .with_x_label("x")
        .with_y_label("y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/standalone.svg"), svg).unwrap();
}

/// Band attached to a line via `LinePlot::with_band`.
///
/// `.with_band(y_lower, y_upper)` is a convenience shorthand: it creates a
/// `BandPlot` using the line's own x positions and inherits the line color
/// automatically. The band is rendered behind the line.
fn attached_line() {
    let x: Vec<f64> = (0..80).map(|i| i as f64 * std::f64::consts::PI / 20.0).collect();
    let y: Vec<f64> = x.iter().map(|&v| (-v * 0.15).exp() * v.cos()).collect();
    let lower: Vec<f64> = y.iter().map(|&v| v - 0.2).collect();
    let upper: Vec<f64> = y.iter().map(|&v| v + 0.2).collect();

    let line = LinePlot::new()
        .with_data(x.iter().copied().zip(y.iter().copied()))
        .with_color("firebrick")
        .with_band(lower, upper);

    let plots = vec![Plot::Line(line)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Band Attached to Line")
        .with_x_label("x")
        .with_y_label("e^(−0.15x) · cos(x)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/attached_line.svg"), svg).unwrap();
}

/// Band attached to a scatter plot via `ScatterPlot::with_band`.
///
/// Same pattern as `LinePlot::with_band`: the band inherits the scatter color
/// and is drawn behind the points.
fn attached_scatter() {
    let x: Vec<f64> = (0..30).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&v| 0.4 * v + 2.0).collect();
    let lower: Vec<f64> = y.iter().map(|&v| v - 2.5).collect();
    let upper: Vec<f64> = y.iter().map(|&v| v + 2.5).collect();

    let scatter = ScatterPlot::new()
        .with_data(x.iter().copied().zip(y.iter().copied()))
        .with_color("seagreen")
        .with_band(lower, upper);

    let plots = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Band Attached to Scatter")
        .with_x_label("x")
        .with_y_label("y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/attached_scatter.svg"), svg).unwrap();
}

/// Two series each with their own confidence band.
///
/// Each `LinePlot` carries an independent band. All four plot elements share
/// the same axes and are combined in one `plots` vector.
fn multi_band() {
    let x: Vec<f64> = (0..60).map(|i| i as f64 * 0.2).collect();

    let y1: Vec<f64> = x.iter().map(|&v| v.sin()).collect();
    let y2: Vec<f64> = x.iter().map(|&v| (v * 0.5).cos() * 0.8).collect();

    let line1 = LinePlot::new()
        .with_data(x.iter().copied().zip(y1.iter().copied()))
        .with_color("steelblue")
        .with_band(
            y1.iter().map(|&v| v - 0.25),
            y1.iter().map(|&v| v + 0.25),
        )
        .with_legend("sin(x)");

    let line2 = LinePlot::new()
        .with_data(x.iter().copied().zip(y2.iter().copied()))
        .with_color("darkorange")
        .with_band(
            y2.iter().map(|&v| v - 0.25),
            y2.iter().map(|&v| v + 0.25),
        )
        .with_legend("0.8 · cos(0.5x)");

    let plots = vec![Plot::Line(line1), Plot::Line(line2)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Multiple Series with Bands")
        .with_x_label("x")
        .with_y_label("y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/multi.svg"), svg).unwrap();
}
