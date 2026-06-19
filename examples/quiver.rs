//! Quiver plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example quiver
//! ```
//!
//! SVGs are written to `docs/src/assets/quiver/`.

use kuva::backend::svg::SvgBackend;
use kuva::plot::ColorMap;
use kuva::plot::QuiverPivot;
use kuva::plot::QuiverPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use std::fs;

const OUT: &str = "docs/src/assets/quiver";

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all(OUT).unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

fn main() {
    // ── Basic: rotational field ────────────────────────────────────────────
    let plot =
        QuiverPlot::from_function((-5.0, 5.0, 10), (-5.0, 5.0, 10), |x, y| (-y * 0.3, x * 0.3))
            .with_auto_scale(0.85)
            .with_pivot(QuiverPivot::Middle)
            .with_color("steelblue");
    let plots = vec![Plot::Quiver(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Rotational Field")
        .with_x_label("x")
        .with_y_label("y");
    write("basic", plots, layout);

    // ── Saddle point colored by magnitude ──────────────────────────────────
    let plot =
        QuiverPlot::from_function((-5.0, 5.0, 12), (-5.0, 5.0, 12), |x, y| (x * 0.4, -y * 0.4))
            .with_auto_scale(0.8)
            .with_pivot(QuiverPivot::Middle)
            .with_magnitude_colormap(ColorMap::Viridis, "Speed");
    let plots = vec![Plot::Quiver(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Saddle Point (u = x, v = -y)")
        .with_x_label("x")
        .with_y_label("y");
    write("colormap", plots, layout);

    // ── Divergent source (u = x, v = y) ────────────────────────────────────
    let plot = QuiverPlot::from_function((-5.0, 5.0, 10), (-5.0, 5.0, 10), |x, y| {
        (x * 0.25, y * 0.25)
    })
    .with_auto_scale(0.8)
    .with_pivot(QuiverPivot::Middle)
    .with_color_map(ColorMap::Plasma)
    .with_color_legend_label("|v|")
    .with_shaft_width(1.4);
    let plots = vec![Plot::Quiver(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Divergent Source")
        .with_x_label("x")
        .with_y_label("y");
    write("source", plots, layout);
}
