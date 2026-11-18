//! Clustermap documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example clustermap
//! ```
//!
//! SVGs are written to `docs/src/assets/clustermap/`.

use kuva::backend::svg::SvgBackend;
use kuva::plot::clustermap::{AnnotationPlacement, AnnotationTrack, DiscreteColorMap};
use kuva::plot::{Clustermap, ClustermapNorm, ColLabelSide, ColorMap, RowLabelSide};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use std::fs;

const OUT: &str = "docs/src/assets/clustermap";

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all(OUT).unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

fn main() {
    // ── Basic ──────────────────────────────────────────────────────────────
    // Two-group structure: rows A/D/E (high col1/col5) vs B/C (high col2/col3)
    let data = vec![
        vec![0.9, 0.1, 0.2, 0.1, 0.8], // A
        vec![0.1, 0.8, 0.9, 0.1, 0.2], // B
        vec![0.2, 0.9, 0.8, 0.1, 0.1], // C
        vec![0.8, 0.1, 0.1, 0.2, 0.9], // D
        vec![0.9, 0.2, 0.1, 0.1, 0.8], // E
    ];
    let cm = Clustermap::new()
        .with_data(data.clone())
        .with_row_labels(["A", "B", "C", "D", "E"])
        .with_col_labels(["X1", "X2", "X3", "X4", "X5"])
        .with_legend("Value");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("Clustermap");
    write("basic", plots, layout);

    // ── Gene expression ──────────────────────────────────────────────────
    // 8 genes × 6 conditions; three expression modules
    let expr = vec![
        vec![8.2, 7.9, 0.4, 0.2, 0.1, 0.3], // Gene1
        vec![7.8, 8.1, 0.3, 0.1, 0.4, 0.2], // Gene2
        vec![0.2, 0.3, 7.5, 8.0, 0.1, 0.2], // Gene3
        vec![0.1, 0.4, 8.1, 7.6, 0.3, 0.1], // Gene4
        vec![0.3, 0.1, 0.2, 0.1, 8.3, 7.9], // Gene5
        vec![0.2, 0.2, 0.1, 0.3, 7.8, 8.2], // Gene6
        vec![4.1, 0.3, 3.9, 0.2, 4.2, 0.1], // Gene7 (mixed)
        vec![0.1, 4.3, 0.2, 4.0, 0.3, 4.1], // Gene8 (mixed)
    ];
    let gene_names = [
        "Gene1", "Gene2", "Gene3", "Gene4", "Gene5", "Gene6", "Gene7", "Gene8",
    ];
    let cond_names = ["CtrlA", "CtrlB", "TreatA", "TreatB", "StimA", "StimB"];
    let cm = Clustermap::new()
        .with_data(expr)
        .with_row_labels(gene_names)
        .with_col_labels(cond_names)
        .with_color_map(ColorMap::Viridis)
        .with_legend("Expression");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Gene Expression Clustermap")
        .with_width(520.0)
        .with_height(420.0);
    write("gene_expression", plots, layout);

    // ── Z-score normalised ────────────────────────────────────────────────
    let cm = Clustermap::new()
        .with_data(data.clone())
        .with_row_labels(["A", "B", "C", "D", "E"])
        .with_col_labels(["X1", "X2", "X3", "X4", "X5"])
        .with_normalization(ClustermapNorm::ColZScore)
        .with_color_map(ColorMap::Inferno)
        .with_legend("Z-score");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("Z-score (columns)");
    write("zscore", plots, layout);

    // ── Equal-distance dendrogram (issue #59 reproducer) ──────────────────
    // Rows A, D, E have identical profiles → they should appear at the same
    // dendrogram level, not at artificially staggered levels.
    let eq_data = vec![
        vec![5.0, 0.1, 0.1, 0.1], // A — identical profile
        vec![0.1, 5.0, 0.1, 0.1], // B
        vec![0.1, 5.0, 0.1, 0.1], // C — identical to B
        vec![5.0, 0.1, 0.1, 0.1], // D — identical to A
        vec![5.0, 0.1, 0.1, 0.1], // E — identical to A
    ];
    let cm = Clustermap::new()
        .with_data(eq_data)
        .with_row_labels(["A", "B", "C", "D", "E"])
        .with_col_labels(["c1", "c2", "c3", "c4"])
        .with_legend("Value");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("Equal-distance dendrogram");
    write("equal_distance", plots, layout);

    // ── Row-only clustering ───────────────────────────────────────────────
    let cm = Clustermap::new()
        .with_data(data.clone())
        .with_row_labels(["A", "B", "C", "D", "E"])
        .with_col_labels(["X1", "X2", "X3", "X4", "X5"])
        .with_cluster_cols(false)
        .with_legend("Value");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("Row clustering only");
    write("row_only", plots, layout);

    // ── No clustering ─────────────────────────────────────────────────────
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(["A", "B", "C", "D", "E"])
        .with_col_labels(["X1", "X2", "X3", "X4", "X5"])
        .with_cluster_rows(false)
        .with_cluster_cols(false)
        .with_values()
        .with_legend("Value");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("No clustering (values shown)");
    write("no_clustering", plots, layout);

    annotated();
    genotypes();
    copy_number();
    str_panel();
}

