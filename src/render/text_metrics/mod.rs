//! Real text measurement backed by the bundled DejaVu Sans metrics — horizontal
//! advance widths and per-style vertical metrics (ascent, descent, line height,
//! cap/x-height).
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

use std::sync::{Arc, OnceLock};

mod data;

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
    /// Bundled DejaVu Sans Mono — the face a label rendered in a monospace
    /// family resolves to, and the one its width must be reserved from.
    Mono,
    /// Axis tick labels: the tick font family (or the body family when none is
    /// set). A [`FontSet`] resolves it to that family's regular face; for the
    /// built-in tables it means whichever DejaVu face [`FontStyle::for_family`]
    /// picks for that family.
    Tick,
}

impl FontStyle {
    /// Builds a style from independent bold/italic flags (the common call-site shape).
    #[cfg_attr(
        not(any(feature = "png", feature = "typst-math", feature = "custom_fonts")),
        allow(dead_code)
    )]
    pub(crate) fn from_flags(bold: bool, italic: bool) -> Self {
        match (bold, italic) {
            (false, false) => FontStyle::Regular,
            (false, true) => FontStyle::Italic,
            (true, false) => FontStyle::Bold,
            (true, true) => FontStyle::BoldItalic,
        }
    }

    /// The style a rich-text span is drawn in: bold first, then code, then
    /// italic, then plain text (the tick face inside a tick-label group).
    #[cfg_attr(not(any(feature = "png", feature = "custom_fonts")), allow(dead_code))]
    pub(crate) fn for_span(span: &crate::render::render::TextSpan, in_tick_group: bool) -> Self {
        if span.bold {
            FontStyle::from_flags(true, span.italic)
        } else if span.code {
            FontStyle::Mono
        } else if span.italic {
            FontStyle::Italic
        } else if in_tick_group {
            FontStyle::Tick
        } else {
            FontStyle::Regular
        }
    }

    /// Picks the face to *measure* a label that will be *rendered* in the CSS
    /// `font-family` list `family` — [`FontStyle::Mono`] when that list resolves
    /// to a monospace face, [`FontStyle::Regular`] otherwise (including `None`,
    /// the default sans cascade).
    ///
    /// Only the first family is consulted, the same one a renderer picks when it
    /// is available: the rest of the list is a fallback chain for consumers that
    /// lack it, and measuring a fallback would mis-reserve the common case. A
    /// family counts as monospace when it carries `mono`/`monospace` as a word
    /// (`"DejaVu Sans Mono"`, `"ui-monospace"`) or is one of the widely-deployed
    /// fixed-width faces that do not (`"Consolas"`, `"Fira Code"`, …). This is a
    /// width-reservation choice, not a rendering one, so an unrecognised
    /// monospace family is merely reserved from sans metrics as before.
    pub(crate) fn for_family(family: Option<&str>) -> Self {
        match family {
            Some(list) if first_family_is_monospace(list) => FontStyle::Mono,
            _ => FontStyle::Regular,
        }
    }
}

/// Fixed-width families whose names carry no `mono` word, so the word test in
/// [`first_family_is_monospace`] alone would miss them.
const KNOWN_MONOSPACE_FAMILIES: &[&str] = &[
    "consolas",
    "menlo",
    "monaco",
    "courier",
    "courier new",
    "inconsolata",
    "fira code",
    "source code pro",
    "cascadia code",
    "lucida console",
    "andale",
    "hack",
    "anonymous pro",
    "operator mono",
];

/// Whether the first family of a CSS `font-family` list names a monospace face.
///
/// Word-based rather than substring-based on purpose: `"Monotype Corsiva"` is a
/// script face, not a fixed-width one, and a `contains("mono")` test would
/// reserve it from the wrong metrics.
fn first_family_is_monospace(list: &str) -> bool {
    let first = list
        .split(',')
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .trim()
        .to_ascii_lowercase();
    if first.is_empty() {
        return false;
    }
    if KNOWN_MONOSPACE_FAMILIES.contains(&first.as_str()) {
        return true;
    }
    first
        .split(|c: char| c.is_whitespace() || c == '-' || c == '_')
        .any(|word| word == "mono" || word == "monospace")
}

/// Vertical metrics for one face, in font units (scale to pixels by
/// `size / units_per_em`). `descent` is stored as a positive depth below the
/// baseline. `cap_height`/`x_height` are measured from the `H`/`x` glyph outlines
/// (robust to faces whose OS/2 table omits them, as DejaVu does).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct VMetrics {
    ascent: i16,
    descent: i16,
    line_gap: i16,
    cap_height: i16,
    x_height: i16,
}

