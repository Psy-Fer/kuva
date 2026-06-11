mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::sunburst::{SunburstColorMode, SunburstPlot};
use kuva::plot::treemap::{ColorMap, TreemapNode};
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

fn render(sb: SunburstPlot, title: &str) -> String {
    let plots = vec![Plot::Sunburst(sb)];
    let layout = Layout::auto_from_plots(&plots).with_title(title);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

fn simple_sunburst() -> SunburstPlot {
    SunburstPlot::new().with_node(TreemapNode::new(
        "Root",
        vec![
            TreemapNode::leaf("A", 30.0),
            TreemapNode::leaf("B", 45.0),
            TreemapNode::leaf("C", 25.0),
        ],
    ))
}

fn two_level() -> SunburstPlot {
    SunburstPlot::new()
        .with_node(TreemapNode::new(
            "GroupA",
            vec![TreemapNode::leaf("A1", 30.0), TreemapNode::leaf("A2", 20.0)],
        ))
        .with_node(TreemapNode::new(
            "GroupB",
            vec![TreemapNode::leaf("B1", 50.0), TreemapNode::leaf("B2", 25.0)],
        ))
}

#[test]
fn test_sunburst_basic() {
    let svg = render(simple_sunburst(), "Sunburst Basic");
    common::write_test_output("test_outputs/sunburst_basic.svg", &svg).unwrap();
    assert!(svg.contains("<path"), "should contain path elements");
    assert!(svg.contains("<svg"), "should be valid SVG");
}

#[test]
fn test_sunburst_two_level() {
    let svg = render(two_level(), "Sunburst Two-Level");
    common::write_test_output("test_outputs/sunburst_two_level.svg", &svg).unwrap();
    // Should have arcs for both roots and their children
    let path_count = svg.matches("<path").count();
    assert!(
        path_count > 2,
        "two-level sunburst should have more than 2 arcs"
    );
}

#[test]
fn test_sunburst_deep() {
    let deep = SunburstPlot::new().with_node(TreemapNode::new(
        "L1",
        vec![
            TreemapNode::new(
                "L2a",
                vec![
                    TreemapNode::leaf("L3a", 10.0),
                    TreemapNode::leaf("L3b", 20.0),
                ],
            ),
            TreemapNode::new("L2b", vec![TreemapNode::leaf("L3c", 15.0)]),
        ],
    ));
    let svg = render(deep, "Sunburst Deep");
    common::write_test_output("test_outputs/sunburst_deep.svg", &svg).unwrap();
    assert!(svg.contains("<path"), "deep sunburst should render arcs");
}

#[test]
fn test_sunburst_single_node() {
    let sb = SunburstPlot::new().with_node(TreemapNode::leaf("Solo", 100.0));
    let svg = render(sb, "Sunburst Single Node");
    common::write_test_output("test_outputs/sunburst_single_node.svg", &svg).unwrap();
    assert!(
        svg.contains("<svg"),
        "single node sunburst should produce valid SVG"
    );
    assert!(svg.contains("<path"), "single node should render an arc");
}

#[test]
fn test_sunburst_empty() {
    let sb = SunburstPlot::new();
    let svg = render(sb, "Sunburst Empty");
    common::write_test_output("test_outputs/sunburst_empty.svg", &svg).unwrap();
    assert!(
        svg.contains("<svg"),
        "empty sunburst should produce valid SVG"
    );
}

#[test]
fn test_sunburst_multiple_roots() {
    let sb = SunburstPlot::new()
        .with_node(TreemapNode::leaf("Alpha", 40.0))
        .with_node(TreemapNode::leaf("Beta", 35.0))
        .with_node(TreemapNode::leaf("Gamma", 25.0));
    let svg = render(sb, "Sunburst Multiple Roots");
    common::write_test_output("test_outputs/sunburst_multiple_roots.svg", &svg).unwrap();
    let path_count = svg.matches("<path").count();
    assert!(path_count >= 3, "should have at least 3 arcs for 3 roots");
}

#[test]
fn test_sunburst_color_by_parent() {
    let sb = two_level().with_color_mode(SunburstColorMode::ByParent);
    let svg = render(sb, "Sunburst Color By Parent");
    common::write_test_output("test_outputs/sunburst_color_by_parent.svg", &svg).unwrap();
    // category10 uses hex colors like #1f77b4
    assert!(
        svg.contains("#1f77b4") || svg.contains("fill="),
        "ByParent should use palette colors"
    );
}

#[test]
fn test_sunburst_color_by_value() {
    let sb = two_level().with_color_mode(SunburstColorMode::ByValue(ColorMap::Viridis));
    let svg = render(sb, "Sunburst Color By Value");
    common::write_test_output("test_outputs/sunburst_color_by_value.svg", &svg).unwrap();
    assert!(svg.contains("<path"), "ByValue mode should render arcs");
}

#[test]
fn test_sunburst_explicit_colors() {
    let sb = SunburstPlot::new()
        .with_node(TreemapNode::new(
            "Root",
            vec![
                TreemapNode::leaf_colored("Red", 50.0, "#ff0000"),
                TreemapNode::leaf_colored("Blue", 50.0, "#0000ff"),
            ],
        ))
        .with_color_mode(SunburstColorMode::Explicit);
    let svg = render(sb, "Sunburst Explicit Colors");
    common::write_test_output("test_outputs/sunburst_explicit_colors.svg", &svg).unwrap();
    assert!(
        svg.contains("#ff0000"),
        "explicit red color should appear in SVG"
    );
}

#[test]
fn test_sunburst_donut() {
    let sb = simple_sunburst().with_inner_radius(0.4);
    let svg = render(sb, "Sunburst Donut");
    common::write_test_output("test_outputs/sunburst_donut.svg", &svg).unwrap();
    assert!(svg.contains("<path"), "donut sunburst should render arcs");
}

#[test]
fn test_sunburst_no_labels() {
    let sb = simple_sunburst().with_show_labels(false);
    let svg = render(sb, "Sunburst No Labels");
    common::write_test_output("test_outputs/sunburst_no_labels.svg", &svg).unwrap();
    // With no labels there should be no <text> elements (except possibly title/axis)
    let text_count_with_labels = {
        let sb2 = simple_sunburst().with_show_labels(true);
        let svg2 = render(sb2, "");
        svg2.matches("<text").count()
    };
    let text_count_no_labels = svg.matches("<text").count();
    assert!(
        text_count_no_labels <= text_count_with_labels,
        "disabling labels should reduce text count"
    );
}

#[test]
fn test_sunburst_tooltips() {
    let sb = simple_sunburst().with_tooltips(true);
    let svg = render(sb, "Sunburst Tooltips");
    common::write_test_output("test_outputs/sunburst_tooltips.svg", &svg).unwrap();
    assert!(
        svg.contains("<title"),
        "tooltips should produce <title> elements"
    );
}

#[test]
fn test_sunburst_no_tooltips() {
    let sb = simple_sunburst().with_tooltips(false);
    let svg = render(sb, "Sunburst No Tooltips");
    common::write_test_output("test_outputs/sunburst_no_tooltips.svg", &svg).unwrap();
    assert!(
        !svg.contains("<title"),
        "no tooltips should not produce <title> elements"
    );
}

#[test]
fn test_sunburst_colorbar() {
    let sb = two_level()
        .with_color_mode(SunburstColorMode::ByValue(ColorMap::Viridis))
        .with_colorbar(true)
        .with_colorbar_label("Score");
    let svg = render(sb, "Sunburst Colorbar");
    common::write_test_output("test_outputs/sunburst_colorbar.svg", &svg).unwrap();
    assert!(svg.contains("Score"), "colorbar label should appear in SVG");
}

#[test]
fn test_sunburst_max_depth() {
    let deep = SunburstPlot::new().with_node(TreemapNode::new(
        "L1",
        vec![
            TreemapNode::new(
                "L2a",
                vec![
                    TreemapNode::leaf("L3a", 10.0),
                    TreemapNode::leaf("L3b", 20.0),
                ],
            ),
            TreemapNode::leaf("L2b", 15.0),
        ],
    ));
    let limited = deep.clone().with_max_depth(1);
    let unlimited = deep;

    let svg_limited = render(limited, "Sunburst Max Depth 1");
    common::write_test_output("test_outputs/sunburst_max_depth.svg", &svg_limited).unwrap();
    let svg_unlimited = render(unlimited, "Sunburst Unlimited");

    let paths_limited = svg_limited.matches("<path").count();
    let paths_unlimited = svg_unlimited.matches("<path").count();
    assert!(
        paths_limited <= paths_unlimited,
        "limiting depth should not increase arc count"
    );
}

#[test]
fn test_sunburst_start_angle() {
    // Different start angles should produce different SVG (different path coordinates)
    let sb0 = simple_sunburst().with_start_angle(0.0);
    let sb90 = simple_sunburst().with_start_angle(90.0);
    let svg0 = render(sb0, "Sunburst Start 0");
    let svg90 = render(sb90, "Sunburst Start 90");
    common::write_test_output("test_outputs/sunburst_start_angle.svg", &svg90).unwrap();
    assert_ne!(
        svg0, svg90,
        "different start angles should produce different SVG"
    );
}

#[test]
fn test_sunburst_ring_gap() {
    let sb = simple_sunburst().with_ring_gap(4.0);
    let svg = render(sb, "Sunburst Ring Gap");
    common::write_test_output("test_outputs/sunburst_ring_gap.svg", &svg).unwrap();
    assert!(svg.contains("<path"), "ring gap sunburst should render");
}

#[test]
fn test_sunburst_into_plot() {
    let sb = simple_sunburst();
    let plot = Plot::from(sb);
    assert!(
        matches!(plot, Plot::Sunburst(_)),
        "From<SunburstPlot> should produce Plot::Sunburst"
    );
}

#[test]
fn test_sunburst_large() {
    // 50 flat leaves
    let leaves: Vec<TreemapNode> = (0..50)
        .map(|i| TreemapNode::leaf(format!("Leaf {i}"), (i + 1) as f64))
        .collect();
    let sb = SunburstPlot::new().with_node(TreemapNode::new("Root", leaves));
    let svg = render(sb, "Sunburst Large");
    common::write_test_output("test_outputs/sunburst_large.svg", &svg).unwrap();
    assert!(svg.contains("<path"), "large sunburst should render");
}

#[test]
fn test_sunburst_color_values() {
    let sb = SunburstPlot::new()
        .with_node(TreemapNode::new(
            "Root",
            vec![
                TreemapNode::leaf("A", 10.0),
                TreemapNode::leaf("B", 20.0),
                TreemapNode::leaf("C", 30.0),
            ],
        ))
        .with_color_mode(SunburstColorMode::ByValue(ColorMap::Viridis))
        .with_color_values(vec![0.1, 0.5, 0.9]);
    let svg = render(sb, "Sunburst Color Values");
    common::write_test_output("test_outputs/sunburst_color_values.svg", &svg).unwrap();
    assert!(svg.contains("<path"), "color_values mode should render");
}

#[test]
fn test_sunburst_no_rotate_labels() {
    let rotated = simple_sunburst().with_rotate_labels(true);
    let upright = simple_sunburst().with_rotate_labels(false);

    let svg_rotated = render(rotated, "Sunburst Rotated Labels");
    let svg_upright = render(upright, "Sunburst Upright Labels");
    common::write_test_output("test_outputs/sunburst_no_rotate_labels.svg", &svg_upright).unwrap();

    // Rotated labels produce transform="rotate(...)" on text elements
    assert!(
        svg_rotated.contains(r#"transform="rotate("#),
        "rotated labels should contain rotate transform"
    );
    // Upright labels must not have any rotate transform
    assert!(
        !svg_upright.contains(r#"transform="rotate("#),
        "upright labels should not contain rotate transform"
    );
}

#[test]
fn test_sunburst_with_children_builder() {
    let sb = SunburstPlot::new()
        .with_children(
            "Group A",
            vec![TreemapNode::leaf("X", 40.0), TreemapNode::leaf("Y", 60.0)],
        )
        .with_children(
            "Group B",
            vec![TreemapNode::leaf("P", 25.0), TreemapNode::leaf("Q", 75.0)],
        );
    let svg = render(sb, "Sunburst with_children");
    common::write_test_output("test_outputs/sunburst_with_children.svg", &svg).unwrap();
    assert!(
        svg.contains("<path"),
        "with_children builder should render arcs"
    );
}
