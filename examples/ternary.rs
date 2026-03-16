//! Ternary plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example ternary
//! ```
//!
//! SVGs are written to `docs/src/assets/ternary/`.

use kuva::plot::ternary::TernaryPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::backend::svg::SvgBackend;

const OUT: &str = "docs/src/assets/ternary";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/ternary");

    basic();
    marker_density();

    println!("Ternary SVGs written to {OUT}/");
}

/// Three groups clustering near each vertex — small dataset, solid markers.
fn basic() {
    let mut plot = TernaryPlot::new()
        .with_corner_labels("A", "B", "C")
        .with_grid_lines(5)
        .with_legend(true);

    for (a, b, c) in [
        (0.75, 0.15, 0.10), (0.80, 0.12, 0.08),
        (0.70, 0.18, 0.12), (0.82, 0.10, 0.08), (0.68, 0.20, 0.12),
    ] {
        plot = plot.with_point_group(a, b, c, "A-rich");
    }
    for (a, b, c) in [
        (0.12, 0.75, 0.13), (0.10, 0.80, 0.10),
        (0.15, 0.72, 0.13), (0.08, 0.82, 0.10), (0.13, 0.70, 0.17),
    ] {
        plot = plot.with_point_group(a, b, c, "B-rich");
    }
    for (a, b, c) in [
        (0.10, 0.12, 0.78), (0.08, 0.10, 0.82),
        (0.13, 0.15, 0.72), (0.09, 0.09, 0.82), (0.12, 0.18, 0.70),
    ] {
        plot = plot.with_point_group(a, b, c, "C-rich");
    }

    let plots = vec![Plot::Ternary(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Ternary Plot");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Soil texture diagram: four compositional classes, 100 points each.
///
/// At 400 total points the clusters overlap at their boundaries. Solid markers
/// obscure whether a boundary sample belongs to one class or straddles two.
/// Semi-transparent fill + stroke makes overlap regions visibly darker while
/// keeping individual boundary points distinguishable.
fn marker_density() {
    let mut seed: u64 = 2_718_281_828;
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

    // (center_a, center_b, center_c, label)
    let classes = [
        (0.70, 0.20, 0.10, "Sandy loam"),
        (0.15, 0.70, 0.15, "Silt loam"),
        (0.15, 0.15, 0.70, "Clay"),
        (1.0_f64 / 3.0, 1.0 / 3.0, 1.0 / 3.0, "Loam"),
    ];

    let mut plot = TernaryPlot::new()
        .with_corner_labels("Sand", "Silt", "Clay")
        .with_normalize(true)
        .with_legend(true)
        .with_marker_size(5.0)
        .with_marker_opacity(0.3)
        .with_marker_stroke_width(0.8);

    for &(ca, cb, cc, label) in &classes {
        for _ in 0..100 {
            let a = (ca + gauss(&mut lcg) * 0.07).max(0.0);
            let b = (cb + gauss(&mut lcg) * 0.07).max(0.0);
            let c = (cc + gauss(&mut lcg) * 0.07).max(0.0);
            plot = plot.with_point_group(a, b, c, label);
        }
    }

    let plots = vec![Plot::Ternary(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Soil texture — semi-transparent markers (400 pts)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/marker_density.svg"), svg).unwrap();
}
