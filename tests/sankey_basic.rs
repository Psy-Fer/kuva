use visus::plot::SankeyPlot;
use visus::render::{plots::Plot, layout::Layout, render::render_multiple};
use visus::backend::svg::SvgBackend;

/// Clean 3-stage pipeline: no skip links, all outputs go to the immediately
/// next column only.
///   col 0: Input
///   col 1: Process A, Process B
///   col 2: Output X, Output Y
#[test]
fn sankey_basic() {
    let sankey = SankeyPlot::new()
        .with_link("Input", "Process A", 50.0)
        .with_link("Input", "Process B", 30.0)
        .with_link("Process A", "Output X", 40.0)
        .with_link("Process A", "Output Y", 10.0)
        .with_link("Process B", "Output X", 10.0)
        .with_link("Process B", "Output Y", 20.0);
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Sankey Basic");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/sankey_basic.svg", svg).unwrap();
}

/// Same clean topology with gradient ribbons.
#[test]
fn sankey_gradient() {
    let sankey = SankeyPlot::new()
        .with_link("Input", "Process A", 50.0)
        .with_link("Input", "Process B", 30.0)
        .with_link("Process A", "Output X", 40.0)
        .with_link("Process A", "Output Y", 10.0)
        .with_link("Process B", "Output X", 10.0)
        .with_link("Process B", "Output Y", 20.0)
        .with_gradient_links()
        .with_link_opacity(0.6);
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Sankey Gradient");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/sankey_gradient.svg", svg).unwrap();
}

/// Explicit node colors and legend.
#[test]
fn sankey_legend() {
    let sankey = SankeyPlot::new()
        .with_node_color("Input", "#e41a1c")
        .with_node_color("Process A", "#377eb8")
        .with_node_color("Process B", "#4daf4a")
        .with_node_color("Output", "#984ea3")
        .with_link("Input", "Process A", 40.0)
        .with_link("Input", "Process B", 30.0)
        .with_link("Process A", "Output", 35.0)
        .with_link("Process B", "Output", 25.0)
        .with_link_opacity(0.5)
        .with_node_width(24.0)
        .with_legend("Flow");
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Sankey Legend");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/sankey_legend.svg", svg).unwrap();
}

/// Dead-end nodes: "Discard A" and "Discard B" receive flow but have no
/// outgoing links.  Their node heights should still be proportional to their
/// in-flow and they must not crash the renderer.
///   col 0: Source
///   col 1: Filter, Discard A          (Discard A is a dead end)
///   col 2: Pass, Discard B            (Discard B is a dead end)
///   col 3: Output
#[test]
fn sankey_dead_end() {
    let sankey = SankeyPlot::new()
        .with_link("Source", "Filter", 70.0)
        .with_link("Source", "Discard A", 30.0)   // dead end at col 1
        .with_link("Filter", "Pass", 55.0)
        .with_link("Filter", "Discard B", 15.0)   // dead end at col 2
        .with_link("Pass", "Output", 55.0);
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Sankey Dead Ends");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/sankey_dead_end.svg", svg).unwrap();
}

/// Wide 4-stage variant-filtering pipeline representative of a bioinformatics use-case.
///   col 0: Raw Variants
///   col 1: QC Pass, QC Fail
///   col 2: High Conf, Low Conf       (from QC Pass only)
///   col 3: SNP, Indel, Filtered Out  (from High Conf and Low Conf)
#[test]
fn sankey_variant_filter() {
    let sankey = SankeyPlot::new()
        .with_node_color("Raw Variants",  "#888888")
        .with_node_color("QC Pass",       "#4daf4a")
        .with_node_color("QC Fail",       "#e41a1c")
        .with_node_color("High Conf",     "#377eb8")
        .with_node_color("Low Conf",      "#ff7f00")
        .with_node_color("SNP",           "#984ea3")
        .with_node_color("Indel",         "#a65628")
        .with_node_color("Filtered Out",  "#cccccc")
        .with_link("Raw Variants", "QC Pass",      8000.0)
        .with_link("Raw Variants", "QC Fail",      2000.0)
        .with_link("QC Pass",      "High Conf",    6000.0)
        .with_link("QC Pass",      "Low Conf",     2000.0)
        .with_link("High Conf",    "SNP",          4500.0)
        .with_link("High Conf",    "Indel",        1200.0)
        .with_link("High Conf",    "Filtered Out",  300.0)
        .with_link("Low Conf",     "SNP",           800.0)
        .with_link("Low Conf",     "Filtered Out", 1200.0)
        .with_link_opacity(0.45)
        .with_legend("Stage");
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Variant Filtering Pipeline");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/sankey_variant_filter.svg", svg).unwrap();
}

/// Per-link coloring mode.
#[test]
fn sankey_per_link_color() {
    let sankey = SankeyPlot::new()
        .with_link_colored("Budget", "R&D",       40.0, "#377eb8")
        .with_link_colored("Budget", "Marketing", 25.0, "#e41a1c")
        .with_link_colored("Budget", "Ops",       35.0, "#4daf4a")
        .with_link_colored("R&D",       "Product A", 25.0, "#377eb8")
        .with_link_colored("R&D",       "Product B", 15.0, "#984ea3")
        .with_link_colored("Marketing", "Product A", 15.0, "#e41a1c")
        .with_link_colored("Marketing", "Product B", 10.0, "#ff7f00")
        .with_link_colored("Ops",       "Product A", 20.0, "#4daf4a")
        .with_link_colored("Ops",       "Product B", 15.0, "#a65628")
        .with_per_link_colors()
        .with_link_opacity(0.55);
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Sankey Per-Link Color");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/sankey_per_link_color.svg", svg).unwrap();
}
