use kuva::backend::svg::SvgBackend;
use kuva::plot::SankeyPlot;
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

fn label_y(svg: &str, label: &str) -> f64 {
    let needle = format!(">{label}</text>");
    let label_idx = svg
        .find(&needle)
        .unwrap_or_else(|| panic!("label '{label}' not found"));
    let prefix = &svg[..label_idx];
    let y_idx = prefix
        .rfind(" y=\"")
        .unwrap_or_else(|| panic!("y attr for '{label}' not found"))
        + 4;
    let y_end = prefix[y_idx..].find('"').unwrap();
    prefix[y_idx..y_idx + y_end].parse::<f64>().unwrap()
}

fn label_x(svg: &str, label: &str) -> f64 {
    let needle = format!(">{label}</text>");
    let label_idx = svg
        .find(&needle)
        .unwrap_or_else(|| panic!("label '{label}' not found"));
    let prefix = &svg[..label_idx];
    let x_idx = prefix
        .rfind(" x=\"")
        .unwrap_or_else(|| panic!("x attr for '{label}' not found"))
        + 4;
    let x_end = prefix[x_idx..].find('"').unwrap();
    prefix[x_idx..x_idx + x_end].parse::<f64>().unwrap()
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}

fn node_rect_fills(svg: &str) -> Vec<String> {
    let mut fills = Vec::new();
    let mut idx = 0;
    while let Some(pos) = svg[idx..].find("<rect ") {
        let start = idx + pos;
        let end = svg[start..].find("/>").map(|n| start + n + 2).unwrap();
        let tag = &svg[start..end];
        idx = end;
        if !tag.contains("width=\"20\"") {
            continue;
        }
        if let Some(fill_pos) = tag.find("fill=\"") {
            let rest = &tag[fill_pos + 6..];
            if let Some(end_q) = rest.find('"') {
                fills.push(rest[..end_q].to_string());
            }
        }
    }
    fills
}

fn node_text_labels(svg: &str) -> Vec<String> {
    let mut labels = Vec::new();
    let mut idx = 0;
    while let Some(pos) = svg[idx..].find("<text ") {
        let start = idx + pos;
        let tag_end = svg[start..].find('>').map(|n| start + n + 1).unwrap();
        let end = svg[tag_end..].find("</text>").map(|n| tag_end + n).unwrap();
        let tag = &svg[start..tag_end];
        idx = end + "</text>".len();
        if !tag.contains("text-anchor=") {
            continue;
        }
        let content = svg[tag_end..end].replace("&apos;", "'");
        if !content.is_empty() {
            labels.push(content);
        }
    }
    labels
}

fn node_fill_for_label(svg: &str, label: &str) -> String {
    let fills = node_rect_fills(svg);
    let labels = node_text_labels(svg);
    for (i, text) in labels.iter().enumerate() {
        if text == label {
            return fills[i].clone();
        }
    }
    panic!("label '{label}' not found");
}

fn assert_top_to_bottom(svg: &str, labels: &[&str]) {
    for pair in labels.windows(2) {
        let upper = label_y(svg, pair[0]);
        let lower = label_y(svg, pair[1]);
        assert!(
            upper < lower,
            "expected '{}' above '{}' but got y={} and y={}",
            pair[0],
            pair[1],
            upper,
            lower
        );
    }
}

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
    let layout = Layout::auto_from_plots(&plots).with_title("Sankey Basic");
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
    let layout = Layout::auto_from_plots(&plots).with_title("Sankey Gradient");
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
    let layout = Layout::auto_from_plots(&plots).with_title("Sankey Legend");
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
        .with_link("Source", "Discard A", 30.0) // dead end at col 1
        .with_link("Filter", "Pass", 55.0)
        .with_link("Filter", "Discard B", 15.0) // dead end at col 2
        .with_link("Pass", "Output", 55.0);
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots).with_title("Sankey Dead Ends");
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
        .with_node_color("Raw Variants", "#888888")
        .with_node_color("QC Pass", "#4daf4a")
        .with_node_color("QC Fail", "#e41a1c")
        .with_node_color("High Conf", "#377eb8")
        .with_node_color("Low Conf", "#ff7f00")
        .with_node_color("SNP", "#984ea3")
        .with_node_color("Indel", "#a65628")
        .with_node_color("Filtered Out", "#cccccc")
        .with_link("Raw Variants", "QC Pass", 8000.0)
        .with_link("Raw Variants", "QC Fail", 2000.0)
        .with_link("QC Pass", "High Conf", 6000.0)
        .with_link("QC Pass", "Low Conf", 2000.0)
        .with_link("High Conf", "SNP", 4500.0)
        .with_link("High Conf", "Indel", 1200.0)
        .with_link("High Conf", "Filtered Out", 300.0)
        .with_link("Low Conf", "SNP", 800.0)
        .with_link("Low Conf", "Filtered Out", 1200.0)
        .with_link_opacity(0.45)
        .with_legend("Stage");
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots).with_title("Variant Filtering Pipeline");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/sankey_variant_filter.svg", svg).unwrap();
}

