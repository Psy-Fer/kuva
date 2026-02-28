//! Bar plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example bar
//! ```
//!
//! SVGs are written to `docs/src/assets/bar/`.

use kuva::plot::BarPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

const OUT: &str = "docs/src/assets/bar";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/bar");

    basic();
    grouped();
    stacked();

    println!("Bar SVGs written to {OUT}/");
}

/// Simple bar chart — one bar per category, uniform color.
fn basic() {
    let plot = BarPlot::new()
        .with_bars(vec![
            ("Apples",  42.0),
            ("Bananas", 58.0),
            ("Cherries", 31.0),
            ("Dates",   47.0),
            ("Elderberry", 25.0),
        ])
        .with_color("steelblue");

    let plots = vec![Plot::Bar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Bar Chart")
        .with_y_label("Count");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Grouped bar chart — multiple series side-by-side within each category.
fn grouped() {
    let plot = BarPlot::new()
        .with_group("Q1", vec![(18.0, "steelblue"), (12.0, "crimson"), (9.0,  "seagreen")])
        .with_group("Q2", vec![(22.0, "steelblue"), (17.0, "crimson"), (14.0, "seagreen")])
        .with_group("Q3", vec![(19.0, "steelblue"), (21.0, "crimson"), (11.0, "seagreen")])
        .with_group("Q4", vec![(25.0, "steelblue"), (15.0, "crimson"), (18.0, "seagreen")])
        .with_legend(vec!["Product A", "Product B", "Product C"]);

    let plots = vec![Plot::Bar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Grouped Bar Chart")
        .with_y_label("Sales (units)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/grouped.svg"), svg).unwrap();
}

/// Stacked bar chart — series stacked vertically within each category.
fn stacked() {
    let plot = BarPlot::new()
        .with_group("Q1", vec![(18.0, "steelblue"), (12.0, "crimson"), (9.0,  "seagreen")])
        .with_group("Q2", vec![(22.0, "steelblue"), (17.0, "crimson"), (14.0, "seagreen")])
        .with_group("Q3", vec![(19.0, "steelblue"), (21.0, "crimson"), (11.0, "seagreen")])
        .with_group("Q4", vec![(25.0, "steelblue"), (15.0, "crimson"), (18.0, "seagreen")])
        .with_legend(vec!["Product A", "Product B", "Product C"])
        .with_stacked();

    let plots = vec![Plot::Bar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Stacked Bar Chart")
        .with_y_label("Sales (units)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/stacked.svg"), svg).unwrap();
}
