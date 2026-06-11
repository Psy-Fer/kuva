mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::clustermap::{AnnotationTrack, Clustermap, ClustermapNorm};
use kuva::plot::{ColorMap, PhyloTree};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use std::fs;

fn write_svg(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all("test_outputs").unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output(format!("test_outputs/{name}.svg"), &svg).unwrap();
    assert!(!svg.is_empty());
}

fn make_data_5x5() -> Vec<Vec<f64>> {
    vec![
        vec![5.0, 0.1, 0.2, 0.0, 4.8],
        vec![0.2, 4.9, 0.1, 0.3, 0.1],
        vec![0.1, 4.8, 0.2, 0.1, 0.0],
        vec![0.0, 0.2, 5.1, 0.1, 0.2],
        vec![4.7, 0.1, 0.0, 0.2, 5.0],
    ]
}

#[test]
fn test_clustermap_basic() {
    let data = make_data_5x5();
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(["A", "B", "C", "D", "E"])
        .with_col_labels(["X1", "X2", "X3", "X4", "X5"])
        .with_legend("Value");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("Clustermap Basic");
    write_svg("clustermap_basic", plots, layout);
}

#[test]
fn test_clustermap_row_only() {
    let data = make_data_5x5();
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(["A", "B", "C", "D", "E"])
        .with_col_labels(["X1", "X2", "X3", "X4", "X5"])
        .with_cluster_rows(true)
        .with_cluster_cols(false)
        .with_legend("Value");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("Clustermap Row Only");
    write_svg("clustermap_row_only", plots, layout);
}

#[test]
fn test_clustermap_col_only() {
    let data = make_data_5x5();
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(["A", "B", "C", "D", "E"])
        .with_col_labels(["X1", "X2", "X3", "X4", "X5"])
        .with_cluster_rows(false)
        .with_cluster_cols(true)
        .with_legend("Value");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("Clustermap Col Only");
    write_svg("clustermap_col_only", plots, layout);
}

#[test]
fn test_clustermap_no_clustering() {
    let data = make_data_5x5();
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(["A", "B", "C", "D", "E"])
        .with_col_labels(["X1", "X2", "X3", "X4", "X5"])
        .with_cluster_rows(false)
        .with_cluster_cols(false)
        .with_legend("Value");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("Labeled Heatmap via Clustermap");
    write_svg("clustermap_no_clustering", plots, layout);
}

#[test]
fn test_clustermap_with_row_annotation() {
    let data = make_data_5x5();
    let group_colors = vec![
        "#e41a1c".to_string(),
        "#377eb8".to_string(),
        "#377eb8".to_string(),
        "#4daf4a".to_string(),
        "#e41a1c".to_string(),
    ];
    let track = AnnotationTrack::new(group_colors).with_label("Group");
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(["A", "B", "C", "D", "E"])
        .with_col_labels(["X1", "X2", "X3", "X4", "X5"])
        .with_row_annotation(track)
        .with_legend("Value");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("Clustermap with Row Annotation");
    write_svg("clustermap_row_annotation", plots, layout);
}

#[test]
fn test_clustermap_with_both_annotations() {
    let data = make_data_5x5();
    let row_colors = vec![
        "#e41a1c".to_string(),
        "#377eb8".to_string(),
        "#377eb8".to_string(),
        "#4daf4a".to_string(),
        "#e41a1c".to_string(),
    ];
    let col_colors = vec![
        "#ff7f00".to_string(),
        "#ff7f00".to_string(),
        "#984ea3".to_string(),
        "#984ea3".to_string(),
        "#ff7f00".to_string(),
    ];
    let row_track = AnnotationTrack::new(row_colors).with_label("Sample");
    let col_track = AnnotationTrack::new(col_colors).with_label("Feature");
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(["A", "B", "C", "D", "E"])
        .with_col_labels(["X1", "X2", "X3", "X4", "X5"])
        .with_row_annotation(row_track)
        .with_col_annotation(col_track)
        .with_legend("Value");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("Clustermap Both Annotations");
    write_svg("clustermap_both_annotations", plots, layout);
}

#[test]
fn test_clustermap_zscore_row() {
    let data = vec![
        vec![100.0, 0.5, 0.3, 0.1, 90.0],
        vec![0.2, 50.0, 0.1, 0.4, 0.3],
        vec![0.1, 45.0, 0.2, 0.5, 0.1],
        vec![0.3, 0.1, 80.0, 0.2, 0.2],
        vec![95.0, 0.2, 0.1, 0.3, 88.0],
    ];
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(["A", "B", "C", "D", "E"])
        .with_col_labels(["X1", "X2", "X3", "X4", "X5"])
        .with_normalization(ClustermapNorm::RowZScore)
        .with_legend("Z-score");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("Clustermap Row Z-score");
    write_svg("clustermap_zscore_row", plots, layout);
}

#[test]
fn test_clustermap_zscore_col() {
    let data = vec![
        vec![100.0, 0.5, 0.3, 0.1, 90.0],
        vec![0.2, 50.0, 0.1, 0.4, 0.3],
        vec![0.1, 45.0, 0.2, 0.5, 0.1],
        vec![0.3, 0.1, 80.0, 0.2, 0.2],
        vec![95.0, 0.2, 0.1, 0.3, 88.0],
    ];
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(["A", "B", "C", "D", "E"])
        .with_col_labels(["X1", "X2", "X3", "X4", "X5"])
        .with_normalization(ClustermapNorm::ColZScore)
        .with_legend("Z-score");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("Clustermap Col Z-score");
    write_svg("clustermap_zscore_col", plots, layout);
}

