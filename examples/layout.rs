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
    text_annotation();
    reference_line();
    shaded_region();

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

/// Text annotation — label "Outlier" with arrow pointing to the high point.
fn text_annotation() {
    let data = vec![
        (1.0_f64, 1.2_f64), (1.8, 2.5), (2.4, 1.9), (3.1, 3.4),
        (3.8, 2.8), (4.5, 4.1), (5.2, 3.9), (6.0, 9.0),
    ];

    let plot = ScatterPlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_size(5.0);

    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Text Annotation")
        .with_x_label("X")
        .with_y_label("Y")
        .with_annotation(
            TextAnnotation::new("Outlier", 5.0, 7.5)
                .with_arrow(6.0, 9.0)
                .with_color("crimson")
                .with_font_size(12),
        );

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/text_annotation.svg"), svg).unwrap();
}

/// Reference lines — horizontal p = 0.05 threshold and vertical x = 3.5 cutoff.
fn reference_line() {
    let data = vec![
        (0.5_f64, 0.28_f64), (1.2, 0.14), (1.8, 0.04), (2.4, 0.19),
        (2.9, 0.08), (3.8, 0.21), (4.5, 0.12), (5.1, 0.07),
        (5.8, 0.25), (6.5, 0.09),
    ];

    let plot = ScatterPlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_size(5.0);

    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Reference Lines")
        .with_x_label("X")
        .with_y_label("p-value")
        .with_reference_line(
            ReferenceLine::horizontal(0.05)
                .with_color("crimson")
                .with_label("p = 0.05"),
        )
        .with_reference_line(
            ReferenceLine::vertical(3.5)
                .with_color("steelblue")
                .with_label("cutoff")
                .with_stroke_width(1.5)
                .with_dasharray("8 4"),
        );

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/reference_line.svg"), svg).unwrap();
}

/// Shaded regions — horizontal gold band and vertical blue band.
fn shaded_region() {
    let data = vec![
        (2.0_f64, 1.0_f64), (5.0, 3.2), (8.0, 0.8), (12.0, 5.5),
        (15.0, 2.1), (18.0, 6.3), (22.0, 3.8), (25.0, 1.5), (28.0, 4.7),
    ];

    let plot = ScatterPlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_size(5.0);

    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Shaded Regions")
        .with_x_label("X")
        .with_y_label("Y")
        .with_shaded_region(
            ShadedRegion::horizontal(2.0, 4.0)
                .with_color("gold")
                .with_opacity(0.2),
        )
        .with_shaded_region(
            ShadedRegion::vertical(10.0, 20.0)
                .with_color("steelblue")
                .with_opacity(0.15),
        );

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/shaded_region.svg"), svg).unwrap();
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
