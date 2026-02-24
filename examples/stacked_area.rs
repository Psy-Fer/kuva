//! Stacked area plot documentation examples.
//!
//! Generates canonical SVG outputs used in the visus documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example stacked_area
//! ```
//!
//! SVGs are written to `docs/src/assets/stacked_area/`.

use visus::plot::{StackedAreaPlot, LegendPosition};
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

const OUT: &str = "docs/src/assets/stacked_area";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/stacked_area");

    basic();
    normalized();
    no_strokes();
    legend_position();

    println!("Stacked area SVGs written to {OUT}/");
}

/// Absolute stacking — monthly variant counts by type across a year.
///
/// Four series (SNVs, Indels, SVs, CNVs) are stacked so the reader can see
/// both the individual contribution of each type and the growing total.
fn basic() {
    let months: Vec<f64> = (1..=12).map(|m| m as f64).collect();

    let sa = StackedAreaPlot::new()
        .with_x(months)
        .with_series([420.0, 445.0, 398.0, 510.0, 488.0, 501.0, 467.0, 523.0, 495.0, 540.0, 518.0, 555.0])
        .with_color("steelblue").with_legend("SNVs")
        .with_series([ 95.0, 102.0,  88.0, 115.0, 108.0, 112.0,  98.0, 125.0, 118.0, 130.0, 122.0, 140.0])
        .with_color("orange").with_legend("Indels")
        .with_series([ 22.0,  25.0,  20.0,  28.0,  26.0,  27.0,  24.0,  31.0,  28.0,  33.0,  30.0,  35.0])
        .with_color("mediumseagreen").with_legend("SVs")
        .with_series([ 15.0,  17.0,  14.0,  19.0,  18.0,  18.0,  16.0,  21.0,  19.0,  23.0,  21.0,  24.0])
        .with_color("tomato").with_legend("CNVs");

    let plots = vec![Plot::StackedArea(sa)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Monthly Variant Counts by Type")
        .with_x_label("Month")
        .with_y_label("Variant count");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Normalized (100 %) stacking — same data rescaled to show proportions.
///
/// Each column sums to 100 %, shifting focus from absolute totals to the
/// changing composition of variant types across months.
fn normalized() {
    let months: Vec<f64> = (1..=12).map(|m| m as f64).collect();

    let sa = StackedAreaPlot::new()
        .with_x(months)
        .with_series([420.0, 445.0, 398.0, 510.0, 488.0, 501.0, 467.0, 523.0, 495.0, 540.0, 518.0, 555.0])
        .with_color("steelblue").with_legend("SNVs")
        .with_series([ 95.0, 102.0,  88.0, 115.0, 108.0, 112.0,  98.0, 125.0, 118.0, 130.0, 122.0, 140.0])
        .with_color("orange").with_legend("Indels")
        .with_series([ 22.0,  25.0,  20.0,  28.0,  26.0,  27.0,  24.0,  31.0,  28.0,  33.0,  30.0,  35.0])
        .with_color("mediumseagreen").with_legend("SVs")
        .with_series([ 15.0,  17.0,  14.0,  19.0,  18.0,  18.0,  16.0,  21.0,  19.0,  23.0,  21.0,  24.0])
        .with_color("tomato").with_legend("CNVs")
        .with_normalized();

    let plots = vec![Plot::StackedArea(sa)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Variant Type Composition (Normalized)")
        .with_x_label("Month")
        .with_y_label("Proportion (%)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/normalized.svg"), svg).unwrap();
}

/// No stroke lines — flat, smooth bands without top-edge outlines.
fn no_strokes() {
    let months: Vec<f64> = (1..=12).map(|m| m as f64).collect();

    let sa = StackedAreaPlot::new()
        .with_x(months)
        .with_series([420.0, 445.0, 398.0, 510.0, 488.0, 501.0, 467.0, 523.0, 495.0, 540.0, 518.0, 555.0])
        .with_color("steelblue").with_legend("SNVs")
        .with_series([ 95.0, 102.0,  88.0, 115.0, 108.0, 112.0,  98.0, 125.0, 118.0, 130.0, 122.0, 140.0])
        .with_color("orange").with_legend("Indels")
        .with_series([ 22.0,  25.0,  20.0,  28.0,  26.0,  27.0,  24.0,  31.0,  28.0,  33.0,  30.0,  35.0])
        .with_color("mediumseagreen").with_legend("SVs")
        .with_series([ 15.0,  17.0,  14.0,  19.0,  18.0,  18.0,  16.0,  21.0,  19.0,  23.0,  21.0,  24.0])
        .with_color("tomato").with_legend("CNVs")
        .with_strokes(false);

    let plots = vec![Plot::StackedArea(sa)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Without Stroke Lines")
        .with_x_label("Month")
        .with_y_label("Variant count");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/no_strokes.svg"), svg).unwrap();
}

/// Legend positioned at the bottom-left corner.
fn legend_position() {
    let months: Vec<f64> = (1..=12).map(|m| m as f64).collect();

    let sa = StackedAreaPlot::new()
        .with_x(months)
        .with_series([420.0, 445.0, 398.0, 510.0, 488.0, 501.0, 467.0, 523.0, 495.0, 540.0, 518.0, 555.0])
        .with_color("steelblue").with_legend("SNVs")
        .with_series([ 95.0, 102.0,  88.0, 115.0, 108.0, 112.0,  98.0, 125.0, 118.0, 130.0, 122.0, 140.0])
        .with_color("orange").with_legend("Indels")
        .with_series([ 22.0,  25.0,  20.0,  28.0,  26.0,  27.0,  24.0,  31.0,  28.0,  33.0,  30.0,  35.0])
        .with_color("mediumseagreen").with_legend("SVs")
        .with_series([ 15.0,  17.0,  14.0,  19.0,  18.0,  18.0,  16.0,  21.0,  19.0,  23.0,  21.0,  24.0])
        .with_color("tomato").with_legend("CNVs")
        .with_legend_position(LegendPosition::BottomLeft);

    let plots = vec![Plot::StackedArea(sa)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Legend at Bottom Left")
        .with_x_label("Month")
        .with_y_label("Variant count");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/legend_position.svg"), svg).unwrap();
}
