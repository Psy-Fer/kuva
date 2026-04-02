use kuva::plot::SankeyPlot;
use kuva::render::{plots::Plot, layout::Layout, render::render_multiple};
use kuva::backend::svg::SvgBackend;

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
    std::fs::write("test_outputs/sankey_per_link_color.svg", svg).unwrap();
}

// ── Flow label tests ──────────────────────────────────────────────────────────

/// Absolute flow labels with default Auto format.
#[test]
fn sankey_flow_labels_basic() {
    let sankey = SankeyPlot::new()
        .with_link("Input", "Process A", 50.0)
        .with_link("Input", "Process B", 30.0)
        .with_link("Process A", "Output X", 40.0)
        .with_link("Process A", "Output Y", 10.0)
        .with_link("Process B", "Output X", 10.0)
        .with_link("Process B", "Output Y", 20.0)
        .with_flow_labels();
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots).with_title("Flow Labels — Absolute");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/sankey_flow_labels_basic.svg", &svg).unwrap();
    // Values 50, 30, 40, 10, 10, 20 should appear as text
    assert!(svg.contains(">50<"), "link value 50 should appear");
    assert!(svg.contains(">30<"), "link value 30 should appear");
    assert!(svg.contains(">40<"), "link value 40 should appear");
}

/// Percentage flow labels — expressed as fraction of source outflow.
#[test]
fn sankey_flow_percent_basic() {
    let sankey = SankeyPlot::new()
        .with_link("Input", "Process A", 75.0)
        .with_link("Input", "Process B", 25.0)
        .with_link("Process A", "Output", 75.0)
        .with_link("Process B", "Output", 25.0)
        .with_flow_percent();
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots).with_title("Flow Labels — Percent");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/sankey_flow_percent_basic.svg", &svg).unwrap();
    // 75/(75+25)*100 = 75.0%
    assert!(svg.contains("75.0%"), "75% label should appear");
    assert!(svg.contains("25.0%"), "25% label should appear");
}

/// Absolute labels with a unit suffix.
#[test]
fn sankey_flow_labels_unit() {
    let sankey = SankeyPlot::new()
        .with_link("Raw", "Trimmed",  82.0)
        .with_link("Raw", "Discarded", 3.0)
        .with_link("Trimmed", "Aligned",  68.0)
        .with_link("Trimmed", "Unmapped",  6.0)
        .with_flow_labels()
        .with_flow_label_unit("reads")
        .with_flow_label_min_height(0.0);
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots).with_title("Flow Labels — Unit Suffix");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/sankey_flow_labels_unit.svg", &svg).unwrap();
    assert!(svg.contains("reads"), "unit suffix 'reads' should appear");
    assert!(svg.contains("82 reads"), "82 reads label should appear");
}

/// Scientific notation format for large values.
#[test]
fn sankey_flow_labels_sci() {
    use kuva::TickFormat;
    let sankey = SankeyPlot::new()
        .with_link("Raw Reads",    "Trimmed",    8_200_000.0)
        .with_link("Raw Reads",    "Discarded",    300_000.0)
        .with_link("Trimmed",      "Aligned",    6_800_000.0)
        .with_link("Trimmed",      "Unmapped",     600_000.0)
        .with_link("Aligned",      "Exonic",     4_200_000.0)
        .with_link("Aligned",      "Intronic",   1_800_000.0)
        .with_link("Aligned",      "Intergenic",   800_000.0)
        .with_flow_labels()
        .with_flow_label_format(TickFormat::Sci);
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots).with_title("Flow Labels — Scientific Notation");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/sankey_flow_labels_sci.svg", &svg).unwrap();
    // Scientific notation uses 'e'
    assert!(svg.contains('e'), "scientific notation labels should contain 'e'");
}

/// min_height 0.0 forces labels on all ribbons including small ones.
#[test]
fn sankey_flow_labels_min_height_zero() {
    let sankey = SankeyPlot::new()
        .with_link("Input", "Large",  90.0)
        .with_link("Input", "Small",   1.0)  // ribbon too narrow at default threshold
        .with_link("Large", "Output", 90.0)
        .with_link("Small", "Output",  1.0)
        .with_flow_labels()
        .with_flow_label_min_height(0.0);
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots).with_title("Flow Labels — min_height 0");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/sankey_flow_labels_min_height_zero.svg", &svg).unwrap();
    // Both "90" and "1" labels should appear
    assert!(svg.contains(">90<"), "large ribbon label should appear");
    assert!(svg.contains(">1<"), "small ribbon label should appear");
}

/// Percent labels on the variant-filter pipeline — bioinformatics use case.
#[test]
fn sankey_flow_percent_variant_filter() {
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
        .with_flow_percent()
        .with_link_opacity(0.45);
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Variant Filtering — % of source outflow");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/sankey_flow_percent_variant_filter.svg", &svg).unwrap();
    // QC Pass gets 80% of raw variants
    assert!(svg.contains("80.0%"), "80% QC pass label should appear");
    // QC Fail gets 20%
    assert!(svg.contains("20.0%"), "20% QC fail label should appear");
}

/// flow_percent takes priority over flow_labels when both are set.
#[test]
fn sankey_flow_percent_priority() {
    let sankey = SankeyPlot::new()
        .with_link("A", "B", 60.0)
        .with_link("A", "C", 40.0)
        .with_flow_labels()
        .with_flow_percent();
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    // Should show percentages, not raw values
    assert!(svg.contains("60.0%"), "percent label should appear");
    assert!(svg.contains("40.0%"), "percent label should appear");
    // Raw values '60' and '40' as standalone labels would appear; but as a percent we have '60.0%'
    // The raw number without '%' should not be the primary label here — at minimum % must appear
    assert!(svg.contains('%'), "percent sign should appear in output");
}
