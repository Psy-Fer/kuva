//! 3D surface plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example surface3d
//! ```
//!
//! SVGs are written to `docs/src/assets/surface3d/`.

use kuva::plot::surface3d::Surface3DPlot;
use kuva::plot::heatmap::ColorMap;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use std::fs;

const OUT: &str = "docs/src/assets/surface3d";

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all(OUT).unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

fn main() {
    // ── Paraboloid ─────────────────────────────────────────────────────────
    let n = 25;
    let z_data: Vec<Vec<f64>> = (0..n).map(|i| {
        (0..n).map(|j| {
            let x = (i as f64 - n as f64 / 2.0) / (n as f64 / 4.0);
            let y = (j as f64 - n as f64 / 2.0) / (n as f64 / 4.0);
            x * x + y * y
        }).collect()
    }).collect();

    let surface = Surface3DPlot::new(z_data)
        .with_z_colormap(ColorMap::Viridis)
        .with_x_label("X")
        .with_y_label("Y")
        .with_z_label("Z");

    let plots = vec![Plot::Surface3D(surface)];
    let layout = Layout::auto_from_plots(&plots).with_title("Surface3D — Paraboloid");
    write("paraboloid", plots, layout);

    // ── Wave ───────────────────────────────────────────────────────────────
    let n = 30;
    let z_data: Vec<Vec<f64>> = (0..n).map(|i| {
        (0..n).map(|j| {
            let x = i as f64 * 0.3;
            let y = j as f64 * 0.3;
            (x.sin() + y.cos()) * 2.0
        }).collect()
    }).collect();

    let surface = Surface3DPlot::new(z_data)
        .with_z_colormap(ColorMap::Inferno)
        .with_alpha(0.9);

    let plots = vec![Plot::Surface3D(surface)];
    let layout = Layout::auto_from_plots(&plots).with_title("Surface3D — Wave");
    write("wave", plots, layout);
}
