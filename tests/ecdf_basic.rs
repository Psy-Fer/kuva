mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::EcdfPlot;
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use std::fs;

fn render_svg(plots: Vec<Plot>, layout: Layout) -> String {
    let scene = render_multiple(plots, layout);
    SvgBackend.render_scene(&scene)
}

fn outdir() {
    fs::create_dir_all("test_outputs").ok();
}

#[test]
fn test_ecdf_basic() {
    outdir();
    let data: Vec<f64> = vec![1.2, 3.4, 2.1, 5.6, 4.0, 0.8, 3.3, 2.7, 4.5, 1.9];
    let plot = EcdfPlot::new()
        .with_data("Sample", data)
        .with_color("steelblue");
    let plots = vec![Plot::Ecdf(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("ECDF Basic")
        .with_x_label("Value")
        .with_y_label("F(x)");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/ecdf_basic.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"), "ECDF should produce a path element");
}

#[test]
fn test_ecdf_step_function_shape() {
    outdir();
    // 4 data points → 4 steps at F = 0.25, 0.5, 0.75, 1.0
    let data = vec![1.0, 2.0, 3.0, 4.0];
    let plot = EcdfPlot::new().with_data("", data).with_color("steelblue");
    let plots = vec![Plot::Ecdf(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/ecdf_step.svg", &svg).unwrap();
    assert!(
        svg.contains('V'),
        "step function should use V (vertical) path commands"
    );
    assert!(
        svg.contains('H'),
        "step function should use H (horizontal) path commands"
    );
}

#[test]
fn test_ecdf_complementary() {
    outdir();
    let data: Vec<f64> = (1..=20).map(|i| i as f64).collect();
    let plot = EcdfPlot::new()
        .with_data("Read lengths", data)
        .with_color("tomato")
        .with_complementary();
    let plots = vec![Plot::Ecdf(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("CCDF (Complementary)")
        .with_y_label("1 - F(x)");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/ecdf_complementary.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"));
}

#[test]
fn test_ecdf_confidence_band() {
    outdir();
    let data: Vec<f64> = vec![
        0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 1.2, 2.2, 3.2, 1.8, 2.8, 3.8, 0.8, 4.2,
        2.4, 3.6,
    ];
    let plot = EcdfPlot::new()
        .with_data("Sample", data)
        .with_color("steelblue")
        .with_confidence_band();
    let plots = vec![Plot::Ecdf(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("ECDF with DKW Band");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/ecdf_confidence_band.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    // Band is a filled path; main line is unfilled
    let path_count = svg.matches("<path").count();
    assert!(
        path_count >= 2,
        "expected band + line paths, got {path_count}"
    );
}

#[test]
fn test_ecdf_rug() {
    outdir();
    let data = vec![1.0, 1.5, 2.0, 3.5, 4.0, 4.5];
    let plot = EcdfPlot::new()
        .with_data("", data)
        .with_color("seagreen")
        .with_rug();
    let plots = vec![Plot::Ecdf(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("ECDF with Rug");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/ecdf_rug.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    // Rug produces line elements
    assert!(svg.contains("<line"), "rug should produce line elements");
}

#[test]
fn test_ecdf_percentile_lines() {
    outdir();
    let data: Vec<f64> = (1..=100).map(|i| i as f64).collect();
    let plot = EcdfPlot::new()
        .with_data("", data)
        .with_color("steelblue")
        .with_percentile_lines(vec![0.25, 0.5, 0.75]);
    let plots = vec![Plot::Ecdf(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("ECDF with Percentiles");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/ecdf_percentile_lines.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    // 25%, 50%, 75% labels
    assert!(svg.contains("25%"), "should contain 25% label");
    assert!(svg.contains("50%"), "should contain 50% label");
    assert!(svg.contains("75%"), "should contain 75% label");
}

#[test]
fn test_ecdf_markers() {
    outdir();
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let plot = EcdfPlot::new()
        .with_data("", data)
        .with_color("orchid")
        .with_markers();
    let plots = vec![Plot::Ecdf(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("ECDF with Markers");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/ecdf_markers.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(
        svg.contains("<circle"),
        "markers should produce circle elements"
    );
    // 5 data points → 5 markers
    let circle_count = svg.matches("<circle").count();
    assert_eq!(
        circle_count, 5,
        "expected 5 circle markers, got {circle_count}"
    );
}

#[test]
fn test_ecdf_smooth() {
    outdir();
    let data: Vec<f64> = (0..50).map(|i| (i as f64) * 0.1 + 0.5).collect();
    let plot = EcdfPlot::new()
        .with_data("", data)
        .with_color("darkorange")
        .with_smooth();
    let plots = vec![Plot::Ecdf(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Smooth ECDF");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/ecdf_smooth.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"));
    // Smooth should not contain H/V (it uses L commands)
    assert!(
        !svg.contains(" V ") || !svg.contains(" H "),
        "smooth mode should not produce H/V step commands in isolation"
    );
}

#[test]
fn test_ecdf_multigroup() {
    outdir();
    let pal = Palette::category10();
    let control: Vec<f64> = vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 2.2, 1.8, 3.2];
    let treated: Vec<f64> = vec![2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5, 3.8, 4.2, 5.2];

    let plot = EcdfPlot::new()
        .with_data_colored("Control", control, pal[0].to_string())
        .with_data_colored("Treated", treated, pal[1].to_string())
        .with_legend("Groups");

    let plots = vec![Plot::Ecdf(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("ECDF Multi-Group");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/ecdf_multigroup.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    let path_count = svg.matches("<path").count();
    assert!(
        path_count >= 2,
        "two groups should produce at least 2 paths, got {path_count}"
    );
    assert!(svg.contains("Control"), "legend should contain 'Control'");
    assert!(svg.contains("Treated"), "legend should contain 'Treated'");
}

#[test]
fn test_ecdf_multigroup_with_bands() {
    outdir();
    let a: Vec<f64> = (0..30).map(|i| i as f64 * 0.1).collect();
    let b: Vec<f64> = (0..30).map(|i| i as f64 * 0.1 + 1.0).collect();

    let plot = EcdfPlot::new()
        .with_data("A", a)
        .with_data("B", b)
        .with_confidence_band()
        .with_legend("Groups");

    let plots = vec![Plot::Ecdf(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Multi-Group ECDF with Bands");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/ecdf_multigroup_bands.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    // 2 bands + 2 step lines = 4 paths minimum
    let path_count = svg.matches("<path").count();
    assert!(
        path_count >= 4,
        "expected band+line per group, got {path_count}"
    );
}

#[test]
fn test_ecdf_complementary_rug() {
    outdir();
    // Simulated nanopore read-length-like distribution
    let data: Vec<f64> = vec![
        500.0, 800.0, 1200.0, 2000.0, 3500.0, 5000.0, 8000.0, 12000.0, 500.0, 1000.0, 1500.0,
        2500.0, 4000.0, 6000.0, 10000.0, 800.0, 1100.0, 3000.0,
    ];
    let plot = EcdfPlot::new()
        .with_data("Sample", data)
        .with_color("steelblue")
        .with_complementary()
        .with_rug();
    let plots = vec![Plot::Ecdf(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Read Length Distribution (CCDF)")
        .with_x_label("Read length (bp)")
        .with_y_label("Fraction ≥ length")
        .with_log_x();
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/ecdf_ccdf_rug.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(!svg.contains("NaN"), "SVG should not contain NaN");
}

#[test]
fn test_ecdf_all_features() {
    outdir();
    let data: Vec<f64> = (1..=50).map(|i| i as f64 * 0.5).collect();
    let plot = EcdfPlot::new()
        .with_data("Sample", data)
        .with_color("steelblue")
        .with_confidence_band()
        .with_rug()
        .with_percentile_lines(vec![0.5])
        .with_markers()
        .with_legend("Full Example");
    let plots = vec![Plot::Ecdf(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("ECDF All Features");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/ecdf_all_features.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(!svg.contains("NaN"));
}

#[test]
fn test_ecdf_empty() {
    outdir();
    let plot = EcdfPlot::new();
    let plots = vec![Plot::Ecdf(plot)];
    // Empty plot has no bounds — provide explicit range
    let layout = Layout::auto_from_plots(&plots)
        .with_x_axis_min(0.0)
        .with_x_axis_max(1.0)
        .with_y_axis_min(0.0)
        .with_y_axis_max(1.0);
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/ecdf_empty.svg", &svg).unwrap();
    assert!(
        svg.contains("<svg"),
        "empty ECDF should still produce valid SVG"
    );
}

#[test]
fn test_ecdf_single_point() {
    outdir();
    let plot = EcdfPlot::new().with_data("", vec![5.0]);
    let plots = vec![Plot::Ecdf(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Single Point ECDF");
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/ecdf_single.svg", &svg).unwrap();
    assert!(
        svg.contains("<svg"),
        "single-point ECDF should produce valid SVG"
    );
}

#[test]
fn test_ecdf_bounds() {
    let data = vec![2.0, 5.0, 8.0, 1.0, 9.0];
    let plot = EcdfPlot::new().with_data("", data);
    let b = Plot::Ecdf(plot).bounds().expect("bounds should be Some");
    assert_eq!(b.0, (1.0, 9.0), "x range should be data min/max");
    assert_eq!(b.1, (0.0, 1.0), "y range should always be [0, 1]");
}

#[test]
fn test_ecdf_from_impls() {
    // Verify Into<Plot> works
    let plot = EcdfPlot::new().with_data("A", vec![1.0, 2.0, 3.0]);
    let _: Plot = plot.into();
}
