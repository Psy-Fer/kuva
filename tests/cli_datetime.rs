/// CLI date/time X-axis support (issue #107): `--x-date-format` parses the X
/// column as a date/time instead of a plain number; `--x-date-unit`,
/// `--x-date-tick-format`, and `--x-date-tick-step` control tick generation.
/// Available on `scatter` and `line`.
use std::io::Write;
use std::process::{Command, Stdio};

fn kuva_bin() -> Command {
    let bin = env!("CARGO_BIN_EXE_kuva");
    Command::new(bin)
}

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

const MONTHLY_TSV: &str = "date\tvalue\n\
2024-01-01\t10\n\
2024-02-15\t14\n\
2024-03-10\t9\n\
2024-04-22\t18\n\
2024-06-01\t25\n";

#[test]
fn line_auto_mode_produces_month_year_ticks() {
    let (stdout, stderr, code) = run_with_stdin(
        &[
            "line",
            "--x",
            "date",
            "--y",
            "value",
            "--x-date-format",
            "%Y-%m-%d",
        ],
        MONTHLY_TSV,
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("Jan 2024") && stdout.contains("Jun 2024"),
        "auto mode should pick a months+year format for a ~5-month range: {stdout}"
    );
}

#[test]
fn scatter_explicit_unit_and_tick_format() {
    let (stdout, stderr, code) = run_with_stdin(
        &[
            "scatter",
            "--x",
            "date",
            "--y",
            "value",
            "--x-date-format",
            "%Y-%m-%d",
            "--x-date-unit",
            "months",
            "--x-date-tick-format",
            "%b %y",
        ],
        MONTHLY_TSV,
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains(">Jan 24<") || stdout.contains(">Jan 24 <"),
        "explicit tick format '%b %y' should render as e.g. 'Jan 24': {stdout}"
    );
    assert!(
        !stdout.contains("Jan 2024"),
        "explicit tick format should override the auto default: {stdout}"
    );
}

#[test]
fn tick_step_widens_tick_spacing() {
    let (default_ticks, _, code1) = run_with_stdin(
        &[
            "line",
            "--x",
            "date",
            "--y",
            "value",
            "--x-date-format",
            "%Y-%m-%d",
            "--x-date-unit",
            "months",
        ],
        MONTHLY_TSV,
    );
    let (stepped_ticks, _, code2) = run_with_stdin(
        &[
            "line",
            "--x",
            "date",
            "--y",
            "value",
            "--x-date-format",
            "%Y-%m-%d",
            "--x-date-unit",
            "months",
            "--x-date-tick-step",
            "2",
        ],
        MONTHLY_TSV,
    );
    assert_eq!(code1, 0);
    assert_eq!(code2, 0);
    let count = |svg: &str| svg.matches("text-anchor=\"middle\">").count();
    assert!(
        count(&stepped_ticks) < count(&default_ticks),
        "every-2-months should produce fewer x ticks than every-1-month \
         (stepped {} vs default {})",
        count(&stepped_ticks),
        count(&default_ticks)
    );
}

#[test]
fn unparseable_date_reports_a_clear_error() {
    let (_stdout, stderr, code) = run_with_stdin(
        &[
            "line",
            "--x",
            "date",
            "--y",
            "value",
            "--x-date-format",
            "%d/%m/%Y",
        ],
        MONTHLY_TSV,
    );
    assert_ne!(
        code, 0,
        "should fail: input dates don't match the given format"
    );
    assert!(
        stderr.contains("cannot parse") && stderr.contains("2024-01-01"),
        "error should name the offending value: {stderr}"
    );
}

#[test]
fn datetime_x_axis_works_with_color_by() {
    let tsv = "date\tvalue\tgroup\n\
2024-01-01\t10\tA\n\
2024-02-01\t12\tA\n\
2024-01-15\t20\tB\n\
2024-02-15\t22\tB\n";
    let (stdout, stderr, code) = run_with_stdin(
        &[
            "scatter",
            "--x",
            "date",
            "--y",
            "value",
            "--color-by",
            "group",
            "--x-date-format",
            "%Y-%m-%d",
        ],
        tsv,
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("<svg"),
        "should produce valid SVG: {stdout}"
    );
}

#[test]
fn datetime_x_axis_works_with_multi_y() {
    let tsv = "date\ta\tb\n\
2024-01-01\t10\t5\n\
2024-02-01\t12\t7\n\
2024-03-01\t9\t6\n";
    let (stdout, stderr, code) = run_with_stdin(
        &[
            "line",
            "--x",
            "date",
            "--y",
            "a,b",
            "--x-date-format",
            "%Y-%m-%d",
        ],
        tsv,
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("<svg"),
        "should produce valid SVG: {stdout}"
    );
}

#[test]
fn without_x_date_format_x_column_is_still_parsed_as_a_plain_number() {
    // Regression guard: --x-date-* flags must be fully inert unless
    // --x-date-format is set, so ordinary numeric-x usage is unaffected.
    let tsv = "x\ty\n1\t2\n3\t4\n5\t3\n";
    let (stdout, stderr, code) = run_with_stdin(&["scatter", "--x", "x", "--y", "y"], tsv);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("<svg"));
}

#[test]
fn dates_near_the_end_of_chronos_range_render() {
    // Year 262142 is the last full year chrono can hold. Stepping past it used
    // to panic in the tick generator.
    let tsv = "date\tvalue\n+262142-01-01\t1\n+262142-12-31\t2\n";
    for unit in ["years", "months", "weeks", "days"] {
        for cmd in ["line", "scatter"] {
            let (stdout, stderr, code) = run_with_stdin(
                &[
                    cmd,
                    "--x",
                    "date",
                    "--y",
                    "value",
                    "--x-date-format",
                    "%Y-%m-%d",
                    "--x-date-unit",
                    unit,
                ],
                tsv,
            );
            assert_eq!(code, 0, "{cmd} {unit}: {stderr}");
            assert!(stdout.contains("<svg"), "{cmd} {unit}");
        }
    }
}

#[test]
fn tick_format_chrono_cannot_render_is_an_error() {
    for fmt in ["%Q", "%Z", "%Y %"] {
        let (_, stderr, code) = run_with_stdin(
            &[
                "line",
                "--x",
                "date",
                "--y",
                "value",
                "--x-date-format",
                "%Y-%m-%d",
                "--x-date-unit",
                "months",
                "--x-date-tick-format",
                fmt,
            ],
            MONTHLY_TSV,
        );
        assert_ne!(code, 0, "{fmt}");
        assert!(
            stderr.contains("--x-date-tick-format") && stderr.contains(fmt),
            "{fmt}: {stderr}"
        );
    }
}
