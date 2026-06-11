mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::radar::RadarPlot;
#[allow(unused_imports)]
use kuva::plot::radar::RadarReference;
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

fn render_svg(plots: Vec<Plot>, layout: Layout) -> String {
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

#[test]
fn test_radar_basic() {
    let plot = RadarPlot::new(vec!["Speed", "Power", "Agility", "Stamina", "Technique"])
        .with_series(vec![0.8_f64, 0.6, 0.9, 0.7, 0.75]);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Radar Basic");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_basic.svg", &svg).unwrap();
    assert!(svg.contains("Speed"), "should contain axis label");
    assert!(svg.contains("Agility"), "should contain axis label");
    assert!(svg.contains(" Z"), "should contain closed polygon path");
}

#[test]
fn test_radar_two_series() {
    let plot = RadarPlot::new(vec!["A", "B", "C", "D", "E"])
        .with_series_labeled(vec![1.0_f64, 0.8, 0.6, 0.9, 0.7], "Group 1")
        .with_series_labeled(vec![0.5_f64, 0.7, 1.0, 0.4, 0.8], "Group 2");
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Radar Two Series");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_two_series.svg", &svg).unwrap();
    assert!(svg.contains("A"));
    assert!(svg.contains("B"));
}

#[test]
fn test_radar_filled() {
    let plot = RadarPlot::new(vec!["Alpha", "Beta", "Gamma", "Delta"])
        .with_series_labeled(vec![0.9_f64, 0.6, 0.8, 0.4], "Series A")
        .with_filled(true)
        .with_opacity(0.3);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Radar Filled");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_filled.svg", &svg).unwrap();
    assert!(svg.contains("Alpha"));
    assert!(svg.contains(" Z"));
}

#[test]
fn test_radar_with_legend() {
    let plot = RadarPlot::new(vec!["X", "Y", "Z", "W"])
        .with_series_labeled(vec![1.0_f64, 0.5, 0.8, 0.3], "Method A")
        .with_series_labeled(vec![0.6_f64, 0.9, 0.4, 0.7], "Method B")
        .with_legend(true);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Radar Legend");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_legend.svg", &svg).unwrap();
    assert!(svg.contains("Method A"), "legend entry A");
    assert!(svg.contains("Method B"), "legend entry B");
}

#[test]
fn test_radar_normalize() {
    let plot = RadarPlot::new(vec!["Precision", "Recall", "F1", "AUC"])
        .with_series_labeled(vec![0.92_f64, 0.85, 0.88, 0.94], "Model A")
        .with_series_labeled(vec![0.78_f64, 0.91, 0.84, 0.89], "Model B")
        .with_normalize(true)
        .with_filled(true);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Radar Normalized");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_normalize.svg", &svg).unwrap();
    assert!(svg.contains("%"), "normalize mode should show % labels");
}

#[test]
fn test_radar_explicit_range() {
    let plot = RadarPlot::new(vec!["Memory", "Speed", "Accuracy", "Throughput", "Latency"])
        .with_series_color(vec![80.0_f64, 95.0, 88.0, 72.0, 91.0], "Tool A", "#e41a1c")
        .with_series_color(vec![65.0_f64, 78.0, 92.0, 88.0, 75.0], "Tool B", "#377eb8")
        .with_range(0.0, 100.0)
        .with_grid_lines(4)
        .with_filled(true)
        .with_legend(true);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Radar Explicit Range");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_explicit_range.svg", &svg).unwrap();
    assert!(svg.contains("Memory"));
    assert!(svg.contains("Tool A"));
    assert!(svg.contains("Tool B"));
}

#[test]
fn test_radar_with_dots() {
    let plot = RadarPlot::new(vec!["A", "B", "C", "D", "E", "F"])
        .with_series(vec![0.5_f64, 0.8, 0.6, 0.9, 0.7, 0.4])
        .with_dot_size(4.0);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Radar Dots");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_dots.svg", &svg).unwrap();
    assert!(svg.contains("<circle"), "should render vertex dots");
}

