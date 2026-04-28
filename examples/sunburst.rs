//! Sunburst chart documentation examples.
use kuva::plot::{SunburstPlot, TreemapNode};
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use std::fs;

const OUT: &str = "docs/src/assets/sunburst";

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all(OUT).unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

fn main() {
    // Basic sunburst — operating system market share
    let plot = SunburstPlot::new()
        .with_children("Desktop", vec![
            TreemapNode::leaf("Windows", 72.0),
            TreemapNode::leaf("macOS", 16.0),
            TreemapNode::leaf("Linux", 4.0),
            TreemapNode::leaf("ChromeOS", 2.0),
        ])
        .with_children("Mobile", vec![
            TreemapNode::leaf("Android", 72.0),
            TreemapNode::leaf("iOS", 28.0),
        ])
        .with_children("Other", vec![
            TreemapNode::leaf("Smart TV", 2.5),
            TreemapNode::leaf("Console", 1.5),
        ]);

    let plots = vec![Plot::Sunburst(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("OS Market Share")
        .with_width(560.0)
        .with_height(460.0);
    write("basic", plots, layout);

    // Donut style (inner radius)
    let plot = SunburstPlot::new()
        .with_children("Q1", vec![
            TreemapNode::leaf("Jan", 42.0),
            TreemapNode::leaf("Feb", 38.0),
            TreemapNode::leaf("Mar", 55.0),
        ])
        .with_children("Q2", vec![
            TreemapNode::leaf("Apr", 61.0),
            TreemapNode::leaf("May", 70.0),
            TreemapNode::leaf("Jun", 65.0),
        ])
        .with_children("Q3", vec![
            TreemapNode::leaf("Jul", 58.0),
            TreemapNode::leaf("Aug", 52.0),
            TreemapNode::leaf("Sep", 63.0),
        ])
        .with_children("Q4", vec![
            TreemapNode::leaf("Oct", 71.0),
            TreemapNode::leaf("Nov", 85.0),
            TreemapNode::leaf("Dec", 92.0),
        ])
        .with_inner_radius(0.30);

    let plots = vec![Plot::Sunburst(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Quarterly Revenue by Month")
        .with_width(560.0)
        .with_height(460.0);
    write("donut", plots, layout);

    println!("Sunburst SVGs written to {OUT}/");
}
