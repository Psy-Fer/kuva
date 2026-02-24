//! Series plot documentation examples.
//!
//! Generates canonical SVG outputs used in the visus documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example series
//! ```
//!
//! SVGs are written to `docs/src/assets/series/`.

use visus::plot::SeriesPlot;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

const OUT: &str = "docs/src/assets/series";
const N: usize = 80;

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/series");

    basic();
    multi_series();
    styles();
    styled();

    println!("Series SVGs written to {OUT}/");
}

/// Single sine wave — default line style.
///
/// `with_data` accepts any iterable of numeric values. The x-axis is assigned
/// automatically as sequential integers 0, 1, 2, … matching the value index.
fn basic() {
    let data: Vec<f64> = (0..N)
        .map(|i| (i as f64 * 2.0 * std::f64::consts::PI / N as f64).sin())
        .collect();

    let series = SeriesPlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_line_style();

    let plots = vec![Plot::Series(series)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Series Plot — Line Style")
        .with_x_label("Sample")
        .with_y_label("Amplitude");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Multiple series on one canvas — sin, cos, and a damped oscillation.
///
/// Each `SeriesPlot` has its own color and legend label. All three share the
/// same x-axis length (N points) so they align automatically.
fn multi_series() {
    let pi2 = 2.0 * std::f64::consts::PI;

    let sin_data: Vec<f64> = (0..N)
        .map(|i| (i as f64 * pi2 / N as f64).sin())
        .collect();

    let cos_data: Vec<f64> = (0..N)
        .map(|i| (i as f64 * pi2 / N as f64).cos())
        .collect();

    let damped: Vec<f64> = (0..N)
        .map(|i| {
            let t = i as f64 * pi2 / N as f64;
            (-t * 0.5).exp() * t.sin()
        })
        .collect();

    let s1 = SeriesPlot::new()
        .with_data(sin_data)
        .with_color("steelblue")
        .with_line_style()
        .with_legend("sin(t)");

    let s2 = SeriesPlot::new()
        .with_data(cos_data)
        .with_color("firebrick")
        .with_line_style()
        .with_legend("cos(t)");

    let s3 = SeriesPlot::new()
        .with_data(damped)
        .with_color("seagreen")
        .with_line_style()
        .with_stroke_width(1.5)
        .with_legend("e^(−t/2)·sin(t)");

    let plots = vec![Plot::Series(s1), Plot::Series(s2), Plot::Series(s3)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Multiple Series")
        .with_x_label("Sample")
        .with_y_label("Amplitude");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/multi.svg"), svg).unwrap();
}

/// Three display styles — Line, Point, and Both — applied to the same data.
fn styles() {
    let data: Vec<f64> = (0..30)
        .map(|i| (i as f64 * 2.0 * std::f64::consts::PI / 30.0).sin())
        .collect();

    let make = |name: &str, title: &str, series: SeriesPlot| {
        let plots = vec![Plot::Series(series)];
        let layout = Layout::auto_from_plots(&plots)
            .with_title(title)
            .with_x_label("Sample")
            .with_y_label("Amplitude");
        let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
        std::fs::write(format!("{OUT}/style_{name}.svg"), svg).unwrap();
    };

    make("line", "SeriesStyle::Line",
        SeriesPlot::new().with_data(data.clone()).with_color("steelblue").with_line_style());
    make("point", "SeriesStyle::Point",
        SeriesPlot::new().with_data(data.clone()).with_color("steelblue").with_point_style());
    make("both", "SeriesStyle::Both",
        SeriesPlot::new().with_data(data).with_color("steelblue").with_line_point_style());
}

/// Custom stroke width and point radius.
///
/// `.with_stroke_width(f)` controls the line thickness;
/// `.with_point_radius(f)` controls the circle size in `Both` and `Point` modes.
fn styled() {
    let data: Vec<f64> = (0..N)
        .map(|i| {
            let t = i as f64 * 2.0 * std::f64::consts::PI / N as f64;
            t.sin() + (3.0 * t).sin() * 0.3
        })
        .collect();

    let series = SeriesPlot::new()
        .with_data(data)
        .with_color("darkorchid")
        .with_line_point_style()
        .with_stroke_width(1.5)
        .with_point_radius(4.0)
        .with_legend("signal");

    let plots = vec![Plot::Series(series)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Custom Stroke & Point Size")
        .with_x_label("Sample")
        .with_y_label("Amplitude");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/styled.svg"), svg).unwrap();
}
