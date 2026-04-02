//! Forest plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example forest
//! ```
//!
//! SVGs are written to `docs/src/assets/forest/`.

use kuva::plot::ForestPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

const OUT: &str = "docs/src/assets/forest";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/forest");

    basic();
    weighted();

    println!("Forest SVGs written to {OUT}/");
}

/// Basic forest plot — 8 studies + an overall summary row.
fn basic() {
    let forest = ForestPlot::new()
        .with_row("Smith 2019",    0.50,  0.10, 0.90)
        .with_row("Johnson 2020", -0.30, -0.80, 0.20)
        .with_row("Williams 2020", 0.20, -0.10, 0.50)
        .with_row("Brown 2021",    0.65,  0.30, 1.00)
        .with_row("Davis 2021",   -0.10, -0.50, 0.30)
        .with_row("Miller 2022",   0.35,  0.05, 0.65)
        .with_row("Wilson 2022",   0.80,  0.40, 1.20)
        .with_row("Moore 2023",    0.15, -0.20, 0.50)
        .with_row("Overall",       0.28,  0.10, 0.46)
        .with_null_value(0.0);

    let plots = vec![Plot::Forest(forest)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Meta-Analysis: Treatment Effect")
        .with_x_label("Effect Size (95% CI)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Weighted forest plot — marker radius scales with study weight.
fn weighted() {
    let forest = ForestPlot::new()
        .with_weighted_row("Smith 2019",    0.50,  0.10, 0.90, 5.2)
        .with_weighted_row("Johnson 2020", -0.30, -0.80, 0.20, 3.8)
        .with_weighted_row("Williams 2020", 0.20, -0.10, 0.50, 8.1)
        .with_weighted_row("Brown 2021",    0.65,  0.30, 1.00, 6.0)
        .with_weighted_row("Davis 2021",   -0.10, -0.50, 0.30, 4.5)
        .with_weighted_row("Miller 2022",   0.35,  0.05, 0.65, 7.3)
        .with_weighted_row("Wilson 2022",   0.80,  0.40, 1.20, 3.2)
        .with_weighted_row("Moore 2023",    0.15, -0.20, 0.50, 9.0)
        .with_weighted_row("Overall",       0.28,  0.10, 0.46, 10.0)
        .with_null_value(0.0)
        .with_marker_size(6.0);

    let plots = vec![Plot::Forest(forest)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Meta-Analysis: Weighted Markers")
        .with_x_label("Effect Size (95% CI)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/weighted.svg"), svg).unwrap();
}
