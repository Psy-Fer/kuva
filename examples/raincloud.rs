//! Raincloud plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example raincloud
//! ```
//!
//! SVGs are written to `docs/src/assets/raincloud/`.

use kuva::backend::svg::SvgBackend;
use kuva::plot::RaincloudPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use rand::SeedableRng;
use rand_distr::{Distribution, Exp, Normal};

const OUT: &str = "docs/src/assets/raincloud";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/raincloud");

    basic();
    flipped();
    group_colors();
    bandwidth_scale();
    elements();
    horizontal();

    println!("Raincloud SVGs written to {OUT}/");
}

fn normal_samples(mean: f64, std: f64, n: usize, seed: u64) -> Vec<f64> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    Normal::new(mean, std)
        .unwrap()
        .sample_iter(&mut rng)
        .take(n)
        .collect()
}

fn bimodal_samples(mean1: f64, mean2: f64, std: f64, n: usize, seed: u64) -> Vec<f64> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    let d1 = Normal::new(mean1, std).unwrap();
    let d2 = Normal::new(mean2, std).unwrap();
    let half = n / 2;
    d1.sample_iter(&mut rng.clone())
        .take(half)
        .chain(d2.sample_iter(&mut rng).take(n - half))
        .collect()
}

fn skewed_samples(shift: f64, n: usize, seed: u64) -> Vec<f64> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    Exp::new(1.2_f64)
        .unwrap()
        .sample_iter(&mut rng)
        .take(n)
        .map(|v| v + shift)
        .collect()
}

/// Three groups with distinct distribution shapes — the canonical raincloud use-case.
fn basic() {
    let plot = RaincloudPlot::new()
        .with_group("Normal", normal_samples(5.0, 1.0, 150, 1))
        .with_group("Bimodal", bimodal_samples(3.0, 7.0, 0.7, 150, 2))
        .with_group("Skewed", skewed_samples(2.0, 150, 3));

    let plots = vec![Plot::Raincloud(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Raincloud Plot")
        .with_x_label("Group")
        .with_y_label("Value");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Cloud left, rain right — `.with_flip(true)`.
fn flipped() {
    let plot = RaincloudPlot::new()
        .with_group("Normal", normal_samples(5.0, 1.0, 150, 1))
        .with_group("Bimodal", bimodal_samples(3.0, 7.0, 0.7, 150, 2))
        .with_group("Skewed", skewed_samples(2.0, 150, 3))
        .with_flip(true);

    let plots = vec![Plot::Raincloud(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Raincloud Plot — Flipped")
        .with_x_label("Group")
        .with_y_label("Value");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/flipped.svg"), svg).unwrap();
}

/// Per-group colors via `.with_group_colors()`.
fn group_colors() {
    let plot = RaincloudPlot::new()
        .with_group("Control", normal_samples(5.0, 1.0, 150, 10))
        .with_group("Low dose", normal_samples(6.2, 1.1, 150, 11))
        .with_group("High dose", normal_samples(7.8, 0.9, 150, 12))
        .with_group_colors(["#4878d0", "#ee854a", "#6acc65"]);

    let plots = vec![Plot::Raincloud(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Raincloud Plot — Per-group Colors")
        .with_x_label("Treatment")
        .with_y_label("Response");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/group_colors.svg"), svg).unwrap();
}

/// Effect of bandwidth_scale on cloud shape sensitivity.
fn bandwidth_scale() {
    let data = bimodal_samples(3.0, 7.0, 0.8, 200, 42);

    for (scale, name) in [(2.0_f64, "wide"), (1.0, "default"), (0.4, "tight")] {
        let plot = RaincloudPlot::new()
            .with_group("", data.clone())
            .with_bandwidth_scale(scale);

        let title = match name {
            "wide" => "bandwidth_scale = 2.0 (over-smoothed)",
            "tight" => "bandwidth_scale = 0.4 (sensitive)",
            _ => "bandwidth_scale = 1.0 (Silverman default)",
        };
        let plots = vec![Plot::Raincloud(plot)];
        let layout = Layout::auto_from_plots(&plots)
            .with_title(title)
            .with_y_label("Value");

        let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
        std::fs::write(format!("{OUT}/bandwidth_{name}.svg"), svg).unwrap();
    }
}

/// Horizontal raincloud — categories on Y-axis, values on X-axis.
fn horizontal() {
    let plot = RaincloudPlot::new()
        .with_group("Control", normal_samples(5.0, 1.0, 150, 10))
        .with_group("Low dose", normal_samples(6.2, 1.1, 150, 11))
        .with_group("High dose", normal_samples(7.8, 0.9, 150, 12))
        .with_group_colors(["#4878d0", "#ee854a", "#6acc65"])
        .with_horizontal(true);

    let plots = vec![Plot::Raincloud(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Horizontal Raincloud Plot")
        .with_x_label("Value")
        .with_y_label("Treatment");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/horizontal.svg"), svg).unwrap();
}

/// Individual element variants: cloud+box only, cloud+rain only, box+rain only.
fn elements() {
    let data_a = normal_samples(5.0, 1.2, 120, 20);
    let data_b = bimodal_samples(3.5, 6.5, 0.7, 120, 21);

    let variants: &[(&str, bool, bool, bool, &str)] = &[
        ("cloud_box", true, true, false, "Cloud + Box"),
        ("cloud_rain", true, false, true, "Cloud + Rain"),
        ("box_rain", false, true, true, "Box + Rain"),
    ];

    for (name, cloud, box_, rain, title) in variants {
        let plot = RaincloudPlot::new()
            .with_group("A", data_a.clone())
            .with_group("B", data_b.clone())
            .with_cloud(*cloud)
            .with_box(*box_)
            .with_rain(*rain);

        let plots = vec![Plot::Raincloud(plot)];
        let layout = Layout::auto_from_plots(&plots)
            .with_title(*title)
            .with_y_label("Value");

        let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
        std::fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
    }
}
