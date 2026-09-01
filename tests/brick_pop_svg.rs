mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::BrickPlot;
use kuva::plot::ColorMap;
use kuva::render::brick_pop::{BrickPopPlot, MetricColumn};
use std::collections::HashMap;

/// A small RFC1-like locus: five frequency-sorted alleles of differing expansion sizes
/// (one interrupted), a frequency bar, and two metric heatbox columns (one with a
/// missing value). Exercises the whole composite: name gutter, freq bar, heatboxes,
/// and the embedded run-length-merged brick column, all row-aligned.
#[test]
fn test_brick_pop_basic() {
    let strigars: Vec<(String, String)> = vec![
        ("AATGG:A".to_string(), "40A".to_string()),
        ("AATGG:A".to_string(), "22A".to_string()),
        ("AATGG:A,AAGGG:B".to_string(), "20A5B15A".to_string()),
        ("AATGG:A".to_string(), "12A".to_string()),
        ("AATGG:A".to_string(), "8A".to_string()),
    ];
    let names = vec!["allele_1", "allele_2", "allele_3", "allele_4", "allele_5"];

    let mut motif_colors: HashMap<String, String> = HashMap::new();
    motif_colors.insert("AATGG".to_string(), "#4c78a8".to_string());
    motif_colors.insert("AAGGG".to_string(), "#e45756".to_string());

    let brick = BrickPlot::new()
        .with_names(names)
        .with_motif_colors(motif_colors)
        .with_merge_runs(true)
        .with_strigars(strigars);

    let plot = BrickPopPlot::new(brick)
        .with_title("RFC1-like locus - population alleles")
        .with_frequencies(vec![0.55, 0.20, 0.12, 0.08, 0.05])
        .with_metric(
            "methylation",
            vec![Some(0.9), Some(0.6), None, Some(0.3), Some(0.1)],
            ColorMap::Viridis,
        )
        .with_metric_column(
            MetricColumn::new(
                "motif entropy",
                vec![Some(0.10), Some(0.15), Some(0.80), Some(0.05), Some(0.0)],
                ColorMap::Inferno,
            )
            .with_range(0.0, 1.0),
        );

    let scene = plot.render(900.0);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/brick_pop_basic.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"), "must produce valid SVG");
    // Frequency bar colour, pinned motif colour, and NA cell colour should all appear.
    assert!(svg.contains("#4c78a8"), "freq bars / AATGG motif colour");
    assert!(
        svg.contains("#eeeeee"),
        "NA metric cell colour (allele_3 methylation)"
    );
}
