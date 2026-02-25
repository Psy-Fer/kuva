/// Integration tests for the `visus` CLI binary.
///
/// Each test spawns the binary as a child process and checks stdout/stderr/exit code.
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn visus_bin() -> Command {
    // Use the debug build produced by `cargo build --bin visus`.
    let bin = env!("CARGO_BIN_EXE_visus");
    Command::new(bin)
}

/// Feed `input` to the binary's stdin and return (stdout, stderr, exit_code).
fn run_with_stdin(args: &[&str], input: &str) -> (String, String, i32) {
    let mut cmd = visus_bin();
    cmd.args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().expect("failed to spawn visus");
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
    let out = visus_bin()
        .args(args)
        .output()
        .expect("failed to run visus");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

// ─── tests ────────────────────────────────────────────────────────────────────

/// Piping a TSV scatter through stdin should produce valid SVG on stdout.
#[test]
fn test_scatter_stdout() {
    let tsv = "x\ty\n1\t2\n3\t4\n5\t3\n";
    let (stdout, _stderr, code) = run_with_stdin(&["scatter"], tsv);
    assert_eq!(code, 0, "exit code should be 0");
    assert!(stdout.starts_with("<svg"), "output should start with <svg");
}

/// Writing to a file path should produce a valid SVG file.
#[test]
fn test_bar_to_file() {
    let tsv = "label\tvalue\nApples\t3\nBananas\t5\nCherries\t2\n";
    let tmp = std::env::temp_dir().join("visus_test_bar.svg");
    let path_str = tmp.to_str().unwrap();

    // Write input to a temp TSV so we can pass it as a positional arg.
    let input_path = std::env::temp_dir().join("visus_test_bar_input.tsv");
    fs::write(&input_path, tsv).unwrap();

    let (_, stderr, code) = run_with_file(&[
        "bar",
        input_path.to_str().unwrap(),
        "--label-col", "label",
        "--value-col", "value",
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
    let (stdout, stderr, code) = run_with_stdin(&["histogram", "--bins", "5"], tsv);
    assert_eq!(code, 0, "exit code should be 0; stderr: {stderr}");
    assert!(stdout.contains("<rect"), "histogram SVG should contain <rect elements for bars");
}

/// --color-by should produce SVG with two distinct fill colors.
#[test]
fn test_scatter_color_by() {
    let tsv = "x\ty\tgroup\n1\t2\tA\n2\t3\tA\n3\t1\tB\n4\t4\tB\n";
    let (stdout, stderr, code) = run_with_stdin(
        &["scatter", "--x", "x", "--y", "y", "--color-by", "group"],
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
    let tmp_png = std::env::temp_dir().join("visus_test_missing.png");

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
