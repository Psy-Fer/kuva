use kuva::plot::bump::{BumpPlot, CurveStyle, BumpTieBreak};
use kuva::render::{plots::Plot, layout::Layout, render::render_multiple};
use kuva::backend::svg::SvgBackend;

fn render(bp: BumpPlot, title: &str) -> String {
    let plots = vec![Plot::Bump(bp)];
    let layout = Layout::auto_from_plots(&plots).with_title(title);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

fn simple_bump() -> BumpPlot {
    BumpPlot::new()
        .with_series("Alpha", vec![1, 3, 2, 1])
        .with_series("Beta",  vec![2, 1, 1, 3])
        .with_series("Gamma", vec![3, 2, 3, 2])
        .with_x_labels(["2021", "2022", "2023", "2024"])
}

#[test]
fn test_bump_basic() {
    let svg = render(simple_bump(), "Bump Basic");
    std::fs::write("test_outputs/bump_basic.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "should be valid SVG");
    assert!(svg.contains("<path"), "should contain path elements (curves)");
    assert!(svg.contains("<circle"), "should contain circle elements (dots)");
}

#[test]
fn test_bump_x_labels() {
    let svg = render(simple_bump(), "Bump X Labels");
    std::fs::write("test_outputs/bump_x_labels.svg", &svg).unwrap();
    assert!(svg.contains("2021"), "x-axis label 2021 should appear");
    assert!(svg.contains("2024"), "x-axis label 2024 should appear");
}

#[test]
fn test_bump_y_rank_labels() {
    let svg = render(simple_bump(), "Bump Rank Labels");
    std::fs::write("test_outputs/bump_y_rank_labels.svg", &svg).unwrap();
    // y-axis should show rank numbers 1, 2, 3
    assert!(svg.contains(">1<") || svg.contains(">1 <") || svg.matches(">1").count() > 0,
        "rank 1 should appear in SVG");
}

#[test]
fn test_bump_straight_curves() {
    let bp = simple_bump().with_curve_style(CurveStyle::Straight);
    let svg = render(bp, "Bump Straight");
    std::fs::write("test_outputs/bump_straight.svg", &svg).unwrap();
    assert!(svg.contains("<path"), "straight lines should render as paths");
    // Straight lines use L command, not C
    assert!(svg.contains(" L "), "straight lines should use L command");
    assert!(!svg.contains(" C "), "straight lines should not use C command");
}

#[test]
fn test_bump_sigmoid_curves() {
    let bp = simple_bump().with_curve_style(CurveStyle::Sigmoid);
    let svg = render(bp, "Bump Sigmoid");
    std::fs::write("test_outputs/bump_sigmoid.svg", &svg).unwrap();
    // Sigmoid uses cubic Bézier (C command)
    assert!(svg.contains(" C "), "sigmoid should use C (cubic bezier) command");
}

#[test]
fn test_bump_show_rank_labels() {
    let bp = simple_bump().with_show_rank_labels(true);
    let svg = render(bp, "Bump Rank Labels Inside Dots");
    std::fs::write("test_outputs/bump_rank_labels.svg", &svg).unwrap();
    let text_count_with = svg.matches("<text").count();

    let bp2 = simple_bump().with_show_rank_labels(false);
    let svg2 = render(bp2, "");
    let text_count_without = svg2.matches("<text").count();

    assert!(text_count_with > text_count_without,
        "showing rank labels should add <text> elements");
}

#[test]
fn test_bump_no_series_labels() {
    let bp = simple_bump().with_show_series_labels(false);
    let svg = render(bp, "Bump No Series Labels");
    std::fs::write("test_outputs/bump_no_series_labels.svg", &svg).unwrap();
    // With series labels off, series names should not appear as endpoint labels
    let text_count_off = svg.matches("<text").count();

    let bp2 = simple_bump().with_show_series_labels(true);
    let svg2 = render(bp2, "");
    let text_count_on = svg2.matches("<text").count();

    assert!(text_count_off <= text_count_on,
        "disabling series labels should not add more text");
}

#[test]
fn test_bump_highlight() {
    let bp = simple_bump().with_highlight("Alpha");
    let svg = render(bp, "Bump Highlight Alpha");
    std::fs::write("test_outputs/bump_highlight.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "highlight mode should produce valid SVG");
    // Highlighted series should have higher stroke-width path
    assert!(svg.contains("<path"), "should contain path elements");
}

#[test]
fn test_bump_no_legend() {
    let bp = simple_bump().with_legend(false);
    let svg = render(bp, "Bump No Legend");
    std::fs::write("test_outputs/bump_no_legend.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "valid SVG");
}

#[test]
fn test_bump_with_legend() {
    let bp = simple_bump().with_legend(true);
    let svg = render(bp, "Bump With Legend");
    std::fs::write("test_outputs/bump_with_legend.svg", &svg).unwrap();
    assert!(svg.contains("Alpha"), "legend should contain series name");
}

#[test]
fn test_bump_dot_radius() {
    let bp = simple_bump().with_dot_radius(10.0);
    let svg = render(bp, "Bump Large Dots");
    std::fs::write("test_outputs/bump_dot_radius.svg", &svg).unwrap();
    assert!(svg.contains("r=\"10\"") || svg.contains("r=\"10."), "large dot radius should appear in SVG");
}

#[test]
fn test_bump_stroke_width() {
    let bp = simple_bump().with_stroke_width(5.0);
    let svg = render(bp, "Bump Thick Lines");
    std::fs::write("test_outputs/bump_stroke_width.svg", &svg).unwrap();
    assert!(svg.contains("<path"), "thick lines should render");
}

#[test]
fn test_bump_raw_series() {
    let bp = BumpPlot::new()
        .with_raw_series("A", vec![100.0, 80.0, 90.0])
        .with_raw_series("B", vec![90.0, 95.0, 70.0])
        .with_raw_series("C", vec![70.0, 85.0, 100.0])
        .with_x_labels(["Q1", "Q2", "Q3"]);
    let svg = render(bp, "Bump Raw Series");
    std::fs::write("test_outputs/bump_raw_series.svg", &svg).unwrap();
    assert!(svg.contains("<circle"), "auto-ranked series should render dots");
}

#[test]
fn test_bump_rank_ascending() {
    // With ascending=true, lower value = rank 1 (best)
    let bp = BumpPlot::new()
        .with_raw_series("Low", vec![10.0, 20.0, 30.0])
        .with_raw_series("High", vec![90.0, 80.0, 70.0])
        .with_rank_ascending(true);
    let svg = render(bp, "Bump Rank Ascending");
    std::fs::write("test_outputs/bump_rank_ascending.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "ascending rank mode should produce valid SVG");
}

#[test]
fn test_bump_tie_break_min() {
    let bp = BumpPlot::new()
        .with_raw_series("A", vec![50.0, 50.0])
        .with_raw_series("B", vec![50.0, 50.0])
        .with_tie_break(BumpTieBreak::Min);
    let svg = render(bp, "Bump Tie Min");
    std::fs::write("test_outputs/bump_tie_min.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "tie break min should produce valid SVG");
}

#[test]
fn test_bump_tie_break_max() {
    let bp = BumpPlot::new()
        .with_raw_series("A", vec![50.0])
        .with_raw_series("B", vec![50.0])
        .with_raw_series("C", vec![50.0])
        .with_tie_break(BumpTieBreak::Max);
    let svg = render(bp, "Bump Tie Max");
    std::fs::write("test_outputs/bump_tie_max.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "tie break max should produce valid SVG");
}

#[test]
fn test_bump_tie_break_stable() {
    let bp = BumpPlot::new()
        .with_raw_series("A", vec![50.0, 50.0])
        .with_raw_series("B", vec![50.0, 50.0])
        .with_tie_break(BumpTieBreak::Stable);
    let svg = render(bp, "Bump Tie Stable");
    std::fs::write("test_outputs/bump_tie_stable.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "tie break stable should produce valid SVG");
}

#[test]
fn test_bump_missing_values() {
    // Series with None at some time points — line should break
    let bp = BumpPlot::new()
        .with_ranked_series("Alpha", vec![Some(1.0), None, Some(2.0), Some(1.0)])
        .with_ranked_series("Beta",  vec![Some(2.0), Some(1.0), Some(1.0), None])
        .with_x_labels(["A", "B", "C", "D"]);
    let svg = render(bp, "Bump Missing Values");
    std::fs::write("test_outputs/bump_missing.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "missing values should produce valid SVG");
}

#[test]
fn test_bump_single_series() {
    let bp = BumpPlot::new()
        .with_series("Solo", vec![1, 1, 1])
        .with_x_labels(["X", "Y", "Z"]);
    let svg = render(bp, "Bump Single Series");
    std::fs::write("test_outputs/bump_single.svg", &svg).unwrap();
    assert!(svg.contains("<circle"), "single series should render dots");
}

#[test]
fn test_bump_empty() {
    let bp = BumpPlot::new();
    let svg = render(bp, "Bump Empty");
    std::fs::write("test_outputs/bump_empty.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "empty bump chart should produce valid SVG");
}

#[test]
fn test_bump_large() {
    // 10 series × 6 time points
    let mut bp = BumpPlot::new();
    for i in 0..10 {
        let ranks: Vec<u32> = (0..6).map(|t| ((i + t) % 10 + 1) as u32).collect();
        bp = bp.with_series(format!("S{i}"), ranks);
    }
    bp = bp.with_x_labels(["T1","T2","T3","T4","T5","T6"]);
    let svg = render(bp, "Bump Large");
    std::fs::write("test_outputs/bump_large.svg", &svg).unwrap();
    assert!(svg.contains("<circle"), "large bump chart should render dots");
}

#[test]
fn test_bump_into_plot() {
    let bp = simple_bump();
    let plot = Plot::from(bp);
    assert!(matches!(plot, Plot::Bump(_)), "From<BumpPlot> should produce Plot::Bump");
}

#[test]
fn test_bump_mixed_ranked_and_raw() {
    // Mix pre-ranked and raw-value series
    let bp = BumpPlot::new()
        .with_ranked_series("Pre-ranked", vec![Some(1.0), Some(2.0), Some(1.0)])
        .with_raw_series("Auto-ranked", vec![10.0, 5.0, 8.0]);
    let svg = render(bp, "Bump Mixed");
    std::fs::write("test_outputs/bump_mixed.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "mixed series should produce valid SVG");
}
