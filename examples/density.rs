//! Density plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example density
//! ```
//!
//! SVGs are written to `docs/src/assets/density/`.

use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use kuva::plot::DensityPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;
use kuva::render::plots::Plot;

const OUT: &str = "docs/src/assets/density";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/density");

    basic();
    filled();
    multigroup();
    bandwidth();

    println!("Density SVGs written to {OUT}/");
}

fn normal_samples(mean: f64, std: f64, n: usize, seed: u64) -> Vec<f64> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    Normal::new(mean, std).unwrap()
        .sample_iter(&mut rng)
        .take(n)
        .collect()
}

/// Basic density — 300 samples, Silverman bandwidth, no fill.
fn basic() {
    let data = normal_samples(0.0, 1.0, 300, 42);

    let density = DensityPlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_stroke_width(2.0);

    let plots = vec![Plot::Density(density)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Density Plot")
        .with_x_label("Value")
        .with_y_label("Density");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Filled density — area under curve shaded.
fn filled() {
    let data = normal_samples(0.0, 1.0, 300, 42);

    let density = DensityPlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_filled(true)
        .with_opacity(0.25)
        .with_stroke_width(2.0);

    let plots = vec![Plot::Density(density)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Filled Density")
        .with_x_label("Value")
        .with_y_label("Density");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/filled.svg"), svg).unwrap();
}

/// Multi-group — three overlapping filled curves with palette colors.
/// This is the canonical gallery image.
fn multigroup() {
    let control    = normal_samples(0.0, 1.0, 300, 1);
    let treatment_a = normal_samples(1.5, 0.8, 300, 2);
    let treatment_b = normal_samples(-0.5, 1.3, 300, 3);

    let pal = Palette::category10();

    let plots = vec![
        Plot::Density(
            DensityPlot::new()
                .with_data(control)
                .with_color(&pal[0])
                .with_filled(true)
                .with_opacity(0.2)
                .with_stroke_width(2.0)
                .with_legend("Control"),
        ),
        Plot::Density(
            DensityPlot::new()
                .with_data(treatment_a)
                .with_color(&pal[1])
                .with_filled(true)
                .with_opacity(0.2)
                .with_stroke_width(2.0)
                .with_legend("Treatment A"),
        ),
        Plot::Density(
            DensityPlot::new()
                .with_data(treatment_b)
                .with_color(&pal[2])
                .with_filled(true)
                .with_opacity(0.2)
                .with_stroke_width(2.0)
                .with_legend("Treatment B"),
        ),
    ];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Expression by Group")
        .with_x_label("Expression")
        .with_y_label("Density");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/multigroup.svg"), svg).unwrap();
}

/// Bandwidth comparison — narrow / auto (Silverman) / wide.
/// Uses a bimodal mixture so the effect of over-smoothing is visible.
fn bandwidth() {
    // Bimodal: mix of two normals
    let mut a = normal_samples(-1.0, 0.4, 150, 10);
    let b = normal_samples(1.0, 0.4, 150, 11);
    a.extend(b);

    let save = |name: &str, bw: Option<f64>, title: &str| {
        let mut dp = DensityPlot::new()
            .with_data(a.clone())
            .with_color("steelblue")
            .with_stroke_width(2.0);
        if let Some(h) = bw {
            dp = dp.with_bandwidth(h);
        }
        let plots = vec![Plot::Density(dp)];
        let layout = Layout::auto_from_plots(&plots)
            .with_title(title)
            .with_x_label("Value")
            .with_y_label("Density")
            .with_width(280.0)
            .with_height(220.0);
        let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
        std::fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
    };

    save("bandwidth_narrow", Some(0.1), "h = 0.1 (too narrow)");
    save("bandwidth_auto",   None,      "Auto — Silverman");
    save("bandwidth_wide",   Some(2.0), "h = 2.0 (too wide)");
}