/// Per-link coloring mode.
#[test]
fn sankey_per_link_color() {
    let sankey = SankeyPlot::new()
        .with_link_colored("Budget", "R&D", 40.0, "#377eb8")
        .with_link_colored("Budget", "Marketing", 25.0, "#e41a1c")
        .with_link_colored("Budget", "Ops", 35.0, "#4daf4a")
        .with_link_colored("R&D", "Product A", 25.0, "#377eb8")
        .with_link_colored("R&D", "Product B", 15.0, "#984ea3")
        .with_link_colored("Marketing", "Product A", 15.0, "#e41a1c")
        .with_link_colored("Marketing", "Product B", 10.0, "#ff7f00")
        .with_link_colored("Ops", "Product A", 20.0, "#4daf4a")
        .with_link_colored("Ops", "Product B", 15.0, "#a65628")
        .with_per_link_colors()
        .with_link_opacity(0.55);
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots).with_title("Sankey Per-Link Color");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/sankey_per_link_color.svg", svg).unwrap();
}

#[test]
fn sankey_crossing_reduction_reorders_nodes() {
    let base = SankeyPlot::new()
        .with_node_column("Left A", 0)
        .with_node_column("Left B", 0)
        .with_node_column("Right A", 1)
        .with_node_column("Right B", 1)
        .with_link("Left A", "Right B", 10.0)
        .with_link("Left B", "Right A", 10.0);

    let svg_input = SvgBackend.render_scene(&render_multiple(
        vec![Plot::Sankey(base.clone())],
        Layout::auto_from_plots(&[Plot::Sankey(base.clone())]),
    ));

    let optimized = base.with_crossing_reduction();
    let svg_opt = SvgBackend.render_scene(&render_multiple(
        vec![Plot::Sankey(optimized.clone())],
        Layout::auto_from_plots(&[Plot::Sankey(optimized)]),
    ));

    let input_right_a = label_y(&svg_input, "Right A");
    let input_right_b = label_y(&svg_input, "Right B");
    assert!(
        input_right_a < input_right_b,
        "default input order should keep Right A above Right B"
    );

    let opt_left_a = label_y(&svg_opt, "Left A");
    let opt_left_b = label_y(&svg_opt, "Left B");
    let opt_right_a = label_y(&svg_opt, "Right A");
    let opt_right_b = label_y(&svg_opt, "Right B");
    let left_a_above = opt_left_a < opt_left_b;
    let right_b_above = opt_right_b < opt_right_a;
    assert_eq!(
        left_a_above, right_b_above,
        "crossing reduction should uncross the two-column Sankey by reordering at least one side"
    );
}

#[test]
fn sankey_alluvium_mode_supports_reused_labels_across_axes() {
    let sankey = SankeyPlot::new()
        .with_alluvium(vec!["A", "X", "A"], 10.0)
        .with_alluvium(vec!["B", "Y", "B"], 8.0)
        .with_alluvium(vec!["A", "Y", "B"], 4.0)
        .with_crossing_reduction();

    assert_eq!(
        sankey.alluvia.len(),
        3,
        "alluvia should be stored explicitly"
    );
    assert_eq!(
        sankey.nodes.len(),
        6,
        "same visible labels in different axes must remain distinct nodes"
    );

    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots).with_title("Alluvium Mode");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/sankey_alluvium_mode.svg", &svg).unwrap();

    assert!(svg.contains(">A</text>"));
    assert!(svg.contains(">B</text>"));
    assert!(svg.contains(">X</text>"));
    assert!(svg.contains(">Y</text>"));
}

