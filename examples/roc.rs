//! ROC curve documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example roc
//! ```
//!
//! SVGs are written to `docs/src/assets/roc/`.

use kuva::plot::roc::{RocPlot, RocGroup};
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use std::fs;

const OUT: &str = "docs/src/assets/roc";

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all(OUT).unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

/// Build a dataset from logistic distribution quantiles.
///
/// Positive scores are drawn from Logistic(+mu, scale),
/// negative scores from Logistic(-mu, scale), then mapped to [0,1] via sigmoid.
/// This produces genuinely overlapping distributions without any RNG.
fn logistic_dataset(n: usize, mu: f64, scale: f64) -> Vec<(f64, bool)> {
    let mut data = Vec::with_capacity(2 * n);
    for i in 1..=n {
        let p = i as f64 / (n + 1) as f64;
        let logit = (p / (1.0 - p)).ln();
        let pos_score = 1.0 / (1.0 + (-(mu + scale * logit)).exp());
        let neg_score = 1.0 / (1.0 + (-(-mu + scale * logit)).exp());
        data.push((pos_score, true));
        data.push((neg_score, false));
    }
    data
}

fn main() {
    // ── Basic single-group ROC ────────────────────────────────────────────────
    let group = RocGroup::new("Classifier")
        .with_raw(logistic_dataset(150, 1.0, 0.5));
    let roc = RocPlot::new().with_group(group);
    let plots = vec![Plot::Roc(roc)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("ROC Curve")
        .with_x_label("False Positive Rate")
        .with_y_label("True Positive Rate");
    write("basic", plots, layout);

    // ── Single group with DeLong 95% CI ──────────────────────────────────────
    let group = RocGroup::new("Classifier")
        .with_raw(logistic_dataset(150, 1.0, 0.5))
        .with_ci(true)
        .with_optimal_point();
    let roc = RocPlot::new().with_group(group);
    let plots = vec![Plot::Roc(roc)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("ROC Curve with 95% CI")
        .with_x_label("False Positive Rate")
        .with_y_label("True Positive Rate");
    write("with_ci", plots, layout);

    // ── Three classifiers — model comparison ─────────────────────────────────
    let g1 = RocGroup::new("Model A").with_raw(logistic_dataset(150, 1.2, 0.5));
    let g2 = RocGroup::new("Model B").with_raw(logistic_dataset(150, 0.6, 0.5));
    let g3 = RocGroup::new("Model C").with_raw(logistic_dataset(150, 0.2, 0.5));
    let roc = RocPlot::new()
        .with_group(g1)
        .with_group(g2)
        .with_group(g3)
        .with_legend("Classifier");
    let plots = vec![Plot::Roc(roc)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Multi-model ROC Comparison")
        .with_x_label("False Positive Rate")
        .with_y_label("True Positive Rate")
        .with_width(520.0)
        .with_height(440.0);
    write("multi_model", plots, layout);

    // ── Genomics: two biomarker classifiers with CI ───────────────────────────
    let g1 = RocGroup::new("Biomarker A")
        .with_raw(logistic_dataset(120, 1.1, 0.45))
        .with_ci(true)
        .with_optimal_point();
    let g2 = RocGroup::new("Biomarker B")
        .with_raw(logistic_dataset(120, 0.7, 0.5))
        .with_ci(true);
    let roc = RocPlot::new()
        .with_group(g1)
        .with_group(g2)
        .with_legend("Biomarker");
    let plots = vec![Plot::Roc(roc)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Diagnostic Biomarker Comparison")
        .with_x_label("1 − Specificity")
        .with_y_label("Sensitivity")
        .with_width(520.0)
        .with_height(440.0);
    write("biomarkers", plots, layout);
}
