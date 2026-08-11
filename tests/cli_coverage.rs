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
