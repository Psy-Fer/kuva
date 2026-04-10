use kuva::plot::{QQPlot, QQMode};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::render::palette::Palette;
use kuva::backend::svg::SvgBackend;
use std::fs;

fn render_svg(plots: Vec<Plot>, layout: Layout) -> String {
    let scene = render_multiple(plots, layout);
    SvgBackend.render_scene(&scene)
}

fn outdir() { fs::create_dir_all("test_outputs").ok(); }

// ── Normal Q-Q ────────────────────────────────────────────────────────────────

#[test]
fn test_qq_normal_basic() {
    outdir();
    let data: Vec<f64> = (1..=50).map(|i| i as f64 * 0.1).collect();
    let plot = QQPlot::new()
        .with_data("Sample", data)
        .with_color("steelblue");
    let plots = vec![Plot::QQ(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Normal Q-Q")
        .with_x_label("Theoretical Quantiles")
        .with_y_label("Sample Quantiles");
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/qq_normal_basic.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    // Should have scatter circles
    assert!(svg.contains("<circle"), "normal Q-Q should produce circle elements");
}

#[test]
fn test_qq_normal_reference_line() {
    outdir();
    let data: Vec<f64> = (1..=30).map(|i| i as f64).collect();
    let plot = QQPlot::new().with_data("", data).with_color("steelblue");
    let plots = vec![Plot::QQ(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/qq_normal_refline.svg", &svg).unwrap();
    assert!(svg.contains("<line"), "reference line should produce a line element");
}

#[test]
fn test_qq_normal_no_reference_line() {
    outdir();
    let data: Vec<f64> = (1..=20).map(|i| i as f64).collect();
    let plot = QQPlot::new()
        .with_data("", data)
        .without_reference_line();
    let plots = vec![Plot::QQ(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/qq_normal_no_refline.svg", &svg).unwrap();
    assert!(!svg.contains("stroke-dasharray=\"5,3\""),
        "no reference line should not produce dashed line");
}

#[test]
fn test_qq_normal_multigroup() {
    outdir();
    let pal = Palette::category10();
    let a: Vec<f64> = (1..=40).map(|i| i as f64 * 0.1).collect();
    let b: Vec<f64> = (1..=40).map(|i| i as f64 * 0.1 + 2.0).collect();
    let plot = QQPlot::new()
        .with_data_colored("Group A", a, pal[0].to_string())
        .with_data_colored("Group B", b, pal[1].to_string())
        .with_legend("Groups");
    let plots = vec![Plot::QQ(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Multi-group Normal Q-Q");
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/qq_normal_multigroup.svg", &svg).unwrap();
    assert!(svg.contains("Group A"), "legend should contain Group A");
    assert!(svg.contains("Group B"), "legend should contain Group B");
}

// ── Genomic Q-Q ──────────────────────────────────────────────────────────────

#[test]
fn test_qq_genomic_basic() {
    outdir();
    let pvals: Vec<f64> = (1..=100).map(|i| i as f64 / 101.0).collect();
    let plot = QQPlot::new()
        .with_pvalues("GWAS", pvals)
        .with_color("steelblue");
    let plots = vec![Plot::QQ(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Genomic Q-Q")
        .with_x_label("Expected −log₁₀(p)")
        .with_y_label("Observed −log₁₀(p)");
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/qq_genomic_basic.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"), "genomic Q-Q should produce circles");
    assert!(!svg.contains("NaN"), "no NaN in output");
}

#[test]
fn test_qq_genomic_mode_explicit() {
    outdir();
    let pvals: Vec<f64> = vec![0.0001, 0.001, 0.01, 0.05, 0.1, 0.3, 0.5, 0.8, 0.9];
    let plot = QQPlot::new()
        .with_pvalues("", pvals);
    assert_eq!(plot.mode, QQMode::Genomic, "with_pvalues should set Genomic mode");
    let plots = vec![Plot::QQ(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/qq_genomic_explicit.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_qq_genomic_ci_band() {
    outdir();
    let pvals: Vec<f64> = (1..=200).map(|i| i as f64 / 201.0).collect();
    let plot = QQPlot::new()
        .with_pvalues("GWAS", pvals)
        .with_ci_band();
    let plots = vec![Plot::QQ(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Genomic Q-Q with CI band");
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/qq_genomic_ci_band.svg", &svg).unwrap();
    // CI band is a filled path
    let path_count = svg.matches("<path").count();
    assert!(path_count >= 1, "CI band should produce at least one path, got {path_count}");
}

#[test]
fn test_qq_genomic_lambda() {
    outdir();
    // Inflate p-values slightly (λ > 1)
    let pvals: Vec<f64> = (1..=500).map(|i| (i as f64 / 501.0).powi(2)).collect();
    let plot = QQPlot::new()
        .with_pvalues("Inflated", pvals)
        .with_lambda();
    let plots = vec![Plot::QQ(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Genomic Q-Q with λ");
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/qq_genomic_lambda.svg", &svg).unwrap();
    assert!(svg.contains('λ') || svg.contains("&#955;") || svg.contains("λ"),
        "lambda annotation should be present");
}

#[test]
fn test_qq_genomic_no_inflation() {
    outdir();
    // Uniform p-values — null distributed, λ ≈ 1
    let pvals: Vec<f64> = (1..=1000).map(|i| i as f64 / 1001.0).collect();
    let plot = QQPlot::new()
        .with_pvalues("Null", pvals)
        .with_ci_band()
        .with_lambda();
    let plots = vec![Plot::QQ(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/qq_genomic_null.svg", &svg).unwrap();
    assert!(!svg.contains("NaN"));
    assert!(svg.contains("<svg"));
}

#[test]
fn test_qq_genomic_multigroup() {
    outdir();
    let pal = Palette::category10();
    let p1: Vec<f64> = (1..=100).map(|i| i as f64 / 101.0).collect();
    let p2: Vec<f64> = (1..=100).map(|i| (i as f64 / 101.0).powi(2)).collect();
    let plot = QQPlot::new()
        .with_pvalues_colored("Study A", p1, pal[0].to_string())
        .with_pvalues_colored("Study B", p2, pal[1].to_string())
        .with_legend("")
        .with_lambda();
    let plots = vec![Plot::QQ(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Multi-study Genomic Q-Q");
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/qq_genomic_multigroup.svg", &svg).unwrap();
    assert!(svg.contains("Study A"));
    assert!(svg.contains("Study B"));
}

// ── Edge cases ────────────────────────────────────────────────────────────────

#[test]
fn test_qq_single_point() {
    outdir();
    let plot = QQPlot::new().with_data("", vec![5.0]);
    let plots = vec![Plot::QQ(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/qq_single.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "single-point Q-Q should produce valid SVG");
}

#[test]
fn test_qq_empty() {
    outdir();
    let plot = QQPlot::new();
    let plots = vec![Plot::QQ(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_x_axis_min(-3.0).with_x_axis_max(3.0)
        .with_y_axis_min(-3.0).with_y_axis_max(3.0);
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/qq_empty.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "empty Q-Q should produce valid SVG");
}

#[test]
fn test_qq_bounds_normal() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let plot = QQPlot::new().with_data("", data.clone());
    let b = Plot::QQ(plot).bounds().expect("bounds should be Some");
    // y range should cover data min/max
    assert_eq!(b.1.0, 1.0);
    assert_eq!(b.1.1, 5.0);
    // x range should be finite theoretical quantile range
    assert!(b.0.0 < 0.0, "theoretical min should be negative");
    assert!(b.0.1 > 0.0, "theoretical max should be positive");
}

#[test]
fn test_qq_bounds_genomic() {
    let pvals = vec![0.001, 0.01, 0.1, 0.5, 0.9];
    let plot = QQPlot::new().with_pvalues("", pvals);
    let b = Plot::QQ(plot).bounds().expect("bounds should be Some");
    // Both axes should start at 0
    assert_eq!(b.0.0, 0.0, "genomic x_min should be 0");
    assert_eq!(b.1.0, 0.0, "genomic y_min should be 0");
    // y_max should be -log10(0.001) = 3.0
    assert!((b.1.1 - 3.0).abs() < 0.01, "y_max should be ~3.0, got {}", b.1.1);
}

#[test]
fn test_qq_from_impls() {
    let plot = QQPlot::new().with_data("A", vec![1.0, 2.0, 3.0]);
    let _: Plot = plot.into();
}

#[test]
fn test_qq_fill_opacity() {
    outdir();
    let data: Vec<f64> = (1..=50).map(|i| i as f64).collect();
    let plot = QQPlot::new()
        .with_data("", data)
        .with_color("steelblue")
        .with_fill_opacity(0.5);
    let plots = vec![Plot::QQ(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/qq_fill_opacity.svg", &svg).unwrap();
    assert!(svg.contains("fill-opacity"));
}
