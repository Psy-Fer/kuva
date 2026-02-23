//! Violin plot documentation examples.
//!
//! Generates canonical SVG outputs used in the visus documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example violin
//! ```
//!
//! SVGs are written to `docs/src/assets/violin/`.

use rand::SeedableRng;
use rand_distr::{Distribution, Normal, Exp};
use visus::plot::ViolinPlot;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

const OUT: &str = "docs/src/assets/violin";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/violin");

    basic();
    bandwidth();
    swarm_overlay();

    println!("Violin SVGs written to {OUT}/");
}

fn normal_samples(mean: f64, std: f64, n: usize, seed: u64) -> Vec<f64> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    Normal::new(mean, std).unwrap()
        .sample_iter(&mut rng)
        .take(n)
        .collect()
}

fn bimodal_samples(mean1: f64, mean2: f64, std: f64, n: usize, seed: u64) -> Vec<f64> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    let d1 = Normal::new(mean1, std).unwrap();
    let d2 = Normal::new(mean2, std).unwrap();
    let half = n / 2;
    d1.sample_iter(&mut rng.clone()).take(half)
        .chain(d2.sample_iter(&mut rng).take(n - half))
        .collect()
}

fn skewed_samples(n: usize, seed: u64) -> Vec<f64> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    Exp::new(0.8_f64).unwrap()
        .sample_iter(&mut rng)
        .take(n)
        .collect()
}

/// Three groups with distinct distribution shapes — the core use-case for violins.
fn basic() {
    let plot = ViolinPlot::new()
        .with_group("Normal",  normal_samples(0.0, 1.0, 500, 1))
        .with_group("Bimodal", bimodal_samples(-2.0, 2.0, 0.6, 500, 2))
        .with_group("Skewed",  skewed_samples(500, 3))
        .with_color("steelblue")
        .with_width(30.0);

    let plots = vec![Plot::Violin(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Violin Plot")
        .with_y_label("Value");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Effect of KDE bandwidth on shape smoothness.
fn bandwidth() {
    let data = normal_samples(0.0, 1.0, 300, 42);

    for (bw, name) in [(0.15_f64, "narrow"), (0.5, "auto"), (2.0, "wide")] {
        let mut plot = ViolinPlot::new()
            .with_group("", data.clone())
            .with_color("steelblue")
            .with_width(30.0);
        if name == "auto" {
            // leave bandwidth unset — Silverman's rule
        } else {
            plot = plot.with_bandwidth(bw);
        }

        let title = match name {
            "narrow" => "Narrow (h = 0.15)",
            "wide"   => "Wide (h = 2.0)",
            _        => "Auto (Silverman)",
        };
        let plots = vec![Plot::Violin(plot)];
        let layout = Layout::auto_from_plots(&plots).with_title(title);
        let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
        std::fs::write(format!("{OUT}/bandwidth_{name}.svg"), svg).unwrap();
    }
}

/// Bimodal violin with a beeswarm overlay showing individual points.
fn swarm_overlay() {
    let plot = ViolinPlot::new()
        .with_group("Normal",  normal_samples(0.0, 1.0, 120, 1))
        .with_group("Bimodal", bimodal_samples(-2.0, 2.0, 0.6, 120, 2))
        .with_group("Skewed",  skewed_samples(120, 3))
        .with_color("steelblue")
        .with_width(30.0)
        .with_swarm_overlay()
        .with_overlay_color("rgba(0,0,0,0.35)")
        .with_overlay_size(2.5);

    let plots = vec![Plot::Violin(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Violin + Swarm Overlay")
        .with_y_label("Value");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/swarm_overlay.svg"), svg).unwrap();
}
