mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::pr::{
    auc_pr_trapz, compute_pr_group, compute_pr_points, optimal_f1_idx, PrGroup, PrPlot,
};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use std::fs;

fn write_svg(name: &str, plots: Vec<Plot>, layout: Layout) -> String {
    fs::create_dir_all("test_outputs").unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output(format!("test_outputs/{name}.svg"), &svg).unwrap();
    assert!(svg.contains("<svg"));
    svg
}

// ── Realistic data generators (same approach as roc_basic.rs) ─────────────────

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

fn good_classifier() -> Vec<(f64, bool)> {
    logistic_dataset(100, 1.0, 0.5)
}
fn moderate_classifier() -> Vec<(f64, bool)> {
    logistic_dataset(100, 0.5, 0.5)
}
fn poor_classifier() -> Vec<(f64, bool)> {
    logistic_dataset(100, 0.2, 0.5)
}

fn perfect_classifier() -> Vec<(f64, bool)> {
    let mut data: Vec<(f64, bool)> = (0..50).map(|i| (0.51 + i as f64 * 0.009, true)).collect();
    data.extend((0..50).map(|i| (0.01 + i as f64 * 0.009, false)));
    data
}

/// Imbalanced dataset: 10% positive prevalence.
fn imbalanced_dataset() -> Vec<(f64, bool)> {
    let mut data = Vec::new();
    for i in 0..100 {
        let score = (i as f64 + 1.0) / 101.0;
        data.push((score, i >= 90)); // top 10 are positive
    }
    data
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_pr_single_group() {
    let group = PrGroup::new("Classifier").with_raw(good_classifier());
    let pr = PrPlot::new().with_group(group);
    let plots = vec![Plot::Pr(pr)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Precision-Recall Curve")
        .with_x_label("Recall")
        .with_y_label("Precision");
    let svg = write_svg("pr_single_group", plots, layout);
    assert!(svg.contains("<path"));
}

#[test]
fn test_pr_auc_values() {
    let (pts_good, _) = compute_pr_points(&good_classifier());
    let auc_good = auc_pr_trapz(&pts_good);
    assert!(
        auc_good > 0.80,
        "Good classifier AUC-PR too low: {auc_good}"
    );
    assert!(auc_good <= 1.0, "AUC-PR cannot exceed 1.0");

    let (pts_mod, _) = compute_pr_points(&moderate_classifier());
    let auc_mod = auc_pr_trapz(&pts_mod);
    assert!(auc_mod > 0.60, "Moderate AUC-PR too low: {auc_mod}");
    assert!(
        auc_mod < auc_good,
        "Moderate AUC-PR should be lower than good: {auc_mod} vs {auc_good}"
    );

    let (pts_poor, _) = compute_pr_points(&poor_classifier());
    let auc_poor = auc_pr_trapz(&pts_poor);
    assert!(
        auc_poor < auc_mod,
        "Poor AUC-PR should be lower than moderate: {auc_poor} vs {auc_mod}"
    );
}

#[test]
fn test_pr_prevalence() {
    let (_, prevalence) = compute_pr_points(&good_classifier());
    // good_classifier: equal positives and negatives → prevalence = 0.5
    assert!(
        (prevalence - 0.5).abs() < 1e-9,
        "Expected prevalence 0.5, got {prevalence}"
    );

    let (_, prev_imb) = compute_pr_points(&imbalanced_dataset());
    // imbalanced: 10 positives out of 100 → prevalence ≈ 0.1
    assert!(
        (prev_imb - 0.1).abs() < 1e-9,
        "Expected prevalence 0.1, got {prev_imb}"
    );
}

#[test]
fn test_pr_curve_starts_at_recall_zero() {
    let (pts, _) = compute_pr_points(&good_classifier());
    assert!(!pts.is_empty(), "PR points should not be empty");
    assert!(
        (pts[0].recall - 0.0).abs() < 1e-9,
        "First point should have recall=0"
    );
    assert!(
        (pts[0].precision - 1.0).abs() < 1e-9,
        "First point should have precision=1"
    );
}

#[test]
fn test_pr_curve_ends_at_recall_one() {
    let (pts, _) = compute_pr_points(&good_classifier());
    let last = pts.last().unwrap();
    assert!(
        (last.recall - 1.0).abs() < 1e-9,
        "Last point should have recall=1, got {}",
        last.recall
    );
}

#[test]
fn test_pr_perfect_classifier() {
    let (pts, _) = compute_pr_points(&perfect_classifier());
    let auc = auc_pr_trapz(&pts);
    assert!(
        auc > 0.99,
        "Perfect classifier AUC-PR should be > 0.99, got {auc}"
    );
}

#[test]
fn test_pr_two_groups() {
    let g1 = PrGroup::new("Good classifier").with_raw(good_classifier());
    let g2 = PrGroup::new("Moderate classifier").with_raw(moderate_classifier());
    let pr = PrPlot::new()
        .with_group(g1)
        .with_group(g2)
        .with_legend("Model");
    let plots = vec![Plot::Pr(pr)];
    let layout = Layout::auto_from_plots(&plots).with_title("PR Curve Comparison");
    let svg = write_svg("pr_two_groups", plots, layout);
    assert!(svg.contains("Good classifier"));
    assert!(svg.contains("Moderate classifier"));
}

#[test]
fn test_pr_three_groups() {
    let g1 = PrGroup::new("Good").with_raw(good_classifier());
    let g2 = PrGroup::new("Moderate").with_raw(moderate_classifier());
    let g3 = PrGroup::new("Poor").with_raw(poor_classifier());
    let pr = PrPlot::new()
        .with_group(g1)
        .with_group(g2)
        .with_group(g3)
        .with_legend("Classifier");
    let plots = vec![Plot::Pr(pr)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Three-model PR Comparison")
        .with_width(520.0)
        .with_height(440.0);
    let svg = write_svg("pr_three_groups", plots, layout);
    assert!(svg.contains("Good"));
    assert!(svg.contains("Moderate"));
    assert!(svg.contains("Poor"));
}

#[test]
fn test_pr_optimal_point() {
    let group = PrGroup::new("Classifier")
        .with_raw(good_classifier())
        .with_optimal_point();
    let pr = PrPlot::new().with_group(group);
    let plots = vec![Plot::Pr(pr)];
    let layout = Layout::auto_from_plots(&plots).with_title("PR with Optimal F1 Threshold");
    let svg = write_svg("pr_optimal_point", plots, layout);
    assert!(svg.contains("<circle") || svg.contains("circle"));
}

#[test]
fn test_pr_no_baseline() {
    let group = PrGroup::new("Classifier").with_raw(good_classifier());
    let pr = PrPlot::new().with_group(group).with_baseline(false);
    let plots = vec![Plot::Pr(pr)];
    let layout = Layout::auto_from_plots(&plots).with_title("PR — No Baseline");
    let svg = write_svg("pr_no_baseline", plots, layout);
    assert!(svg.contains("<path"));
}

#[test]
fn test_pr_precomputed() {
    // A concave PR curve (precision stays high)
    let pts: Vec<(f64, f64)> = (0..=20)
        .map(|i| {
            let recall = i as f64 / 20.0;
            let precision = 1.0 - 0.5 * recall; // linearly declining
            (recall, precision)
        })
        .collect();
    let group = PrGroup::new("Linear curve")
        .with_points(pts)
        .with_prevalence(0.3);
    let pr = PrPlot::new().with_group(group);
    let plots = vec![Plot::Pr(pr)];
    let layout = Layout::auto_from_plots(&plots).with_title("Precomputed PR Curve");
    let svg = write_svg("pr_precomputed", plots, layout);
    assert!(svg.contains("<path"));
}

#[test]
fn test_pr_auc_label_in_svg() {
    let group = PrGroup::new("Classifier")
        .with_raw(good_classifier())
        .with_auc_label(true);
    let pr = PrPlot::new().with_group(group).with_legend("Model");
    let plots = vec![Plot::Pr(pr)];
    let layout = Layout::auto_from_plots(&plots).with_title("PR AUC Label");
    let svg = write_svg("pr_auc_label", plots, layout);
    assert!(svg.contains("AUC-PR"), "AUC-PR annotation missing from SVG");
}

#[test]
fn test_pr_auc_label_off() {
    let group = PrGroup::new("Classifier")
        .with_raw(good_classifier())
        .with_auc_label(false);
    let pr = PrPlot::new().with_group(group).with_legend("Model");
    let plots = vec![Plot::Pr(pr)];
    let layout = Layout::auto_from_plots(&plots).with_title("PR No AUC Label");
    let svg = write_svg("pr_auc_label_off", plots, layout);
    assert!(
        !svg.contains("AUC-PR"),
        "AUC-PR should be absent when show_auc_label=false"
    );
}

#[test]
fn test_pr_imbalanced_baseline() {
    // Baseline should be at prevalence ≈ 0.1, well below 0.5.
    let group = PrGroup::new("Imbalanced").with_raw(imbalanced_dataset());
    let pr = PrPlot::new().with_group(group).with_legend("Model");
    let plots = vec![Plot::Pr(pr)];
    let layout = Layout::auto_from_plots(&plots).with_title("PR — Imbalanced (10% prevalence)");
    let svg = write_svg("pr_imbalanced_baseline", plots, layout);
    assert!(svg.contains("<path"));
}

#[test]
fn test_pr_optimal_f1_idx_basic() {
    let (pts, _) = compute_pr_points(&good_classifier());
    let idx = optimal_f1_idx(&pts);
    assert!(idx < pts.len(), "Optimal index out of bounds");
    let opt = &pts[idx];
    let f1 = 2.0 * opt.precision * opt.recall / (opt.precision + opt.recall + 1e-12);
    // Should be meaningfully above 0.5
    assert!(f1 > 0.5, "Optimal F1 too low: {f1}");
}

#[test]
fn test_pr_render_multiple() {
    let group = PrGroup::new("Test").with_raw(good_classifier());
    let pr = PrPlot::new().with_group(group);
    let plots = vec![Plot::Pr(pr)];
    let layout = Layout::auto_from_plots(&plots).with_title("PR via render_multiple");
    let scene = render_multiple(plots, layout);
    assert!(scene.width > 0.0);
    assert!(!scene.elements.is_empty());
}

#[test]
fn test_pr_compute_group() {
    let group = PrGroup::new("Good").with_raw(good_classifier());
    let computed = compute_pr_group(&group);
    assert!(!computed.points.is_empty());
    assert!(computed.auc > 0.8);
    assert!((computed.prevalence - 0.5).abs() < 1e-9);
    assert!(computed.optimal_idx.is_none()); // show_optimal_point = false by default
}

#[test]
fn test_pr_showcase() {
    // Full-featured showcase: 3 classifiers, legend, baseline, optimal F1 markers.
    let g1 = PrGroup::new("Good model")
        .with_raw(good_classifier())
        .with_optimal_point();
    let g2 = PrGroup::new("Moderate model")
        .with_raw(moderate_classifier())
        .with_dasharray("6,3".to_string());
    let g3 = PrGroup::new("Poor model")
        .with_raw(poor_classifier())
        .with_dasharray("2,4".to_string());
    let pr = PrPlot::new()
        .with_group(g1)
        .with_group(g2)
        .with_group(g3)
        .with_legend("PR comparison");
    let plots = vec![Plot::Pr(pr)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Precision-Recall Curve — 3 Classifiers")
        .with_x_label("Recall")
        .with_y_label("Precision")
        .with_width(560.0)
        .with_height(460.0);
    let svg = write_svg("pr_showcase", plots, layout);
    assert!(svg.contains("Good model"));
    assert!(svg.contains("AUC-PR"));
}
