//! Pie chart documentation examples.
//!
//! Generates canonical SVG outputs used in the visus documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example pie
//! ```
//!
//! SVGs are written to `docs/src/assets/pie/`.

use visus::plot::{PiePlot, PieLabelPosition};
use visus::backend::svg::SvgBackend;
use visus::render::render::{render_pie, render_multiple};
use visus::render::layout::Layout;
use visus::render::plots::Plot;

const OUT: &str = "docs/src/assets/pie";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/pie");

    basic();
    donut();
    percent();
    outside_labels();
    legend();

    println!("Pie SVGs written to {OUT}/");
}

/// Simple pie chart — four slices, default Auto label positioning.
fn basic() {
    let pie = PiePlot::new()
        .with_slice("Rust",   40.0, "steelblue")
        .with_slice("Python", 30.0, "tomato")
        .with_slice("R",      20.0, "seagreen")
        .with_slice("Other",  10.0, "gold");

    let plots = vec![Plot::Pie(pie.clone())];
    let layout = Layout::auto_from_plots(&plots).with_title("Pie Chart");

    let svg = SvgBackend.render_scene(&render_pie(&pie, &layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Donut chart — same slices with a hollow center.
fn donut() {
    let pie = PiePlot::new()
        .with_slice("Rust",   40.0, "steelblue")
        .with_slice("Python", 30.0, "tomato")
        .with_slice("R",      20.0, "seagreen")
        .with_slice("Other",  10.0, "gold")
        .with_inner_radius(60.0);

    let plots = vec![Plot::Pie(pie.clone())];
    let layout = Layout::auto_from_plots(&plots).with_title("Donut Chart");

    let svg = SvgBackend.render_scene(&render_pie(&pie, &layout));
    std::fs::write(format!("{OUT}/donut.svg"), svg).unwrap();
}

/// Percentage labels — appends the percentage to each slice label.
fn percent() {
    let pie = PiePlot::new()
        .with_slice("Rust",   40.0, "steelblue")
        .with_slice("Python", 30.0, "tomato")
        .with_slice("R",      20.0, "seagreen")
        .with_slice("Other",  10.0, "gold")
        .with_percent();

    let plots = vec![Plot::Pie(pie.clone())];
    let layout = Layout::auto_from_plots(&plots).with_title("Pie Chart – Percentages");

    let svg = SvgBackend.render_scene(&render_pie(&pie, &layout));
    std::fs::write(format!("{OUT}/percent.svg"), svg).unwrap();
}

/// Outside labels — explicit leader lines, useful when slices vary widely in size.
fn outside_labels() {
    let pie = PiePlot::new()
        .with_slice("Apples",  30.0, "seagreen")
        .with_slice("Oranges", 25.0, "darkorange")
        .with_slice("Bananas", 20.0, "gold")
        .with_slice("Grapes",  12.0, "mediumpurple")
        .with_slice("Mango",    8.0, "coral")
        .with_slice("Kiwi",     5.0, "olivedrab")
        .with_label_position(PieLabelPosition::Outside);

    let plots = vec![Plot::Pie(pie.clone())];
    let layout = Layout::auto_from_plots(&plots).with_title("Outside Labels");

    let svg = SvgBackend.render_scene(&render_pie(&pie, &layout));
    std::fs::write(format!("{OUT}/outside_labels.svg"), svg).unwrap();
}

/// Legend instead of slice labels — use render_multiple so the legend renders.
fn legend() {
    let pie = PiePlot::new()
        .with_slice("Apples",  40.0, "seagreen")
        .with_slice("Oranges", 35.0, "darkorange")
        .with_slice("Grapes",  25.0, "mediumpurple")
        .with_legend("Fruit")
        .with_percent()
        .with_label_position(PieLabelPosition::None);

    let plots = vec![Plot::Pie(pie)];
    let layout = Layout::auto_from_plots(&plots).with_title("Pie with Legend");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/legend.svg"), svg).unwrap();
}
