//! Scatter plot documentation examples.
//!
//! Generates canonical SVG outputs used in the visus documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example scatter
//! ```
//!
//! SVGs are written to `docs/src/assets/scatter/`.

use visus::plot::scatter::{MarkerShape, ScatterPlot, TrendLine};
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

const OUT: &str = "docs/src/assets/scatter";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/scatter");

    basic();
    trend();
    confidence_band();
    error_bars();
    markers();
    bubble();

    println!("Scatter SVGs written to {OUT}/");
}

/// Basic scatter plot — x/y data, color, point size.
fn basic() {
    let data = vec![
        (0.5_f64, 1.2_f64),
        (1.4, 3.1),
        (2.1, 2.4),
        (3.3, 5.0),
        (4.0, 4.3),
        (5.2, 6.8),
        (6.1, 6.0),
        (7.0, 8.5),
        (8.4, 7.9),
        (9.1, 9.8),
    ];

    let plot = ScatterPlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_size(5.0);

    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Scatter Plot")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Scatter with a linear trend line, regression equation, and R² annotation.
fn trend() {
    let data = vec![
        (1.0_f64, 2.1_f64),
        (2.0, 3.9),
        (3.0, 6.2),
        (4.0, 7.8),
        (5.0, 10.1),
        (6.0, 12.3),
        (7.0, 13.9),
        (8.0, 16.2),
        (9.0, 17.8),
        (10.0, 19.7),
    ];

    let plot = ScatterPlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_size(5.0)
        .with_trend(TrendLine::Linear)
        .with_trend_color("crimson")
        .with_equation()
        .with_correlation();

    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Linear Trend Line")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/trend.svg"), svg).unwrap();
}

/// Scatter with a shaded confidence band.
fn confidence_band() {
    let xs: Vec<f64> = (1..=10).map(|i| i as f64).collect();
    let ys: Vec<f64> = xs.iter().map(|&x| x * 1.8 + 0.5).collect();
    let lower: Vec<f64> = ys.iter().map(|&y| y - 1.2).collect();
    let upper: Vec<f64> = ys.iter().map(|&y| y + 1.2).collect();

    let data: Vec<(f64, f64)> = xs.into_iter().zip(ys).collect();

    let plot = ScatterPlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_size(5.0)
        .with_band(lower, upper);

    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Confidence Band")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/confidence_band.svg"), svg).unwrap();
}

/// Scatter with symmetric error bars on both axes.
fn error_bars() {
    let data = vec![
        (1.0_f64, 2.0_f64),
        (2.0, 4.5),
        (3.0, 5.8),
        (4.0, 8.2),
        (5.0, 10.1),
    ];
    let x_err = vec![0.2_f64, 0.15, 0.3, 0.1, 0.25];
    let y_err = vec![0.6_f64, 0.8, 0.4, 0.9, 0.5];

    let plot = ScatterPlot::new()
        .with_data(data)
        .with_x_err(x_err)
        .with_y_err(y_err)
        .with_color("steelblue")
        .with_size(5.0);

    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Error Bars")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/error_bars.svg"), svg).unwrap();
}

/// All six marker shapes shown together.
fn markers() {
    let y_offsets = [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0];
    let shapes = [
        (MarkerShape::Circle,   "Circle",   "steelblue"),
        (MarkerShape::Square,   "Square",   "crimson"),
        (MarkerShape::Triangle, "Triangle", "seagreen"),
        (MarkerShape::Diamond,  "Diamond",  "darkorange"),
        (MarkerShape::Cross,    "Cross",    "purple"),
        (MarkerShape::Plus,     "Plus",     "saddlebrown"),
    ];

    let plots: Vec<Plot> = shapes
        .iter()
        .zip(y_offsets.iter())
        .map(|((shape, label, color), y)| {
            let data = vec![(1.0_f64, *y), (2.0, *y), (3.0, *y)];
            Plot::Scatter(
                ScatterPlot::new()
                    .with_data(data)
                    .with_color(*color)
                    .with_size(7.0)
                    .with_marker(*shape)
                    .with_legend(*label),
            )
        })
        .collect();

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Marker Shapes")
        .with_x_label("X")
        .with_y_label("");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/markers.svg"), svg).unwrap();
}

/// Bubble plot — per-point sizes via `.with_sizes()`.
fn bubble() {
    let data = vec![
        (1.0_f64, 3.0_f64),
        (2.5, 6.5),
        (4.0, 4.0),
        (5.5, 8.0),
        (7.0, 5.5),
        (8.5, 9.0),
    ];
    let sizes = vec![5.0_f64, 14.0, 9.0, 18.0, 11.0, 7.0];

    let plot = ScatterPlot::new()
        .with_data(data)
        .with_sizes(sizes)
        .with_color("steelblue");

    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Bubble Plot")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/bubble.svg"), svg).unwrap();
}