/// Read a TSV from `examples/data`: (header, rows).
fn read_tsv(name: &str) -> (Vec<String>, Vec<Vec<String>>) {
    let text = fs::read_to_string(format!("examples/data/{name}")).unwrap();
    let mut lines = text.lines();
    let header = lines
        .next()
        .unwrap()
        .split('\t')
        .map(String::from)
        .collect();
    let rows = lines
        .filter(|l| !l.is_empty())
        .map(|l| l.split('\t').map(String::from).collect())
        .collect();
    (header, rows)
}

/// Look up `column` of `file` for each key (matched on the file's first column).
fn annotation(file: &str, column: usize, keys: &[String]) -> Vec<String> {
    let (_, rows) = read_tsv(file);
    keys.iter()
        .map(|k| {
            rows.iter()
                .find(|r| &r[0] == k)
                .map_or(String::new(), |r| r[column].clone())
        })
        .collect()
}

/// The full annotated layout: expression z-scores with condition and batch column tracks, a
/// pathway row track, labels top and left. Same data as the CLI example.
fn annotated() {
    let (header, rows) = read_tsv("clustermap_expr.tsv");
    let samples: Vec<String> = header[1..].to_vec();
    let genes: Vec<String> = rows.iter().map(|r| r[0].clone()).collect();
    let data: Vec<Vec<f64>> = rows
        .iter()
        .map(|r| r[1..].iter().map(|v| v.parse().unwrap()).collect())
        .collect();
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(genes.clone())
        .with_col_labels(samples.clone())
        .with_normalization(ClustermapNorm::RowZScore)
        .with_col_annotation(
            AnnotationTrack::categorical(annotation("clustermap_samples.tsv", 1, &samples))
                .with_label("Condition")
                .with_category_color("ctrl", "#999999")
                .with_category_label("treat", "Treated (10 uM)")
                .with_category_order(["ctrl", "treat", "rescue"]),
        )
        .with_col_annotation(
            AnnotationTrack::categorical(annotation("clustermap_samples.tsv", 2, &samples))
                .with_label("Batch"),
        )
        .with_row_annotation(
            AnnotationTrack::categorical(annotation("clustermap_genes.tsv", 1, &genes))
                .with_label("Pathway"),
        )
        .with_col_labels_position(ColLabelSide::Top)
        .with_row_labels_position(RowLabelSide::Left)
        .with_cell_size(26.0, 16.0)
        .with_legend("z-score");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("Annotated expression heatmap");
    write("annotated", plots, layout);

    // The simpler two-track figure the "Annotation tracks" section shows.
    let data = vec![
        vec![0.9, 0.1, 0.2, 0.1, 0.8],
        vec![0.1, 0.8, 0.9, 0.1, 0.2],
        vec![0.2, 0.9, 0.8, 0.1, 0.1],
        vec![0.8, 0.1, 0.1, 0.2, 0.9],
        vec![0.9, 0.2, 0.1, 0.1, 0.8],
    ];
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(["A", "B", "C", "D", "E"])
        .with_col_labels(["X1", "X2", "X3", "X4", "X5"])
        .with_row_annotation(
            AnnotationTrack::categorical(["case", "control", "control", "case", "case"])
                .with_label("Status"),
        )
        .with_col_annotation(
            AnnotationTrack::categorical(["early", "late", "late", "late", "early"])
                .with_label("Timepoint"),
        )
        .with_legend("Value");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("Annotation tracks");
    write("both_annotations", plots, layout);
}

