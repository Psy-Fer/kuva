use kuva::plot::funnel::{FunnelPlot, FunnelStage, FunnelColorMode, FunnelOrientation};
use kuva::render::{plots::Plot, layout::Layout, render::render_multiple};
use kuva::backend::svg::SvgBackend;

fn render(fp: FunnelPlot, title: &str) -> String {
    let plots = vec![Plot::Funnel(fp)];
    let layout = Layout::auto_from_plots(&plots).with_title(title);
    let backend = SvgBackend;
    backend.render_scene(&render_multiple(plots, layout))
}

fn render_size(fp: FunnelPlot, title: &str, w: f64, h: f64) -> String {
    let plots = vec![Plot::Funnel(fp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title(title)
        .with_width(w)
        .with_height(h);
    let backend = SvgBackend;
    backend.render_scene(&render_multiple(plots, layout))
}

#[test]
fn test_funnel_basic() {
    let fp = FunnelPlot::new()
        .with_stage("Screened",  1200)
        .with_stage("Eligible",   800)
        .with_stage("Enrolled",   600)
        .with_stage("Completed",  540);
    let svg = render(fp, "Basic Funnel");
    std::fs::write("test_outputs/funnel_basic.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "should have rect elements");
}

#[test]
fn test_funnel_empty() {
    let fp = FunnelPlot::new();
    let svg = render(fp, "Empty Funnel");
    std::fs::write("test_outputs/funnel_empty.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "should produce valid SVG");
}

#[test]
fn test_funnel_single_stage() {
    let fp = FunnelPlot::new().with_stage("Only", 500);
    let svg = render(fp, "Single Stage");
    std::fs::write("test_outputs/funnel_single.svg", &svg).unwrap();
    assert!(svg.contains("<rect"));
}

#[test]
fn test_funnel_with_stage_color() {
    let fp = FunnelPlot::new()
        .with_stage_color("A", 1000.0, "#e74c3c")
        .with_stage_color("B",  700.0, "#3498db")
        .with_stage_color("C",  400.0, "#2ecc71");
    let svg = render(fp, "Explicit Colors");
    std::fs::write("test_outputs/funnel_explicit_colors.svg", &svg).unwrap();
    assert!(svg.contains("#e74c3c") || svg.contains("#E74C3C"));
}

#[test]
fn test_funnel_color_by_stage() {
    let fp = FunnelPlot::new()
        .with_stage("A", 1000)
        .with_stage("B",  700)
        .with_stage("C",  400)
        .with_color_mode(FunnelColorMode::ByStage);
    let svg = render(fp, "Color By Stage");
    std::fs::write("test_outputs/funnel_by_stage.svg", &svg).unwrap();
    assert!(svg.contains("<rect"));
}

#[test]
fn test_funnel_color_gradient() {
    let fp = FunnelPlot::new()
        .with_stage("A", 1000)
        .with_stage("B",  700)
        .with_stage("C",  400)
        .with_color_mode(FunnelColorMode::Gradient);
    let svg = render(fp, "Color Gradient");
    std::fs::write("test_outputs/funnel_gradient.svg", &svg).unwrap();
    assert!(svg.contains("<rect"));
}

#[test]
fn test_funnel_no_connectors() {
    let fp = FunnelPlot::new()
        .with_stage("A", 1000)
        .with_stage("B",  700)
        .with_connectors(false);
    let svg = render(fp, "No Connectors");
    std::fs::write("test_outputs/funnel_no_connectors.svg", &svg).unwrap();
    assert!(svg.contains("<rect"));
}

#[test]
fn test_funnel_connector_opacity() {
    let fp = FunnelPlot::new()
        .with_stage("A", 1000)
        .with_stage("B",  700)
        .with_stage("C",  400)
        .with_connector_opacity(0.8);
    let svg = render(fp, "Connector Opacity");
    std::fs::write("test_outputs/funnel_connector_opacity.svg", &svg).unwrap();
    assert!(svg.contains("<rect"));
}

#[test]
fn test_funnel_show_percents() {
    let fp = FunnelPlot::new()
        .with_stage("A", 1000)
        .with_stage("B",  700)
        .with_stage("C",  400)
        .with_show_percents(true);
    let svg = render(fp, "Show Percents");
    std::fs::write("test_outputs/funnel_percents.svg", &svg).unwrap();
    assert!(svg.contains('%'), "should show percentage signs");
}

#[test]
fn test_funnel_no_values() {
    let fp = FunnelPlot::new()
        .with_stage("A", 1000)
        .with_stage("B",  700)
        .with_show_values(false)
        .with_show_conversion(false);
    let svg = render(fp, "No Values");
    std::fs::write("test_outputs/funnel_no_values.svg", &svg).unwrap();
    assert!(svg.contains("<rect"));
}

#[test]
fn test_funnel_no_conversion() {
    let fp = FunnelPlot::new()
        .with_stage("A", 1000)
        .with_stage("B",  700)
        .with_stage_gap(30.0)   // large gap so conversion would show if enabled
        .with_show_conversion(false);
    let svg = render(fp, "No Conversion");
    std::fs::write("test_outputs/funnel_no_conversion.svg", &svg).unwrap();
    assert!(svg.contains("<rect"));
}

#[test]
fn test_funnel_stage_gap() {
    let fp = FunnelPlot::new()
        .with_stage("A", 1000)
        .with_stage("B",  700)
        .with_stage("C",  400)
        .with_stage_gap(20.0);
    let svg = render(fp, "Stage Gap");
    std::fs::write("test_outputs/funnel_stage_gap.svg", &svg).unwrap();
    assert!(svg.contains("<rect"));
}

#[test]
fn test_funnel_horizontal() {
    let fp = FunnelPlot::new()
        .with_stage("Screened",  1200)
        .with_stage("Eligible",   800)
        .with_stage("Enrolled",   600)
        .with_stage("Completed",  540)
        .with_orientation(FunnelOrientation::Horizontal);
    let svg = render(fp, "Horizontal Funnel");
    std::fs::write("test_outputs/funnel_horizontal.svg", &svg).unwrap();
    assert!(svg.contains("<rect"));
}

#[test]
fn test_funnel_mirror_vertical() {
    let fp = FunnelPlot::new()
        .with_stage("Screened",  1200)
        .with_stage("Eligible",   800)
        .with_stage("Enrolled",   600)
        .with_mirror_stages([
            ("Screened", 1100.0),
            ("Eligible",  750.0),
            ("Enrolled",  520.0),
        ])
        .with_mirror_labels("Treatment", "Control");
    let svg = render(fp, "Mirror Vertical");
    std::fs::write("test_outputs/funnel_mirror_vertical.svg", &svg).unwrap();
    assert!(svg.contains("<rect"));
}

#[test]
fn test_funnel_mirror_with_mirror_stages_api() {
    let stages = vec![
        FunnelStage::new("A", 1000.0),
        FunnelStage::new("B",  800.0),
        FunnelStage::new("C",  600.0),
    ];
    let fp = FunnelPlot::new()
        .with_stage("A", 1000)
        .with_stage("B",  800)
        .with_stage("C",  600)
        .with_mirror(stages);
    let svg = render(fp, "Mirror API");
    std::fs::write("test_outputs/funnel_mirror_api.svg", &svg).unwrap();
    assert!(svg.contains("<rect"));
}

#[test]
fn test_funnel_mirror_horizontal() {
    let fp = FunnelPlot::new()
        .with_stage("A", 1000)
        .with_stage("B",  700)
        .with_stage("C",  400)
        .with_orientation(FunnelOrientation::Horizontal)
        .with_mirror_stages([
            ("A", 900.0),
            ("B", 650.0),
            ("C", 380.0),
        ])
        .with_mirror_labels("Group 1", "Group 2");
    let svg = render(fp, "Mirror Horizontal");
    std::fs::write("test_outputs/funnel_mirror_horizontal.svg", &svg).unwrap();
    assert!(svg.contains("<rect"));
}

#[test]
fn test_funnel_with_stages_api() {
    let fp = FunnelPlot::new().with_stages([
        ("Awareness", 5000.0),
        ("Interest",  3000.0),
        ("Desire",    2000.0),
        ("Action",    1200.0),
    ]);
    let svg = render(fp, "Bulk Stages");
    std::fs::write("test_outputs/funnel_with_stages.svg", &svg).unwrap();
    assert!(svg.contains("<rect"));
}

#[test]
fn test_funnel_legend() {
    let fp = FunnelPlot::new()
        .with_stage("Screened",  1200)
        .with_stage("Eligible",   800)
        .with_stage("Enrolled",   600)
        .with_color_mode(FunnelColorMode::ByStage)
        .with_legend("Stages");
    let svg = render(fp, "Funnel Legend");
    std::fs::write("test_outputs/funnel_legend.svg", &svg).unwrap();
    assert!(svg.contains("Screened"), "should have stage labels in legend");
}

#[test]
fn test_funnel_into_plot() {
    let fp = FunnelPlot::new().with_stage("A", 100).with_stage("B", 80);
    let p: Plot = fp.into();
    assert!(matches!(p, Plot::Funnel(_)));
}

#[test]
fn test_funnel_large() {
    let fp = FunnelPlot::new().with_stages(
        (0..20).map(|i| (format!("Stage {}", i), (1000 - i * 40) as f64))
    );
    let svg = render(fp, "Large Funnel");
    std::fs::write("test_outputs/funnel_large.svg", &svg).unwrap();
    assert!(svg.contains("<rect"));
}

#[test]
fn test_funnel_equal_values() {
    let fp = FunnelPlot::new()
        .with_stage("A", 500)
        .with_stage("B", 500)
        .with_stage("C", 500);
    let svg = render(fp, "Equal Values");
    std::fs::write("test_outputs/funnel_equal.svg", &svg).unwrap();
    assert!(svg.contains("<rect"));
}

#[test]
fn test_funnel_conversion_rate_with_large_gap() {
    let fp = FunnelPlot::new()
        .with_stage("A", 1000)
        .with_stage("B",  600)
        .with_stage("C",  300)
        .with_stage_gap(40.0)
        .with_show_conversion(true);
    let svg = render_size(fp, "Conversion Rates", 600.0, 500.0);
    std::fs::write("test_outputs/funnel_conversion.svg", &svg).unwrap();
    assert!(svg.contains('%'), "conversion rates show % signs");
}

#[test]
fn test_funnel_dramatic_connectors() {
    // Very large gap (60px) + steep attrition so connectors are wide trapezoids
    let fp = FunnelPlot::new()
        .with_stage("Candidates",   5000)
        .with_stage("Screened",     2800)
        .with_stage("Eligible",     1200)
        .with_stage("Enrolled",      400)
        .with_stage("Completed",     120)
        .with_stage_gap(60.0)
        .with_connector_opacity(0.65)
        .with_show_percents(true)
        .with_show_conversion(true)
        .with_color_mode(FunnelColorMode::Gradient);
    let svg = render_size(fp, "Dramatic Connectors", 700.0, 600.0);
    std::fs::write("test_outputs/funnel_dramatic_connectors.svg", &svg).unwrap();
    assert!(svg.contains('<'), "should produce SVG content");
    // With steep attrition (5000 → 120) the connectors taper dramatically
    // Check connector paths are present
    assert!(svg.contains("<path"), "should have connector path elements");
}
