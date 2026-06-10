mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::scatter::ScatterPlot;
use kuva::render::layout::{ComputedLayout, Layout};
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

fn make_layout() -> Layout {
    let points: Vec<(f64, f64)> = vec![(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)];
    let plots = vec![Plot::Scatter(ScatterPlot::new().with_data(points))];
    Layout::auto_from_plots(&plots)
}

#[test]
fn test_scale_1x_is_identity() {
    let layout = make_layout().with_scale(1.0);
    let computed = ComputedLayout::from_layout(&layout);
    assert_eq!(computed.title_size, 18);
    assert_eq!(computed.label_size, 14);
    assert_eq!(computed.tick_size, 12);
    assert_eq!(computed.body_size, 12);
    assert!((computed.tick_mark_major - 5.0).abs() < 1e-9);
    assert!((computed.tick_mark_minor - 3.0).abs() < 1e-9);
    assert!((computed.axis_stroke_width - 1.0).abs() < 1e-9);
    assert!((computed.legend_padding - 10.0).abs() < 1e-9);
    assert!((computed.legend_swatch_size - 12.0).abs() < 1e-9);
}

#[test]
fn test_scale_2x_doubles_font_sizes() {
    let layout = make_layout().with_scale(2.0);
    let computed = ComputedLayout::from_layout(&layout);
    assert_eq!(computed.title_size, 36);
    assert_eq!(computed.label_size, 28);
    assert_eq!(computed.tick_size, 24);
    assert_eq!(computed.body_size, 24);
}

#[test]
fn test_scale_2x_doubles_chrome() {
    let layout = make_layout().with_scale(2.0);
    let computed = ComputedLayout::from_layout(&layout);
    assert!((computed.tick_mark_major - 10.0).abs() < 1e-9);
    assert!((computed.tick_mark_minor - 6.0).abs() < 1e-9);
    assert!((computed.axis_stroke_width - 2.0).abs() < 1e-9);
    assert!((computed.legend_padding - 20.0).abs() < 1e-9);
    assert!((computed.legend_swatch_size - 24.0).abs() < 1e-9);
    assert!((computed.annotation_arrow_len - 16.0).abs() < 1e-9);
}

#[test]
fn test_scale_2x_wider_margins() {
    let layout_1x = make_layout().with_scale(1.0);
    let layout_2x = make_layout().with_scale(2.0);
    let c1 = ComputedLayout::from_layout(&layout_1x);
    let c2 = ComputedLayout::from_layout(&layout_2x);
    // All four margins should be larger at scale=2
    assert!(c2.margin_top > c1.margin_top);
    assert!(c2.margin_bottom > c1.margin_bottom);
    assert!(c2.margin_left > c1.margin_left);
}

#[test]
fn test_scale_2x_svg_contains_scaled_font_size() {
    let points: Vec<(f64, f64)> = vec![(1.0, 2.0), (3.0, 4.0)];
    let plots = vec![Plot::Scatter(ScatterPlot::new().with_data(points))];
    let layout = Layout::auto_from_plots(&plots)
        .with_scale(2.0)
        .with_title("Big Plot")
        .with_x_label("X")
        .with_y_label("Y");
    let computed = ComputedLayout::from_layout(&layout);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/scale_2x.svg", svg.clone()).unwrap();
    // title font-size = 36
    assert!(
        svg.contains("font-size=\"36\""),
        "SVG should contain scaled title font-size=36"
    );
    // tick font-size = 24
    assert!(
        svg.contains("font-size=\"24\""),
        "SVG should contain scaled tick font-size=24"
    );
    assert_eq!(computed.title_size, 36);
}

#[test]
fn test_scale_0_5x_halves_font_sizes() {
    let layout = make_layout().with_scale(0.5);
    let computed = ComputedLayout::from_layout(&layout);
    // 18 * 0.5 = 9, 14 * 0.5 = 7, 12 * 0.5 = 6
    assert_eq!(computed.title_size, 9);
    assert_eq!(computed.label_size, 7);
    assert_eq!(computed.tick_size, 6);
    assert!((computed.tick_mark_major - 2.5).abs() < 1e-9);
}

#[test]
fn test_scale_0_5x_svg() {
    let points: Vec<(f64, f64)> = vec![(1.0, 2.0), (3.0, 4.0)];
    let plots = vec![Plot::Scatter(ScatterPlot::new().with_data(points))];
    let layout = Layout::auto_from_plots(&plots)
        .with_scale(0.5)
        .with_title("Small");
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/scale_0_5x.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"), "SVG should be valid at scale=0.5");
    assert!(
        svg.contains("font-size=\"9\""),
        "Should contain title font-size=9"
    );
}

#[test]
fn test_scale_3x_svg() {
    let points: Vec<(f64, f64)> = vec![(1.0, 2.0), (3.0, 4.0)];
    let plots = vec![Plot::Scatter(ScatterPlot::new().with_data(points))];
    let layout = Layout::auto_from_plots(&plots)
        .with_scale(3.0)
        .with_title("Extra Large")
        .with_x_label("X axis")
        .with_y_label("Y axis");
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/scale_3x.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"), "SVG should be valid at scale=3.0");
    // title_size = round(18 * 3) = 54
    assert!(
        svg.contains("font-size=\"54\""),
        "Should contain title font-size=54"
    );
}

#[test]
fn test_scale_0_25x_svg() {
    let points: Vec<(f64, f64)> = vec![(1.0, 2.0), (3.0, 4.0)];
    let plots = vec![Plot::Scatter(ScatterPlot::new().with_data(points))];
    let layout = Layout::auto_from_plots(&plots).with_scale(0.25);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/scale_0_25x.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"), "SVG should be valid at scale=0.25");
}

#[test]
fn test_scale_floor_clamp() {
    // with_scale(0.0) is clamped to 0.1 — no zero font sizes
    let layout = make_layout().with_scale(0.0);
    let computed = ComputedLayout::from_layout(&layout);
    assert!(computed.title_size >= 1, "title_size must be >= 1");
    assert!(computed.tick_size >= 1, "tick_size must be >= 1");
    assert!(computed.axis_stroke_width > 0.0);
}

#[test]
fn test_scale_1_5x_svg() {
    let points: Vec<(f64, f64)> = vec![(1.0, 2.0), (3.0, 4.0)];
    let plots = vec![Plot::Scatter(ScatterPlot::new().with_data(points))];
    let layout = Layout::auto_from_plots(&plots)
        .with_scale(1.5)
        .with_title("Medium Large");
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/scale_1_5x.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"), "SVG should be valid at scale=1.5");
    // title_size = round(18 * 1.5) = 27
    assert!(
        svg.contains("font-size=\"27\""),
        "Should contain title font-size=27"
    );
}