/// Categorical cells: a genotype matrix with a population track.
fn genotypes() {
    let (header, rows) = read_tsv("clustermap_genotypes.tsv");
    let samples: Vec<String> = header[1..].to_vec();
    let variants: Vec<String> = rows.iter().map(|r| r[0].clone()).collect();
    let calls: Vec<Vec<String>> = rows
        .iter()
        .map(|r| {
            r[1..]
                .iter()
                .map(|c| if c == "NA" { String::new() } else { c.clone() })
                .collect()
        })
        .collect();
    let cm = Clustermap::new()
        .with_categorical_data(calls)
        .with_row_labels(variants)
        .with_col_labels(samples.clone())
        .with_category_colors([("ref", "#eeeeee"), ("het", "#4c9be8"), ("hom", "#1a3d6e")])
        .with_category_labels([
            ("ref", "Reference"),
            ("het", "Heterozygous"),
            ("hom", "Homozygous alt"),
        ])
        .with_category_order(["ref", "het", "hom"])
        .with_col_annotation(
            AnnotationTrack::categorical(annotation("clustermap_populations.tsv", 1, &samples))
                .with_label("Population"),
        )
        .with_col_labels_position(ColLabelSide::Top)
        .with_col_label_rotation(90.0)
        .with_cell_size(22.0, 18.0)
        .with_cell_gap(2.0)
        .with_legend("Genotype");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("Genotype calls");
    write("genotypes", plots, layout);
}

/// Numeric cells binned with a DiscreteColorMap: copy number per chromosome arm.
fn copy_number() {
    let arms: Vec<String> = (1..=12).map(|c| format!("chr{c}p")).collect();
    let tumours: Vec<String> = (1..=10).map(|t| format!("T{t:02}")).collect();
    let data: Vec<Vec<f64>> = (0..arms.len())
        .map(|a| {
            (0..tumours.len())
                .map(|t| [2.0, 2.0, 3.0, 1.0, 2.0, 4.0, 2.0, 0.0, 5.0][(a * a + t * 3 + a * t) % 9])
                .collect()
        })
        .collect();
    let map = DiscreteColorMap::new()
        .with_bin(0.0, "#2166ac", "0 (deep deletion)")
        .with_bin(1.0, "#92c5de", "1 (loss)")
        .with_bin(2.0, "#dddddd", "2 (neutral)")
        .with_bin(3.0, "#f4a582", "3 (gain)")
        .with_bin(4.0, "#b2182b", "4+ (amplification)");
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(arms)
        .with_col_labels(tumours)
        .with_discrete_color_map(map)
        .with_cluster_rows(false)
        .with_cell_size(28.0, 18.0)
        .with_legend("Copy number");
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("Copy number");
    write("copy_number", plots, layout);
}

