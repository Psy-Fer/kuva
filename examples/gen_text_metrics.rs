//! Regenerates `src/render/text_metrics/data.rs` from the bundled DejaVu Sans
//! TTFs. Run after changing the fonts:
//!
//! ```sh
//! cargo run --example gen_text_metrics
//! ```
//!
//! The output is, per font style (Regular, Italic, Bold, BoldItalic): a
//! run-length-encoded table of horizontal advance widths over the Basic
//! Multilingual Plane, plus the per-style vertical metrics (ascent, descent,
//! line-gap, cap-height, x-height) as `VMETRICS_*` constants. Each is measured
//! from its own face so nothing relies on one style's metrics matching
//! another's — swap the fonts and regenerate. No
//! bold-oblique face is bundled, so BoldItalic falls back to the Bold face until
//! one is added. The `text_metrics::tests::committed_table_matches_font` test
//! fails if the committed table drifts from the fonts, so regenerate when it does.

use std::fmt::Write as _;

use ttf_parser::Face;

const ASSETS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts");
const OUT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/render/text_metrics/data.rs"
);

/// Dense BMP advance table in font units; 0 marks a codepoint with no glyph.
/// `(advance RLE bytes, (codepoint, advance) pairs, units-per-em, vertical metrics)`
/// as extracted from one font face.
type FaceMetrics = (Vec<u8>, Vec<(u16, u16)>, u16, [i16; 5]);

/// The same metrics tagged with the const name they will be emitted under.
type NamedFaceMetrics<'a> = (&'a str, Vec<(u16, u16)>, u16, [i16; 5]);

fn dense(face: &Face) -> Vec<u16> {
    let mut advances = vec![0u16; 0x10000];
    for cp in 0u32..=0xFFFF {
        if let Some(c) = char::from_u32(cp) {
            if let Some(gid) = face.glyph_index(c) {
                if let Some(advance) = face.glyph_hor_advance(gid) {
                    advances[cp as usize] = advance;
                }
            }
        }
    }
    advances
}

/// Per-face vertical metrics in font units: `[ascent, descent, line_gap,
/// cap_height, x_height]`, where `descent` is stored as a positive depth below
/// the baseline. hhea ascender/descender/lineGap are used (they match the
/// rendered line box); cap/x-height are measured from the 'H'/'x' glyph outlines,
/// falling back to OS/2 sCapHeight/sxHeight and then a derived value only when the
/// glyph is absent (DejaVu's OS/2 omits both, so the outline path is what runs).
fn vmetrics(face: &Face) -> [i16; 5] {
    let ascent = face.ascender();
    let descent = -face.descender(); // store as a positive depth below baseline
    let line_gap = face.line_gap();
    // Measure cap-height and x-height from the actual outlines of 'H' and 'x'
    // (the top of their bounding boxes). This is robust to faces whose OS/2 table
    // omits sCapHeight/sxHeight — DejaVu does, so `capital_height()`/`x_height()`
    // return None. Fall back to OS/2, then a derived value, only if the glyph is
    // missing entirely.
    let glyph_top = |ch: char| {
        face.glyph_index(ch)
            .and_then(|g| face.glyph_bounding_box(g))
            .map(|bb| bb.y_max)
    };
    let cap_height = glyph_top('H')
        .or_else(|| face.capital_height())
        .unwrap_or(ascent);
    let x_height = glyph_top('x')
        .or_else(|| face.x_height())
        .unwrap_or((f32::from(cap_height) * 0.72) as i16);
    [ascent, descent, line_gap, cap_height, x_height]
}

/// Run-length-encodes a dense table into `(advance, run_length)` pairs.
fn rle(advances: &[u16]) -> Vec<(u16, u16)> {
    let mut runs = vec![];
    let mut i = 0usize;
    while i < advances.len() {
        let value = advances[i];
        let start = i;
        while i < advances.len() && advances[i] == value && (i - start) < u16::MAX as usize {
            i += 1;
        }
        runs.push((value, (i - start) as u16));
    }
    runs
}

fn emit_rle(buf: &mut String, name: &str, runs: &[(u16, u16)]) {
    // Keep the compact 8-pairs-per-line packing; without this rustfmt would
    // explode the array to one pair per line (thousands of lines).
    buf.push_str("#[rustfmt::skip]\n");
    writeln!(buf, "pub(super) const {name}: &[(u16, u16)] = &[").unwrap();
    for chunk in runs.chunks(8) {
        buf.push_str("    ");
        for (value, len) in chunk {
            write!(buf, "({value},{len}),").unwrap();
        }
        buf.push('\n');
    }
    buf.push_str("];\n");
}

