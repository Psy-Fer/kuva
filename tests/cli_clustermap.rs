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

/// Run the binary with `input` on stdin.
fn run_stdin(args: &[&str], input: &str) -> (String, String, i32) {
    use std::io::Write;
    use std::process::Stdio;
    let mut child = Command::new(env!("CARGO_BIN_EXE_kuva"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn kuva");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().expect("wait for kuva");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

#[test]
fn ragged_rows_follow_the_header_width() {
    // The width comes from the header. Before, it came from the first row, so
    // here the extra column label made the renderer index past the matrix.
    let short_first = "gene\ts1\ts2\ts3\ng1\ta\ng2\tb\ta\tc\ng3\tc\tb\ta\n";
    // Categorical: a short row is padded with missing cells.
    let (svg, stderr, code) = run_stdin(&["clustermap", "--categorical"], short_first);
    assert_eq!(code, 0, "{stderr}");
    assert!(svg.contains(">s3</text>"), "the third column is drawn");
    assert!(svg.contains(">NA</text>"), "missing cells are listed as NA");
    // Numeric: a missing cell is not a number, as for an empty one.
    let numeric = short_first.replace(['a', 'b', 'c'], "1");
    let (_, stderr, code) = run_stdin(&["clustermap"], &numeric);
    assert_eq!(code, 1, "{stderr}");
    assert!(
        stderr.contains("column 's2': '' is not a number"),
        "{stderr}"
    );

    // Without a header the longest row sets the width.
    let (_, stderr, code) = run_stdin(
        &["clustermap", "--categorical", "--no-header"],
        "g1\ta\ng2\tb\ta\tc\ng3\tc\tb\ta\n",
    );
    assert_eq!(code, 0, "{stderr}");

    // A row longer than the header is an error, not silently cut.
    let long = "gene\ts1\ts2\ng1\t1\t2\ng2\t2\t3\t4\n";
    let (_, stderr, code) = run_stdin(&["clustermap"], long);
    assert_eq!(code, 1, "{stderr}");
    assert!(
        stderr.contains("data row 2 has 4 fields but the header has 3"),
        "{stderr}"
    );
}

#[test]
fn non_finite_rotation_and_cell_gap_error() {
    let input = "gene\ts1\ts2\ng1\t1\t2\ng2\t3\t4\n";
    for args in [
        &["clustermap", "--col-label-rotation", "NaN"][..],
        &["clustermap", "--col-label-rotation", "inf"][..],
        &["clustermap", "--cell-gap", "NaN"][..],
        &["clustermap", "--cell-gap=-2"][..],
    ] {
        let (_, stderr, code) = run_stdin(args, input);
        assert_eq!(code, 1, "expected error for {args:?}, got: {stderr}");
    }
}

#[test]
fn annot_spec_matches_longest_existing_file_prefix() {
    let dir = std::env::temp_dir().join(format!("kuva_clustermap_colon_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let short = dir.join("meta");
    let longer = dir.join("meta:v2");
    std::fs::write(&short, "gene\tother\ng1\tx\ng2\ty\n").unwrap();
    std::fs::write(&longer, "gene\tgroup\ng1\tA\ng2\tB\n").unwrap();

    let spec = format!("{}:group", longer.display());
    let input = "gene\ts1\ts2\ng1\t1\t2\ng2\t3\t4\n";
    let (stdout, stderr, code) = run_stdin(&["clustermap", "--row-annot", &spec], input);
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains(">group</text>"), "expected group track");
}