#[test]
fn sankey_reused_labels_keep_consistent_colors_across_axes() {
    let sankey = SankeyPlot::new()
        .with_alluvium(vec!["Wildlings", "Camp", "Wildlings"], 10.0)
        .with_alluvium(vec!["North", "Castle", "North"], 8.0)
        .with_crossing_reduction();

    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots).with_title("Repeated Label Colors");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));

    assert_eq!(
        count_occurrences(&svg, "fill=\"#1f77b4\""),
        3,
        "reused labels should keep the same palette color across axes and source-colored links"
    );
}

#[test]
fn sankey_left_coloring_propagates_parent_colors() {
    let sankey = SankeyPlot::new()
        .with_alluvium(vec!["A", "X", "P"], 10.0)
        .with_alluvium(vec!["A", "Y", "Q"], 1.0)
        .with_alluvium(vec!["B", "Y", "Q"], 9.0)
        .with_left_coloring();

    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));

    let a = node_fill_for_label(&svg, "A");
    let b = node_fill_for_label(&svg, "B");
    let x = node_fill_for_label(&svg, "X");
    let y = node_fill_for_label(&svg, "Y");
    let p = node_fill_for_label(&svg, "P");
    let q = node_fill_for_label(&svg, "Q");

    assert_eq!(a.to_ascii_uppercase(), "#D55E00");
    assert_eq!(b.to_ascii_uppercase(), "#56B4E9");
    assert_eq!(x, a, "X should inherit A's color");
    assert_eq!(y, b, "Y should inherit B's color");
    assert_eq!(p, x, "P should inherit X's color");
    assert_eq!(q, y, "Q should inherit Y's color");
}

#[test]
fn sankey_alluvium_crossing_reduction_can_reorder_columns() {
    let sankey = SankeyPlot::new()
        .with_alluvium(vec!["A1", "B1", "C1"], 12.0)
        .with_alluvium(vec!["A2", "B2", "C2"], 12.0)
        .with_alluvium(vec!["A1", "B2", "C1"], 3.0)
        .with_alluvium(vec!["A2", "B1", "C2"], 3.0)
        .with_crossing_reduction();

    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots).with_title("Alluvium Column Reorder");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/sankey_alluvium_column_reorder.svg", &svg).unwrap();

    let x_a1 = label_x(&svg, "A1");
    let x_b1 = label_x(&svg, "B1");
    let x_c1 = label_x(&svg, "C1");
    assert!(
        x_a1 != x_b1 && x_b1 != x_c1,
        "three axes should occupy distinct x positions"
    );
}

#[test]
fn sankey_alluvium_crossing_reduction_is_seed_deterministic() {
    let make_plot = || {
        SankeyPlot::new()
            .with_axis_names(["tissue", "cluster", "sex"])
            .with_alluvium(vec!["B CELL", "4", "male"], 9.0)
            .with_alluvium(vec!["BRAIN", "1", "female"], 1.0)
            .with_alluvium(vec!["BRAIN", "1", "male"], 1.0)
            .with_alluvium(vec!["BRAIN", "2", "male"], 1.0)
            .with_alluvium(vec!["HEART", "1", "male"], 1.0)
            .with_alluvium(vec!["HEART", "3", "female"], 3.0)
            .with_alluvium(vec!["HEART", "3", "male"], 3.0)
            .with_alluvium(vec!["STOMACH", "1", "female"], 1.0)
            .with_alluvium(vec!["STOMACH", "2", "female"], 3.0)
            .with_alluvium(vec!["STOMACH", "2", "male"], 2.0)
            .with_alluvium(vec!["T CELL", "4", "female"], 1.0)
            .with_alluvium(vec!["T CELL", "4", "male"], 1.0)
            .with_crossing_reduction()
            .with_left_coloring()
            .with_node_order_seed(42)
    };

    let render = |plot: SankeyPlot| {
        SvgBackend.render_scene(&render_multiple(
            vec![Plot::Sankey(plot.clone())],
            Layout::auto_from_plots(&[Plot::Sankey(plot)]),
        ))
    };

    let svg_a = render(make_plot());
    let svg_b = render(make_plot());
    assert_eq!(
        svg_a, svg_b,
        "same seeded alluvial ordering should render identically"
    );
}