/// Everything layout needs to measure text in one face: horizontal advances over
/// the Basic Multilingual Plane plus the vertical metrics, measured **once** when
/// the face is loaded and then only looked up.
///
/// Every font is measured through this one type. The bundled DejaVu faces are
/// decoded from the generated tables in [`data`] (see [`FontMetrics::builtin`]), so
/// the dependency-free default build needs no font bytes or parser.
#[derive(Debug)]
pub(crate) struct FontMetrics {
    /// Font design units per em: the divisor that turns units into em fractions.
    units_per_em: u16,
    /// Advance per BMP codepoint, in font units; 0 marks a codepoint with no glyph.
    advances: Box<[u16; 0x10000]>,
    vmetrics: VMetrics,
    /// Mean advance over printable ASCII, in em, for the few inverse
    /// width-to-character-count estimates that have no concrete string.
    mean_advance_em: f64,
}

impl FontMetrics {
    /// Metrics of the bundled DejaVu face for `style`, decoded on first use and
    /// shared for the life of the process.
    pub(crate) fn builtin(style: FontStyle) -> &'static Arc<FontMetrics> {
        static REGULAR: OnceLock<Arc<FontMetrics>> = OnceLock::new();
        static ITALIC: OnceLock<Arc<FontMetrics>> = OnceLock::new();
        static BOLD: OnceLock<Arc<FontMetrics>> = OnceLock::new();
        static BOLD_ITALIC: OnceLock<Arc<FontMetrics>> = OnceLock::new();
        static MONO: OnceLock<Arc<FontMetrics>> = OnceLock::new();
        let (cell, rle, vmetrics) = match style {
            FontStyle::Regular => (&REGULAR, data::ADVANCE_RLE_REGULAR, data::VMETRICS_REGULAR),
            FontStyle::Italic => (&ITALIC, data::ADVANCE_RLE_ITALIC, data::VMETRICS_ITALIC),
            FontStyle::Bold => (&BOLD, data::ADVANCE_RLE_BOLD, data::VMETRICS_BOLD),
            FontStyle::BoldItalic => (
                &BOLD_ITALIC,
                data::ADVANCE_RLE_BOLD_ITALIC,
                data::VMETRICS_BOLD_ITALIC,
            ),
            FontStyle::Mono => (&MONO, data::ADVANCE_RLE_MONO, data::VMETRICS_MONO),
            // The tick role is resolved by `FontSet`; on its own it is the body face.
            FontStyle::Tick => (&REGULAR, data::ADVANCE_RLE_REGULAR, data::VMETRICS_REGULAR),
        };
        cell.get_or_init(|| {
            Arc::new(FontMetrics {
                units_per_em: data::UNITS_PER_EM,
                advances: decode_rle(rle),
                vmetrics,
                // The generator records the Regular face's mean for every style,
                // which is what `mean_char_width` has always used.
                mean_advance_em: data::MEAN_ADVANCE_EM,
            })
        })
    }

    /// Measure a parsed font face (feature `custom_fonts`): advances over the BMP,
    /// vertical metrics and the printable-ASCII mean advance. Called once per face,
    /// when it is registered. Uses exactly the rules `examples/gen_text_metrics.rs`
    /// applies to the bundled faces, so DejaVu measured here equals the built-in table.
    #[cfg(feature = "custom_fonts")]
    pub(crate) fn from_face(face: &ttf_parser::Face) -> FontMetrics {
        let units_per_em = face.units_per_em();
        let advance = |c: char| face.glyph_index(c).and_then(|g| face.glyph_hor_advance(g));
        let mut advances: Box<[u16; 0x10000]> = vec![0u16; 0x10000]
            .into_boxed_slice()
            .try_into()
            .expect("65536 entries");
        // Visit only the codepoints the font maps: `glyph_index` consults just the
        // Unicode cmap subtables, and those enumerate exactly what they map, so this
        // fills the same table as probing all 65536 codepoints, much faster.
        // U+FFFF is probed explicitly: a format-4 subtable's closing sentinel segment
        // sits there, which `glyph_index` resolves but enumeration skips.
        let mut mapped = vec![0xFFFF];
        if let Some(cmap) = face.tables().cmap {
            for subtable in cmap.subtables.into_iter().filter(|s| s.is_unicode()) {
                subtable.codepoints(|cp| {
                    if cp <= 0xFFFF {
                        mapped.push(cp);
                    }
                });
            }
        }
        for cp in mapped {
            if let Some(a) = char::from_u32(cp).and_then(advance) {
                advances[cp as usize] = a;
            }
        }

        let ascent = face.ascender();
        // Cap and x height from the 'H' and 'x' outlines (DejaVu's OS/2 table omits
        // them), then OS/2, then a derived value.
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
        let vmetrics = VMetrics {
            ascent,
            descent: -face.descender(),
            line_gap: face.line_gap(),
            cap_height,
            x_height,
        };

        let ascii: Vec<f64> = (0x20u8..=0x7E)
            .filter_map(|b| advance(char::from(b)))
            .map(|a| f64::from(a) / f64::from(units_per_em))
            .collect();
        let mean_advance_em = if ascii.is_empty() {
            data::MEAN_ADVANCE_EM
        } else {
            ascii.iter().sum::<f64>() / ascii.len() as f64
        };

        FontMetrics {
            units_per_em,
            advances,
            vmetrics,
            mean_advance_em,
        }
    }

    /// Scales a font-unit metric to pixels at `font_size`.
    fn scale(&self, units: impl Into<i32>, font_size: f64) -> f64 {
        f64::from(units.into()) / f64::from(self.units_per_em) * font_size
    }

    /// Advance of one codepoint in font units. A codepoint the face has no glyph
    /// for (CJK, emoji, exotic scripts) reserves one em: deliberately over-reserving
    /// rather than clipping, since such glyphs render in a substituted font of
    /// unknown metrics anyway.
    fn advance_units(&self, c: char) -> u16 {
        let cp = c as u32;
        let units = if cp <= 0xFFFF {
            self.advances[cp as usize]
        } else {
            0
        };
        if units == 0 {
            self.units_per_em
        } else {
            units
        }
    }

    /// Rendered width of a single line of `text` at `font_size` pixels.
    ///
    /// Counts Unicode scalar values (not UTF-8 bytes), so multi-byte labels are
    /// measured correctly. Newlines are not interpreted: callers wrap first.
    pub(crate) fn text_width(&self, text: &str, font_size: f64) -> f64 {
        let total_units: u64 = text.chars().map(|c| u64::from(self.advance_units(c))).sum();
        total_units as f64 / f64::from(self.units_per_em) * font_size
    }

    /// Width of the widest of `labels` at `font_size`; `0.0` for an empty set.
    pub(crate) fn widest_text_width<'a>(
        &self,
        labels: impl IntoIterator<Item = &'a str>,
        font_size: f64,
    ) -> f64 {
        labels
            .into_iter()
            .map(|s| self.text_width(s, font_size))
            .fold(0.0, f64::max)
    }

    /// Mean rendered width of one character at `font_size`.
    pub(crate) fn mean_char_width(&self, font_size: f64) -> f64 {
        font_size * self.mean_advance_em
    }

    /// Single-spaced line height (ascent + descent + line gap) in pixels.
    pub(crate) fn line_height(&self, font_size: f64) -> f64 {
        let m = self.vmetrics;
        self.scale(
            i32::from(m.ascent) + i32::from(m.descent) + i32::from(m.line_gap),
            font_size,
        )
    }

    /// Distance from the baseline up to the top of the ascenders, in pixels.
    pub(crate) fn ascent(&self, font_size: f64) -> f64 {
        self.scale(self.vmetrics.ascent, font_size)
    }

    /// Distance from the baseline down to the bottom of descenders (positive), in px.
    pub(crate) fn descent(&self, font_size: f64) -> f64 {
        self.scale(self.vmetrics.descent, font_size)
    }

    /// Ascent + descent, without line leading.
    pub(crate) fn text_height(&self, font_size: f64) -> f64 {
        let m = self.vmetrics;
        self.scale(i32::from(m.ascent) + i32::from(m.descent), font_size)
    }

    /// Cap height (top of uppercase letters above the baseline), in pixels.
    pub(crate) fn cap_height(&self, font_size: f64) -> f64 {
        self.scale(self.vmetrics.cap_height, font_size)
    }

    /// x-height (top of lowercase letters above the baseline), in pixels.
    pub(crate) fn x_height(&self, font_size: f64) -> f64 {
        self.scale(self.vmetrics.x_height, font_size)
    }

    /// Baseline offset that vertically centres a line of cap-height text on a
    /// target y (half the cap height).
    pub(crate) fn center_offset(&self, font_size: f64) -> f64 {
        self.cap_height(font_size) / 2.0
    }
}

