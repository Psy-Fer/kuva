//! 3D scatter plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example scatter3d
//! ```
//!
//! SVGs are written to `docs/src/assets/scatter3d/`.

use kuva::plot::scatter3d::Scatter3DPlot;
use kuva::plot::heatmap::ColorMap;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use std::fs;

const OUT: &str = "docs/src/assets/scatter3d";

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all(OUT).unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

fn main() {
    // ── Basic ──────────────────────────────────────────────────────────────
    let data: Vec<(f64, f64, f64)> = (0..50)
        .map(|i| {
            let t = i as f64 / 49.0 * std::f64::consts::TAU;
            (t.cos() * (1.0 + t * 0.3), t.sin() * (1.0 + t * 0.3), t)
        })
        .collect();

    let plot = Scatter3DPlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_x_label("X")
        .with_y_label("Y")
        .with_z_label("Z");

    let plots = vec![Plot::Scatter3D(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("3D Scatter — Helix");
    write("basic", plots, layout);

    // ── Z-colored ──────────────────────────────────────────────────────────
    let data: Vec<(f64, f64, f64)> = (0..100)
        .map(|i| {
            let t = i as f64 / 99.0;
            let x = t * 10.0 - 5.0;
            let y = (t * 6.0).sin() * 3.0;
            let z = x * x + y * y;
            (x, y, z)
        })
        .collect();

    let plot = Scatter3DPlot::new()
        .with_data(data)
        .with_z_colormap(ColorMap::Viridis)
        .with_x_label("X")
        .with_y_label("Y")
        .with_z_label("Z = X² + Y²");

    let plots = vec![Plot::Scatter3D(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("3D Scatter — Z Colormap");
    write("zcolor", plots, layout);

    // ── Custom view ────────────────────────────────────────────────────────
    let data: Vec<(f64, f64, f64)> = (0..30)
        .map(|i| {
            let t = i as f64 / 29.0;
            (t * 10.0, (t * 4.0).sin() * 5.0, (t * 3.0).cos() * 5.0)
        })
        .collect();

    let plot = Scatter3DPlot::new()
        .with_data(data)
        .with_color("crimson")
        .with_azimuth(-120.0)
        .with_elevation(20.0)
        .with_depth_shade(true)
        .with_x_label("X")
        .with_y_label("Y")
        .with_z_label("Z");

    let plots = vec![Plot::Scatter3D(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("3D Scatter — Custom View");
    write("view", plots, layout);
}
