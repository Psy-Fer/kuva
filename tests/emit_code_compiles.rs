#![cfg(all(feature = "cli", feature = "full", feature = "emit_code"))]
//! Verifies `--emit-code` output for every wired-up Tier-1 subcommand.
//!
//! Two layers here: a structural smoke check for all wired commands, and a
//! small full-feature trybuild sample (`pie`, `surface3d`). The all-command
//! compile proof lives in `emit_code_minimal_compiles.rs`, which typechecks
//! freshly generated snippets in an isolated package without `full`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "support/emit_code_representative.rs"]
mod representative;

/// Run `kuva <args> --emit-code` and return its stdout.
fn run_emit_code(args: &[&str]) -> String {
    let exe = env!("CARGO_BIN_EXE_kuva");
    let output = Command::new(exe)
        .args(args)
        .arg("--emit-code")
        .output()
        .unwrap_or_else(|e| panic!("failed to run kuva {args:?}: {e}"));
    assert!(
        output.status.success(),
        "kuva {args:?} --emit-code failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("--emit-code output was not valid UTF-8")
}

/// The subcommands wired up for `--emit-code` so far, with a CLI invocation
/// (excluding the trailing `--emit-code`, added by `run_emit_code`) known to
/// produce output against the checked-in example data.
fn covered_subcommands() -> Vec<(&'static str, Vec<&'static str>)> {
    representative::all_cases()
}

#[test]
fn emit_code_snippets_are_structurally_sound() {
    for (name, args) in covered_subcommands() {
        let snippet = run_emit_code(&args);
        assert!(
            snippet.contains("use kuva::backend::svg::SvgBackend;"),
            "{name}: missing SvgBackend import"
        );
        assert!(
            snippet.contains("Layout::auto_from_plots(&plots)"),
            "{name}: missing Layout::auto_from_plots"
        );
        assert!(
            snippet.contains("render_multiple(plots, layout)"),
            "{name}: missing render_multiple call"
        );
        assert!(
            !snippet.to_lowercase().contains("todo!") && !snippet.contains("unimplemented!"),
            "{name}: emitted a placeholder, not real code"
        );
    }
}

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/emit_code_fixtures")
}

fn representative_subcommands() -> Vec<(&'static str, Vec<&'static str>)> {
    representative::full_feature_cases()
}

#[test]
fn emit_code_snippets_compile() {
    let dir = fixtures_dir();
    fs::create_dir_all(&dir).expect("failed to create fixtures dir");

    let cases = trybuild::TestCases::new();
    for (name, args) in representative_subcommands() {
        let snippet = run_emit_code(&args);
        let program = representative::wrap_as_program(&snippet);
        let path = dir.join(format!("{name}.rs"));
        fs::write(&path, program).unwrap_or_else(|e| panic!("failed to write {path:?}: {e}"));
        cases.pass(path);
    }
}

/// `--emit-code` validates `--signif` exactly like a real run: a malformed
/// spec or unknown format fails instead of silently emitting no bracket.
#[test]
fn emit_code_rejects_bad_signif_specs() {
    let exe = env!("CARGO_BIN_EXE_kuva");
    let cases: [(&[&str], &str); 2] = [
        (&["--signif", "a,b"], "three comma-separated fields"),
        (
            &["--signif", "#0,#1,0.1", "--signif-format", "fancy"],
            "expected stars, p, or both",
        ),
    ];
    for (extra, needle) in cases {
        let output = Command::new(exe)
            .args([
                "box",
                "examples/data/measurements.tsv",
                "--group-col",
                "group",
                "--value-col",
                "value",
            ])
            .args(extra)
            .arg("--emit-code")
            .output()
            .expect("run kuva");
        assert!(!output.status.success(), "{extra:?} should fail");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(needle), "{extra:?}: {stderr}");
    }
}