#[test]
fn test_radar_no_grid() {
    let plot = RadarPlot::new(vec!["A", "B", "C", "D"])
        .with_series(vec![0.7_f64, 0.5, 0.9, 0.6])
        .with_grid(false);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Radar No Grid");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_no_grid.svg", &svg).unwrap();
    assert!(!svg.contains("4,3"), "dashed grid rings should be absent");
}

#[test]
fn test_radar_six_axes() {
    let plot = RadarPlot::new(vec![
        "Sensitivity",
        "Specificity",
        "PPV",
        "NPV",
        "F1",
        "MCC",
    ])
    .with_series_labeled(
        vec![0.91_f64, 0.87, 0.84, 0.93, 0.875, 0.78],
        "Classifier A",
    )
    .with_series_labeled(
        vec![0.85_f64, 0.92, 0.90, 0.88, 0.875, 0.77],
        "Classifier B",
    )
    .with_filled(true)
    .with_legend(true);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Radar Six Axes");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_six_axes.svg", &svg).unwrap();
    assert!(svg.contains("Sensitivity"));
    assert!(svg.contains("MCC"));
    assert!(svg.contains("Classifier A"));
}

#[test]
fn test_radar_three_axes_minimum() {
    let plot = RadarPlot::new(vec!["X", "Y", "Z"]).with_series(vec![1.0_f64, 0.5, 0.8]);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Radar Three Axes");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_three_axes.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains(" Z"));
}

#[test]
fn test_radar_custom_stroke_width() {
    let plot = RadarPlot::new(vec!["A", "B", "C", "D", "E"])
        .with_series(vec![0.6_f64, 0.8, 0.5, 0.9, 0.7])
        .with_stroke_width(3.0);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Radar Stroke Width");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_stroke_width.svg", &svg).unwrap();
    assert!(svg.contains("stroke-width=\"3\"") || svg.contains("stroke-width=\"3.0\""));
}

#[test]
fn test_radar_into_plot() {
    let plot: Plot = RadarPlot::new(vec!["A", "B", "C", "D"])
        .with_series(vec![1.0_f64, 0.5, 0.8, 0.6])
        .into();
    assert!(matches!(plot, Plot::Radar(_)));
}

#[test]
fn test_radar_benchmarking_use_case() {
    let plot = RadarPlot::new(vec![
        "Precision",
        "Recall",
        "Speed",
        "Memory",
        "Scalability",
    ])
    .with_series_color(vec![0.94_f64, 0.91, 0.78, 0.85, 0.72], "Tool A", "#e41a1c")
    .with_series_color(vec![0.88_f64, 0.95, 0.92, 0.70, 0.88], "Tool B", "#377eb8")
    .with_series_color(vec![0.81_f64, 0.83, 0.96, 0.92, 0.95], "Tool C", "#4daf4a")
    .with_filled(true)
    .with_opacity(0.2)
    .with_legend(true)
    .with_range(0.0, 1.0);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Tool Comparison");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_benchmarking.svg", &svg).unwrap();
    assert!(svg.contains("Precision"));
    assert!(svg.contains("Tool A"));
    assert!(svg.contains("Tool C"));
}

// ── New feature tests ─────────────────────────────────────────────────────────

#[test]
fn test_radar_inverted_axis() {
    let plot = RadarPlot::new(vec!["Latency", "Throughput", "Precision", "Recall"])
        .with_series_labeled(vec![5.0_f64, 0.8, 0.9, 0.85], "System A")
        .with_series_labeled(vec![2.0_f64, 0.6, 0.95, 0.78], "System B")
        .with_inverted_axis(0) // low latency = good → invert so it plots at rim
        .with_legend(true);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Inverted Axis");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_inverted_axis.svg", &svg).unwrap();
    assert!(svg.contains("Latency"));
    assert!(svg.contains("System A"));
}

