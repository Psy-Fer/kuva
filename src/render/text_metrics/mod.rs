//! Real text-width measurement backed by the bundled DejaVu Sans advance widths.
//!
//! kuva reserves space for every label, tick, legend entry, and title from a
//! width estimate computed at layout time. Historically that estimate was
//! `char_count * font_size * <some factor>`, a monospace approximation applied to
//! a proportional font; the factor was guessed (and re-guessed) per call site,
//! and non-ASCII labels measured by byte length were 2–3× off.
//!
//! This module replaces all of that with one function, [`measure_text_width`]
//! (and [`widest_text_width`] for "size to the widest of these labels"), that
//! sums the *real* horizontal advances of DejaVu Sans — the font kuva renders for
//! PNG/PDF, embeds under `embed_font`, and lists first in the default SVG cascade.
//! The advances live in generated, run-length-encoded tables ([`data`]) compiled
//! into every build (including the dependency-free SVG-only default build), so
//! layout never needs the font bytes or a parser at runtime. See
//! `examples/gen_text_metrics.rs` for regeneration.
//!
//! Each [`FontStyle`] is measured from its own face, so callers ask for the style
//! they will render and get correct widths even if the bundled font is swapped
//! for one whose italic or bold metrics differ from regular — a font change is a
//! regenerate, not a code change. (Across the full BMP the DejaVu faces really do
//! differ, so this matters.) The one exception: no bold-oblique face is bundled,
//! so [`FontStyle::BoldItalic`] is sourced from the Bold face until one is added.
//!
//! Accuracy by backend: exact for PNG/PDF and `embed_font` SVG (kuva controls the
//! font); a close, fails-safe prediction for bare SVG (the consumer picks the
//! font, but the cascade resolves to DejaVu or a near-metric-match like Verdana).

use std::sync::OnceLock;

mod data;

/// Advance assumed for a codepoint DejaVu Sans has no glyph for (CJK, emoji,
/// exotic scripts). One em deliberately over-reserves rather than clipping, and
/// such glyphs render in a substituted font of unknown metrics anyway.
const FALLBACK_ADVANCE_UNITS: u16 = data::UNITS_PER_EM;

/// Which face to measure against. Each variant has its own advance table; do not
/// assume any two share metrics (they do not, across the full character set).
///
/// `Italic`/`BoldItalic` (and [`FontStyle::from_flags`]) complete the API so
/// callers can measure italic text correctly — the faces genuinely differ — but
/// no current width site measures italic text, so they carry `allow(dead_code)`
/// until one does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FontStyle {
    Regular,
    #[allow(dead_code)]
    Italic,
    Bold,
    #[allow(dead_code)]
    BoldItalic,
}

impl FontStyle {
    /// Builds a style from independent bold/italic flags (the common call-site shape).
    #[allow(dead_code)]
    pub(crate) fn from_flags(bold: bool, italic: bool) -> Self {
        match (bold, italic) {
            (false, false) => FontStyle::Regular,
            (false, true) => FontStyle::Italic,
            (true, false) => FontStyle::Bold,
            (true, true) => FontStyle::BoldItalic,
        }
    }
}

/// Estimated rendered width of a single line of `text` at `font_size` pixels,
/// using real DejaVu Sans advance widths for `style`.
///
/// Counts Unicode scalar values (not UTF-8 bytes), so multi-byte labels are
/// measured correctly. Newlines are not interpreted — callers wrap first.
pub(crate) fn measure_text_width(text: &str, font_size: f64, style: FontStyle) -> f64 {
    let table = advance_table(style);
    let total_units: u64 = text
        .chars()
        .map(|c| {
            let cp = c as u32;
            let units = if cp <= 0xFFFF { table[cp as usize] } else { 0 };
            u64::from(if units == 0 { FALLBACK_ADVANCE_UNITS } else { units })
        })
        .sum();
    total_units as f64 / f64::from(data::UNITS_PER_EM) * font_size
}

/// Width of the widest of `labels` at `font_size` — the correct way to size a box
/// to fit any of a set of strings. Returns `0.0` for an empty set.
///
/// Prefer this over picking the longest-by-`len()` label and measuring it: more
/// characters does not mean more width (`"iiiii"` is narrower than `"MMM"`), and
/// `len()` counts bytes, not glyphs.
pub(crate) fn widest_text_width<'a>(
    labels: impl IntoIterator<Item = &'a str>,
    font_size: f64,
    style: FontStyle,
) -> f64 {
    labels
        .into_iter()
        .map(|s| measure_text_width(s, font_size, style))
        .fold(0.0, f64::max)
}

/// Mean rendered width of one character at `font_size`, for the few sites that
/// invert the relationship (fitting a character *budget* into an available
/// width) and so have no concrete string to measure.
pub(crate) fn mean_char_width(font_size: f64) -> f64 {
    font_size * data::MEAN_ADVANCE_EM
}

/// Returns the lazily-decoded dense advance table (font units, indexed by BMP
/// codepoint; 0 = no glyph) for `style`.
fn advance_table(style: FontStyle) -> &'static [u16; 0x10000] {
    static REGULAR: OnceLock<Box<[u16; 0x10000]>> = OnceLock::new();
    static ITALIC: OnceLock<Box<[u16; 0x10000]>> = OnceLock::new();
    static BOLD: OnceLock<Box<[u16; 0x10000]>> = OnceLock::new();
    static BOLD_ITALIC: OnceLock<Box<[u16; 0x10000]>> = OnceLock::new();
    match style {
        FontStyle::Regular => REGULAR.get_or_init(|| decode_rle(data::ADVANCE_RLE_REGULAR)),
        FontStyle::Italic => ITALIC.get_or_init(|| decode_rle(data::ADVANCE_RLE_ITALIC)),
        FontStyle::Bold => BOLD.get_or_init(|| decode_rle(data::ADVANCE_RLE_BOLD)),
        FontStyle::BoldItalic => {
            BOLD_ITALIC.get_or_init(|| decode_rle(data::ADVANCE_RLE_BOLD_ITALIC))
        }
    }
}

