//! 3D quiver plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example quiver3d
//! ```
//!
//! SVGs are written to `docs/src/assets/quiver3d/`.

use kuva::backend::svg::SvgBackend;
use kuva::plot::ColorMap;
use kuva::plot::Quiver3DPlot;
use kuva::plot::QuiverPivot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use std::fs;

const OUT: &str = "docs/src/assets/quiver3d";

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all(OUT).unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

/// Tornado funnel: rings of arrows stacked up the z axis, radius growing
/// with height, each arrow swirling tangentially with a steady updraft.
/// Arrows per ring scale with the circumference so the field is evenly
/// spaced from the narrow base to the wide top.
fn tornado() -> Vec<(f64, f64, f64, f64, f64, f64)> {
    let mut arrows = Vec::new();
    let rings = 8;
    for k in 0..rings {
        let z = k as f64;
        let r = 0.5 + 0.4 * z;
        let per_ring = ((std::f64::consts::TAU * r / 0.9).round() as usize).max(4);
        for i in 0..per_ring {
            let theta = i as f64 / per_ring as f64 * std::f64::consts::TAU;
            let (x, y) = (r * theta.cos(), r * theta.sin());
            // Tangential swirl (counter-clockwise from above) plus updraft;
            // the core spins a little faster than the rim.
            let speed = 1.3 - 0.05 * z;
            let (u, v) = (-theta.sin() * speed, theta.cos() * speed);
            arrows.push((x, y, z, u, v, 0.6));
        }
    }
    arrows
}

fn main() {
    // ── Tornado funnel ─────────────────────────────────────────────────────
    let plot = Quiver3DPlot::new()
        .with_arrows(tornado())
        .with_pivot(QuiverPivot::Middle)
        .with_auto_scale(0.7)
        .with_color("steelblue")
        .with_x_label("x")
        .with_y_label("y")
        .with_z_label("height");
    let plots = vec![Plot::Quiver3D(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Tornado Funnel");
    write("tornado", plots, layout);

    // ── Vortex on a grid, colored by magnitude ─────────────────────────────
    let plot =
        Quiver3DPlot::from_function((-2.0, 2.0, 5), (-2.0, 2.0, 5), (0.0, 4.0, 5), |x, y, z| {
            let swirl = 1.0 + 0.3 * z;
            (-y * swirl, x * swirl, 0.8)
        })
        .with_pivot(QuiverPivot::Middle)
        .with_magnitude_colormap(ColorMap::Viridis, "Speed")
        .with_x_label("x")
        .with_y_label("y")
        .with_z_label("z");
    let plots = vec![Plot::Quiver3D(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Vortex — colored by |v|");
    write("magnitude", plots, layout);

    // ── Radial source with depth shading, alternate view ───────────────────
    let plot =
        Quiver3DPlot::from_function((-1.0, 1.0, 4), (-1.0, 1.0, 4), (-1.0, 1.0, 4), |x, y, z| {
            (x, y, z)
        })
        .with_depth_shade()
        .with_color("crimson")
        .with_azimuth(-120.0)
        .with_elevation(20.0)
        .with_x_label("x")
        .with_y_label("y")
        .with_z_label("z");
    let plots = vec![Plot::Quiver3D(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Radial Source — depth shaded");
    write("depth_shade", plots, layout);
}
