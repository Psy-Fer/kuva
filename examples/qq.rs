//! Q-Q plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example qq
//! ```
//!
//! SVGs are written to `docs/src/assets/qq/`.

use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use kuva::plot::QQPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;
use kuva::render::plots::Plot;

const OUT: &str = "docs/src/assets/qq";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/qq");

    normal_basic();
    normal_skewed();
    normal_multigroup();
    genomic_basic();
    genomic_ci_lambda();
    genomic_multigroup();

    println!("Q-Q plot SVGs written to {OUT}/");
}

fn normal_samples(mean: f64, std: f64, n: usize, seed: u64) -> Vec<f64> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    Normal::new(mean, std).unwrap()
        .sample_iter(&mut rng)
        .take(n)
        .collect()
}

/// Normal Q-Q for normally distributed data — points lie on the reference line.
fn normal_basic() {
    let data = normal_samples(0.0, 1.0, 200, 42);

    let plot = QQPlot::new()
        .with_data("Sample", data)
        .with_color("steelblue");

    let plots = vec![Plot::QQ(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Normal Q-Q")
        .with_x_label("Theoretical Quantiles")
        .with_y_label("Sample Quantiles");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/normal_basic.svg"), svg).unwrap();
}

/// Normal Q-Q for right-skewed data — S-curve deviation in the tails.
fn normal_skewed() {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(7);
    // Log-normal data (heavy right tail)
    let data: Vec<f64> = Normal::new(0.0_f64, 1.0).unwrap()
        .sample_iter(&mut rng)
        .take(200)
        .map(|v: f64| v.exp())
        .collect();

    let plot = QQPlot::new()
        .with_data("Log-normal sample", data)
        .with_color("tomato");

    let plots = vec![Plot::QQ(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Normal Q-Q — right-skewed data")
        .with_x_label("Theoretical Quantiles")
        .with_y_label("Sample Quantiles");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/normal_skewed.svg"), svg).unwrap();
}

/// Multi-group normal Q-Q — compare two distributions side by side.
fn normal_multigroup() {
    let pal = Palette::category10();
    let control = normal_samples(0.0, 1.0, 150, 1);
    let treated  = normal_samples(1.5, 1.0, 150, 2);

    let plot = QQPlot::new()
        .with_data_colored("Control", control, pal[0].to_string())
        .with_data_colored("Treated", treated, pal[1].to_string())
        .with_legend("");

    let plots = vec![Plot::QQ(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Normal Q-Q — two groups")
        .with_x_label("Theoretical Quantiles")
        .with_y_label("Sample Quantiles");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/normal_multigroup.svg"), svg).unwrap();
}

/// Genomic Q-Q for null-distributed p-values — points hug the diagonal, λ ≈ 1.
fn genomic_basic() {
    // Uniform p-values (null hypothesis holds for all tests)
    let pvals: Vec<f64> = (1..=2000).map(|i| i as f64 / 2001.0).collect();

    let plot = QQPlot::new()
        .with_pvalues("Null GWAS", pvals)
        .with_color("steelblue")
        .with_lambda();

    let plots = vec![Plot::QQ(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Genomic Q-Q (null)")
        .with_x_label("Expected −log₁₀(p)")
        .with_y_label("Observed −log₁₀(p)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/genomic_basic.svg"), svg).unwrap();
}

/// Genomic Q-Q with CI band and λ — inflated test statistics curve above diagonal.
fn genomic_ci_lambda() {
    // Mix: mostly null p-values + a tail of genuinely small p-values (associations)
    let mut null: Vec<f64> = (1..=1800).map(|i| i as f64 / 1801.0).collect();
    let signal: Vec<f64> = (1..=200).map(|i| (i as f64 / 2000.0).powi(3)).collect();
    null.extend(signal);

    let plot = QQPlot::new()
        .with_pvalues("GWAS study", null)
        .with_color("steelblue")
        .with_ci_band()
        .with_lambda();

    let plots = vec![Plot::QQ(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Genomic Q-Q with CI band and λ")
        .with_x_label("Expected −log₁₀(p)")
        .with_y_label("Observed −log₁₀(p)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/genomic_ci_lambda.svg"), svg).unwrap();
}

/// Multi-study genomic Q-Q — overlay two GWAS datasets for comparison.
fn genomic_multigroup() {
    let pal = Palette::category10();
    let study_a: Vec<f64> = (1..=500).map(|i| i as f64 / 501.0).collect();
    // Study B with mild inflation
    let study_b: Vec<f64> = (1..=500).map(|i| (i as f64 / 501.0).powf(0.85)).collect();

    let plot = QQPlot::new()
        .with_pvalues_colored("Study A", study_a, pal[0].to_string())
        .with_pvalues_colored("Study B", study_b, pal[1].to_string())
        .with_ci_band()
        .with_legend("")
        .with_lambda();

    let plots = vec![Plot::QQ(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Multi-study Genomic Q-Q")
        .with_x_label("Expected −log₁₀(p)")
        .with_y_label("Observed −log₁₀(p)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/genomic_multigroup.svg"), svg).unwrap();
}
