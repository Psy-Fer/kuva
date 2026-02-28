//! Histogram documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example histogram
//! ```
//!
//! SVGs are written to `docs/src/assets/histogram/`.

use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use kuva::plot::Histogram;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

const OUT: &str = "docs/src/assets/histogram";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/histogram");

    basic();
    bins();
    normalized();
    overlapping();

    println!("Histogram SVGs written to {OUT}/");
}

fn normal_samples(mean: f64, std: f64, n: usize, seed: u64) -> Vec<f64> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    let dist = Normal::new(mean, std).unwrap();
    (0..n).map(|_| dist.sample(&mut rng)).collect()
}

/// Basic histogram — 300 samples from a normal distribution.
fn basic() {
    let data = normal_samples(0.0, 1.0, 300, 42);

    let hist = Histogram::new()
        .with_data(data)
        .with_bins(30)
        .with_range((-3.0, 3.0))
        .with_color("steelblue");

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Histogram")
        .with_x_label("Value")
        .with_y_label("Count");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Effect of bin count — coarse (6 bins) vs fine (60 bins).
fn bins() {
    let data = normal_samples(0.0, 1.0, 300, 42);

    // Coarse — 6 bins over [-3, 3]: bin_width = 1.0, ticks at integers
    {
        let hist = Histogram::new()
            .with_data(data.clone())
            .with_bins(6)
            .with_range((-3.0, 3.0))
            .with_color("steelblue");
        let plots = vec![Plot::Histogram(hist)];
        let layout = Layout::auto_from_plots(&plots)
            .with_title("6 Bins")
            .with_x_label("Value")
            .with_y_label("Count");
        let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
        std::fs::write(format!("{OUT}/bins_coarse.svg"), svg).unwrap();
    }

    // Fine — 60 bins over [-3, 3]: bin_width = 0.1, ticks at integers
    {
        let hist = Histogram::new()
            .with_data(data)
            .with_bins(60)
            .with_range((-3.0, 3.0))
            .with_color("steelblue");
        let plots = vec![Plot::Histogram(hist)];
        let layout = Layout::auto_from_plots(&plots)
            .with_title("60 Bins")
            .with_x_label("Value")
            .with_y_label("Count");
        let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
        std::fs::write(format!("{OUT}/bins_fine.svg"), svg).unwrap();
    }
}

/// Normalized histogram — tallest bar scaled to 1.0.
fn normalized() {
    let data = normal_samples(0.0, 1.0, 300, 42);

    let hist = Histogram::new()
        .with_data(data)
        .with_bins(30)
        .with_range((-3.0, 3.0))
        .with_color("steelblue")
        .with_normalize();

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Normalized Histogram")
        .with_x_label("Value")
        .with_y_label("Relative frequency");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/normalized.svg"), svg).unwrap();
}

/// Two overlapping distributions using semi-transparent fill colors.
///
/// Both histograms share the same range so their x-axes align.
fn overlapping() {
    let group_a = normal_samples(-1.0, 0.8, 300, 1);
    let group_b = normal_samples(1.0, 0.8, 300, 2);

    // Fixed range covering both distributions — 16 bins over [-4,4]: bin_width=0.5, ticks at even integers.
    let range = (-4.0_f64, 4.0_f64);

    // #4682b4 = steelblue, #dc143c = crimson — 80 = ~50% opacity in 8-digit hex (RRGGBBAA)
    let hist_a = Histogram::new()
        .with_data(group_a)
        .with_bins(16)
        .with_range(range)
        .with_color("#4682b480")
        .with_legend("Group A");

    let hist_b = Histogram::new()
        .with_data(group_b)
        .with_bins(16)
        .with_range(range)
        .with_color("#dc143c80")
        .with_legend("Group B");

    let plots = vec![Plot::Histogram(hist_a), Plot::Histogram(hist_b)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Overlapping Distributions")
        .with_x_label("Value")
        .with_y_label("Count");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/overlapping.svg"), svg).unwrap();
}