/// The faces one layout measures its text with, resolved **once** from its font
/// families (see [`FontSet::resolve`]) and then only looked up.
///
/// Every measuring site asks the layout's `FontSet` rather than a fixed font, so a
/// registered custom font (feature `custom_fonts`) is measured with its own
/// metrics, and the bundled DejaVu faces are simply the default entries.
#[derive(Clone, Debug)]
pub(crate) struct FontSet {
    regular: ResolvedFace,
    bold: ResolvedFace,
    italic: ResolvedFace,
    bold_italic: ResolvedFace,
    mono: ResolvedFace,
    tick: ResolvedFace,
}

/// Where a resolved face comes from, so a backend that draws glyphs itself (PNG)
/// draws exactly the face layout measured.
#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "png"), allow(dead_code))]
pub(crate) enum FaceSource {
    /// A bundled DejaVu face, by the style its built-in table was taken from.
    Builtin(FontStyle),
    /// A face registered at runtime.
    #[cfg(feature = "custom_fonts")]
    Registered(Arc<crate::fonts::Face>),
}

/// One slot of a [`FontSet`]: the metrics layout measures with and their source.
#[derive(Clone, Debug)]
pub(crate) struct ResolvedFace {
    pub(crate) metrics: Arc<FontMetrics>,
    #[cfg_attr(not(feature = "png"), allow(dead_code))]
    pub(crate) source: FaceSource,
}

