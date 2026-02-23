//! Box plot documentation examples.
//!
//! Generates canonical SVG outputs used in the visus documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example boxplot
//! ```
//!
//! SVGs are written to `docs/src/assets/boxplot/`.

use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use visus::plot::BoxPlot;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

const OUT: &str = "docs/src/assets/boxplot";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/boxplot");

    basic();
    strip_overlay();
    swarm_overlay();

    println!("Box plot SVGs written to {OUT}/");
}

fn samples(mean: f64, std: f64, n: usize, seed: u64) -> Vec<f64> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    Normal::new(mean, std).unwrap()
        .sample_iter(&mut rng)
        .take(n)
        .collect()
}

/// Four groups of normally-distributed data.
fn basic() {
    let plot = BoxPlot::new()
        .with_group("Control",    samples(5.0, 1.0, 60, 1))
        .with_group("Treatment A", samples(6.5, 1.2, 60, 2))
        .with_group("Treatment B", samples(4.2, 0.9, 60, 3))
        .with_group("Treatment C", samples(7.1, 1.5, 60, 4))
        .with_color("steelblue");

    let plots = vec![Plot::Box(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Box Plot")
        .with_y_label("Value");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Boxes with a jittered strip overlay showing individual points.
fn strip_overlay() {
    let plot = BoxPlot::new()
        .with_group("Control",    samples(5.0, 1.0, 60, 1))
        .with_group("Treatment A", samples(6.5, 1.2, 60, 2))
        .with_group("Treatment B", samples(4.2, 0.9, 60, 3))
        .with_group("Treatment C", samples(7.1, 1.5, 60, 4))
        .with_color("steelblue")
        .with_strip(0.2)                        // jitter width
        .with_overlay_color("rgba(0,0,0,0.4)")  // semi-transparent points
        .with_overlay_size(3.0);

    let plots = vec![Plot::Box(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Box Plot + Strip Overlay")
        .with_y_label("Value");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/strip_overlay.svg"), svg).unwrap();
}

/// Boxes with a beeswarm overlay — points spread to avoid overlap.
fn swarm_overlay() {
    let plot = BoxPlot::new()
        .with_group("Control",    samples(5.0, 1.0, 60, 1))
        .with_group("Treatment A", samples(6.5, 1.2, 60, 2))
        .with_group("Treatment B", samples(4.2, 0.9, 60, 3))
        .with_group("Treatment C", samples(7.1, 1.5, 60, 4))
        .with_color("steelblue")
        .with_swarm_overlay()
        .with_overlay_color("rgba(0,0,0,0.4)")
        .with_overlay_size(3.0);

    let plots = vec![Plot::Box(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Box Plot + Swarm Overlay")
        .with_y_label("Value");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/swarm_overlay.svg"), svg).unwrap();
}
