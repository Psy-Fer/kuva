//! Polar plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example polar
//! ```
//!
//! SVGs are written to `docs/src/assets/polar/`.

use kuva::plot::polar::{PolarMode, PolarPlot};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::backend::svg::SvgBackend;

const OUT: &str = "docs/src/assets/polar";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/polar");

    basic();
    marker_density();

    println!("Polar SVGs written to {OUT}/");
}

/// Cardioid line curve + reference unit circle.
fn basic() {
    let n = 72;
    let theta_cardioid: Vec<f64> = (0..n).map(|i| i as f64 * 360.0 / n as f64).collect();
    let r_cardioid: Vec<f64> = theta_cardioid
        .iter()
        .map(|&t| 1.0 + t.to_radians().cos())
        .collect();

    let theta_circle: Vec<f64> = (0..=n).map(|i| i as f64 * 360.0 / n as f64).collect();
    let r_circle: Vec<f64> = vec![1.0; theta_circle.len()];

    let plot = PolarPlot::new()
        .with_series_labeled(r_cardioid, theta_cardioid, "Cardioid", PolarMode::Line)
        .with_series_labeled(r_circle, theta_circle, "Unit circle", PolarMode::Line)
        .with_r_max(2.1)
        .with_r_grid_lines(4)
        .with_theta_divisions(12)
        .with_legend(true);

    let plots = vec![Plot::Polar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Polar Plot");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// 500 scattered observations dominated by two directional modes (NE and SW).
///
/// With solid markers, each direction becomes an opaque wedge — the internal
/// spread and any secondary structure disappear. Semi-transparent markers with
/// a thin stroke reveal the density gradient and let individual points show
/// through even where hundreds overlap.
fn marker_density() {
    let mut seed: u64 = 3_141_592_653;
    let mut lcg = || -> f64 {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (seed >> 33) as f64 / ((1u64 << 31) as f64)
    };
    let gauss = |lcg: &mut dyn FnMut() -> f64| -> f64 {
        let u1 = lcg().max(1e-10);
        let u2 = lcg();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    };

    // Two dominant directions: NE (45°) and SW (225°), 250 pts each.
    let mut r_vals: Vec<f64> = Vec::new();
    let mut t_vals: Vec<f64> = Vec::new();
    for &dir in &[45.0_f64, 225.0] {
        for _ in 0..250 {
            let r = (0.7 + gauss(&mut lcg) * 0.12).clamp(0.1, 1.1);
            let t = (dir + gauss(&mut lcg) * 22.0).rem_euclid(360.0);
            r_vals.push(r);
            t_vals.push(t);
        }
    }

    let plot = PolarPlot::new()
        .with_series(r_vals, t_vals)
        .with_color("steelblue")
        .with_marker_opacity(0.2)
        .with_marker_stroke_width(0.7)
        .with_r_max(1.2)
        .with_theta_divisions(24);

    let plots = vec![Plot::Polar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Directional scatter — semi-transparent markers (500 pts)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/marker_density.svg"), svg).unwrap();
}