impl ResolvedFace {
    fn builtin(style: FontStyle) -> Self {
        ResolvedFace {
            metrics: Arc::clone(FontMetrics::builtin(style)),
            source: FaceSource::Builtin(style),
        }
    }
}

impl Default for FontSet {
    /// The bundled DejaVu faces: what an unregistered family measures as.
    fn default() -> Self {
        Self::resolve(None, None)
    }
}

impl FontSet {
    /// Resolve the faces for a body font-family list and an optional tick-label
    /// family list (`None` means the tick labels use the body family).
    ///
    /// For each list only the **first** family counts, the one a renderer picks
    /// when it is available; the rest is fallback for SVG viewers without it.
    /// - A registered family (feature `custom_fonts`) gets its own faces, chosen
    ///   by CSS font matching. A style the family lacks resolves to its nearest
    ///   face: kuva never synthesises bold or italic.
    /// - Otherwise the bundled DejaVu faces are used, with tick labels measured
    ///   as DejaVu Sans Mono when their family is monospace.
    ///
    /// Code spans use DejaVu Sans Mono unless [`FontSet::with_mono`] names a
    /// registered family.
    pub(crate) fn resolve(body: Option<&str>, tick: Option<&str>) -> FontSet {
        let body_face = |weight: u16, italic: bool, fallback: FontStyle| {
            registered_face(body, weight, italic).unwrap_or_else(|| ResolvedFace::builtin(fallback))
        };
        let tick_family = tick.or(body);
        let tick_face = registered_face(tick_family, 400, false)
            .unwrap_or_else(|| ResolvedFace::builtin(FontStyle::for_family(tick_family)));
        FontSet {
            regular: body_face(400, false, FontStyle::Regular),
            bold: body_face(700, false, FontStyle::Bold),
            italic: body_face(400, true, FontStyle::Italic),
            bold_italic: body_face(700, true, FontStyle::BoldItalic),
            mono: ResolvedFace::builtin(FontStyle::Mono),
            tick: tick_face,
        }
    }

    /// Set the face code spans (`FontStyle::Mono`) use: the regular face of a
    /// registered `family`, else the bundled DejaVu Sans Mono (an unregistered
    /// family is measured as that, as before).
    pub(crate) fn with_mono(mut self, family: Option<&str>) -> Self {
        if let Some(face) = registered_face(family, 400, false) {
            self.mono = face;
        }
        self
    }

    /// The resolved face for a `style`: its metrics and where it comes from.
    #[cfg_attr(not(feature = "png"), allow(dead_code))]
    pub(crate) fn resolved(&self, style: FontStyle) -> &ResolvedFace {
        match style {
            FontStyle::Regular => &self.regular,
            FontStyle::Bold => &self.bold,
            FontStyle::Italic => &self.italic,
            FontStyle::BoldItalic => &self.bold_italic,
            FontStyle::Mono => &self.mono,
            FontStyle::Tick => &self.tick,
        }
    }

