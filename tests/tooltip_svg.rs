use kuva::prelude::*;
use kuva::backend::svg::SvgBackend;
use kuva::plot::waterfall::WaterfallPlot;

fn render_plots(plots: Vec<Plot>) -> String {
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    SvgBackend::new().render_scene(&scene)
}

// ── Scatter ──────────────────────────────────────────────────────────────────

#[test]
fn test_scatter_auto_tooltips() {
    let plots = vec![ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
        .with_tooltips()
        .into()];
    let svg = render_plots(plots);
    std::fs::write("test_outputs/tooltip_scatter_auto.svg", &svg).unwrap();
    assert!(svg.contains("<title>"), "expected <title> in SVG");
    assert!(svg.contains("x=1.00, y=2.00"), "expected auto tooltip text");
    assert!(svg.contains("x=3.00, y=4.00"));
}

#[test]
fn test_scatter_custom_tooltip_labels() {
    let plots = vec![ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
        .with_tooltip_labels(["Sample A", "Sample B"])
        .into()];
    let svg = render_plots(plots);
    std::fs::write("test_outputs/tooltip_scatter_labels.svg", &svg).unwrap();
    assert!(svg.contains("<title>Sample A</title>"));
    assert!(svg.contains("<title>Sample B</title>"));
}

#[test]
fn test_scatter_no_tooltips_by_default() {
    let plots = vec![ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0)])
        .into()];
    let svg = render_plots(plots);
    assert!(!svg.contains("<title>"), "no tooltip expected when not requested");
}

// ── Bar ───────────────────────────────────────────────────────────────────────

#[test]
fn test_bar_auto_tooltips() {
    let plots = vec![BarPlot::new()
        .with_bar("A", 10.0)
        .with_bar("B", 20.0)
        .with_tooltips()
        .into()];
    let svg = render_plots(plots);
    std::fs::write("test_outputs/tooltip_bar.svg", &svg).unwrap();
    assert!(svg.contains("<title>"), "expected <title> in bar SVG");
    assert!(svg.contains("A: 10.00"));
    assert!(svg.contains("B: 20.00"));
}

// ── Histogram ─────────────────────────────────────────────────────────────────

#[test]
fn test_histogram_auto_tooltips() {
    let plots = vec![Histogram::new()
        .with_data(vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0])
        .with_bins(3)
        .with_tooltips()
        .into()];
    let svg = render_plots(plots);
    std::fs::write("test_outputs/tooltip_histogram.svg", &svg).unwrap();
    assert!(svg.contains("<title>"), "expected <title> in histogram SVG");
}

// ── Pie ───────────────────────────────────────────────────────────────────────

#[test]
fn test_pie_auto_tooltips() {
    let plots = vec![PiePlot::new()
        .with_slice("Cat", 30.0, "steelblue")
        .with_slice("Dog", 70.0, "orange")
        .with_tooltips()
        .into()];
    let svg = render_plots(plots);
    std::fs::write("test_outputs/tooltip_pie.svg", &svg).unwrap();
    assert!(svg.contains("<title>"), "expected <title> in pie SVG");
    assert!(svg.contains("Cat: 30.00 (30.0%)"));
    assert!(svg.contains("Dog: 70.00 (70.0%)"));
}

// ── Waterfall ─────────────────────────────────────────────────────────────────

#[test]
fn test_waterfall_auto_tooltips() {
    let plots = vec![WaterfallPlot::new()
        .with_delta("Q1", 100.0)
        .with_delta("Q2", -20.0)
        .with_total("Total")
        .with_tooltips()
        .into()];
    let svg = render_plots(plots);
    std::fs::write("test_outputs/tooltip_waterfall.svg", &svg).unwrap();
    assert!(svg.contains("<title>"), "expected <title> in waterfall SVG");
    assert!(svg.contains("Q1: 100.00"));
}

// ── XML escaping ──────────────────────────────────────────────────────────────

#[test]
fn test_tooltip_xml_escaping() {
    let plots = vec![ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0)])
        .with_tooltip_labels(["gene<BRCA1> & p<0.05"])
        .into()];
    let svg = render_plots(plots);
    assert!(
        svg.contains("gene&lt;BRCA1&gt; &amp; p&lt;0.05"),
        "tooltip text should be XML-escaped; got:\n{}",
        svg.lines().find(|l| l.contains("<title>")).unwrap_or("(not found)")
    );
}