/// A large annotated table: repeat-expansion calls for 48 STR disease loci (rows) across 20
/// samples (columns). No clustering, labels top and left, categorical cells, a cohort strip
/// over the columns and an inheritance strip down the rows, and a legend with a titled section
/// for each. The calls are simulated.
fn str_panel() {
    // (locus, inheritance)
    let loci = [
        ("HTT", "AD"),
        ("ATXN1", "AD"),
        ("ATXN2", "AD"),
        ("ATXN3", "AD"),
        ("CACNA1A", "AD"),
        ("ATXN7", "AD"),
        ("TBP", "AD"),
        ("ATXN8OS", "AD"),
        ("ATXN10", "AD"),
        ("PPP2R2B", "AD"),
        ("FMR1", "XL"),
        ("AFF2", "XL"),
        ("DMPK", "AD"),
        ("CNBP", "AD"),
        ("FXN", "AR"),
        ("C9orf72", "AD"),
        ("NOTCH2NLC", "AD"),
        ("RFC1", "AR"),
        ("AR", "XL"),
        ("ATN1", "AD"),
        ("PHOX2B", "AD"),
        ("PABPN1", "AD"),
        ("JPH3", "AD"),
        ("NOP56", "AD"),
        ("DAB1", "AD"),
        ("BEAN1", "AD"),
        ("TNRC6A", "AD"),
        ("RAPGEF2", "AD"),
        ("YEATS2", "AD"),
        ("SAMD12", "AD"),
        ("STARD7", "AD"),
        ("MARCHF6", "AD"),
        ("XYLT1", "AR"),
        ("GLS", "AR"),
        ("LRP12", "AD"),
        ("GIPC1", "AD"),
        ("ZFHX3", "AD"),
        ("FGF14", "AD"),
        ("THAP11", "AD"),
        ("CSTB", "AR"),
        ("COMP", "AD"),
        ("FOXL2", "AD"),
        ("HOXD13", "AD"),
        ("ARX", "XL"),
        ("SOX3", "XL"),
        ("ZIC2", "AD"),
        ("RUNX2", "AD"),
        ("TCF4", "AD"),
    ];
    let samples: Vec<String> = (1..=20).map(|i| format!("PT{i:03}")).collect();
    let cohort: Vec<&str> = (0..samples.len())
        .map(|i| match i {
            0..=7 => "Ataxia",
            8..=13 => "Neuromuscular",
            _ => "Control",
        })
        .collect();

    // Deterministic pseudo-random calls: mostly normal, some intermediate, a few expansions in
    // the affected cohorts, and the odd failed call.
    let mut seed: u64 = 0x5eed_2026;
    let mut next = || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) as f64 / (1u64 << 31) as f64
    };
    let calls: Vec<Vec<String>> = loci
        .iter()
        .map(|_| {
            (0..samples.len())
                .map(|s| {
                    let affected = cohort[s] != "Control";
                    let r = next();
                    let call = if r < 0.03 {
                        "" // no call
                    } else if affected && r < 0.07 {
                        "pathogenic"
                    } else if affected && r < 0.10 {
                        "premutation"
                    } else if r < 0.19 {
                        "intermediate"
                    } else {
                        "normal"
                    };
                    call.to_string()
                })
                .collect()
        })
        .collect();

    let cm = Clustermap::new()
        .with_categorical_data(calls)
        .with_row_labels(loci.iter().map(|(l, _)| *l))
        .with_col_labels(samples)
        .with_category_colors([
            ("normal", "#e3e8ef"),
            ("intermediate", "#f6c85f"),
            ("premutation", "#f08a3e"),
            ("pathogenic", "#c0262d"),
        ])
        .with_category_labels([
            ("normal", "Normal"),
            ("intermediate", "Intermediate"),
            ("premutation", "Premutation"),
            ("pathogenic", "Pathogenic expansion"),
        ])
        .with_category_order(["normal", "intermediate", "premutation", "pathogenic"])
        .with_na_label("No call")
        .with_legend("Repeat status")
        .with_col_annotation(
            AnnotationTrack::categorical(cohort)
                .with_label("Cohort")
                .with_category_color("Ataxia", "#4e79a7")
                .with_category_color("Neuromuscular", "#59a14f")
                .with_category_color("Control", "#bab0ac"),
        )
        .with_row_annotation(
            AnnotationTrack::categorical(loci.iter().map(|(_, m)| *m))
                .with_label("Inheritance")
                .with_category_color("AD", "#76b7b2")
                .with_category_color("AR", "#b07aa1")
                .with_category_color("XL", "#ff9da7")
                .with_category_label("AD", "Autosomal dominant")
                .with_category_label("AR", "Autosomal recessive")
                .with_category_label("XL", "X-linked")
                .with_category_order(["AD", "AR", "XL"]),
        )
        .with_col_labels_position(ColLabelSide::Top)
        .with_row_labels_position(RowLabelSide::Left)
        .with_col_label_rotation(90.0)
        .with_cell_size(18.0, 14.0)
        .with_cell_gap(1.5);
    // The same table with the strips between the labels and the cells.
    let inner = cm
        .clone()
        .with_annotation_placement(AnnotationPlacement::NextToCells);
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title("STR expansion panel");
    write("str_panel", plots, layout);
    let plots = vec![Plot::Clustermap(inner)];
    let layout = Layout::auto_from_plots(&plots).with_title("STR expansion panel");
    write("str_panel_next_to_cells", plots, layout);
}
