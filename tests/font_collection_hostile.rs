//! Font collections (TTC/OTC) whose face list costs far more than the file's
//! size. The face count was checked only against the file length, so a 1 MB
//! file could repeat one offset to claim 250,000 faces, each parsed, named and
//! inserted into the registry (which compares it with every face already there).
#![cfg(feature = "custom_fonts")]

#[path = "../src/render/text_metrics/test_font.rs"]
mod font_builder;
mod hostile;

use font_builder::{cmap, collection, collection_repeating, font, format12};
use kuva::fonts::{self, FontError, MAX_COLLECTION_FACES, MAX_FAMILY_NAME_CHARS};

fn small_font(family: &str) -> Vec<u8> {
    font(family, cmap(1, format12(&[(0x41, 0x5A, 1)])))
}

/// Write `bytes` to a file under the test's temporary directory.
fn temp_file(name: &str, bytes: &[u8]) -> std::path::PathBuf {
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

fn parse_reason(r: Result<Vec<fonts::FontInfo>, FontError>) -> String {
    match r {
        Err(FontError::Parse { reason, .. }) => reason,
        other => panic!("expected a parse error, got {other:?}"),
    }
}

#[test]
fn a_collection_repeating_one_face_is_rejected() {
    let bytes = collection_repeating(&small_font("Kuva Repeated"), 250_000);
    let path = temp_file("repeated.ttc", &bytes);
    let (from_bytes, from_file) = hostile::bounded(64 << 20, move || {
        (
            fonts::register_font_bytes(bytes),
            fonts::register_font_file(path),
        )
    });
    for r in [from_bytes, from_file] {
        let reason = parse_reason(r);
        assert!(reason.contains("more than the 1024"), "{reason}");
    }

    // Under the face limit, a repeated face is still refused.
    let bytes = collection_repeating(&small_font("Kuva Repeated"), 3);
    let path = temp_file("repeated3.ttc", &bytes);
    for r in [
        fonts::register_font_bytes(bytes),
        fonts::register_font_file(path),
    ] {
        let reason = parse_reason(r);
        assert!(reason.contains("same face more than once"), "{reason}");
    }
}

#[test]
fn the_face_limit_is_inclusive() {
    let face = small_font("Kuva Many Faces");
    let at_limit = collection(&vec![face.clone(); MAX_COLLECTION_FACES as usize]);
    let over = collection(&vec![face; MAX_COLLECTION_FACES as usize + 1]);
    let path = temp_file("at_limit.ttc", &at_limit);
    let infos = fonts::register_font_file(path).unwrap();
    assert_eq!(infos.len(), MAX_COLLECTION_FACES as usize);
    let infos = fonts::register_font_bytes(at_limit).unwrap();
    assert_eq!(infos.len(), MAX_COLLECTION_FACES as usize);

    let path = temp_file("over_limit.ttc", &over);
    for r in [
        fonts::register_font_bytes(over),
        fonts::register_font_file(path),
    ] {
        let reason = parse_reason(r);
        assert!(reason.contains("1025 faces"), "{reason}");
    }
}

#[test]
fn an_ordinary_collection_registers_every_face() {
    let bytes = collection(&[small_font("Kuva Coll A"), small_font("Kuva Coll B")]);
    let path = temp_file("ordinary.ttc", &bytes);
    for infos in [
        fonts::register_font_bytes(bytes).unwrap(),
        fonts::register_font_file(path).unwrap(),
    ] {
        let families: Vec<_> = infos.iter().map(|i| i.family.as_str()).collect();
        assert_eq!(families, ["Kuva Coll A", "Kuva Coll B"]);
    }
}

#[test]
fn an_overlong_family_name_is_not_used() {
    let ok = "N".repeat(MAX_FAMILY_NAME_CHARS);
    let infos = fonts::register_font_bytes(small_font(&ok)).unwrap();
    assert_eq!(infos[0].family, ok);

    let long = "N".repeat(MAX_FAMILY_NAME_CHARS + 1);
    let path = temp_file("long_name.ttf", &small_font(&long));
    for r in [
        fonts::register_font_bytes(small_font(&long)),
        fonts::register_font_file(path),
    ] {
        assert!(matches!(r, Err(FontError::NoFamilyName { .. })), "{r:?}");
    }
}
