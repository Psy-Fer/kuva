//! Line plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example line
//! ```
//!
//! SVGs are written to `docs/src/assets/line/`.

use kuva::plot::{LinePlot, LineStyle};
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

const OUT: &str = "docs/src/assets/line";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/line");

    basic();
    styles();
    fill();
    step();
    band();
    error_bars();

    println!("Line SVGs written to {OUT}/");
}

/// Basic line plot — a single series.
fn basic() {
    let data: Vec<(f64, f64)> = (0..=100)
        .map(|i| {
            let x = i as f64 * 0.1;
            (x, x.sin())
        })
        .collect();

    let plot = LinePlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_stroke_width(2.0);

    let plots = vec![Plot::Line(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Line Plot")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// All four built-in line styles.
fn styles() {
    let xs: Vec<f64> = (0..=80).map(|i| i as f64 * 0.125).collect();

    let plots = vec![
        Plot::Line(
            LinePlot::new()
                .with_data(xs.iter().map(|&x| (x, x.sin())))
                .with_color("steelblue")
                .with_stroke_width(2.0)
                .with_line_style(LineStyle::Solid)
                .with_legend("Solid"),
        ),
        Plot::Line(
            LinePlot::new()
                .with_data(xs.iter().map(|&x| (x, x.cos())))
                .with_color("crimson")
                .with_stroke_width(2.0)
                .with_dashed()
                .with_legend("Dashed"),
        ),
        Plot::Line(
            LinePlot::new()
                .with_data(xs.iter().map(|&x| (x, (x * 0.7).sin())))
                .with_color("seagreen")
                .with_stroke_width(2.0)
                .with_dotted()
                .with_legend("Dotted"),
        ),
        Plot::Line(
            LinePlot::new()
                .with_data(xs.iter().map(|&x| (x, (x * 0.7).cos())))
                .with_color("darkorange")
                .with_stroke_width(2.0)
                .with_dashdot()
                .with_legend("Dash-dot"),
        ),
    ];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Line Styles")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/styles.svg"), svg).unwrap();
}

/// Area plot — line with a filled region down to zero.
fn fill() {
    let data: Vec<(f64, f64)> = (0..=100)
        .map(|i| {
            let x = i as f64 * 0.1;
            (x, x.sin())
        })
        .collect();

    let plot = LinePlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_stroke_width(2.0)
        .with_fill()
        .with_fill_opacity(0.3);

    let plots = vec![Plot::Line(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Area Plot")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/fill.svg"), svg).unwrap();
}

/// Step plot — horizontal-then-vertical transitions between points.
fn step() {
    let data: Vec<(f64, f64)> = vec![
        (0.0, 2.0), (1.0, 5.0), (2.0, 3.0), (3.0, 7.0),
        (4.0, 4.0), (5.0, 8.0), (6.0, 5.0), (7.0, 9.0),
    ];

    let plot = LinePlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_stroke_width(2.0)
        .with_step();

    let plots = vec![Plot::Line(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Step Plot")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/step.svg"), svg).unwrap();
}

/// Line with a shaded confidence band.
fn band() {
    let xs: Vec<f64> = (0..=80).map(|i| i as f64 * 0.125).collect();
    let ys: Vec<f64> = xs.iter().map(|&x| x.sin()).collect();
    let lower: Vec<f64> = ys.iter().map(|&y| y - 0.3).collect();
    let upper: Vec<f64> = ys.iter().map(|&y| y + 0.3).collect();

    let data: Vec<(f64, f64)> = xs.into_iter().zip(ys).collect();

    let plot = LinePlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_stroke_width(2.0)
        .with_band(lower, upper);

    let plots = vec![Plot::Line(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Confidence Band")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/band.svg"), svg).unwrap();
}

/// Line with symmetric Y error bars.
fn error_bars() {
    let data: Vec<(f64, f64)> = (0..=8)
        .map(|i| (i as f64, (i as f64 * 0.8).sin()))
        .collect();
    let y_err: Vec<f64> = vec![0.15, 0.20, 0.12, 0.18, 0.22, 0.14, 0.19, 0.16, 0.21];

    let plot = LinePlot::new()
        .with_data(data)
        .with_y_err(y_err)
        .with_color("steelblue")
        .with_stroke_width(2.0);

    let plots = vec![Plot::Line(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Error Bars")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/error_bars.svg"), svg).unwrap();
}
