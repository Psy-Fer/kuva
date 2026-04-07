use kuva::prelude::*;
use kuva::plot::scatter::{MarkerShape, TrendLine};
use std::fs;

fn sample_data(n: usize, seed: u64) -> (Vec<f64>, Vec<f64>) {
    // Simple LCG for deterministic test data without rand dependency
    let mut v = seed;
    let mut x = Vec::with_capacity(n);
    let mut y = Vec::with_capacity(n);
    for _ in 0..n {
        v = v.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let fx = (v >> 33) as f64 / (u32::MAX as f64);
        v = v.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let fy = (v >> 33) as f64 / (u32::MAX as f64);
        x.push(fx * 10.0 - 5.0);
        y.push(fy * 10.0 - 5.0);
    }
    (x, y)
}

fn write(name: &str, svg: &str) {
    fs::create_dir_all("test_outputs").ok();
    fs::write(format!("test_outputs/{name}"), svg).unwrap();
}

#[test]
fn test_jointplot_basic() {
    let (x, y) = sample_data(100, 42);
    let jp = JointPlot::new()
        .with_xy(x, y)
        .with_x_label("X")
        .with_y_label("Y");
    let layout = Layout::new((-6.0, 6.0), (-6.0, 6.0)).with_title("JointPlot Basic");
    let svg = SvgBackend.render_scene(&render_jointplot(jp, layout));
    write("jointplot_basic.svg", &svg);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_jointplot_density() {
    let (x, y) = sample_data(200, 1);
    let jp = JointPlot::new()
        .with_xy(x, y)
        .with_marginal_type(MarginalType::Density)
        .with_x_label("Feature A")
        .with_y_label("Feature B");
    let layout = Layout::new((-6.0, 6.0), (-6.0, 6.0)).with_title("JointPlot Density");
    let svg = SvgBackend.render_scene(&render_jointplot(jp, layout));
    write("jointplot_density.svg", &svg);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_jointplot_two_groups() {
    let (x1, y1) = sample_data(80, 10);
    let (x2, y2) = sample_data(80, 20);
    let x2: Vec<f64> = x2.iter().map(|v| v + 3.0).collect();
    let y2: Vec<f64> = y2.iter().map(|v| v + 3.0).collect();
    let jp = JointPlot::new()
        .with_group("Group A", x1, y1, "#4e79a7")
        .with_group("Group B", x2, y2, "#f28e2b")
        .with_x_label("X")
        .with_y_label("Y");
    let layout = Layout::new((-6.0, 9.0), (-6.0, 9.0)).with_title("JointPlot Two Groups");
    let svg = SvgBackend.render_scene(&render_jointplot(jp, layout));
    write("jointplot_two_groups.svg", &svg);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_jointplot_top_only() {
    let (x, y) = sample_data(100, 3);
    let jp = JointPlot::new()
        .with_xy(x, y)
        .with_right_marginal(false);
    let layout = Layout::new((-6.0, 6.0), (-6.0, 6.0)).with_title("JointPlot Top Only");
    let svg = SvgBackend.render_scene(&render_jointplot(jp, layout));
    write("jointplot_top_only.svg", &svg);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_jointplot_right_only() {
    let (x, y) = sample_data(100, 4);
    let jp = JointPlot::new()
        .with_xy(x, y)
        .with_top_marginal(false);
    let layout = Layout::new((-6.0, 6.0), (-6.0, 6.0)).with_title("JointPlot Right Only");
    let svg = SvgBackend.render_scene(&render_jointplot(jp, layout));
    write("jointplot_right_only.svg", &svg);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_jointplot_no_marginals() {
    let (x, y) = sample_data(50, 5);
    let jp = JointPlot::new()
        .with_xy(x, y)
        .with_top_marginal(false)
        .with_right_marginal(false)
        .with_x_label("X")
        .with_y_label("Y");
    let layout = Layout::new((-6.0, 6.0), (-6.0, 6.0)).with_title("JointPlot Scatter Only");
    let svg = SvgBackend.render_scene(&render_jointplot(jp, layout));
    write("jointplot_no_marginals.svg", &svg);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_jointplot_large_marginals() {
    let (x, y) = sample_data(150, 6);
    let jp = JointPlot::new()
        .with_xy(x, y)
        .with_marginal_size(120.0)
        .with_bins(30);
    let layout = Layout::new((-6.0, 6.0), (-6.0, 6.0)).with_title("JointPlot Large Marginals");
    let svg = SvgBackend.render_scene(&render_jointplot(jp, layout));
    write("jointplot_large_marginals.svg", &svg);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_jointplot_custom_size() {
    let (x, y) = sample_data(100, 7);
    let jp = JointPlot::new().with_xy(x, y);
    let layout = Layout::new((-6.0, 6.0), (-6.0, 6.0))
        .with_title("JointPlot 700×600")
        .with_width(700.0)
        .with_height(600.0);
    let svg = SvgBackend.render_scene(&render_jointplot(jp, layout));
    write("jointplot_custom_size.svg", &svg);
    assert!(svg.contains("700"));
}

#[test]
fn test_jointplot_gene_expression() {
    // Realistic: log2 TPM vs log2 fold change
    let (x, y) = sample_data(300, 99);
    let x: Vec<f64> = x.iter().map(|v| v * 1.5).collect(); // log2 TPM range
    let y: Vec<f64> = y.iter().map(|v| v * 0.8).collect(); // log2FC range
    let jp = JointPlot::new()
        .with_xy(x, y)
        .with_marginal_type(MarginalType::Density)
        .with_x_label("log2 TPM")
        .with_y_label("log2 Fold Change")
        .with_marker_size(3.0)
        .with_marker_opacity(0.5);
    let layout = Layout::new((-8.0, 8.0), (-5.0, 5.0))
        .with_title("Expression vs Fold Change")
        .with_width(520.0)
        .with_height(520.0);
    let svg = SvgBackend.render_scene(&render_jointplot(jp, layout));
    write("jointplot_gene_expression.svg", &svg);
    assert!(svg.contains("<svg"));
}

// ── Feature parity tests ──────────────────────────────────────────────────

#[test]
fn test_jointplot_legend() {
    let (x1, y1) = sample_data(80, 30);
    let (x2, y2) = sample_data(80, 40);
    let x2: Vec<f64> = x2.iter().map(|v| v + 2.0).collect();
    let y2: Vec<f64> = y2.iter().map(|v| v + 2.0).collect();
    let jp = JointPlot::new()
        .with_group("Control", x1, y1, "#4e79a7")
        .with_group("Treated", x2, y2, "#f28e2b")
        .with_x_label("X")
        .with_y_label("Y");
    // Legend is auto-enabled because both groups have labels
    let layout = Layout::new((-6.0, 9.0), (-6.0, 9.0))
        .with_title("JointPlot Legend");
    let svg = SvgBackend.render_scene(&render_jointplot(jp, layout));
    write("jointplot_legend.svg", &svg);
    // Both group labels should appear in the SVG
    assert!(svg.contains("Control"), "legend entry 'Control' missing");
    assert!(svg.contains("Treated"), "legend entry 'Treated' missing");
}

#[test]
fn test_jointplot_trend_line() {
    let (x, y) = sample_data(100, 50);
    let group = JointGroup::new(x, y)
        .with_color("#e15759")
        .with_trend(TrendLine::Linear)
        .with_trend_color("#333333")
        .with_correlation();
    let jp = JointPlot::new()
        .with_joint_group(group)
        .with_x_label("X")
        .with_y_label("Y");
    let layout = Layout::new((-6.0, 6.0), (-6.0, 6.0)).with_title("JointPlot Trend Line");
    let svg = SvgBackend.render_scene(&render_jointplot(jp, layout));
    write("jointplot_trend_line.svg", &svg);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_jointplot_error_bars() {
    let (x, y) = sample_data(30, 60);
    let x_err: Vec<f64> = vec![0.2; 30];
    let y_err: Vec<f64> = vec![0.3; 30];
    let group = JointGroup::new(x, y)
        .with_color("#76b7b2")
        .with_x_err(x_err)
        .with_y_err(y_err);
    let jp = JointPlot::new()
        .with_joint_group(group)
        .with_x_label("Measurement")
        .with_y_label("Response");
    let layout = Layout::new((-6.0, 6.0), (-6.0, 6.0)).with_title("JointPlot Error Bars");
    let svg = SvgBackend.render_scene(&render_jointplot(jp, layout));
    write("jointplot_error_bars.svg", &svg);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_jointplot_marker_shape() {
    let (x, y) = sample_data(60, 70);
    let group = JointGroup::new(x, y)
        .with_color("#59a14f")
        .with_marker(MarkerShape::Square)
        .with_marker_size(5.0)
        .with_marker_stroke_width(1.0);
    let jp = JointPlot::new()
        .with_joint_group(group)
        .with_marginal_type(MarginalType::Density);
    let layout = Layout::new((-6.0, 6.0), (-6.0, 6.0)).with_title("JointPlot Square Markers");
    let svg = SvgBackend.render_scene(&render_jointplot(jp, layout));
    write("jointplot_marker_shape.svg", &svg);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_jointplot_per_point_colors() {
    let (x, y) = sample_data(50, 80);
    // Color points by x value (positive = blue, negative = red)
    let colors: Vec<String> = x.iter()
        .map(|&v| if v > 0.0 { "#4e79a7".to_string() } else { "#e15759".to_string() })
        .collect();
    let group = JointGroup::new(x, y)
        .with_colors(colors);
    let jp = JointPlot::new()
        .with_joint_group(group)
        .with_x_label("X")
        .with_y_label("Y");
    let layout = Layout::new((-6.0, 6.0), (-6.0, 6.0)).with_title("JointPlot Per-Point Colors");
    let svg = SvgBackend.render_scene(&render_jointplot(jp, layout));
    write("jointplot_per_point_colors.svg", &svg);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_jointplot_tooltips() {
    let (x, y) = sample_data(40, 90);
    let labels: Vec<String> = (0..40).map(|i| format!("Point {i}")).collect();
    let group = JointGroup::new(x, y)
        .with_color("#b07aa1")
        .with_tooltips()
        .with_tooltip_labels(labels);
    let jp = JointPlot::new()
        .with_joint_group(group);
    let layout = Layout::new((-6.0, 6.0), (-6.0, 6.0)).with_title("JointPlot Tooltips");
    let svg = SvgBackend.render_scene(&render_jointplot(jp, layout));
    write("jointplot_tooltips.svg", &svg);
    // Tooltip groups inject JS / title elements
    assert!(svg.contains("<svg"));
}
