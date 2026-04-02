//! 2D histogram documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example histogram2d
//! ```
//!
//! SVGs are written to `docs/src/assets/histogram2d/`.

use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use kuva::plot::Histogram2D;
use kuva::plot::histogram2d::ColorMap;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::{Layout, TickFormat};
use kuva::render::plots::Plot;

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
    log_count();
    colorbar_formats();

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

/// Log-scale color mapping — linear vs log side by side.
///
/// When a small number of bins dominate the count (e.g. a dense core surrounded
/// by a sparse tail), `with_log_count()` compresses the dynamic range via
/// `ln(count + 1)` so low-density structure stays visible. The colorbar label
/// updates to "log(Count)" automatically.
fn log_count() {
    // Highly skewed: 8 000 points in a tight core, 2 000 scattered in a halo.
    let core  = bivariate(8_000, 15.0, 15.0, 1.0, 1.0, 10);
    let halo  = bivariate(2_000, 15.0, 15.0, 5.0, 5.0, 11);
    let mut data = core;
    data.extend(halo);

    // Linear — dense core washes out the halo completely.
    let hist_linear = Histogram2D::new()
        .with_data(data.clone(), (0.0, 30.0), (0.0, 30.0), 30, 30)
        .with_color_map(ColorMap::Inferno);

    let plots = vec![Plot::Histogram2d(hist_linear)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Linear Color Scale")
        .with_x_label("X")
        .with_y_label("Y");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/log_count_linear.svg"), svg).unwrap();

    // Log — halo structure is visible alongside the dense core.
    let hist_log = Histogram2D::new()
        .with_data(data, (0.0, 30.0), (0.0, 30.0), 30, 30)
        .with_color_map(ColorMap::Inferno)
        .with_log_count();

    let plots = vec![Plot::Histogram2d(hist_log)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Log Color Scale")
        .with_x_label("X")
        .with_y_label("Y");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/log_count_log.svg"), svg).unwrap();
}

/// Colorbar tick formats — Auto (default) and Sci on a high-count dataset.
///
/// `with_colorbar_tick_format` accepts any `TickFormat` variant.
/// `TickFormat::Auto` (the default) renders integer counts cleanly and
/// switches to scientific notation automatically when counts reach 10 000.
/// `TickFormat::Sci` forces scientific notation regardless of magnitude.
fn colorbar_formats() {
    // 50 000 points in one tight cluster → max bin count well above 10 000,
    // so Auto will switch to scientific notation on the colorbar.
    let data = bivariate(50_000, 15.0, 15.0, 2.0, 2.0, 12);

    // Auto format — sci notation kicks in automatically for large counts.
    let hist_auto = Histogram2D::new()
        .with_data(data.clone(), (0.0, 30.0), (0.0, 30.0), 30, 30)
        .with_color_map(ColorMap::Viridis);

    let plots = vec![Plot::Histogram2d(hist_auto)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Colorbar: Auto Format (large counts)")
        .with_x_label("X")
        .with_y_label("Y");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/colorbar_auto.svg"), svg).unwrap();

    // Sci format — always scientific notation, regardless of magnitude.
    let hist_sci = Histogram2D::new()
        .with_data(data, (0.0, 30.0), (0.0, 30.0), 30, 30)
        .with_color_map(ColorMap::Viridis);

    let plots = vec![Plot::Histogram2d(hist_sci)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Colorbar: Sci Format")
        .with_x_label("X")
        .with_y_label("Y")
        .with_colorbar_tick_format(TickFormat::Sci);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/colorbar_sci.svg"), svg).unwrap();
}

/// Effect of bin resolution — coarse (10×10, Grayscale) vs fine (50×50, Viridis).
///
/// Coarse binning uses `ColorMap::Grayscale` (white → black) to show overall
/// shape; fine binning uses `ColorMap::Viridis` to reveal finer density
/// structure with a perceptually uniform colour scale.
fn bin_resolution() {
    let data = bivariate(8_000, 15.0, 15.0, 3.5, 3.5, 5);

    let configs: &[(usize, &str, ColorMap, &str)] = &[
        (10, "coarse", ColorMap::Grayscale, "Grayscale — 10×10 bins"),
        (50, "fine",   ColorMap::Viridis,   "Viridis — 50×50 bins"),
    ];
    for &(bins, name, ref cmap, title) in configs {
        let hist = Histogram2D::new()
            .with_data(data.clone(), (0.0, 30.0), (0.0, 30.0), bins, bins)
            .with_color_map(cmap.clone());

        let plots = vec![Plot::Histogram2d(hist)];
        let layout = Layout::auto_from_plots(&plots)
            .with_title(title)
            .with_x_label("X")
            .with_y_label("Y");

        let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
        std::fs::write(format!("{OUT}/bins_{name}.svg"), svg).unwrap();
    }
}