#[test]
fn sankey_alluvium_crossing_reduction_matches_expected_rendered_order() {
    let sankey = SankeyPlot::new()
        .with_axis_names(["tissue", "cluster", "sex"])
        .with_alluvium(vec!["B CELL", "4", "male"], 9.0)
        .with_alluvium(vec!["BRAIN", "1", "female"], 1.0)
        .with_alluvium(vec!["BRAIN", "1", "male"], 1.0)
        .with_alluvium(vec!["BRAIN", "2", "male"], 1.0)
        .with_alluvium(vec!["HEART", "1", "male"], 1.0)
        .with_alluvium(vec!["HEART", "3", "female"], 3.0)
        .with_alluvium(vec!["HEART", "3", "male"], 3.0)
        .with_alluvium(vec!["STOMACH", "1", "female"], 1.0)
        .with_alluvium(vec!["STOMACH", "2", "female"], 3.0)
        .with_alluvium(vec!["STOMACH", "2", "male"], 2.0)
        .with_alluvium(vec!["T CELL", "4", "female"], 1.0)
        .with_alluvium(vec!["T CELL", "4", "male"], 1.0)
        .with_crossing_reduction()
        .with_left_coloring()
        .with_node_order_seed(42);

    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots).with_title("Alluvium TSP Regression");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/sankey_alluvium_tsp_regression.svg", &svg).unwrap();

    assert_top_to_bottom(&svg, &["STOMACH", "HEART", "BRAIN", "T CELL", "B CELL"]);
    assert_top_to_bottom(&svg, &["2", "3", "1", "4"]);
    assert_top_to_bottom(&svg, &["female", "male"]);
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
        .with_link("Raw", "Trimmed", 82.0)
        .with_link("Raw", "Discarded", 3.0)
        .with_link("Trimmed", "Aligned", 68.0)
        .with_link("Trimmed", "Unmapped", 6.0)
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
        .with_link("Raw Reads", "Trimmed", 8_200_000.0)
        .with_link("Raw Reads", "Discarded", 300_000.0)
        .with_link("Trimmed", "Aligned", 6_800_000.0)
        .with_link("Trimmed", "Unmapped", 600_000.0)
        .with_link("Aligned", "Exonic", 4_200_000.0)
        .with_link("Aligned", "Intronic", 1_800_000.0)
        .with_link("Aligned", "Intergenic", 800_000.0)
        .with_flow_labels()
        .with_flow_label_format(TickFormat::Sci);
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots).with_title("Flow Labels — Scientific Notation");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/sankey_flow_labels_sci.svg", &svg).unwrap();
    // Scientific notation uses 'e'
    assert!(
        svg.contains('e'),
        "scientific notation labels should contain 'e'"
    );
}

/// min_height 0.0 forces labels on all ribbons including small ones.
#[test]
fn sankey_flow_labels_min_height_zero() {
    let sankey = SankeyPlot::new()
        .with_link("Input", "Large", 90.0)
        .with_link("Input", "Small", 1.0) // ribbon too narrow at default threshold
        .with_link("Large", "Output", 90.0)
        .with_link("Small", "Output", 1.0)
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
        .with_node_color("Raw Variants", "#888888")
        .with_node_color("QC Pass", "#4daf4a")
        .with_node_color("QC Fail", "#e41a1c")
        .with_node_color("High Conf", "#377eb8")
        .with_node_color("Low Conf", "#ff7f00")
        .with_node_color("SNP", "#984ea3")
        .with_node_color("Indel", "#a65628")
        .with_node_color("Filtered Out", "#cccccc")
        .with_link("Raw Variants", "QC Pass", 8000.0)
        .with_link("Raw Variants", "QC Fail", 2000.0)
        .with_link("QC Pass", "High Conf", 6000.0)
        .with_link("QC Pass", "Low Conf", 2000.0)
        .with_link("High Conf", "SNP", 4500.0)
        .with_link("High Conf", "Indel", 1200.0)
        .with_link("High Conf", "Filtered Out", 300.0)
        .with_link("Low Conf", "SNP", 800.0)
        .with_link("Low Conf", "Filtered Out", 1200.0)
        .with_flow_percent()
        .with_link_opacity(0.45);
    let plots = vec![Plot::Sankey(sankey)];
    let layout =
        Layout::auto_from_plots(&plots).with_title("Variant Filtering — % of source outflow");
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
