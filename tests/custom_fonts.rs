//! Registering fonts at runtime (feature `custom_fonts`, #134).
//!
//! The registry is process-wide and every test here registers the same DejaVu
//! Serif files (re-registering replaces), so the tests can run in any order.
#![cfg(feature = "custom_fonts")]

use kuva::fonts::{self, FontError};
use std::path::PathBuf;

fn font_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/fonts")
}

fn serif() -> PathBuf {
    font_dir().join("DejaVuSerif.ttf")
}

fn serif_bold() -> PathBuf {
    font_dir().join("DejaVuSerif-Bold.ttf")
}

/// A fresh scratch directory for one test.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kuva_fonts_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn register_file_reads_family_weight_and_style() {
    let infos = fonts::register_font_file(serif()).unwrap();
    assert_eq!(infos.len(), 1);
    let info = &infos[0];
    assert_eq!(info.family, "DejaVu Serif");
    assert_eq!(info.weight, 400);
    assert!(!info.italic && !info.monospace && !info.variable);
    assert!(!info.restricted_embedding);

    let bold = fonts::register_font_file(serif_bold()).unwrap();
    assert_eq!(bold[0].family, "DejaVu Serif", "bold joins the same family");
    assert_eq!(bold[0].weight, 700);
}

#[test]
fn family_styles_reports_missing_italics() {
    fonts::register_font_file(serif()).unwrap();
    fonts::register_font_file(serif_bold()).unwrap();
    // Case-insensitive family lookup.
    let styles = fonts::family_styles("dejavu serif").unwrap();
    assert_eq!(styles.regular.weight, 400);
    assert_eq!(styles.bold.weight, 700);
    // No italic faces registered: italic falls back to the upright faces, by weight.
    assert!(!styles.italic.italic);
    assert_eq!(styles.italic.weight, 400);
    assert_eq!(styles.bold_italic.weight, 700);
    assert_eq!(styles.missing(), ["italic", "bold italic"]);

    assert!(fonts::family_styles("No Such Family").is_none());
}

#[test]
fn re_registering_replaces_instead_of_duplicating() {
    for _ in 0..3 {
        fonts::register_font_file(serif()).unwrap();
    }
    let n = fonts::registered_fonts()
        .iter()
        .filter(|f| f.family == "DejaVu Serif" && f.weight == 400 && !f.italic)
        .count();
    assert_eq!(n, 1);
}

#[test]
fn register_bytes() {
    let bytes = std::fs::read(serif()).unwrap();
    let infos = fonts::register_font_bytes(bytes).unwrap();
    assert_eq!(infos[0].family, "DejaVu Serif");
}

#[test]
fn register_dir_finds_fonts_and_ignores_other_files() {
    // tests/data/fonts also holds LICENSE, which is not a font file.
    let report = fonts::register_fonts_dir(font_dir()).unwrap();
    let mut weights: Vec<u16> = report.registered.iter().map(|f| f.weight).collect();
    weights.sort();
    assert_eq!(weights, [400, 700]);
    assert!(report.skipped.is_empty());
}

#[test]
fn register_dir_skips_bad_files_and_recurses() {
    let dir = scratch("mixed");
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    std::fs::copy(serif(), dir.join("sub/Serif.TTF")).unwrap();
    std::fs::write(dir.join("broken.ttf"), b"not a font").unwrap();
    std::fs::write(dir.join("notes.txt"), b"ignored").unwrap();

    let report = fonts::register_fonts_dir(&dir).unwrap();
    assert_eq!(report.registered.len(), 1, "found in the subdirectory");
    assert_eq!(report.skipped.len(), 1);
    assert!(report.skipped[0].0.ends_with("broken.ttf"));
    assert!(matches!(report.skipped[0].1, FontError::Parse { .. }));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn register_dir_with_nothing_usable_is_an_error() {
    let dir = scratch("empty");
    std::fs::write(dir.join("broken.otf"), b"nope").unwrap();
    let err = fonts::register_fonts_dir(&dir).unwrap_err();
    match &err {
        FontError::NoFonts { skipped, .. } => assert_eq!(skipped.len(), 1),
        other => panic!("expected NoFonts, got {other:?}"),
    }
    assert!(err.to_string().contains("no usable fonts"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn errors_name_what_failed() {
    let err = fonts::register_font_file("/no/such/font.ttf").unwrap_err();
    assert!(matches!(err, FontError::Io { .. }));
    assert!(err.to_string().contains("/no/such/font.ttf"), "{err}");

    let err = fonts::register_font_bytes(b"definitely not a font".to_vec()).unwrap_err();
    assert!(matches!(err, FontError::Parse { .. }), "{err:?}");
    assert!(
        err.to_string().contains("not a font kuva can read"),
        "{err}"
    );

    let err = fonts::register_fonts_dir("/no/such/dir").unwrap_err();
    assert!(matches!(err, FontError::Io { .. }));
}
