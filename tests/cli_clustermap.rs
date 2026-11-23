//! CLI `kuva clustermap`: clustered / annotated / categorical heatmaps.
use std::process::Command;

fn data(name: &str) -> String {
    format!("{}/examples/data/{}", env!("CARGO_MANIFEST_DIR"), name)
}

/// Run the binary; returns (stdout, stderr, exit code). SVG goes to stdout.
fn run(args: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_kuva"))
        .args(args)
        .output()
        .expect("spawn kuva");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

#[test]
fn clustered_matrix_with_annotations_joined_by_label() {
    let samples = format!("{}:condition", data("clustermap_samples.tsv"));
    let (svg, stderr, code) = run(&[
        "clustermap",
        &data("clustermap_expr.tsv"),
        "--normalize",
        "row",
        "--col-annot",
        &samples,
        "--row-annot",
        &data("clustermap_genes.tsv"),
        "--category-colors",
        "ctrl=#999999",
        "--category-labels",
        "treat=Treated (10 uM)",
        "--legend",
        "z-score",
    ]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stderr.is_empty(), "every label matched: {stderr}");
    // Legend groups titled from the annotation files' headers, with the overrides applied.
    for t in [
        "condition",
        "pathway",
        "Treated (10 uM)",
        "DNA repair",
        "z-score",
    ] {
        assert!(svg.contains(&format!(">{t}</text>")), "missing {t}");
    }
    assert!(svg.contains("#999999"));
    // Clustered: dendrogram lines are drawn.
    assert!(svg.contains("<line"), "dendrograms");
}

#[test]
fn categorical_cells_and_legend_order() {
    let (svg, stderr, code) = run(&[
        "clustermap",
        &data("clustermap_genotypes.tsv"),
        "--categorical",
        "--category-colors",
        "ref=#eeeeee,het=#4c9be8,hom=#1a3d6e",
        "--category-labels",
        "ref=Reference,het=Heterozygous,hom=Homozygous alt",
        "--legend",
        "Genotype",
    ]);
    assert_eq!(code, 0, "{stderr}");
    // Legend in --category-colors order, then NA (the file has NA calls).
    let pos = |t: &str| {
        svg.find(&format!(">{t}</text>"))
            .unwrap_or_else(|| panic!("{t}"))
    };
    assert!(pos("Reference") < pos("Heterozygous"));
    assert!(pos("Heterozygous") < pos("Homozygous alt"));
    assert!(pos("Homozygous alt") < pos("NA"));
    assert!(!svg.contains("<line"), "no clustering for text cells");
}

#[test]
fn unmatched_annotation_labels_warn_and_show_na() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("kuva_cm_annot_{}.tsv", std::process::id()));
    // Only two of the twelve samples are listed.
    std::fs::write(&path, "sample\tgroup\nctrl_1\tA\ntreat_1\tB\n").unwrap();
    let (svg, stderr, code) = run(&[
        "clustermap",
        &data("clustermap_expr.tsv"),
        "--col-annot",
        path.to_str().unwrap(),
    ]);
    let _ = std::fs::remove_file(&path);
    assert_eq!(code, 0, "{stderr}");
    assert!(stderr.contains("10 of 12 labels not found"), "{stderr}");
    assert!(svg.contains(">NA</text>"));
}

#[test]
fn bad_input_errors() {
    // Text cells without --categorical.
    let (_, stderr, code) = run(&["clustermap", &data("clustermap_genotypes.tsv")]);
    assert_ne!(code, 0);
    assert!(stderr.contains("--categorical"), "{stderr}");
    // Unknown normalization, bad cell size, missing annotation file.
    for extra in [
        vec!["--normalize", "rows"],
        vec!["--cell-size", "big"],
        vec!["--col-annot", "/no/such/file.tsv:group"],
    ] {
        let mut args = vec!["clustermap".to_string(), data("clustermap_expr.tsv")];
        args.extend(extra.iter().map(|s| s.to_string()));
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let (_, stderr, code) = run(&args);
        assert_ne!(code, 0, "{extra:?} should fail");
        assert!(!stderr.is_empty(), "{extra:?}");
    }
}

#[test]
fn cell_size_grows_the_canvas() {
    let width = |size: &str| {
        let (svg, stderr, code) = run(&[
            "clustermap",
            &data("clustermap_expr.tsv"),
            "--cell-size",
            size,
        ]);
        assert_eq!(code, 0, "{stderr}");
        let from = svg.find("width=\"").unwrap() + 7;
        svg[from..from + svg[from..].find('"').unwrap()]
            .parse::<f64>()
            .unwrap()
    };
    // 12 columns: 10 px more per cell is 120 px wider.
    assert!((width("30x10") - width("20x10") - 120.0).abs() < 0.05);
}
