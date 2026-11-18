//! CLI `kuva coverage` subcommand — genomic coverage figure (issue #2): stacked depth tracks +
//! optional variant marks + below-axis feature bands, sharing a genomic x-axis.
use std::process::Command;

fn kuva_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_kuva"))
}

fn data(name: &str) -> String {
    format!("{}/examples/data/{}", env!("CARGO_MANIFEST_DIR"), name)
}

/// Run the binary, returning (stdout, stderr, exit code). SVG goes to stdout when no `-o`.
fn run(args: &[&str]) -> (String, String, i32) {
    let out = kuva_bin().args(args).output().expect("spawn kuva");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

#[test]
fn coverage_full_renders_all_tracks() {
    let (stdout, stderr, code) = run(&[
        "coverage",
        &data("coverage_depth.tsv"),
        "--samples",
        "tumour,normal",
        "--variants",
        &data("coverage_variants.tsv"),
        "--features",
        &data("coverage_features.tsv"),
        "--locus-start",
        "1000000",
        "--locus-end",
        "1040000",
        "--x-label",
        "chr7 position",
    ]);
    assert_eq!(code, 0, "coverage failed: {stderr}");
    assert!(
        stdout.starts_with("<svg") || stdout.contains("<svg"),
        "no SVG output"
    );
    // Sample depth tracks (y-axis labels).
    assert!(stdout.contains("tumour"), "missing tumour track");
    assert!(stdout.contains("normal"), "missing normal track");
    // Variant lane legend entries.
    assert!(stdout.contains("SNV"), "missing SNV variant legend");
    assert!(stdout.contains("InDel"), "missing InDel variant legend");
    // Feature bands.
    assert!(
        stdout.contains("amp1") && stdout.contains("amp3"),
        "missing feature bands"
    );
    // Axis label + genomic Mb unit (locus is at ~1 Mb).
    assert!(stdout.contains("chr7 position"), "missing axis label");
    assert!(stdout.contains(" Mb"), "expected Mb-unit genomic ticks");
}

#[test]
fn coverage_depth_only_renders() {
    // No variants/features; default sample column (index 1 = tumour).
    let (stdout, stderr, code) = run(&["coverage", &data("coverage_depth.tsv")]);
    assert_eq!(code, 0, "coverage failed: {stderr}");
    assert!(stdout.contains("<svg"), "no SVG output");
    assert!(
        stdout.contains("tumour"),
        "default sample should be column 1 (tumour)"
    );
    // No variant/feature content requested.
    assert!(!stdout.contains("amp1"));
}

#[test]
fn coverage_multi_sample_labels_both() {
    let (stdout, stderr, code) = run(&[
        "coverage",
        &data("coverage_depth.tsv"),
        "--samples",
        "tumour,normal",
    ]);
    assert_eq!(code, 0, "coverage failed: {stderr}");
    assert!(stdout.contains("tumour") && stdout.contains("normal"));
}

#[test]
fn coverage_png_output() {
    // Exercises the raster path end-to-end (feature-gated; skipped without png).
    if cfg!(not(feature = "png")) {
        return;
    }
    let tmp = std::env::temp_dir().join("kuva_cli_coverage_test.png");
    let tmp_s = tmp.to_string_lossy().into_owned();
    let (_stdout, stderr, code) = run(&[
        "coverage",
        &data("coverage_depth.tsv"),
        "--samples",
        "tumour,normal",
        "-o",
        &tmp_s,
    ]);
    assert_eq!(code, 0, "coverage png failed: {stderr}");
    let meta = std::fs::metadata(&tmp).expect("png not written");
    assert!(meta.len() > 0, "empty png");
    let _ = std::fs::remove_file(&tmp);
}

/// Count arrow-shaped bands: a stranded interval is a closed 5-vertex path (`M` + four `L`).
fn count_arrows(svg: &str) -> usize {
    svg.split("<path d=\"M ")
        .skip(1)
        .filter(|p| p.split('"').next().unwrap().matches(" L ").count() == 4)
        .count()
}

/// Gene direction (#126): a fourth `strand` column draws stranded regions as arrows; `.` and a
/// three-column file stay plain boxes.
#[test]
fn coverage_region_strand_column_draws_arrows() {
    let (stdout, stderr, code) = run(&[
        "coverage",
        &data("coverage_depth.tsv"),
        "--regions",
        &data("coverage_genes.tsv"),
    ]);
    assert_eq!(code, 0, "coverage failed: {stderr}");
    // coverage_genes.tsv: five stranded genes (+/-) and one `.` row.
    assert_eq!(
        count_arrows(&stdout),
        5,
        "expected one arrow per stranded gene"
    );
    assert!(stdout.contains(">GENE2<") && stdout.contains(">lncRNA<"));

    // The three-column feature file still loads, with no arrows.
    let (stdout, stderr, code) = run(&[
        "coverage",
        &data("coverage_depth.tsv"),
        "--features",
        &data("coverage_features.tsv"),
    ]);
    assert_eq!(code, 0, "coverage failed: {stderr}");
    assert_eq!(count_arrows(&stdout), 0);
}

#[test]
fn coverage_bad_strand_token_errors() {
    let path = std::env::temp_dir().join(format!("kuva_bad_strand_{}.tsv", std::process::id()));
    std::fs::write(
        &path,
        "start\tend\tlabel\tstrand\n10\t20\tg1\t+\n30\t40\tg2\tsideways\n",
    )
    .unwrap();
    let (_, stderr, code) = run(&[
        "coverage",
        &data("coverage_depth.tsv"),
        "--regions",
        path.to_str().unwrap(),
    ]);
    let _ = std::fs::remove_file(&path);
    assert_ne!(code, 0);
    assert!(
        stderr.contains("row 2") && stderr.contains("invalid strand 'sideways'"),
        "unexpected error: {stderr}"
    );
}