/// Reads a face, falling back to `fallback` (relative to ASSETS) when the
/// primary file is absent. Returns the owned font bytes, the RLE advance table,
/// the units-per-em, and the per-face vertical metrics.
fn read_rle(primary: &str, fallback: Option<&str>) -> FaceMetrics {
    let path = format!("{ASSETS}/{primary}");
    let chosen = if std::path::Path::new(&path).exists() {
        path
    } else if let Some(fb) = fallback {
        eprintln!("note: {primary} not bundled; sourcing from {fb}");
        format!("{ASSETS}/{fb}")
    } else {
        path
    };
    let bytes = std::fs::read(&chosen).unwrap_or_else(|_| panic!("read {chosen}"));
    let face = Face::parse(&bytes, 0).expect("parse face");
    let upem = face.units_per_em();
    let runs = rle(&dense(&face));
    let vm = vmetrics(&face); // end the borrow of `bytes` before it moves into the tuple
    (bytes, runs, upem, vm)
}

/// Emits a `VMetrics` const literal (referencing the struct defined in `mod.rs`).
fn emit_vmetrics(buf: &mut String, name: &str, vm: &[i16; 5]) {
    let [ascent, descent, line_gap, cap_height, x_height] = *vm;
    writeln!(
        buf,
        "pub(super) const {name}: super::VMetrics = super::VMetrics {{ \
         ascent: {ascent}, descent: {descent}, line_gap: {line_gap}, \
         cap_height: {cap_height}, x_height: {x_height} }};"
    )
    .unwrap();
}

fn main() {
    // (const name, primary file, fallback file). Each style is measured from its
    // own face so the API never relies on one style's metrics matching another's.
    let faces = [
        ("ADVANCE_RLE_REGULAR", "DejaVuSans.ttf", None),
        ("ADVANCE_RLE_ITALIC", "DejaVuSans-Oblique.ttf", None),
        ("ADVANCE_RLE_BOLD", "DejaVuSans-Bold.ttf", None),
        (
            "ADVANCE_RLE_BOLD_ITALIC",
            "DejaVuSans-BoldOblique.ttf",
            Some("DejaVuSans-Bold.ttf"),
        ),
    ];

    let tables: Vec<NamedFaceMetrics> = faces
        .iter()
        .map(|(name, primary, fallback)| {
            let (_bytes, runs, upem, vm) = read_rle(primary, *fallback);
            (*name, runs, upem, vm)
        })
        .collect();
    let upem = tables[0].2;
    assert!(
        tables.iter().all(|t| t.2 == upem),
        "faces must share units_per_em"
    );

    // Mean advance over printable ASCII (Regular face), the representative value
    // for inverse width-to-character-count estimates.
    let reg_bytes = std::fs::read(format!("{ASSETS}/DejaVuSans.ttf")).expect("read Regular");
    let reg = Face::parse(&reg_bytes, 0).expect("parse Regular");
    let ascii: Vec<f64> = (0x20u32..=0x7E)
        .filter_map(|cp| {
            let c = char::from_u32(cp).unwrap();
            reg.glyph_index(c)
                .and_then(|g| reg.glyph_hor_advance(g))
                .map(|a| a as f64 / upem as f64)
        })
        .collect();
    let mean_em = ascii.iter().sum::<f64>() / ascii.len() as f64;

    let mut buf = String::new();
    buf.push_str(
        "//! GENERATED FILE — do not edit by hand.\n\
         //!\n\
         //! Regenerate with `cargo run --example gen_text_metrics` after changing the\n\
         //! bundled fonts. `text_metrics::tests::committed_table_matches_font` guards drift.\n\
         //!\n\
         //! Run-length-encoded DejaVu Sans horizontal advance widths over the Basic\n\
         //! Multilingual Plane, in font units, one table per style. Each `(advance, len)`\n\
         //! pair covers `len` consecutive codepoints starting where the previous pair left\n\
         //! off; an advance of 0 marks a codepoint with no glyph in the face. Lengths sum\n\
         //! to 65536.\n\n",
    );
    writeln!(
        buf,
        "/// Font design units per em (the divisor that turns advances into em fractions)."
    )
    .unwrap();
    writeln!(buf, "pub(super) const UNITS_PER_EM: u16 = {upem};\n").unwrap();
    writeln!(
        buf,
        "/// Mean advance over printable ASCII, in em — used for inverse"
    )
    .unwrap();
    writeln!(
        buf,
        "/// width-to-character-count estimates where no concrete string is available."
    )
    .unwrap();
    writeln!(
        buf,
        "pub(super) const MEAN_ADVANCE_EM: f64 = {mean_em:.6};\n"
    )
    .unwrap();
    writeln!(
        buf,
        "/// Per-style vertical metrics in font units (scaled by `size / UNITS_PER_EM`"
    )
    .unwrap();
    writeln!(buf, "/// at runtime); see `super::VMetrics`.").unwrap();
    for (name, _, _, vm) in &tables {
        emit_vmetrics(&mut buf, &name.replace("ADVANCE_RLE", "VMETRICS"), vm);
    }
    buf.push('\n');
    for (name, runs, _, _) in &tables {
        writeln!(buf, "/// {} runs.", runs.len()).unwrap();
        emit_rle(&mut buf, name, runs);
        buf.push('\n');
    }

    std::fs::write(OUT, &buf).expect("write data.rs");
    let summary: Vec<String> = tables
        .iter()
        .map(|(n, r, _, _)| format!("{n}={}", r.len()))
        .collect();
    println!("wrote {OUT} ({})", summary.join(", "));
}