    /// The face a `style` is measured with.
    pub(crate) fn face(&self, style: FontStyle) -> &FontMetrics {
        &self.resolved(style).metrics
    }

    /// Rendered width of a single line of `text`; see [`FontMetrics::text_width`].
    pub(crate) fn measure_text_width(&self, text: &str, font_size: f64, style: FontStyle) -> f64 {
        self.face(style).text_width(text, font_size)
    }

    /// Width of the widest of `labels`; `0.0` for an empty set.
    pub(crate) fn widest_text_width<'a>(
        &self,
        labels: impl IntoIterator<Item = &'a str>,
        font_size: f64,
        style: FontStyle,
    ) -> f64 {
        self.face(style).widest_text_width(labels, font_size)
    }

    /// Mean width of one character of body text, for inverse estimates.
    pub(crate) fn mean_char_width(&self, font_size: f64) -> f64 {
        self.regular.metrics.mean_char_width(font_size)
    }

    pub(crate) fn line_height(&self, font_size: f64, style: FontStyle) -> f64 {
        self.face(style).line_height(font_size)
    }

    pub(crate) fn ascent(&self, font_size: f64, style: FontStyle) -> f64 {
        self.face(style).ascent(font_size)
    }

    pub(crate) fn descent(&self, font_size: f64, style: FontStyle) -> f64 {
        self.face(style).descent(font_size)
    }

    pub(crate) fn text_height(&self, font_size: f64, style: FontStyle) -> f64 {
        self.face(style).text_height(font_size)
    }

    #[allow(dead_code)]
    pub(crate) fn cap_height(&self, font_size: f64, style: FontStyle) -> f64 {
        self.face(style).cap_height(font_size)
    }

    #[allow(dead_code)]
    pub(crate) fn x_height(&self, font_size: f64, style: FontStyle) -> f64 {
        self.face(style).x_height(font_size)
    }

    pub(crate) fn center_offset(&self, font_size: f64, style: FontStyle) -> f64 {
        self.face(style).center_offset(font_size)
    }
}

/// The registered face CSS matching picks in the first family of `family_list`,
/// or `None` when that family is not registered (or `custom_fonts` is off).
#[cfg(feature = "custom_fonts")]
fn registered_face(family_list: Option<&str>, weight: u16, italic: bool) -> Option<ResolvedFace> {
    let first = first_family(family_list?)?;
    crate::fonts::match_face(&first, weight, italic).map(|face| ResolvedFace {
        metrics: Arc::clone(face.metrics()),
        source: FaceSource::Registered(face),
    })
}

#[cfg(not(feature = "custom_fonts"))]
fn registered_face(_: Option<&str>, _: u16, _: bool) -> Option<ResolvedFace> {
    None
}

/// The first family of a CSS `font-family` list, unquoted and trimmed.
#[cfg(feature = "custom_fonts")]
fn first_family(list: &str) -> Option<String> {
    let first = list
        .split(',')
        .next()?
        .trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .trim();
    (!first.is_empty()).then(|| first.to_string())
}

// ── Free functions: the bundled DejaVu faces (tests only) ────────────────────
// Production code measures through a layout's `FontSet`, so a custom font is
// never silently measured as DejaVu. These remain for the table tests.

/// Natural single-spaced line height (ascent + descent + line gap) in pixels —
/// the vertical advance between stacked lines and the minimum height a row or box
/// must reserve to hold one line of text without clipping.
#[cfg(test)]
pub(crate) fn line_height(font_size: f64, style: FontStyle) -> f64 {
    FontMetrics::builtin(style).line_height(font_size)
}

/// Distance from the baseline up to the top of the font's ascenders, in pixels.
/// Use to top-anchor text in a reserved band (baseline = top + ascent).
#[cfg(test)]
pub(crate) fn ascent(font_size: f64, style: FontStyle) -> f64 {
    FontMetrics::builtin(style).ascent(font_size)
}

/// Distance from the baseline down to the bottom of descenders (positive), in px.
#[cfg(test)]
pub(crate) fn descent(font_size: f64, style: FontStyle) -> f64 {
    FontMetrics::builtin(style).descent(font_size)
}

/// Cap height (top of uppercase letters above the baseline), in pixels.
#[cfg(test)]
pub(crate) fn cap_height(font_size: f64, style: FontStyle) -> f64 {
    FontMetrics::builtin(style).cap_height(font_size)
}

