mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::streamgraph::{StreamBaseline, StreamOrder, StreamgraphPlot};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

fn render(plot: StreamgraphPlot) -> String {
    let plots = vec![Plot::Streamgraph(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_x_label("Time")
        .with_y_label("Value");
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

fn weekly_x() -> Vec<f64> {
    (1..=12).map(|w| w as f64).collect()
}

fn make_three_series() -> StreamgraphPlot {
    StreamgraphPlot::new()
        .with_x(weekly_x())
        .with_series([
            10.0, 14.0, 18.0, 22.0, 20.0, 16.0, 12.0, 18.0, 24.0, 28.0, 22.0, 16.0,
        ])
        .with_label("Alpha")
        .with_series([
            5.0, 8.0, 12.0, 15.0, 14.0, 10.0, 8.0, 11.0, 16.0, 18.0, 14.0, 9.0,
        ])
        .with_label("Beta")
        .with_series([3.0, 4.0, 6.0, 8.0, 9.0, 7.0, 5.0, 7.0, 9.0, 10.0, 8.0, 5.0])
        .with_label("Gamma")
}

// ── Basic rendering ───────────────────────────────────────────────────────────

#[test]
fn test_streamgraph_wiggle_default() {
    let svg = render(make_three_series());
    assert!(svg.contains("<svg"), "should produce SVG");
    assert!(svg.contains("<path"), "should contain paths");
    std::fs::create_dir_all("test_outputs").ok();
    common::write_test_output("test_outputs/streamgraph_wiggle.svg", &svg).unwrap();
}

#[test]
fn test_streamgraph_symmetric() {
    let sg = make_three_series().with_baseline(StreamBaseline::Symmetric);
    let svg = render(sg);
    assert!(svg.contains("<path"));
    common::write_test_output("test_outputs/streamgraph_symmetric.svg", &svg).unwrap();
}

#[test]
fn test_streamgraph_zero() {
    let sg = make_three_series().with_baseline(StreamBaseline::Zero);
    let svg = render(sg);
    assert!(svg.contains("<path"));
    common::write_test_output("test_outputs/streamgraph_zero.svg", &svg).unwrap();
}

// ── Ordering ──────────────────────────────────────────────────────────────────

#[test]
fn test_streamgraph_order_by_total() {
    let sg = make_three_series().with_order(StreamOrder::ByTotal);
    let svg = render(sg);
    assert!(svg.contains("<path"));
}

#[test]
fn test_streamgraph_order_original() {
    let sg = make_three_series().with_order(StreamOrder::Original);
    let svg = render(sg);
    assert!(svg.contains("<path"));
}

// ── Interpolation ─────────────────────────────────────────────────────────────

#[test]
fn test_streamgraph_smooth() {
    let sg = make_three_series(); // smooth=true by default
    let svg = render(sg);
    // Smooth paths contain 'C' control-point commands
    assert!(
        svg.contains(" C ") || svg.contains("C "),
        "smooth path should use cubic bezier"
    );
}

#[test]
fn test_streamgraph_linear() {
    let sg = make_three_series().with_linear();
    let svg = render(sg);
    // Linear paths use only L commands; no cubic bezier
    assert!(
        !svg.contains(" C "),
        "linear path should not use cubic bezier"
    );
    common::write_test_output("test_outputs/streamgraph_linear.svg", &svg).unwrap();
}

// ── Visual options ────────────────────────────────────────────────────────────

#[test]
fn test_streamgraph_stroke() {
    let sg = make_three_series().with_stroke();
    let svg = render(sg);
    assert!(svg.contains("<path"));
    common::write_test_output("test_outputs/streamgraph_stroke.svg", &svg).unwrap();
}

#[test]
fn test_streamgraph_fill_opacity() {
    let sg = make_three_series().with_fill_opacity(1.0);
    let svg = render(sg);
    assert!(svg.contains("opacity=\"1\"") || svg.contains("opacity=\"1.0\""));
}

#[test]
fn test_streamgraph_no_labels() {
    let sg = make_three_series().with_stream_labels(false);
    let svg = render(sg);
    // Labels would contain "Alpha", "Beta", "Gamma" in <text> elements; should be absent
    assert!(!svg.contains(">Alpha<") && !svg.contains(">Beta<") && !svg.contains(">Gamma<"));
}

#[test]
fn test_streamgraph_with_labels() {
    let sg = make_three_series(); // show_labels=true by default
    let svg = render(sg);
    // At least one label should appear
    let has_label = svg.contains(">Alpha<") || svg.contains(">Beta<") || svg.contains(">Gamma<");
    assert!(has_label, "at least one inline label should appear");
    common::write_test_output("test_outputs/streamgraph_labels.svg", &svg).unwrap();
}

// ── Legend ────────────────────────────────────────────────────────────────────

#[test]
fn test_streamgraph_legend() {
    let sg = make_three_series().with_legend("Species");
    let svg = render(sg);
    assert!(
        svg.contains("Alpha") && svg.contains("Beta") && svg.contains("Gamma"),
        "legend should contain series names"
    );
    common::write_test_output("test_outputs/streamgraph_legend.svg", &svg).unwrap();
}

// ── Normalization ─────────────────────────────────────────────────────────────

#[test]
fn test_streamgraph_normalized() {
    let sg = make_three_series().with_normalized();
    let svg = render(sg);
    assert!(svg.contains("<path"));
    common::write_test_output("test_outputs/streamgraph_normalized.svg", &svg).unwrap();
}

// ── Bounds ────────────────────────────────────────────────────────────────────

#[test]
fn test_streamgraph_bounds_wiggle() {
    let sg = make_three_series();
    let plot = Plot::Streamgraph(sg);
    let ((x_min, x_max), (y_min, y_max)) = plot.bounds().unwrap();
    assert_eq!(x_min, 1.0);
    assert_eq!(x_max, 12.0);
    // wiggle baseline is symmetric: y_min < 0, y_max > 0
    assert!(y_min < 0.0, "wiggle baseline should go negative");
    assert!(y_max > 0.0, "wiggle stream tops should be positive");
}

#[test]
fn test_streamgraph_bounds_zero() {
    let sg = make_three_series().with_baseline(StreamBaseline::Zero);
    let plot = Plot::Streamgraph(sg);
    let ((_, _), (y_min, y_max)) = plot.bounds().unwrap();
    assert!(y_min >= 0.0, "zero baseline should be non-negative");
    assert!(y_max > 0.0);
}

// ── Edge cases ────────────────────────────────────────────────────────────────

#[test]
fn test_streamgraph_empty() {
    let sg = StreamgraphPlot::new();
    let svg = render(sg);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_streamgraph_single_series() {
    let sg = StreamgraphPlot::new()
        .with_x([1.0, 2.0, 3.0])
        .with_series([10.0, 20.0, 15.0])
        .with_label("Solo");
    let svg = render(sg);
    assert!(svg.contains("<path"));
    common::write_test_output("test_outputs/streamgraph_single.svg", &svg).unwrap();
}

#[test]
fn test_streamgraph_two_points() {
    // Minimum viable x count
    let sg = StreamgraphPlot::new()
        .with_x([0.0, 1.0])
        .with_series([5.0, 8.0])
        .with_label("A")
        .with_series([3.0, 4.0])
        .with_label("B");
    let svg = render(sg);
    assert!(svg.contains("<path"));
}

#[test]
fn test_streamgraph_explicit_colors() {
    let sg = StreamgraphPlot::new()
        .with_x(weekly_x())
        .with_series([
            10.0, 12.0, 14.0, 16.0, 15.0, 13.0, 11.0, 14.0, 17.0, 19.0, 16.0, 12.0,
        ])
        .with_color("steelblue")
        .with_label("A")
        .with_series([5.0, 6.0, 8.0, 9.0, 8.0, 7.0, 6.0, 7.0, 9.0, 10.0, 8.0, 6.0])
        .with_color("tomato")
        .with_label("B");
    let svg = render(sg);
    // steelblue → #4682b4, tomato → #ff6347
    assert!(svg.contains("#4682b4") && svg.contains("#ff6347"));
}

#[test]
fn test_streamgraph_from_impl() {
    let sg: Plot = make_three_series().into();
    assert!(matches!(sg, Plot::Streamgraph(_)));
}