#[test]
fn test_radar_inverted_multiple_axes() {
    let plot = RadarPlot::new(vec!["Error Rate", "Latency", "Recall", "F1"])
        .with_series(vec![0.02_f64, 10.0, 0.88, 0.85])
        .with_inverted_axes([0, 1]) // lower error and latency = better
        .with_range(0.0, 1.0);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Inverted Axes");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_inverted_axes.svg", &svg).unwrap();
    assert!(svg.contains("Error Rate"));
    assert!(svg.contains(" Z"));
}

#[test]
fn test_radar_per_axis_range() {
    let plot = RadarPlot::new(vec!["Speed (ms)", "Accuracy (%)", "Memory (MB)", "Score"])
        .with_series_labeled(vec![120.0_f64, 94.0, 512.0, 0.88], "Model A")
        .with_series_labeled(vec![85.0_f64, 97.0, 1024.0, 0.92], "Model B")
        .with_axis_range(0, 0.0, 200.0)
        .with_axis_range(1, 80.0, 100.0)
        .with_axis_range(2, 0.0, 2048.0)
        .with_axis_range(3, 0.0, 1.0)
        .with_filled(true)
        .with_legend(true);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Per-Axis Ranges");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_per_axis_range.svg", &svg).unwrap();
    assert!(svg.contains("Speed"));
    assert!(svg.contains("Model A"));
}

#[test]
fn test_radar_error_band() {
    let plot = RadarPlot::new(vec!["Sensitivity", "Specificity", "Precision", "F1", "AUC"])
        .with_series_labeled(vec![0.88_f64, 0.92, 0.87, 0.875, 0.95], "Classifier")
        .with_series_errors(vec![0.04_f64, 0.03, 0.05, 0.04, 0.02])
        .with_filled(true)
        .with_opacity(0.3);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Error Bands");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_error_band.svg", &svg).unwrap();
    assert!(svg.contains("Sensitivity"));
    assert!(svg.contains(" Z"), "should have filled polygons");
}

#[test]
fn test_radar_vertex_labels() {
    let plot = RadarPlot::new(vec!["A", "B", "C", "D", "E"])
        .with_series(vec![0.8_f64, 0.6, 0.9, 0.7, 0.75])
        .with_vertex_labels(true)
        .with_dot_size(3.0);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Vertex Labels");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_vertex_labels.svg", &svg).unwrap();
    assert!(
        svg.contains("0.80") || svg.contains("0.8"),
        "vertex value label present"
    );
}

#[test]
fn test_radar_circular_grid() {
    let plot = RadarPlot::new(vec!["Alpha", "Beta", "Gamma", "Delta", "Epsilon"])
        .with_series_labeled(vec![0.9_f64, 0.6, 0.8, 0.7, 0.5], "Series A")
        .with_circular_grid(true)
        .with_grid_lines(4);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Circular Grid");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_circular_grid.svg", &svg).unwrap();
    assert!(svg.contains("Alpha"));
    // Circular grid uses arc paths; polygon grid uses "4,3" but both are dashed
    assert!(svg.contains("4,3"), "dashed ring should be present");
}

#[test]
fn test_radar_reference_polygon() {
    let plot = RadarPlot::new(vec!["Speed", "Power", "Agility", "Stamina", "Technique"])
        .with_series_labeled(vec![0.8_f64, 0.6, 0.9, 0.7, 0.75], "Model")
        .with_reference(vec![0.7_f64, 0.7, 0.7, 0.7, 0.7], "Baseline")
        .with_legend(true);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Reference Polygon");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_reference.svg", &svg).unwrap();
    assert!(svg.contains("Baseline"), "reference label in legend");
    assert!(svg.contains("6,3"), "reference polygon uses 6,3 dasharray");
}

#[test]
fn test_radar_reference_with_color() {
    let plot = RadarPlot::new(vec!["A", "B", "C", "D"])
        .with_series_labeled(vec![0.9_f64, 0.7, 0.8, 0.6], "Current")
        .with_reference_color(vec![0.6_f64, 0.6, 0.6, 0.6], "Target", "#ff7f00")
        .with_legend(true);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Reference With Color");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_reference_color.svg", &svg).unwrap();
    assert!(svg.contains("#ff7f00"), "custom reference color");
    assert!(svg.contains("Target"));
}