#[test]
fn test_clustermap_pretrained_tree() {
    let labels = ["A", "B", "C", "D", "E"];
    let data = make_data_5x5();
    let label_strs: Vec<&str> = labels.iter().copied().collect();
    let row_tree = PhyloTree::from_distance_matrix(&label_strs, &data);
    let cm = Clustermap::new()
        .with_data(data.clone())
        .with_row_labels(labels.iter().copied())
        .with_col_labels(["X1", "X2", "X3", "X4", "X5"])
        .with_row_tree(row_tree)
        .with_cluster_rows(false)
        .with_legend("Value");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("Clustermap Pre-trained Row Tree");
    write_svg("clustermap_pretrained_tree", plots, layout);
}

#[test]
fn test_clustermap_gene_expression() {
    // 20 genes × 6 samples — biologically realistic layout
    let genes = [
        "GAPDH", "ACTB", "TP53", "BRCA1", "MYC", "EGFR", "VEGF", "IL6", "TNF", "IFNG", "CD3D",
        "CD8A", "CD4", "FOXP3", "GZMB", "PRF1", "IL2", "IL10", "TGFB1", "CTLA4",
    ];
    let samples = ["S1", "S2", "S3", "S4", "S5", "S6"];

    // Structured block pattern: genes 0-5 high in S1-S3, genes 6-12 high in S4-S6
    let mut data = vec![vec![0.1f64; 6]; 20];
    for i in 0..6 {
        for j in 0..3 {
            data[i][j] = 8.0 + (i + j) as f64 * 0.2;
        }
    }
    for i in 6..13 {
        for j in 3..6 {
            data[i][j] = 7.0 + (i + j) as f64 * 0.1;
        }
    }
    for i in 13..20 {
        for j in 0..6 {
            data[i][j] = 3.0 + (i * j) as f64 * 0.05;
        }
    }

    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(genes.iter().copied())
        .with_col_labels(samples.iter().copied())
        .with_normalization(ClustermapNorm::RowZScore)
        .with_legend("Z-score");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Gene Expression Clustermap")
        .with_width(700.0)
        .with_height(600.0);
    write_svg("clustermap_gene_expression", plots, layout);
}

#[test]
fn test_clustermap_large() {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    let mut rng = StdRng::seed_from_u64(42);
    let n = 30;
    // Block-structured so clustering produces visible groups
    let mut data = vec![vec![0.0f64; n]; n];
    let block_size = n / 3;
    for bi in 0..3usize {
        for bj in 0..3usize {
            let signal = if bi == bj { 5.0 } else { 0.0 };
            for i in (bi * block_size)..((bi + 1) * block_size) {
                for j in (bj * block_size)..((bj + 1) * block_size) {
                    data[i][j] = signal + rng.random::<f64>() * 0.5;
                }
            }
        }
    }
    let row_labels: Vec<String> = (0..n).map(|i| format!("R{i}")).collect();
    let col_labels: Vec<String> = (0..n).map(|i| format!("C{i}")).collect();
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(row_labels.iter().map(|s| s.as_str()))
        .with_col_labels(col_labels.iter().map(|s| s.as_str()))
        .with_legend("Value");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Large Clustermap (30×30)")
        .with_width(800.0)
        .with_height(700.0);
    write_svg("clustermap_large", plots, layout);
}

#[test]
fn test_clustermap_inferno() {
    let data = make_data_5x5();
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(["A", "B", "C", "D", "E"])
        .with_col_labels(["X1", "X2", "X3", "X4", "X5"])
        .with_color_map(ColorMap::Inferno)
        .with_legend("Value");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("Clustermap Inferno");
    write_svg("clustermap_inferno", plots, layout);
}

#[test]
fn test_clustermap_no_labels() {
    let data = make_data_5x5();
    let cm = Clustermap::new().with_data(data).with_legend("Value");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("Clustermap No Labels");
    write_svg("clustermap_no_labels", plots, layout);
}

#[test]
fn test_clustermap_values_overlay() {
    let data = vec![
        vec![1.0, 2.0, 3.0],
        vec![4.0, 5.0, 6.0],
        vec![7.0, 8.0, 9.0],
    ];
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(["A", "B", "C"])
        .with_col_labels(["X", "Y", "Z"])
        .with_values()
        .with_cluster_rows(false)
        .with_cluster_cols(false)
        .with_legend("Value");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("Clustermap Values Overlay");
    write_svg("clustermap_values_overlay", plots, layout);
}

/// Regression test for issue #59: nodes that merged at the same UPGMA distance
/// must appear at the same x-position in the dendrogram. The 5×5 matrix below
/// has A, D, E all pairwise-equidistant at 1.0, so they should merge at the
/// same level in the row dendrogram.
#[test]
fn test_clustermap_equal_distance_dendrogram() {
    // Distance matrix where A↔D = A↔E = D↔E = 1.0 (all three equidistant)
    // B and C are closer to each other (0.5) and far from A/D/E (3.0+)
    let data = vec![
        // A: high col1
        vec![5.0, 0.1, 0.1, 0.1, 0.1],
        // B: high col2+col3
        vec![0.1, 4.0, 4.0, 0.1, 0.1],
        // C: high col2+col3 (similar to B)
        vec![0.1, 3.9, 4.1, 0.1, 0.1],
        // D: high col1 (same profile as A)
        vec![5.0, 0.1, 0.1, 0.1, 0.1],
        // E: high col1 (same profile as A)
        vec![5.0, 0.1, 0.1, 0.1, 0.1],
    ];
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(["A", "B", "C", "D", "E"])
        .with_col_labels(["c1", "c2", "c3", "c4", "c5"])
        .with_legend("Value");
    let plots = vec![Plot::Clustermap(cm)];
    let layout =
        Layout::auto_from_plots(&plots).with_title("Equal-distance dendrogram (issue #59)");
    write_svg("clustermap_equal_distance", plots, layout);
}