/// x-height (top of lowercase letters above the baseline), in pixels.
#[cfg(test)]
pub(crate) fn x_height(font_size: f64, style: FontStyle) -> f64 {
    FontMetrics::builtin(style).x_height(font_size)
}

/// Baseline offset that vertically centers a line of uppercase/digit text on a
/// target y: draw the baseline at `target_y + center_offset(size, style)` so the
/// cap-height midpoint lands on `target_y`. The real-metric replacement for the
/// ubiquitous `font_size * 0.35` centering guess (`0.35 ≈ cap_height/2` for
/// DejaVu, but this tracks the actual font instead of assuming it).
#[cfg(test)]
pub(crate) fn center_offset(font_size: f64, style: FontStyle) -> f64 {
    FontMetrics::builtin(style).center_offset(font_size)
}

/// Estimated rendered width of a single line of `text` at `font_size` pixels,
/// using real DejaVu Sans advance widths for `style`.
///
/// Counts Unicode scalar values (not UTF-8 bytes), so multi-byte labels are
/// measured correctly. Newlines are not interpreted — callers wrap first.
#[cfg(test)]
pub(crate) fn measure_text_width(text: &str, font_size: f64, style: FontStyle) -> f64 {
    FontMetrics::builtin(style).text_width(text, font_size)
}

