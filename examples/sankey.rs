//! Sankey diagram documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example sankey
//! ```
//!
//! SVGs are written to `docs/src/assets/sankey/`.

use kuva::plot::SankeyPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::TickFormat;

const OUT: &str = "docs/src/assets/sankey";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/sankey");

    basic();
    node_colors();
    gradient();
    variant_filter();
    flow_labels_absolute();
    flow_labels_percent();
    flow_labels_pipeline();

    println!("Sankey SVGs written to {OUT}/");
}

/// Basic 3-stage pipeline — nodes auto-colored, columns auto-assigned.
fn basic() {
    let sankey = SankeyPlot::new()
        .with_link("Input", "Process A", 50.0)
        .with_link("Input", "Process B", 30.0)
        .with_link("Process A", "Output X", 40.0)
        .with_link("Process A", "Output Y", 10.0)
        .with_link("Process B", "Output X", 10.0)
        .with_link("Process B", "Output Y", 20.0);

    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Energy Flow");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Explicit node colors and a per-node legend.
fn node_colors() {
    let sankey = SankeyPlot::new()
        .with_node_color("Input", "#888888")
        .with_node_color("Process A", "#377eb8")
        .with_node_color("Process B", "#4daf4a")
        .with_node_color("Output", "#984ea3")
        .with_link("Input", "Process A", 40.0)
        .with_link("Input", "Process B", 30.0)
        .with_link("Process A", "Output", 35.0)
        .with_link("Process B", "Output", 25.0)
        .with_node_width(24.0)
        .with_legend("Stage");

    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Node Colors & Legend");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/node_colors.svg"), svg).unwrap();
}

/// Gradient ribbons — each ribbon fades from source to target color.
fn gradient() {
    let sankey = SankeyPlot::new()
        .with_node_color("Budget", "#e41a1c")
        .with_node_color("R&D", "#377eb8")
        .with_node_color("Marketing", "#4daf4a")
        .with_node_color("Ops", "#ff7f00")
        .with_node_color("Product A", "#984ea3")
        .with_node_color("Product B", "#a65628")
        .with_link("Budget", "R&D",       40.0)
        .with_link("Budget", "Marketing", 25.0)
        .with_link("Budget", "Ops",       35.0)
        .with_link("R&D",       "Product A", 25.0)
        .with_link("R&D",       "Product B", 15.0)
        .with_link("Marketing", "Product A", 15.0)
        .with_link("Marketing", "Product B", 10.0)
        .with_link("Ops",       "Product A", 20.0)
        .with_link("Ops",       "Product B", 15.0)
        .with_gradient_links()
        .with_link_opacity(0.6);

    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Budget Allocation — Gradient Ribbons");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/gradient.svg"), svg).unwrap();
}

/// Absolute flow labels — RNA-seq read processing pipeline, unit suffix "reads".
fn flow_labels_absolute() {
    let sankey = SankeyPlot::new()
        .with_link("Raw reads",  "Trimmed",        82.0)
        .with_link("Raw reads",  "Discarded",        3.0)
        .with_link("Trimmed",    "Genome aligned",  68.0)
        .with_link("Trimmed",    "rRNA",             8.0)
        .with_link("Trimmed",    "Unmapped",         6.0)
        .with_link("Genome aligned", "Exonic",      42.0)
        .with_link("Genome aligned", "Intronic",    18.0)
        .with_link("Genome aligned", "Intergenic",   8.0)
        .with_flow_labels()
        .with_flow_label_unit("M reads")
        .with_link_opacity(0.5);

    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("RNA-seq Pipeline — Absolute Flow (M reads)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/flow_labels_absolute.svg"), svg).unwrap();
}

/// Percentage flow labels — fraction of each source node's total outflow.
fn flow_labels_percent() {
    let sankey = SankeyPlot::new()
        .with_node_color("Raw reads",        "#888888")
        .with_node_color("Trimmed",          "#4daf4a")
        .with_node_color("Discarded",        "#e41a1c")
        .with_node_color("Genome aligned",   "#377eb8")
        .with_node_color("rRNA",             "#ff7f00")
        .with_node_color("Unmapped",         "#cccccc")
        .with_link("Raw reads",  "Trimmed",        82.0)
        .with_link("Raw reads",  "Discarded",        3.0)
        .with_link("Trimmed",    "Genome aligned",  68.0)
        .with_link("Trimmed",    "rRNA",             8.0)
        .with_link("Trimmed",    "Unmapped",         6.0)
        .with_flow_percent()
        .with_link_opacity(0.5);

    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("RNA-seq Pipeline — % of Source Outflow");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/flow_labels_percent.svg"), svg).unwrap();
}

/// Large-count pipeline with scientific notation labels.
fn flow_labels_pipeline() {
    let sankey = SankeyPlot::new()
        .with_node_color("Raw Variants",  "#888888")
        .with_node_color("QC Pass",       "#4daf4a")
        .with_node_color("QC Fail",       "#e41a1c")
        .with_node_color("High Conf",     "#377eb8")
        .with_node_color("Low Conf",      "#ff7f00")
        .with_node_color("SNP",           "#984ea3")
        .with_node_color("Indel",         "#a65628")
        .with_node_color("Filtered Out",  "#cccccc")
        .with_link("Raw Variants", "QC Pass",      8_000_000.0)
        .with_link("Raw Variants", "QC Fail",      2_000_000.0)
        .with_link("QC Pass",      "High Conf",    6_000_000.0)
        .with_link("QC Pass",      "Low Conf",     2_000_000.0)
        .with_link("High Conf",    "SNP",          4_500_000.0)
        .with_link("High Conf",    "Indel",        1_200_000.0)
        .with_link("High Conf",    "Filtered Out",   300_000.0)
        .with_link("Low Conf",     "SNP",            800_000.0)
        .with_link("Low Conf",     "Filtered Out", 1_200_000.0)
        .with_flow_labels()
        .with_flow_label_format(TickFormat::Sci)
        .with_link_opacity(0.45)
        .with_legend("Stage");

    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Variant Filtering — Scientific Notation Labels");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/flow_labels_pipeline.svg"), svg).unwrap();
}

/// Wide 4-stage variant-filtering pipeline — a bioinformatics use case.
fn variant_filter() {
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
    std::fs::write(format!("{OUT}/variant_filter.svg"), svg).unwrap();
}
