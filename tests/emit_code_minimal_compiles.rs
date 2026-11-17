#![cfg(all(feature = "cli", feature = "emit_code"))]

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "support/emit_code_representative.rs"]
mod representative;

struct TemporaryPackage {
    path: PathBuf,
}

impl TemporaryPackage {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "kuva-emit-code-minimal-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path)
            .unwrap_or_else(|error| panic!("failed to create temporary package {path:?}: {error}"));
        Self { path }
    }
}

impl Drop for TemporaryPackage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn cargo_command() -> OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"))
}

fn run_cargo(package: &Path, args: &[&str]) -> Output {
    Command::new(cargo_command())
        .current_dir(package)
        .env("CARGO_INCREMENTAL", "0")
        .env("CARGO_TARGET_DIR", package.join("target"))
        .env("CARGO_TERM_COLOR", "never")
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to run Cargo in {package:?}: {error}"))
}

fn toml_string(value: &Path) -> String {
    let value = value
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    format!("\"{value}\"")
}

#[test]
fn all_emit_code_snippets_compile_without_full_features() {
    let package = TemporaryPackage::new();
    let source_dir = package.path.join("src/bin");
    fs::create_dir_all(&source_dir).expect("failed to create temporary source directory");

    let manifest = format!(
        r#"[package]
name = "kuva-emit-code-minimal"
version = "0.0.0"
edition = "2021"

[dependencies]
kuva = {{ path = {}, default-features = false }}
"#,
        toml_string(Path::new(env!("CARGO_MANIFEST_DIR")))
    );
    fs::write(package.path.join("Cargo.toml"), manifest)
        .expect("failed to write the temporary package manifest");

    let binary = env!("CARGO_BIN_EXE_kuva");
    for (name, args) in representative::all_cases() {
        let output = Command::new(binary)
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(args)
            .arg("--emit-code")
            .output()
            .unwrap_or_else(|error| panic!("failed to run kuva for {name}: {error}"));
        assert!(
            output.status.success(),
            "kuva --emit-code failed for {name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let snippet = String::from_utf8(output.stdout)
            .unwrap_or_else(|error| panic!("emit-code output for {name} was not UTF-8: {error}"));
        fs::write(
            source_dir.join(format!("{name}.rs")),
            representative::wrap_as_program(&snippet),
        )
        .unwrap_or_else(|error| panic!("failed to write generated {name} fixture: {error}"));
    }

    let lockfile = run_cargo(&package.path, &["generate-lockfile", "--offline"]);
    assert!(
        lockfile.status.success(),
        "failed to generate the minimal package lockfile:\n{}",
        String::from_utf8_lossy(&lockfile.stderr)
    );

    let check = run_cargo(
        &package.path,
        &["check", "--offline", "--locked", "--bins", "--quiet"],
    );
    assert!(
        check.status.success(),
        "minimal-feature generated snippets failed to compile:\n{}",
        String::from_utf8_lossy(&check.stderr)
    );
}
