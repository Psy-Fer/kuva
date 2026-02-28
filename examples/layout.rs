//! Layout & Axes documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example layout
//! ```
//!
//! SVGs are written to `docs/src/assets/layout/`.

use kuva::plot::scatter::ScatterPlot;
use kuva::plot::LinePlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::annotations::{TextAnnotation, ReferenceLine, ShadedRegion};
use kuva::TickFormat;

const OUT: &str = "docs/src/assets/layout";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/layout");

    log_scale();
    tick_formats();
    annotations();

    println!("Layout SVGs written to {OUT}/");
}

/// Log-scale axes — wide-range data that would be unreadable on linear axes.
fn log_scale() {
    let data: Vec<(f64, f64)> = vec![
        (1.0, 0.001), (3.0, 0.02), (10.0, 0.5),
        (30.0, 8.0),  (100.0, 150.0), (300.0, 3_000.0),
        (1000.0, 60_000.0),
    ];

    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_size(5.0);

    let plots = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plots)
        .with_log_scale()
        .with_title("Log-Scale Axes")
        .with_x_label("X (log₁₀)")
        .with_y_label("Y (log₁₀)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/log_scale.svg"), svg).unwrap();
}

/// Four subplots illustrating different TickFormat variants.
/// Each is written as a separate SVG; combined in the docs via a table.
fn tick_formats() {
    let data_linear = vec![(0.0_f64, 0.0_f64), (0.25, 0.25), (0.5, 0.5), (0.75, 0.75), (1.0, 1.0)];
    let data_large  = vec![(0.0_f64, 0.0_f64), (25_000.0, 50_000.0), (50_000.0, 100_000.0)];

    // Auto (default)
    {
        let plot = ScatterPlot::new().with_data(data_linear.clone()).with_color("steelblue").with_size(5.0);
        let plots = vec![Plot::Scatter(plot)];
        let layout = Layout::auto_from_plots(&plots).with_title("Auto");
        let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
        std::fs::write(format!("{OUT}/tick_auto.svg"), svg).unwrap();
    }

    // Fixed(2) — always 2 decimal places
    {
        let plot = ScatterPlot::new().with_data(data_linear.clone()).with_color("steelblue").with_size(5.0);
        let plots = vec![Plot::Scatter(plot)];
        let layout = Layout::auto_from_plots(&plots)
            .with_title("Fixed(2)")
            .with_tick_format(TickFormat::Fixed(2));
        let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
        std::fs::write(format!("{OUT}/tick_fixed.svg"), svg).unwrap();
    }

    // Percent — multiplies by 100 and appends %
    {
        let plot = ScatterPlot::new().with_data(data_linear.clone()).with_color("steelblue").with_size(5.0);
        let plots = vec![Plot::Scatter(plot)];
        let layout = Layout::auto_from_plots(&plots)
            .with_title("Percent")
            .with_tick_format(TickFormat::Percent);
        let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
        std::fs::write(format!("{OUT}/tick_percent.svg"), svg).unwrap();
    }

    // Sci — scientific notation
    {
        let plot = ScatterPlot::new().with_data(data_large.clone()).with_color("steelblue").with_size(5.0);
        let plots = vec![Plot::Scatter(plot)];
        let layout = Layout::auto_from_plots(&plots)
            .with_title("Sci")
            .with_tick_format(TickFormat::Sci);
        let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
        std::fs::write(format!("{OUT}/tick_sci.svg"), svg).unwrap();
    }
}

/// Text annotation with arrow, reference lines, and a shaded region — all combined.
fn annotations() {
    let xs: Vec<f64> = (0..=60).map(|i| i as f64 * 0.1).collect();
    let line = LinePlot::new()
        .with_data(xs.iter().map(|&x| (x, x.sin())))
        .with_color("steelblue")
        .with_stroke_width(2.0);

    let plots = vec![Plot::Line(line)];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Annotations")
        .with_x_label("X")
        .with_y_label("Y")
        // Shade the region where y is positive
        .with_shaded_region(
            ShadedRegion::horizontal(0.0, 1.1)
                .with_color("steelblue")
                .with_opacity(0.08),
        )
        // Horizontal reference line at y = 0
        .with_reference_line(
            ReferenceLine::horizontal(0.0)
                .with_color("black")
                .with_label("y = 0"),
        )
        // Vertical reference line at the first peak
        .with_reference_line(
            ReferenceLine::vertical(1.57)
                .with_color("crimson")
                .with_label("π/2"),
        )
        // Text annotation pointing at the peak
        .with_annotation(
            TextAnnotation::new("peak", 1.0, 1.15)
                .with_arrow(1.57, 1.0)
                .with_color("crimson"),
        );

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/annotations.svg"), svg).unwrap();
}
