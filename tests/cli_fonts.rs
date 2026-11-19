//! CLI font flags (#134): --font, --tick-font, --mono-font, --font-file, --font-dir.
#![cfg(all(feature = "cli", feature = "custom_fonts"))]
use std::process::Command;

fn root(rel: &str) -> String {
    format!("{}/{rel}", env!("CARGO_MANIFEST_DIR"))
}

/// Run `kuva box` on the example measurements; returns (stdout, stderr, exit code).
fn run_box(extra: &[&str]) -> (String, String, i32) {
    let data = root("examples/data/measurements.tsv");
    let out = Command::new(env!("CARGO_BIN_EXE_kuva"))
        .args([
            "box",
            data.as_str(),
            "--group-col",
            "group",
            "--value-col",
            "value",
            "--title",
            "Measurements",
        ])
        .args(extra)
        .output()
        .expect("spawn kuva");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

fn svg_root(svg: &str) -> &str {
    let start = svg.find("<svg").unwrap();
    &svg[start..start + svg[start..].find('>').unwrap()]
}

#[test]
fn font_file_alone_selects_its_family_and_warns_about_missing_styles() {
    let serif = root("tests/data/fonts/DejaVuSerif.ttf");
    let (svg, stderr, code) = run_box(&["--font-file", &serif]);
    assert_eq!(code, 0, "{stderr}");
    let root = svg_root(&svg);
    assert!(root.contains(r#"font-family="DejaVu Serif""#), "{root}");
    assert!(root.contains("font-synthesis:none"), "{root}");
    // One line naming every missing style.
    assert!(
        stderr.contains("DejaVu Serif has no bold, italic or bold italic face"),
        "{stderr}"
    );
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
}

#[test]
fn font_dir_loads_every_face_and_embeds_only_what_is_drawn() {
    let (svg, stderr, code) = run_box(&["--font-dir", &root("tests/data/fonts"), "--embed-font"]);
    assert_eq!(code, 0, "{stderr}");
    // Bold is loaded now; only the italics are missing.
    assert!(stderr.contains("no italic or bold italic face"), "{stderr}");
    // The box plot draws no bold text, so only the regular face is embedded.
    assert_eq!(
        svg.matches("@font-face{font-family:'DejaVu Serif'").count(),
        1
    );
}

#[test]
fn explicit_font_wins_and_unloaded_fonts_warn() {
    let serif = root("tests/data/fonts/DejaVuSerif.ttf");
    let (svg, stderr, code) = run_box(&["--font-file", &serif, "--font", "Comic Neue, sans-serif"]);
    assert_eq!(code, 0, "{stderr}");
    assert!(svg_root(&svg).contains(r#"font-family="Comic Neue, sans-serif""#));
    assert!(
        stderr.contains("font 'Comic Neue' is not loaded"),
        "{stderr}"
    );
    // Generic families and the bundled DejaVu Sans never warn.
    for family in ["serif", "DejaVu Sans, sans-serif"] {
        let (_, stderr, code) = run_box(&["--font", family]);
        assert_eq!(code, 0);
        assert!(stderr.is_empty(), "{family}: {stderr}");
    }
}

#[test]
fn tick_font_reaches_the_svg() {
    let (svg, stderr, code) = run_box(&["--tick-font", "DejaVu Sans Mono"]);
    assert_eq!(code, 0, "{stderr}");
    assert!(
        svg.contains(r#"<g font-family="DejaVu Sans Mono">"#),
        "tick group"
    );
}

#[test]
fn unreadable_fonts_are_errors() {
    let (_, stderr, code) = run_box(&["--font-file", "/no/such/font.ttf"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("/no/such/font.ttf"), "{stderr}");
    let empty = std::env::temp_dir().join(format!("kuva_cli_fonts_{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    let (_, stderr, code) = run_box(&["--font-dir", empty.to_str().unwrap()]);
    let _ = std::fs::remove_dir_all(&empty);
    assert_ne!(code, 0);
    assert!(stderr.contains("no usable fonts"), "{stderr}");
}

#[test]
fn coverage_takes_the_font_flags() {
    let serif = root("tests/data/fonts/DejaVuSerif.ttf");
    let out = Command::new(env!("CARGO_BIN_EXE_kuva"))
        .args([
            "coverage",
            &root("examples/data/covar_depth.tsv"),
            "--font-file",
            &serif,
        ])
        .output()
        .expect("spawn kuva");
    let svg = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(svg_root(&svg).contains(r#"font-family="DejaVu Serif""#));
}