/// Width of the widest of `labels` at `font_size` — the correct way to size a box
/// to fit any of a set of strings. Returns `0.0` for an empty set.
///
/// Prefer this over picking the longest-by-`len()` label and measuring it: more
/// characters does not mean more width (`"iiiii"` is narrower than `"MMM"`), and
/// `len()` counts bytes, not glyphs.
#[cfg(test)]
pub(crate) fn widest_text_width<'a>(
    labels: impl IntoIterator<Item = &'a str>,
    font_size: f64,
    style: FontStyle,
) -> f64 {
    FontMetrics::builtin(style).widest_text_width(labels, font_size)
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
        assert!(
            wide > narrow,
            "M should be wider than i ({wide} vs {narrow})"
        );
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
    fn mono_advances_are_uniform() {
        // Every glyph in a fixed-width face shares one advance, so a string's
        // width depends only on its character count.
        let narrow = measure_text_width("....", SIZE, FontStyle::Mono);
        let wide = measure_text_width("MMMM", SIZE, FontStyle::Mono);
        assert_eq!(narrow, wide);
    }

    #[test]
    fn mono_is_wider_than_sans_for_narrow_glyphs() {
        // The case that drives the tick-label reservation: punctuation and thin
        // letters are far wider in a fixed-width face, so measuring a monospace
        // label with sans metrics under-reserves and clips it.
        let mono = measure_text_width("0.5", SIZE, FontStyle::Mono);
        let sans = measure_text_width("0.5", SIZE, FontStyle::Regular);
        assert!(mono > sans, "mono should be wider ({mono} vs {sans})");
    }

    #[test]
    fn monospace_families_select_the_mono_face() {
        for family in [
            "monospace",
            "ui-monospace",
            "DejaVu Sans Mono",
            "DejaVu Sans Mono, monospace",
            "\"Fira Code\", monospace",
            "Consolas",
            "  MONOSPACE  ",
        ] {
            assert_eq!(
                FontStyle::for_family(Some(family)),
                FontStyle::Mono,
                "{family} should measure as monospace"
            );
        }
    }

    #[test]
    fn proportional_families_keep_the_regular_face() {
        // "Monotype Corsiva" is the trap a substring test falls into: it is a
        // script face, and reserving it from mono metrics would over-size it.
        for family in [
            "DejaVu Sans, Verdana, Liberation Sans, Arial, sans-serif",
            "Monotype Corsiva",
            "Helvetica",
            "",
        ] {
            assert_eq!(
                FontStyle::for_family(Some(family)),
                FontStyle::Regular,
                "{family} should measure as proportional"
            );
        }
        assert_eq!(FontStyle::for_family(None), FontStyle::Regular);
    }

    #[test]
    fn only_the_first_family_decides_the_face() {
        // The tail of a cascade is a fallback for consumers lacking the first
        // family; measuring it would mis-reserve the common case.
        assert_eq!(
            FontStyle::for_family(Some("Verdana, monospace")),
            FontStyle::Regular
        );
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
        assert!(
            sup > 0.0 && sup < two,
            "superscript should be narrow ({sup} vs {two})"
        );
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
            (
                FontStyle::BoldItalic,
                "DejaVuSans-BoldOblique.ttf",
                Some("DejaVuSans-Bold.ttf"),
            ),
            (FontStyle::Mono, "DejaVuSansMono.ttf", None),
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
            assert_eq!(face.units_per_em(), data::UNITS_PER_EM);

            let table = &FontMetrics::builtin(style).advances;
            for cp in 0u32..=0xFFFF {
                let expected = char::from_u32(cp)
                    .and_then(|c| face.glyph_index(c))
                    .and_then(|g| face.glyph_hor_advance(g))
                    .unwrap_or(0);
                assert_eq!(
                    table[cp as usize], expected,
                    "{style:?} mismatch at U+{cp:04X}"
                );
            }
        }
    }

    /// Drift guard for the vertical metrics: every committed `VMetrics` must
    /// reproduce the face it was generated from (hhea ascent/descent/line-gap and
    /// the 'H'/'x' outline tops). Mirrors `vmetrics()` in gen_text_metrics.rs.
    #[test]
    fn committed_vmetrics_match_font() {
        let cases = [
            (FontStyle::Regular, "DejaVuSans.ttf", None),
            (FontStyle::Italic, "DejaVuSans-Oblique.ttf", None),
            (FontStyle::Bold, "DejaVuSans-Bold.ttf", None),
            (
                FontStyle::BoldItalic,
                "DejaVuSans-BoldOblique.ttf",
                Some("DejaVuSans-Bold.ttf"),
            ),
            (FontStyle::Mono, "DejaVuSansMono.ttf", None),
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
            let glyph_top = |ch: char| {
                face.glyph_index(ch)
                    .and_then(|g| face.glyph_bounding_box(g))
                    .map(|bb| bb.y_max)
            };
            let cap = glyph_top('H')
                .or_else(|| face.capital_height())
                .unwrap_or(face.ascender());
            let x = glyph_top('x')
                .or_else(|| face.x_height())
                .unwrap_or((f32::from(cap) * 0.72) as i16);

            let m = FontMetrics::builtin(style).vmetrics;
            assert_eq!(m.ascent, face.ascender(), "{style:?} ascent");
            assert_eq!(m.descent, -face.descender(), "{style:?} descent");
            assert_eq!(m.line_gap, face.line_gap(), "{style:?} line_gap");
            assert_eq!(m.cap_height, cap, "{style:?} cap_height");
            assert_eq!(m.x_height, x, "{style:?} x_height");
        }
    }

    #[test]
    fn line_height_covers_ascent_and_descent() {
        // A row shorter than ascent+descent would clip; line_height must cover it
        // (equals it exactly for DejaVu, whose line_gap is 0).
        let lh = line_height(SIZE, FontStyle::Regular);
        let box_h = ascent(SIZE, FontStyle::Regular) + descent(SIZE, FontStyle::Regular);
        assert!(
            lh >= box_h - 1e-9,
            "line_height {lh} < ascent+descent {box_h}"
        );
    }

    #[test]
    fn line_height_exceeds_cap_height() {
        // Sizing a row to cap height alone (a common bug) clips descenders.
        assert!(line_height(SIZE, FontStyle::Regular) > cap_height(SIZE, FontStyle::Regular));
    }

    #[test]
    fn center_offset_is_half_cap_height() {
        let co = center_offset(SIZE, FontStyle::Regular);
        assert!((co - cap_height(SIZE, FontStyle::Regular) / 2.0).abs() < 1e-9);
    }

    #[test]
    fn x_height_below_cap_below_ascent() {
        let s = FontStyle::Regular;
        assert!(x_height(SIZE, s) < cap_height(SIZE, s));
        assert!(cap_height(SIZE, s) < ascent(SIZE, s));
    }

    /// A face measured at load time must equal the built-in table generated from
    /// the same file: the loader and the generator apply the same rules.
    #[cfg(feature = "custom_fonts")]
    #[test]
    fn loaded_dejavu_matches_builtin_tables() {
        let dir = format!("{}/assets/fonts", env!("CARGO_MANIFEST_DIR"));
        // No bold-oblique face is bundled, so BoldItalic is built from Bold.
        let cases = [
            (FontStyle::Regular, "DejaVuSans.ttf"),
            (FontStyle::Italic, "DejaVuSans-Oblique.ttf"),
            (FontStyle::Bold, "DejaVuSans-Bold.ttf"),
            (FontStyle::BoldItalic, "DejaVuSans-Bold.ttf"),
            (FontStyle::Mono, "DejaVuSansMono.ttf"),
        ];
        for (style, file) in cases {
            let bytes = std::fs::read(format!("{dir}/{file}")).expect("read bundled font");
            let face = ttf_parser::Face::parse(&bytes, 0).expect("parse font");
            let loaded = FontMetrics::from_face(&face);
            let builtin = FontMetrics::builtin(style);
            assert_eq!(loaded.units_per_em, builtin.units_per_em, "{style:?}");
            let diff: Vec<u32> = (0u32..=0xFFFF)
                .filter(|&cp| loaded.advances[cp as usize] != builtin.advances[cp as usize])
                .collect();
            assert!(
                diff.is_empty(),
                "{style:?}: {} codepoints differ, first {:X?}",
                diff.len(),
                &diff[..diff.len().min(8)]
            );
            assert_eq!(loaded.vmetrics, builtin.vmetrics, "{style:?} vmetrics");
        }
        // The generator writes the Regular mean rounded to 6 places.
        let bytes = std::fs::read(format!("{dir}/DejaVuSans.ttf")).unwrap();
        let regular = FontMetrics::from_face(&ttf_parser::Face::parse(&bytes, 0).unwrap());
        assert!((regular.mean_advance_em - data::MEAN_ADVANCE_EM).abs() < 1e-6);
    }

    /// A different font measures differently: DejaVu Serif (the test font) against
    /// the built-in Sans.
    #[cfg(feature = "custom_fonts")]
    #[test]
    fn serif_measures_differently_from_sans() {
        let path = format!(
            "{}/tests/data/fonts/DejaVuSerif.ttf",
            env!("CARGO_MANIFEST_DIR")
        );
        let bytes = std::fs::read(path).expect("read test font");
        let serif = FontMetrics::from_face(&ttf_parser::Face::parse(&bytes, 0).unwrap());
        let sans = FontMetrics::builtin(FontStyle::Regular);
        let text = "Frequency of samples";
        let (w_serif, w_sans) = (serif.text_width(text, SIZE), sans.text_width(text, SIZE));
        assert!(
            (w_serif - w_sans).abs() > 1.0,
            "serif {w_serif} vs sans {w_sans}"
        );
    }

    #[test]
    fn an_unregistered_family_resolves_to_the_builtin_faces() {
        let fonts = FontSet::resolve(Some("No Such Family, sans-serif"), None);
        for style in [
            FontStyle::Regular,
            FontStyle::Bold,
            FontStyle::Italic,
            FontStyle::BoldItalic,
            FontStyle::Mono,
        ] {
            assert!(std::ptr::eq(
                fonts.face(style),
                &**FontMetrics::builtin(style)
            ));
        }
        // Proportional tick family: the regular face.
        assert!(std::ptr::eq(
            fonts.face(FontStyle::Tick),
            &**FontMetrics::builtin(FontStyle::Regular)
        ));
    }

    #[test]
    fn a_monospace_tick_family_measures_as_mono() {
        let fonts = FontSet::resolve(None, Some("monospace"));
        assert!(std::ptr::eq(
            fonts.face(FontStyle::Tick),
            &**FontMetrics::builtin(FontStyle::Mono)
        ));
        // No tick family: ticks follow the body family, here monospace.
        let fonts = FontSet::resolve(Some("DejaVu Sans Mono"), None);
        assert!(std::ptr::eq(
            fonts.face(FontStyle::Tick),
            &**FontMetrics::builtin(FontStyle::Mono)
        ));
        // Body text keeps the regular face even in a monospace family (unchanged
        // behaviour: only tick labels used the monospace heuristic).
        assert!(std::ptr::eq(
            fonts.face(FontStyle::Regular),
            &**FontMetrics::builtin(FontStyle::Regular)
        ));
    }

    #[test]
    fn vertical_metrics_scale_linearly_with_size() {
        let a = line_height(10.0, FontStyle::Regular);
        let b = line_height(20.0, FontStyle::Regular);
        assert!((b - 2.0 * a).abs() < 1e-9);
    }
}

#[cfg(test)]
mod probe_scratch2 {
    use super::*;
    #[test]
    fn probe_digit_widths() {
        for d in "0123456789".chars() {
            println!(
                "{d}: {:.3}",
                measure_text_width(&d.to_string(), 12.0, FontStyle::Regular)
            );
        }
    }
}
