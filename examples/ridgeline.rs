//! Ridgeline plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example ridgeline
//! ```
//!
//! SVGs are written to `docs/src/assets/ridgeline/`.

use kuva::plot::ridgeline::RidgelinePlot;
use kuva::render::plots::Plot;
use kuva::render::layout::Layout;
use kuva::render::render::render_multiple;
use kuva::backend::svg::SvgBackend;

const OUT: &str = "docs/src/assets/ridgeline";

// ── Deterministic data generators ─────────────────────────────────────────────

fn lcg(state: &mut u64) -> f64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((*state >> 33) as f64) / (u32::MAX as f64)
}

fn gaussian(state: &mut u64, mean: f64, std: f64) -> f64 {
    let u1 = lcg(state).max(1e-10);
    let u2 = lcg(state);
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
    mean + z * std
}

fn make_gaussian(seed: u64, n: usize, mean: f64, std: f64) -> Vec<f64> {
    let mut state = seed;
    (0..n).map(|_| gaussian(&mut state, mean, std)).collect()
}

// ── Examples ───────────────────────────────────────────────────────────────────

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/ridgeline");

    basic();
    temperature();

    println!("Ridgeline SVGs written to {OUT}/");
}

/// Seasonal temperature distributions — cold-to-warm gradient across months.
fn temperature() {
    const MONTHS: [(&str, f64, f64); 12] = [
        ("January",   -3.0, 5.0),
        ("February",  -1.5, 5.5),
        ("March",      4.0, 5.0),
        ("April",     10.0, 4.0),
        ("May",       15.5, 3.5),
        ("June",      20.0, 3.0),
        ("July",      23.0, 2.5),
        ("August",    22.5, 2.5),
        ("September", 17.0, 3.0),
        ("October",   10.5, 4.0),
        ("November",   3.5, 5.0),
        ("December",  -1.0, 5.5),
    ];

    let colors = [
        "#3a7abf", "#4589c4", "#6ba3d4", "#a0bfdc",
        "#d4b8a0", "#e8c97a", "#f0a830", "#e86820",
        "#d44a10", "#c06030", "#9070a0", "#5060b0",
    ];

    let mut plot = RidgelinePlot::new()
        .with_overlap(0.6)
        .with_opacity(0.75);

    for (i, &(month, mean, std)) in MONTHS.iter().enumerate() {
        let data = make_gaussian(i as u64 + 1, 200, mean, std);
        plot = plot.with_group_color(month, data, colors[i]);
    }

    let plots = vec![Plot::Ridgeline(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Daily Temperature Distributions by Month")
        .with_x_label("Temperature (°C)")
        .with_y_label("Month");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/temperature.svg"), svg).unwrap();
}

/// Basic ridgeline with 3 groups.
fn basic() {
    let plot = RidgelinePlot::new()
        .with_group("Control",     vec![1.2, 1.5, 1.8, 2.0, 2.2, 1.9, 1.6, 1.3])
        .with_group("Treatment A", vec![2.5, 3.0, 3.5, 4.0, 3.8, 3.2, 2.8, 3.6])
        .with_group("Treatment B", vec![4.5, 5.0, 5.5, 6.0, 5.8, 5.2, 4.8, 5.3]);

    let plots = vec![Plot::Ridgeline(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Expression by Treatment")
        .with_x_label("Expression Level")
        .with_y_label("Group");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}