/// Expands a run-length-encoded advance table into a dense BMP array.
fn decode_rle(rle: &[(u16, u16)]) -> Box<[u16; 0x10000]> {
    let mut values: Vec<u16> = Vec::with_capacity(0x10000);
    for &(advance, len) in rle {
        values.resize(values.len() + len as usize, advance);
    }
    values
        .into_boxed_slice()
        .try_into()
        .expect("advance RLE must cover exactly the 65536 BMP codepoints")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE: f64 = 12.0;

    #[test]
    fn empty_string_has_zero_width() {
        assert_eq!(measure_text_width("", SIZE, FontStyle::Regular), 0.0);
    }

    #[test]
    fn wide_glyphs_measure_wider_than_narrow_ones() {
        let wide = measure_text_width("MMMM", SIZE, FontStyle::Regular);
        let narrow = measure_text_width("iiii", SIZE, FontStyle::Regular);
        assert!(wide > narrow, "M should be wider than i ({wide} vs {narrow})");
    }

    #[test]
    fn digits_are_tabular() {
        // DejaVu digits share one advance, so any two equal-length digit strings
        // measure identically.
        let a = measure_text_width("100000", SIZE, FontStyle::Regular);
        let b = measure_text_width("888888", SIZE, FontStyle::Regular);
        assert_eq!(a, b);
    }

    #[test]
    fn bold_is_wider_than_regular() {
        let bold = measure_text_width("Frequency", SIZE, FontStyle::Bold);
        let regular = measure_text_width("Frequency", SIZE, FontStyle::Regular);
        assert!(bold > regular, "bold should be wider ({bold} vs {regular})");
    }

    #[test]
    fn width_scales_linearly_with_font_size() {
        let small = measure_text_width("Sample", 10.0, FontStyle::Regular);
        let big = measure_text_width("Sample", 20.0, FontStyle::Regular);
        assert!((big - 2.0 * small).abs() < 1e-9);
    }

    #[test]
    fn widest_picks_the_widest_not_the_longest() {
        // "WWW" (3 wide glyphs) is wider than "iiiiiiii" (8 narrow ones), so a
        // count-based heuristic would pick the wrong string.
        let labels = ["iiiiiiii", "WWW", "ab"];
        let widest = widest_text_width(labels, SIZE, FontStyle::Regular);
        let www = measure_text_width("WWW", SIZE, FontStyle::Regular);
        assert_eq!(widest, www);
    }

    #[test]
    fn widest_of_empty_is_zero() {
        let none: [&str; 0] = [];
        assert_eq!(widest_text_width(none, SIZE, FontStyle::Regular), 0.0);
    }

    #[test]
    fn non_ascii_measured_by_glyph_not_byte_length() {
        // A superscript two is a single narrow glyph; measured by chars it is far
        // narrower than the 2-byte UTF-8 length would suggest, and narrower than
        // a normal digit.
        let sup = measure_text_width("\u{00B2}", SIZE, FontStyle::Regular);
        let two = measure_text_width("2", SIZE, FontStyle::Regular);
        assert!(sup > 0.0 && sup < two, "superscript should be narrow ({sup} vs {two})");
    }

    #[test]
    fn unknown_glyph_falls_back_to_one_em() {
        // An astral-plane codepoint (no DejaVu glyph) reserves one em.
        let emoji = measure_text_width("\u{1F600}", SIZE, FontStyle::Regular);
        assert!((emoji - SIZE).abs() < 1e-9);
    }

    /// Drift guard: every committed table must reproduce the advances of the face
    /// it was generated from. Regenerate `data.rs` if this fails after a font swap.
    #[test]
    fn committed_table_matches_font() {
        // (style, primary face file, fallback when the primary is not bundled) —
        // mirrors examples/gen_text_metrics.rs.
        let cases = [
            (FontStyle::Regular, "DejaVuSans.ttf", None),
            (FontStyle::Italic, "DejaVuSans-Oblique.ttf", None),
            (FontStyle::Bold, "DejaVuSans-Bold.ttf", None),
            (FontStyle::BoldItalic, "DejaVuSans-BoldOblique.ttf", Some("DejaVuSans-Bold.ttf")),
        ];
        for (style, primary, fallback) in cases {
            let dir = format!("{}/assets/fonts", env!("CARGO_MANIFEST_DIR"));
            let primary_path = format!("{dir}/{primary}");
            let path = match (std::path::Path::new(&primary_path).exists(), fallback) {
                (true, _) => primary_path,
                (false, Some(fb)) => format!("{dir}/{fb}"),
                (false, None) => primary_path,
            };
            let bytes = std::fs::read(&path).expect("read bundled font");
            let face = ttf_parser::Face::parse(&bytes, 0).expect("parse font");
            assert_eq!(u16::from(face.units_per_em()), data::UNITS_PER_EM);

            let table = advance_table(style);
            for cp in 0u32..=0xFFFF {
                let expected = char::from_u32(cp)
                    .and_then(|c| face.glyph_index(c))
                    .and_then(|g| face.glyph_hor_advance(g))
                    .unwrap_or(0);
                assert_eq!(table[cp as usize], expected, "{style:?} mismatch at U+{cp:04X}");
            }
        }
    }
}