#[test]
fn test_radar_start_angle() {
    let plot = RadarPlot::new(vec!["N", "E", "S", "W"])
        .with_series(vec![0.8_f64, 0.5, 0.6, 0.9])
        .with_start_angle(0.0); // axis 0 points right (east)
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Start Angle East");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_start_angle.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains(" Z"));
}

#[test]
fn test_radar_start_axis() {
    let plot = RadarPlot::new(vec!["Alpha", "Beta", "Gamma", "Delta", "Epsilon"])
        .with_series_labeled(vec![0.9_f64, 0.6, 0.8, 0.7, 0.5], "Data")
        .with_start_axis(2); // Gamma at top
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Start Axis");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_start_axis.svg", &svg).unwrap();
    assert!(svg.contains("Gamma"));
    assert!(svg.contains("Alpha"));
}

#[test]
fn test_radar_per_series_dasharray() {
    let plot = RadarPlot::new(vec!["A", "B", "C", "D", "E"])
        .with_series_labeled(vec![0.8_f64, 0.6, 0.9, 0.7, 0.75], "Solid")
        .with_series_labeled(vec![0.5_f64, 0.8, 0.6, 0.9, 0.4], "Dashed")
        .with_series_dasharray("6,3");
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Series Dash Patterns");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_dasharray.svg", &svg).unwrap();
    assert!(svg.contains("6,3"), "dashed series stroke-dasharray");
}

#[test]
fn test_radar_axis_ticks() {
    let plot = RadarPlot::new(vec!["A", "B", "C", "D", "E"])
        .with_series(vec![0.7_f64, 0.5, 0.9, 0.6, 0.8])
        .with_axis_ticks(true)
        .with_grid_lines(4);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Axis Ticks");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_axis_ticks.svg", &svg).unwrap();
    assert!(svg.contains("<line"), "tick marks are line primitives");
}

#[test]
fn test_radar_long_label_wrap() {
    let plot = RadarPlot::new(vec![
        "Positive Predictive Value",
        "Negative Predictive Value",
        "True Positive Rate",
        "False Positive Rate",
        "Matthews Correlation",
    ])
    .with_series(vec![0.88_f64, 0.91, 0.87, 0.12, 0.79]);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Long Label Wrap");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_long_labels.svg", &svg).unwrap();
    assert!(svg.contains("Positive"), "label text present");
    assert!(svg.contains("Predictive"), "wrapped line present");
}

#[test]
fn test_radar_full_features() {
    // Comprehensive test exercising multiple new features together
    let plot = RadarPlot::new(vec!["Sensitivity", "Specificity", "Precision", "F1", "AUC"])
        .with_series_color(vec![0.91_f64, 0.88, 0.89, 0.90, 0.95], "Model A", "#e41a1c")
        .with_series_errors(vec![0.03_f64, 0.04, 0.03, 0.03, 0.02])
        .with_series_color(vec![0.84_f64, 0.93, 0.92, 0.88, 0.91], "Model B", "#377eb8")
        .with_series_dasharray("5,2")
        .with_reference(vec![0.80_f64, 0.80, 0.80, 0.80, 0.80], "Threshold")
        .with_filled(true)
        .with_opacity(0.2)
        .with_circular_grid(true)
        .with_axis_ticks(true)
        .with_vertex_labels(true)
        .with_legend(true)
        .with_range(0.0, 1.0);
    let plots = vec![Plot::Radar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Full Feature Demo");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/radar_full_features.svg", &svg).unwrap();
    assert!(svg.contains("Model A"));
    assert!(svg.contains("Model B"));
    assert!(svg.contains("Threshold"));
    assert!(svg.contains("5,2"), "dashed series present");
    assert!(svg.contains("6,3"), "reference polygon present");
}
