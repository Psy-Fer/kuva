//! Treemap documentation examples.
use kuva::plot::{TreemapPlot, TreemapNode, TreemapColorMode, ColorMap};
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use std::fs;

const OUT: &str = "docs/src/assets/treemap";

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all(OUT).unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

fn main() {
    // Basic flat treemap
    let plot = TreemapPlot::new()
        .with_node(TreemapNode::leaf("Python", 38.0))
        .with_node(TreemapNode::leaf("JavaScript", 32.0))
        .with_node(TreemapNode::leaf("Rust", 18.0))
        .with_node(TreemapNode::leaf("Go", 14.0))
        .with_node(TreemapNode::leaf("C++", 12.0))
        .with_node(TreemapNode::leaf("Java", 10.0))
        .with_node(TreemapNode::leaf("TypeScript", 9.0))
        .with_node(TreemapNode::leaf("Ruby", 5.0));

    let plots = vec![Plot::Treemap(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Language Popularity")
        .with_width(560.0)
        .with_height(380.0);
    write("basic", plots, layout);

    // Two-level grouped treemap
    let plot = TreemapPlot::new()
        .with_children("Systems", vec![
            TreemapNode::leaf("Rust", 18.0),
            TreemapNode::leaf("C++", 12.0),
            TreemapNode::leaf("C", 9.0),
        ])
        .with_children("Web", vec![
            TreemapNode::leaf("JavaScript", 32.0),
            TreemapNode::leaf("TypeScript", 9.0),
            TreemapNode::leaf("Python", 38.0),
        ])
        .with_children("Enterprise", vec![
            TreemapNode::leaf("Java", 10.0),
            TreemapNode::leaf("Go", 14.0),
            TreemapNode::leaf("Ruby", 5.0),
        ]);

    let plots = vec![Plot::Treemap(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Languages by Category")
        .with_width(560.0)
        .with_height(380.0);
    write("two_level", plots, layout);

    // Colored by value (continuous colormap)
    let plot = TreemapPlot::new()
        .with_node(TreemapNode::leaf("Alpha", 120.0))
        .with_node(TreemapNode::leaf("Beta", 85.0))
        .with_node(TreemapNode::leaf("Gamma", 60.0))
        .with_node(TreemapNode::leaf("Delta", 45.0))
        .with_node(TreemapNode::leaf("Epsilon", 30.0))
        .with_node(TreemapNode::leaf("Zeta", 20.0))
        .with_node(TreemapNode::leaf("Eta", 12.0))
        .with_color_mode(TreemapColorMode::ByValue(ColorMap::Spectral))
        .with_color_values([120.0, 85.0, 60.0, 45.0, 30.0, 20.0, 12.0])
        .with_colorbar(true)
        .with_colorbar_label("value");

    let plots = vec![Plot::Treemap(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Treemap — continuous color scale")
        .with_width(560.0)
        .with_height(380.0);
    write("by_value", plots, layout);

    println!("Treemap SVGs written to {OUT}/");
}
