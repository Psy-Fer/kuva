//! 2D histogram documentation examples.
//!
//! Generates canonical SVG outputs used in the visus documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example histogram2d
//! ```
//!
//! SVGs are written to `docs/src/assets/histogram2d/`.

use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use visus::plot::Histogram2D;
use visus::plot::histogram2d::ColorMap;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

const OUT: &str = "docs/src/assets/histogram2d";

// ── Data helpers ──────────────────────────────────────────────────────────────

/// Independent bivariate Gaussian samples.
fn bivariate(n: usize, mx: f64, my: f64, sx: f64, sy: f64, seed: u64) -> Vec<(f64, f64)> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    let dx = Normal::new(mx, sx).unwrap();
    let dy = Normal::new(my, sy).unwrap();
    (0..n).map(|_| (dx.sample(&mut rng), dy.sample(&mut rng))).collect()
}

/// Correlated bivariate Gaussian: x ~ N(mx, sx), y = rho*z1 + sqrt(1-rho²)*z2
/// scaled to N(my, sy) — Pearson r ≈ rho.
fn correlated(n: usize, mx: f64, my: f64, sx: f64, sy: f64, rho: f64, seed: u64) -> Vec<(f64, f64)> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    let std = Normal::new(0.0_f64, 1.0).unwrap();
    let k = (1.0 - rho * rho).sqrt();
    (0..n).map(|_| {
        let z1 = std.sample(&mut rng);
        let z2 = std.sample(&mut rng);
        let x = mx + sx * z1;
        let y = my + sy * (rho * z1 + k * z2);
        (x, y)
    }).collect()
}

// ── Examples ──────────────────────────────────────────────────────────────────

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/histogram2d");

    basic();
    correlation();
    bimodal();
    bin_resolution();

    println!("Histogram2D SVGs written to {OUT}/");
}

/// Basic 2D histogram — single Gaussian cluster, Viridis colormap.
///
/// `with_data` bins 5 000 scatter points into a 30×30 grid over the range
/// [0, 30) × [0, 30). Points outside the range are silently discarded.
/// A colorbar labeled "Count" is added automatically.
fn basic() {
    let data = bivariate(5_000, 15.0, 15.0, 3.0, 3.0, 1);

    let hist = Histogram2D::new()
        .with_data(data, (0.0, 30.0), (0.0, 30.0), 30, 30);

    let plots = vec![Plot::Histogram2d(hist)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("2D Histogram — Viridis")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Correlated data with Pearson r annotation.
///
/// `with_correlation()` overlays the Pearson correlation coefficient in the
/// top-right corner. Here rho ≈ 0.85 produces a clear diagonal density ridge.
fn correlation() {
    let data = correlated(4_000, 10.0, 10.0, 2.0, 2.0, 0.85, 2);

    let hist = Histogram2D::new()
        .with_data(data, (0.0, 20.0), (0.0, 20.0), 25, 25)
        .with_correlation();

    let plots = vec![Plot::Histogram2d(hist)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Correlated Variables (r ≈ 0.85)")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/correlation.svg"), svg).unwrap();
}

/// Bimodal distribution — two clusters, Inferno colormap.
///
/// `ColorMap::Inferno` uses a dark-to-bright yellow-orange scheme that makes
/// high-density regions stand out against a near-black background.
fn bimodal() {
    let mut data = bivariate(3_000,  9.0,  9.0, 2.0, 2.0, 3);
    data.extend(bivariate(3_000, 21.0, 21.0, 2.0, 2.0, 4));

    let hist = Histogram2D::new()
        .with_data(data, (0.0, 30.0), (0.0, 30.0), 30, 30)
        .with_color_map(ColorMap::Inferno);

    let plots = vec![Plot::Histogram2d(hist)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Bimodal — Inferno")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/bimodal.svg"), svg).unwrap();
}

/// Effect of bin resolution — coarse (10×10) vs fine (50×50), Grayscale.
///
/// `ColorMap::Grayscale` maps zero counts to white and the maximum count to
/// black. Coarse binning reveals the overall shape; fine binning shows
/// finer density structure.
fn bin_resolution() {
    let data = bivariate(8_000, 15.0, 15.0, 3.5, 3.5, 5);

    for (bins, name) in [(10usize, "coarse"), (50, "fine")] {
        let hist = Histogram2D::new()
            .with_data(data.clone(), (0.0, 30.0), (0.0, 30.0), bins, bins)
            .with_color_map(ColorMap::Grayscale);

        let plots = vec![Plot::Histogram2d(hist)];
        let layout = Layout::auto_from_plots(&plots)
            .with_title(format!("Grayscale — {}×{} bins", bins, bins))
            .with_x_label("X")
            .with_y_label("Y");

        let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
        std::fs::write(format!("{OUT}/bins_{name}.svg"), svg).unwrap();
    }
}
