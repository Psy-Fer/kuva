//! CLI missing-value handling (issue #108): `--na-strategy` / `--na-values` on the continuous
//! subcommands (scatter, line, histogram). Empty / NA-token cells no longer abort the command.
use std::process::Command;

fn kuva_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_kuva"))
}
fn data(name: &str) -> String {
    format!("{}/examples/data/{}", env!("CARGO_MANIFEST_DIR"), name)
}
fn run(args: &[&str]) -> (String, String, i32) {
    let out = kuva_bin().args(args).output().expect("spawn kuva");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

// examples/data/missing.tsv has empty cells and an "NA" token in the y column, plus an empty x.

#[test]
fn scatter_default_drops_missing_and_notes_it() {
    let (stdout, stderr, code) = run(&["scatter", &data("missing.tsv"), "--x", "x", "--y", "y"]);
    assert_eq!(code, 0, "should not abort on missing values: {stderr}");
    assert!(stdout.contains("<svg"), "no SVG output");
    assert!(
        stderr.contains("dropped"),
        "expected a stderr note about dropped rows"
    );
}

#[test]
fn scatter_zero_keeps_rows() {
    let (stdout, _stderr, code) = run(&[
        "scatter",
        &data("missing.tsv"),
        "--x",
        "x",
        "--y",
        "y",
        "--na-strategy",
        "zero",
    ]);
    assert_eq!(code, 0);
    assert!(stdout.contains("<svg"));
}

#[test]
fn scatter_error_strategy_fails() {
    let (_stdout, stderr, code) = run(&[
        "scatter",
        &data("missing.tsv"),
        "--x",
        "x",
        "--y",
        "y",
        "--na-strategy",
        "error",
    ]);
    assert_ne!(code, 0, "error strategy should fail on a missing value");
    assert!(stderr.to_lowercase().contains("missing value"));
}

#[test]
fn histogram_and_line_handle_missing() {
    let (h_out, _e, h_code) = run(&["histogram", &data("missing.tsv"), "--value-col", "y"]);
    assert_eq!(h_code, 0);
    assert!(h_out.contains("<svg"));

    let (l_out, _e, l_code) = run(&["line", &data("missing.tsv"), "--x", "x", "--y", "y"]);
    assert_eq!(l_code, 0);
    assert!(l_out.contains("<svg"));
}

#[test]
fn bar_and_parallel_handle_missing() {
    let (b_out, _e, b_code) = run(&[
        "bar",
        &data("missing.tsv"),
        "--label-col",
        "x",
        "--value-col",
        "y",
    ]);
    assert_eq!(b_code, 0);
    assert!(b_out.contains("<svg"));

    let (p_out, _e, p_code) = run(&["parallel", &data("missing.tsv"), "--value-cols", "x", "y"]);
    assert_eq!(p_code, 0);
    assert!(p_out.contains("<svg"));
}

/// A subcommand that reads via the strict `col_f64` (not yet NA-wired) fails fast on a non-finite
/// value instead of silently corrupting the plot.
#[test]
fn non_na_wired_subcommand_rejects_inf() {
    let (_o, stderr, code) = run(&["box", &data("inf_data.tsv"), "--value-col", "y"]);
    assert_ne!(code, 0);
    assert!(stderr.to_lowercase().contains("not finite"));
}

#[test]
fn clamp_caps_infinities_instead_of_dropping() {
    // inf_data.tsv has inf / -inf / 500 rows. Without clamp they'd be dropped (inf) or plotted
    // (500); with --clamp-min/--clamp-max they are capped to the bounds and all rows are kept.
    let (stdout, stderr, code) = run(&[
        "scatter",
        &data("inf_data.tsv"),
        "--x",
        "x",
        "--y",
        "y",
        "--clamp-min",
        "0",
        "--clamp-max",
        "300",
    ]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("<svg"));
    // With clamping, no rows are dropped (inf/-inf became 300/0).
    assert!(
        !stderr.contains("dropped"),
        "clamp should keep the inf rows: {stderr}"
    );
}

#[test]
fn clamp_min_must_not_exceed_max() {
    let (_o, stderr, code) = run(&[
        "scatter",
        &data("inf_data.tsv"),
        "--x",
        "x",
        "--y",
        "y",
        "--clamp-min",
        "300",
        "--clamp-max",
        "0",
    ]);
    assert_ne!(code, 0);
    assert!(stderr.contains("clamp-min"));
}

#[test]
fn custom_na_values_token() {
    // Treat "-999" as missing via --na-values; row with -999 is dropped.
    let (stdout, stderr, code) = run(&[
        "scatter",
        &data("missing.tsv"),
        "--x",
        "x",
        "--y",
        "y",
        "--na-values",
        "-999,NA",
    ]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("<svg"));
}
