use kuva::plot::treemap::{TreemapPlot, TreemapNode, TreemapColorMode, TreemapLayout, ColorMap};
use kuva::render::{plots::Plot, layout::Layout, render::render_multiple};
use kuva::backend::svg::SvgBackend;

fn render(tm: TreemapPlot, title: &str) -> String {
    let plots = vec![Plot::Treemap(tm)];
    let layout = Layout::auto_from_plots(&plots).with_title(title);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

// ── Data helpers ──────────────────────────────────────────────────────────────

fn flat_leaves(n: usize) -> TreemapPlot {
    let leaves: Vec<TreemapNode> = (0..n)
        .map(|i| TreemapNode::leaf(format!("Leaf {i}"), (i + 1) as f64))
        .collect();
    let mut tm = TreemapPlot::new();
    for l in leaves { tm = tm.with_node(l); }
    tm
}

fn two_level() -> TreemapPlot {
    TreemapPlot::new()
        .with_node(TreemapNode::new("GroupA", vec![
            TreemapNode::leaf("A1", 30.0),
            TreemapNode::leaf("A2", 20.0),
            TreemapNode::leaf("A3", 10.0),
        ]))
        .with_node(TreemapNode::new("GroupB", vec![
            TreemapNode::leaf("B1", 50.0),
            TreemapNode::leaf("B2", 25.0),
        ]))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_treemap_basic() {
    let svg = render(flat_leaves(8), "Treemap Basic");
    std::fs::write("test_outputs/treemap_basic.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "should contain rect elements");
    assert!(svg.contains("<svg"), "should be valid SVG");
}

#[test]
fn test_treemap_two_level() {
    let svg = render(two_level(), "Treemap Two-Level");
    std::fs::write("test_outputs/treemap_two_level.svg", &svg).unwrap();
    let rect_count = svg.matches("<rect").count();
    // Parent rects + child rects → more than just the 2 parent rects
    assert!(rect_count > 2, "two-level treemap should have more rects than just top-level nodes");
}

#[test]
fn test_treemap_deep() {
    let deep = TreemapPlot::new().with_node(TreemapNode::new("L1", vec![
        TreemapNode::new("L2a", vec![
            TreemapNode::leaf("L3a", 10.0),
            TreemapNode::leaf("L3b", 20.0),
        ]),
        TreemapNode::new("L2b", vec![
            TreemapNode::leaf("L3c", 15.0),
        ]),
    ]));
    let svg = render(deep, "Treemap Deep");
    std::fs::write("test_outputs/treemap_deep.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "deep treemap should render");
}

#[test]
fn test_treemap_single_node() {
    let tm = TreemapPlot::new().with_node(TreemapNode::leaf("Only", 42.0));
    let svg = render(tm, "Treemap Single Node");
    std::fs::write("test_outputs/treemap_single.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "single-node treemap should render");
}

#[test]
fn test_treemap_empty() {
    let tm = TreemapPlot::new();
    let svg = render(tm, "Treemap Empty");
    std::fs::write("test_outputs/treemap_empty.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "empty treemap should still produce valid SVG");
}

#[test]
fn test_treemap_squarify() {
    let svg = render(flat_leaves(12).with_layout(TreemapLayout::Squarify), "Treemap Squarify");
    std::fs::write("test_outputs/treemap_squarify.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "squarify layout renders");
}

#[test]
fn test_treemap_slicedice() {
    let svg = render(flat_leaves(6).with_layout(TreemapLayout::SliceDice), "Treemap SliceDice");
    std::fs::write("test_outputs/treemap_slicedice.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "slicedice layout renders");
}

#[test]
fn test_treemap_binary() {
    let svg = render(flat_leaves(8).with_layout(TreemapLayout::Binary), "Treemap Binary");
    std::fs::write("test_outputs/treemap_binary.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "binary layout renders");
}

#[test]
fn test_treemap_color_by_parent() {
    // Each top-level group should get a category10 color (e.g. steelblue)
    let svg = render(two_level(), "Treemap By Parent");
    std::fs::write("test_outputs/treemap_color_by_parent.svg", &svg).unwrap();
    // Category10 colors appear as hex strings in SVG fills
    assert!(svg.contains("fill="), "category10 colors should appear in SVG fills");
}

#[test]
fn test_treemap_color_by_value() {
    let tm = flat_leaves(10)
        .with_color_mode(TreemapColorMode::ByValue(ColorMap::Viridis));
    let svg = render(tm, "Treemap By Value");
    std::fs::write("test_outputs/treemap_color_by_value.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "ByValue treemap should render");
}

#[test]
fn test_treemap_explicit_colors() {
    let tm = TreemapPlot::new()
        .with_node(TreemapNode::leaf_colored("Red", 30.0, "#ff0000"))
        .with_node(TreemapNode::leaf_colored("Blue", 20.0, "#0000ff"))
        .with_color_mode(TreemapColorMode::Explicit);
    let svg = render(tm, "Treemap Explicit Colors");
    std::fs::write("test_outputs/treemap_explicit_colors.svg", &svg).unwrap();
    assert!(svg.contains("#ff0000"), "#ff0000 should appear in SVG");
}

#[test]
fn test_treemap_label_suppression() {
    // min_label_area = MAX → no leaf labels should appear
    let tm = flat_leaves(5).with_min_label_area(f64::MAX);
    let svg = render(tm, "Treemap No Labels");
    std::fs::write("test_outputs/treemap_no_labels.svg", &svg).unwrap();
    // No text elements after the title
    let text_after_title = svg.split("Treemap No Labels").nth(1).unwrap_or("");
    assert!(!text_after_title.contains("<text"), "labels should be suppressed");
}

#[test]
fn test_treemap_padding_zero() {
    let tm = two_level().with_padding(0.0);
    let svg = render(tm, "Treemap Zero Padding");
    std::fs::write("test_outputs/treemap_padding_zero.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "zero-padding treemap should render");
}

#[test]
fn test_treemap_padding_large() {
    let tm = two_level().with_padding(12.0);
    let svg = render(tm, "Treemap Large Padding");
    std::fs::write("test_outputs/treemap_padding_large.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "large-padding treemap should render");
}

#[test]
fn test_treemap_max_depth() {
    let deep = TreemapPlot::new().with_node(TreemapNode::new("L1", vec![
        TreemapNode::new("L2a", vec![
            TreemapNode::leaf("L3a", 10.0),
            TreemapNode::leaf("L3b", 20.0),
            TreemapNode::leaf("L3c", 15.0),
        ]),
        TreemapNode::new("L2b", vec![
            TreemapNode::leaf("L3d", 25.0),
            TreemapNode::leaf("L3e", 5.0),
        ]),
    ]));

    let deep_unlimited = deep.clone();
    let deep_limited   = deep.with_max_depth(1);

    let svg_full    = render(deep_unlimited, "Treemap Full Depth");
    let svg_limited = render(deep_limited,  "Treemap Depth 1");
    std::fs::write("test_outputs/treemap_full_depth.svg",    &svg_full).unwrap();
    std::fs::write("test_outputs/treemap_max_depth.svg",  &svg_limited).unwrap();

    let full_rects    = svg_full.matches("<rect").count();
    let limited_rects = svg_limited.matches("<rect").count();
    assert!(full_rects >= limited_rects, "deeper render should have at least as many rects");
}

#[test]
fn test_treemap_tooltips() {
    let tm = flat_leaves(5);
    let svg = render(tm, "Treemap Tooltips");
    std::fs::write("test_outputs/treemap_tooltips.svg", &svg).unwrap();
    assert!(svg.contains("<title"), "tooltips should produce <title> elements");
}

#[test]
fn test_treemap_no_tooltips() {
    let tm = flat_leaves(5).with_tooltips(false);
    let svg = render(tm, "Treemap No Tooltips");
    std::fs::write("test_outputs/treemap_no_tooltips.svg", &svg).unwrap();
    assert!(!svg.contains("<title"), "no-tooltips should suppress <title> elements");
}

#[test]
fn test_treemap_colorbar() {
    let tm = flat_leaves(8)
        .with_color_mode(TreemapColorMode::ByValue(ColorMap::Viridis))
        .with_colorbar_label("p-value");
    let svg = render(tm, "Treemap Colorbar");
    std::fs::write("test_outputs/treemap_colorbar.svg", &svg).unwrap();
    assert!(svg.contains("p-value"), "colorbar label should appear in SVG");
}

#[test]
fn test_treemap_go_enrichment() {
    let terms = vec![
        ("GO:0008150", "biological process", 120, 0.001),
        ("GO:0005575", "cellular component",  80, 0.023),
        ("GO:0003674", "molecular function",   55, 0.045),
        ("GO:0006950", "response to stress",   40, 0.0001),
        ("GO:0065007", "biological regulation",30, 0.12),
    ];
    let tm = TreemapPlot::new()
        .with_go_terms(terms)
        .with_colorbar_label("p-value");
    let svg = render(tm, "Treemap GO Enrichment");
    std::fs::write("test_outputs/treemap_go_enrichment.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "GO enrichment treemap should render");
    assert!(!svg.is_empty(), "should produce non-empty SVG");
}

#[test]
fn test_treemap_into_plot() {
    let tm = TreemapPlot::new().with_node(TreemapNode::leaf("x", 1.0));
    let p: Plot = tm.into();
    assert!(matches!(p, Plot::Treemap(_)), "From<TreemapPlot> should produce Plot::Treemap");
}

#[test]
fn test_treemap_large() {
    let tm = flat_leaves(100);
    let svg = render(tm, "Treemap Large (100 leaves)");
    std::fs::write("test_outputs/treemap_large.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "large treemap should produce valid SVG");
    assert!(svg.contains("<rect"), "large treemap should have rect elements");
}

#[test]
fn test_treemap_unequal_values() {
    // Dominant child 1000× smaller siblings — should not panic
    let tm = TreemapPlot::new().with_node(TreemapNode::new("Root", vec![
        TreemapNode::leaf("Big",    1000.0),
        TreemapNode::leaf("Tiny1",     1.0),
        TreemapNode::leaf("Tiny2",     1.0),
        TreemapNode::leaf("Tiny3",     1.0),
    ]));
    let svg = render(tm, "Treemap Unequal Values");
    std::fs::write("test_outputs/treemap_unequal.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "unequal-value treemap should render without panic");
}

#[test]
fn test_treemap_forest() {
    // Multiple top-level roots (forest)
    let tm = TreemapPlot::new()
        .with_node(TreemapNode::new("Alpha", vec![
            TreemapNode::leaf("a1", 10.0),
            TreemapNode::leaf("a2", 20.0),
        ]))
        .with_node(TreemapNode::new("Beta", vec![
            TreemapNode::leaf("b1", 30.0),
            TreemapNode::leaf("b2", 15.0),
        ]))
        .with_node(TreemapNode::leaf("Gamma", 25.0));
    let svg = render(tm, "Treemap Forest");
    std::fs::write("test_outputs/treemap_forest.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "forest treemap should render");
}
