use kuva::plot::roc::{RocPlot, RocGroup, compute_group, compute_roc_points, auc_trapz, delong_auc, partial_auc};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::backend::svg::SvgBackend;
use std::fs;

fn write_svg(name: &str, plots: Vec<Plot>, layout: Layout) -> String {
    fs::create_dir_all("test_outputs").unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("test_outputs/{name}.svg"), &svg).unwrap();
    assert!(svg.contains("<svg"));
    svg
}

// ── Realistic data generators ─────────────────────────────────────────────────
//
// Scores are drawn from logistic distributions with different separation.
// Positive class: logistic(+mu, scale)  — high scores
// Negative class: logistic(-mu, scale)  — low scores
// Converted to [0,1] via sigmoid so scores are proper probabilities.
//
// No RNG needed: we use evenly-spaced quantiles of the logistic CDF.
//   Q(p) = mu + scale * ln(p / (1-p))
//   sigmoid(x) = 1 / (1 + exp(-x))

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

/// Good classifier — AUC ~0.88 (mu=1.0, 100 pos + 100 neg).
fn good_classifier() -> Vec<(f64, bool)> { logistic_dataset(100, 1.0, 0.5) }

/// Moderate classifier — AUC ~0.73 (mu=0.5, 100 pos + 100 neg).
fn moderate_classifier() -> Vec<(f64, bool)> { logistic_dataset(100, 0.5, 0.5) }

/// Poor classifier — AUC ~0.61 (mu=0.2, 100 pos + 100 neg).
fn poor_classifier() -> Vec<(f64, bool)> { logistic_dataset(100, 0.2, 0.5) }

/// Perfect: all positives score strictly above all negatives.
fn perfect_classifier() -> Vec<(f64, bool)> {
    let mut data: Vec<(f64, bool)> = (0..50)
        .map(|i| (0.51 + i as f64 * 0.009, true))
        .collect();
    data.extend((0..50).map(|i| (0.01 + i as f64 * 0.009, false)));
    data
}

