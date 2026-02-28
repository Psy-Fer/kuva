/// Integration tests for the `kuva` CLI binary.
///
/// Each test spawns the binary as a child process and checks stdout/stderr/exit code.
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn kuva_bin() -> Command {
    // Use the debug build produced by `cargo build --bin kuva`.
    let bin = env!("CARGO_BIN_EXE_kuva");
    Command::new(bin)
}

/// Feed `input` to the binary's stdin and return (stdout, stderr, exit_code).
fn run_with_stdin(args: &[&str], input: &str) -> (String, String, i32) {
    let mut cmd = kuva_bin();
    cmd.args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().expect("failed to spawn kuva");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(input.as_bytes())
        .expect("write stdin");

    let out = child.wait_with_output().expect("wait");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

/// Run the binary with a file argument.
fn run_with_file(args: &[&str]) -> (String, String, i32) {
    let out = kuva_bin()
        .args(args)
        .output()
        .expect("failed to run kuva");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

/// Return the absolute path to an example data file.
fn data(filename: &str) -> String {
    format!("{}/examples/data/{}", env!("CARGO_MANIFEST_DIR"), filename)
}

// ─── tests ────────────────────────────────────────────────────────────────────

/// Piping a TSV scatter through stdin should produce valid SVG on stdout.
#[test]
fn test_scatter_stdout() {
    let tsv = "x\ty\n1\t2\n3\t4\n5\t3\n";
    let (stdout, _stderr, code) = run_with_stdin(
        &["scatter", "--title", "Scatter Plot", "--x-label", "X", "--y-label", "Y"],
        tsv,
    );
    assert_eq!(code, 0, "exit code should be 0");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

/// Writing to a file path should produce a valid SVG file.
#[test]
fn test_bar_to_file() {
    let tsv = "label\tvalue\nApples\t3\nBananas\t5\nCherries\t2\n";
    let tmp = std::env::temp_dir().join("kuva_test_bar.svg");
    let path_str = tmp.to_str().unwrap();

    // Write input to a temp TSV so we can pass it as a positional arg.
    let input_path = std::env::temp_dir().join("kuva_test_bar_input.tsv");
    fs::write(&input_path, tsv).unwrap();

    let (_, stderr, code) = run_with_file(&[
        "bar",
        input_path.to_str().unwrap(),
        "--label-col", "label",
        "--value-col", "value",
        "--title", "Fruit Counts",
        "--x-label", "Fruit",
        "--y-label", "Count",
        "-o", path_str,
    ]);

    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(tmp.exists(), "output file should have been created");
    let content = fs::read_to_string(&tmp).unwrap();
    assert!(content.starts_with("<svg"), "file should contain valid SVG");

    let _ = fs::remove_file(&tmp);
    let _ = fs::remove_file(&input_path);
}

/// Histogram with explicit bin count should produce SVG with rect elements (bars).
#[test]
fn test_histogram_bins() {
    let tsv = "1.5\n2.3\n2.7\n3.2\n3.8\n3.9\n4.0\n1.5\n2.1\n3.5\n";
    let (stdout, stderr, code) = run_with_stdin(
        &["histogram", "--bins", "5", "--title", "Value Distribution", "--x-label", "Value", "--y-label", "Count"],
        tsv,
    );
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.contains("<rect"), "histogram SVG should contain <rect elements for bars");
}

/// --color-by should produce SVG with two distinct fill colors.
#[test]
fn test_scatter_color_by() {
    let tsv = "x\ty\tgroup\n1\t2\tA\n2\t3\tA\n3\t1\tB\n4\t4\tB\n";
    let (stdout, stderr, code) = run_with_stdin(
        &["scatter", "--x", "x", "--y", "y", "--color-by", "group",
          "--title", "Groups", "--x-label", "X", "--y-label", "Y"],
        tsv,
    );
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");

    // Extract all fill="#..." values (circle points), filter out backgrounds.
    let fills: Vec<&str> = stdout
        .split("fill=\"")
        .skip(1)
        .map(|s| s.split('"').next().unwrap_or(""))
        .filter(|s| s.starts_with('#'))
        .collect();

    let unique: std::collections::HashSet<_> = fills.iter().collect();
    assert!(
        unique.len() >= 2,
        "expected at least 2 distinct fill colors for 2 groups; got: {unique:?}"
    );
}

/// Requesting PNG output without the 'png' feature should exit with code 1
/// and print a helpful error containing "--features png".
#[test]
#[cfg(not(feature = "png"))]
fn test_missing_feature_error() {
    let tsv = "x\ty\n1\t2\n3\t4\n";
    let tmp_png = std::env::temp_dir().join("kuva_test_missing.png");

    let (_, stderr, code) = run_with_stdin(
        &["scatter", "-o", tmp_png.to_str().unwrap()],
        tsv,
    );
    assert_ne!(code, 0, "should fail when png feature is missing");
    assert!(
        stderr.contains("--features png") || stderr.contains("png"),
        "error message should mention how to enable png; got: {stderr}"
    );

    let _ = fs::remove_file(&tmp_png);
}

// ─── Tier 1: SVG output tests ─────────────────────────────────────────────────

#[test]
fn test_line_svg() {
    let (stdout, stderr, code) = run_with_file(&[
        "line", &data("measurements.tsv"),
        "--x", "time", "--y", "value", "--color-by", "group",
        "--title", "Growth Curves", "--x-label", "Time", "--y-label", "Value",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

#[test]
fn test_box_svg() {
    let (stdout, stderr, code) = run_with_file(&[
        "box", &data("samples.tsv"),
        "--group-col", "group", "--value-col", "expression",
        "--title", "Expression by Group", "--x-label", "Group", "--y-label", "Expression",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

#[test]
fn test_violin_svg() {
    let (stdout, stderr, code) = run_with_file(&[
        "violin", &data("samples.tsv"),
        "--group-col", "group", "--value-col", "expression",
        "--title", "Expression Distribution", "--x-label", "Group", "--y-label", "Expression",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

#[test]
fn test_pie_svg() {
    let (stdout, stderr, code) = run_with_file(&[
        "pie", &data("pie.tsv"),
        "--label-col", "feature", "--value-col", "percentage",
        "--title", "Genome Composition",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

#[test]
fn test_strip_svg() {
    let (stdout, stderr, code) = run_with_file(&[
        "strip", &data("samples.tsv"),
        "--group-col", "group", "--value-col", "expression",
        "--title", "Expression Spread", "--x-label", "Group", "--y-label", "Expression",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

#[test]
fn test_waterfall_svg() {
    let (stdout, stderr, code) = run_with_file(&[
        "waterfall", &data("waterfall.tsv"),
        "--label-col", "process", "--value-col", "log2fc",
        "--title", "Log2 Fold Change", "--x-label", "Process", "--y-label", "log2FC",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

#[test]
fn test_stacked_area_svg() {
    let (stdout, stderr, code) = run_with_file(&[
        "stacked-area", &data("stacked_area.tsv"),
        "--x-col", "week", "--group-col", "species", "--y-col", "abundance",
        "--title", "Species Abundance", "--x-label", "Week", "--y-label", "Abundance",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

#[test]
fn test_volcano_svg() {
    let (stdout, stderr, code) = run_with_file(&[
        "volcano", &data("volcano.tsv"),
        "--name-col", "gene", "--x-col", "log2fc", "--y-col", "pvalue",
        "--title", "Differential Expression", "--x-label", "log2 Fold Change", "--y-label=-log10(p-value)",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

#[test]
fn test_manhattan_svg() {
    let (stdout, stderr, code) = run_with_file(&[
        "manhattan", &data("gene_stats.tsv"),
        "--chr-col", "chr", "--pvalue-col", "pvalue",
        "--title", "GWAS Results", "--x-label", "Chromosome", "--y-label=-log10(p-value)",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

#[test]
fn test_candlestick_svg() {
    let (stdout, stderr, code) = run_with_file(&[
        "candlestick", &data("candlestick.tsv"),
        "--label-col", "date",
        "--open-col", "open", "--high-col", "high",
        "--low-col", "low", "--close-col", "close",
        "--title", "Stock Price", "--x-label", "Date", "--y-label", "Price (USD)",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

#[test]
fn test_heatmap_svg() {
    let (stdout, stderr, code) = run_with_file(&[
        "heatmap", &data("heatmap.tsv"),
        "--title", "Gene Expression Heatmap", "--x-label", "Sample", "--y-label", "Gene",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

#[test]
fn test_hist2d_svg() {
    let (stdout, stderr, code) = run_with_file(&[
        "hist2d", &data("hist2d.tsv"), "--x", "x", "--y", "y",
        "--title", "2D Density", "--x-label", "X", "--y-label", "Y",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

#[test]
fn test_contour_svg() {
    let (stdout, stderr, code) = run_with_file(&[
        "contour", &data("contour.tsv"), "--x", "x", "--y", "y", "--z", "density",
        "--title", "Density Contour", "--x-label", "X", "--y-label", "Y",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

#[test]
fn test_dot_svg() {
    let (stdout, stderr, code) = run_with_file(&[
        "dot", &data("dot.tsv"),
        "--x-col", "pathway", "--y-col", "cell_type",
        "--size-col", "pct_expressed", "--color-col", "mean_expr",
        "--title", "Gene Expression by Cell Type", "--x-label", "Pathway", "--y-label", "Cell Type",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

#[test]
fn test_upset_svg() {
    let (stdout, stderr, code) = run_with_file(&[
        "upset", &data("upset.tsv"), "--title", "Set Intersections",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

#[test]
fn test_chord_svg() {
    let (stdout, stderr, code) = run_with_file(&[
        "chord", &data("chord.tsv"), "--title", "Cell Type Co-occurrence",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

#[test]
fn test_sankey_svg() {
    let (stdout, stderr, code) = run_with_file(&[
        "sankey", &data("sankey.tsv"),
        "--source-col", "source", "--target-col", "target", "--value-col", "value",
        "--title", "Flow Diagram",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

#[test]
fn test_phylo_svg() {
    let (stdout, stderr, code) = run_with_file(&[
        "phylo", &data("phylo.tsv"),
        "--parent-col", "parent", "--child-col", "child", "--length-col", "length",
        "--title", "Phylogenetic Tree",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

#[test]
fn test_synteny_svg() {
    let (stdout, stderr, code) = run_with_file(&[
        "synteny", &data("synteny_seqs.tsv"),
        "--blocks-file", &data("synteny_blocks.tsv"),
        "--title", "Synteny Map",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

// ─── Tier 2: Content verification tests ──────────────────────────────────────

#[test]
fn test_scatter_has_circles() {
    let tsv = "x\ty\n1\t2\n3\t4\n5\t3\n";
    let (stdout, _stderr, code) = run_with_stdin(
        &["scatter", "--title", "Scatter", "--x-label", "X", "--y-label", "Y"],
        tsv,
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("<circle"), "scatter SVG should contain <circle elements");
}

#[test]
fn test_line_has_path() {
    let (stdout, stderr, code) = run_with_file(&[
        "line", &data("measurements.tsv"),
        "--x", "time", "--y", "value", "--color-by", "group",
        "--title", "Growth Curves", "--x-label", "Time", "--y-label", "Value",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.contains("<path"), "line SVG should contain <path elements");
}

#[test]
fn test_bar_has_rects_and_labels() {
    let (stdout, stderr, code) = run_with_file(&[
        "bar", &data("bar.tsv"),
        "--label-col", "category", "--value-col", "count",
        "--title", "Category Counts", "--x-label", "Category", "--y-label", "Count",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.contains("<rect"), "bar SVG should contain <rect elements");
    assert!(stdout.contains("DNA repair"), "bar SVG should contain category label 'DNA repair'");
}

#[test]
fn test_heatmap_has_grid_cells() {
    let (stdout, stderr, code) = run_with_file(&[
        "heatmap", &data("heatmap.tsv"),
        "--title", "Expression Heatmap", "--x-label", "Sample", "--y-label", "Gene",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    let rect_count = stdout.matches("<rect").count();
    assert!(
        rect_count >= 30,
        "heatmap SVG should have at least 30 <rect elements; got {rect_count}"
    );
}

#[test]
fn test_volcano_threshold_line() {
    let (stdout, stderr, code) = run_with_file(&[
        "volcano", &data("volcano.tsv"),
        "--name-col", "gene", "--x-col", "log2fc", "--y-col", "pvalue",
        "--title", "Differential Expression", "--x-label", "log2 Fold Change", "--y-label=-log10(p-value)",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(
        stdout.contains("stroke-dasharray"),
        "volcano SVG should contain dashed threshold lines"
    );
}

#[test]
fn test_manhattan_chromosome_labels() {
    let (stdout, stderr, code) = run_with_file(&[
        "manhattan", &data("gene_stats.tsv"),
        "--chr-col", "chr", "--pvalue-col", "pvalue",
        "--title", "GWAS Results", "--x-label", "Chromosome", "--y-label=-log10(p-value)",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(
        stdout.contains("chr1"),
        "manhattan SVG should contain chromosome label text 'chr1'"
    );
}

#[test]
fn test_pie_has_paths_and_legend() {
    let (stdout, stderr, code) = run_with_file(&[
        "pie", &data("pie.tsv"),
        "--label-col", "feature", "--value-col", "percentage", "--legend",
        "--title", "Genome Composition",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.contains("<path"), "pie SVG should contain <path elements");
    assert!(stdout.contains("Intron"), "pie SVG should contain legend entry 'Intron'");
}

// ─── Tier 3: Terminal output tests ───────────────────────────────────────────
//
// All tests pass --term-width and --term-height explicitly so the running
// terminal's dimensions never influence the output.

fn is_terminal_output(s: &str) -> bool {
    // Terminal output contains ANSI escape sequences; it must NOT look like SVG.
    !s.trim_start().starts_with("<svg")
        && !s.is_empty()
}

#[test]
fn test_scatter_terminal() {
    let tsv = "x\ty\n1\t2\n3\t4\n5\t3\n2\t5\n4\t1\n";
    let (stdout, stderr, code) = run_with_stdin(
        &["scatter", "--terminal", "--term-width", "80", "--term-height", "24",
          "--x-label", "X", "--y-label", "Y"],
        tsv,
    );
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(is_terminal_output(&stdout), "expected terminal output, not SVG");
    assert!(
        stdout.chars().any(|c| ('\u{2800}'..='\u{28FF}').contains(&c)),
        "scatter terminal output should contain braille dots"
    );
}

#[test]
fn test_bar_terminal() {
    let tsv = "label\tvalue\nA\t10\nB\t20\nC\t15\n";
    let (stdout, stderr, code) = run_with_stdin(
        &["bar", "--label-col", "label", "--value-col", "value",
          "--terminal", "--term-width", "80", "--term-height", "24"],
        tsv,
    );
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(is_terminal_output(&stdout), "expected terminal output, not SVG");
    assert!(stdout.contains('█'), "bar terminal output should contain block characters");
}

#[test]
fn test_histogram_terminal() {
    let tsv = "1.5\n2.3\n2.7\n3.2\n3.8\n3.9\n4.0\n1.5\n2.1\n3.5\n";
    let (stdout, stderr, code) = run_with_stdin(
        &["histogram", "--bins", "5",
          "--terminal", "--term-width", "80", "--term-height", "24"],
        tsv,
    );
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(is_terminal_output(&stdout), "expected terminal output, not SVG");
    assert!(stdout.contains('█'), "histogram terminal output should contain block characters");
}

#[test]
fn test_line_terminal() {
    let (stdout, stderr, code) = run_with_file(&[
        "line", &data("measurements.tsv"),
        "--x", "time", "--y", "value", "--color-by", "group",
        "--terminal", "--term-width", "80", "--term-height", "24",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(is_terminal_output(&stdout), "expected terminal output, not SVG");
}

#[test]
fn test_pie_terminal() {
    let (stdout, stderr, code) = run_with_file(&[
        "pie", &data("pie.tsv"),
        "--label-col", "feature", "--value-col", "percentage",
        "--terminal", "--term-width", "80", "--term-height", "24",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(is_terminal_output(&stdout), "expected terminal output, not SVG");
    assert!(
        stdout.chars().any(|c| ('\u{2800}'..='\u{28FF}').contains(&c)),
        "pie terminal output should contain braille arcs"
    );
}

#[test]
fn test_box_terminal() {
    let (stdout, stderr, code) = run_with_file(&[
        "box", &data("samples.tsv"),
        "--group-col", "group", "--value-col", "expression",
        "--terminal", "--term-width", "80", "--term-height", "24",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(is_terminal_output(&stdout), "expected terminal output, not SVG");
}

#[test]
fn test_violin_terminal() {
    let (stdout, stderr, code) = run_with_file(&[
        "violin", &data("samples.tsv"),
        "--group-col", "group", "--value-col", "expression",
        "--terminal", "--term-width", "80", "--term-height", "24",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(is_terminal_output(&stdout), "expected terminal output, not SVG");
}

#[test]
fn test_volcano_terminal() {
    let (stdout, stderr, code) = run_with_file(&[
        "volcano", &data("volcano.tsv"),
        "--name-col", "gene", "--x-col", "log2fc", "--y-col", "pvalue",
        "--terminal", "--term-width", "80", "--term-height", "24",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(is_terminal_output(&stdout), "expected terminal output, not SVG");
}

#[test]
fn test_heatmap_terminal() {
    let (stdout, stderr, code) = run_with_file(&[
        "heatmap", &data("heatmap.tsv"),
        "--terminal", "--term-width", "80", "--term-height", "24",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(is_terminal_output(&stdout), "expected terminal output, not SVG");
    assert!(stdout.contains('█'), "heatmap terminal output should contain block characters");
}

#[test]
fn test_strip_terminal() {
    let (stdout, stderr, code) = run_with_file(&[
        "strip", &data("samples.tsv"),
        "--group-col", "group", "--value-col", "expression",
        "--terminal", "--term-width", "80", "--term-height", "24",
    ]);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(is_terminal_output(&stdout), "expected terminal output, not SVG");
}

// ─── Tier 4: Error-path tests ─────────────────────────────────────────────────

#[test]
fn test_bad_column_name() {
    let tsv = "x\ty\n1\t2\n3\t4\n";
    let (_, stderr, code) = run_with_stdin(&["scatter", "--x", "nonexistent_col"], tsv);
    assert_ne!(code, 0, "should fail when column name does not exist");
    assert!(
        stderr.contains("nonexistent_col"),
        "error message should mention the bad column name; got: {stderr}"
    );
}

#[test]
fn test_empty_stdin() {
    let (_, stderr, code) = run_with_stdin(&["scatter"], "");
    assert_ne!(code, 0, "should fail on empty input");
    assert!(!stderr.is_empty(), "stderr should be non-empty on empty input");
}

#[test]
#[cfg(not(feature = "pdf"))]
fn test_missing_feature_pdf() {
    let tsv = "x\ty\n1\t2\n3\t4\n";
    let tmp_pdf = std::env::temp_dir().join("kuva_test_missing.pdf");

    let (_, stderr, code) = run_with_stdin(
        &["scatter", "-o", tmp_pdf.to_str().unwrap()],
        tsv,
    );
    assert_ne!(code, 0, "should fail when pdf feature is missing");
    assert!(
        stderr.contains("pdf"),
        "error message should mention pdf; got: {stderr}"
    );

    let _ = fs::remove_file(&tmp_pdf);
}

#[test]
fn test_unknown_subcommand() {
    let (_, _, code) = run_with_file(&["notaplot"]);
    assert_ne!(code, 0, "unknown subcommand should exit with non-zero code");
}