/// Random: alternating pos/neg with identical score distribution.
fn random_classifier() -> Vec<(f64, bool)> {
    (0..100)
        .flat_map(|i| {
            let s = i as f64 / 100.0;
            vec![(s, true), (s, false)]
        })
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_roc_single_group() {
    let group = RocGroup::new("Classifier").with_raw(good_classifier());
    let roc = RocPlot::new().with_group(group);
    let plots = vec![Plot::Roc(roc)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("ROC Curve")
        .with_x_label("False Positive Rate")
        .with_y_label("True Positive Rate");
    let svg = write_svg("roc_single_group", plots, layout);
    assert!(svg.contains("<path"));
}

#[test]
fn test_roc_auc_value() {
    // Good classifier: AUC ~0.95
    let pts = compute_roc_points(&good_classifier());
    let auc = auc_trapz(&pts);
    assert!(auc > 0.88, "Good classifier AUC too low: {auc}");
    assert!(auc < 1.00, "Good classifier AUC too high: {auc}");

    // Moderate classifier: clearly above chance but well below good
    let pts2 = compute_roc_points(&moderate_classifier());
    let auc2 = auc_trapz(&pts2);
    assert!(auc2 > 0.65, "Moderate AUC too low: {auc2}");
    assert!(auc2 < auc, "Moderate AUC should be lower than good: {auc2} vs {auc}");

    // Poor classifier: close to chance
    let pts3 = compute_roc_points(&poor_classifier());
    let auc3 = auc_trapz(&pts3);
    assert!(auc3 > 0.50, "Poor AUC should be above chance: {auc3}");
    assert!(auc3 < auc2, "Poor AUC should be lower than moderate: {auc3} vs {auc2}");
}

#[test]
fn test_roc_two_groups() {
    let g1 = RocGroup::new("Good classifier").with_raw(good_classifier());
    let g2 = RocGroup::new("Moderate classifier").with_raw(moderate_classifier());
    let roc = RocPlot::new()
        .with_group(g1)
        .with_group(g2)
        .with_legend("Model");
    let plots = vec![Plot::Roc(roc)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("ROC Curve Comparison");
    let svg = write_svg("roc_two_groups", plots, layout);
    assert!(svg.contains("Good classifier"));
    assert!(svg.contains("Moderate classifier"));
}

#[test]
fn test_roc_three_groups() {
    let g1 = RocGroup::new("Good").with_raw(good_classifier());
    let g2 = RocGroup::new("Moderate").with_raw(moderate_classifier());
    let g3 = RocGroup::new("Poor").with_raw(poor_classifier());
    let roc = RocPlot::new()
        .with_group(g1)
        .with_group(g2)
        .with_group(g3)
        .with_legend("Classifier");
    let plots = vec![Plot::Roc(roc)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Three-model ROC Comparison")
        .with_width(520.0)
        .with_height(440.0);
    let svg = write_svg("roc_three_groups", plots, layout);
    assert!(svg.contains("Good"), "Missing 'Good' in SVG");
    assert!(svg.contains("Moderate"), "Missing 'Moderate' in SVG");
    assert!(svg.contains("Poor"), "Missing 'Poor' in SVG");
}

#[test]
fn test_roc_with_ci() {
    let group = RocGroup::new("Classifier")
        .with_raw(good_classifier())
        .with_ci(true)
        .with_ci_alpha(0.2);
    let roc = RocPlot::new().with_group(group);
    let plots = vec![Plot::Roc(roc)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("ROC with 95% CI (DeLong)");
    let svg = write_svg("roc_with_ci", plots, layout);
    assert!(svg.contains("<path"));
}

#[test]
fn test_roc_two_groups_with_ci() {
    let g1 = RocGroup::new("Good").with_raw(good_classifier()).with_ci(true);
    let g2 = RocGroup::new("Moderate").with_raw(moderate_classifier()).with_ci(true);
    let roc = RocPlot::new().with_group(g1).with_group(g2).with_legend("Model");
    let plots = vec![Plot::Roc(roc)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("ROC with CI Bands")
        .with_width(520.0)
        .with_height(440.0);
    let svg = write_svg("roc_two_groups_ci", plots, layout);
    assert!(svg.contains("<path"));
}

#[test]
fn test_roc_optimal_point() {
    let group = RocGroup::new("Classifier")
        .with_raw(good_classifier())
        .with_optimal_point();
    let roc = RocPlot::new().with_group(group);
    let plots = vec![Plot::Roc(roc)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("ROC with Optimal Threshold (Youden's J)");
    let svg = write_svg("roc_optimal_point", plots, layout);
    assert!(svg.contains("<circle") || svg.contains("circle"));
}

#[test]
fn test_roc_pauc() {
    let pts = compute_roc_points(&good_classifier());
    let pauc = partial_auc(&pts, 0.0, 0.2);
    // Partial AUC in FPR [0, 0.2] should be meaningful
    assert!(pauc > 0.0, "pAUC should be positive, got {pauc}");
    assert!(pauc <= 1.0, "pAUC should not exceed 1.0, got {pauc}");

    // For a good classifier, high specificity region (low FPR) should be good
    assert!(pauc > 0.5, "Good classifier pAUC[0,0.2] should be >0.5, got {pauc}");
}

#[test]
fn test_roc_precomputed() {
    // Pre-computed points from a smooth curve (square-root shape, AUC~0.75)
    let pts: Vec<(f64, f64)> = (0..=20)
        .map(|i| {
            let fpr = i as f64 / 20.0;
            let tpr = fpr.sqrt(); // concave curve above diagonal
            (fpr, tpr)
        })
        .collect();
    let group = RocGroup::new("sqrt curve").with_points(pts);
    let roc = RocPlot::new().with_group(group);
    let plots = vec![Plot::Roc(roc)];
    let layout = Layout::auto_from_plots(&plots).with_title("Precomputed ROC (smooth curve)");
    let svg = write_svg("roc_precomputed", plots, layout);
    assert!(svg.contains("<path"));
}

#[test]
fn test_roc_no_diagonal() {
    let group = RocGroup::new("Classifier").with_raw(good_classifier());
    let roc = RocPlot::new().with_group(group).with_diagonal(false);
    let plots = vec![Plot::Roc(roc)];
    let layout = Layout::auto_from_plots(&plots).with_title("ROC — No Diagonal");
    let svg = write_svg("roc_no_diagonal", plots, layout);
    assert!(svg.contains("<path"));
}

#[test]
fn test_roc_perfect_classifier() {
    let group = RocGroup::new("Perfect").with_raw(perfect_classifier());
    let rc = compute_group(&group);
    assert!(rc.auc > 0.99, "Perfect classifier AUC should be > 0.99, got {}", rc.auc);
}

#[test]
fn test_roc_random_classifier() {
    let pts = compute_roc_points(&random_classifier());
    let auc = auc_trapz(&pts);
    assert!((auc - 0.5).abs() < 0.05, "Random classifier AUC should be ~0.5, got {auc}");
}

#[test]
fn test_roc_auc_label_in_svg() {
    let group = RocGroup::new("Classifier")
        .with_raw(good_classifier())
        .with_auc_label(true);
    let roc = RocPlot::new().with_group(group).with_legend("Model");
    let plots = vec![Plot::Roc(roc)];
    let layout = Layout::auto_from_plots(&plots).with_title("ROC AUC Label");
    let svg = write_svg("roc_auc_label", plots, layout);
    assert!(svg.contains("AUC"), "AUC annotation missing from SVG");
}

#[test]
fn test_roc_legend() {
    let g1 = RocGroup::new("Model A").with_raw(good_classifier());
    let g2 = RocGroup::new("Model B").with_raw(moderate_classifier());
    let roc = RocPlot::new()
        .with_group(g1)
        .with_group(g2)
        .with_legend("Classifier");
    let plots = vec![Plot::Roc(roc)];
    let layout = Layout::auto_from_plots(&plots).with_title("ROC Legend Test");
    let svg = write_svg("roc_legend", plots, layout);
    assert!(svg.contains("Model A"));
    assert!(svg.contains("Model B"));
}

#[test]
fn test_delong_auc() {
    // Perfect classifier: DeLong AUC = 1.0, variance = 0
    let (auc, var) = delong_auc(&perfect_classifier());
    assert!((auc - 1.0).abs() < 1e-9, "DeLong AUC perfect = {auc}");
    assert!(var < 1e-9, "DeLong variance perfect = {var}");

    // Good classifier: AUC should be in the same range as trapezoidal
    let good = good_classifier();
    let (dauc, dvar) = delong_auc(&good);
    let pts = compute_roc_points(&good);
    let tauc = auc_trapz(&pts);
    assert!((dauc - tauc).abs() < 0.02, "DeLong and trapezoidal AUC should agree: DeLong={dauc}, trapz={tauc}");
    assert!(dvar > 0.0, "DeLong variance should be positive for imperfect classifier");
    assert!(dvar < 0.01, "DeLong variance unreasonably large: {dvar}");
}

#[test]
fn test_roc_plot_render_multiple() {
    let group = RocGroup::new("Test").with_raw(good_classifier());
    let roc = RocPlot::new().with_group(group);
    let plots = vec![Plot::Roc(roc)];
    let layout = Layout::auto_from_plots(&plots).with_title("ROC via render_multiple");
    let scene = render_multiple(plots, layout);
    assert!(scene.width > 0.0);
    assert!(!scene.elements.is_empty());
}
