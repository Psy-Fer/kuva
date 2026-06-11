//! Math notation for `$...$` regions in labels (feature `math` for the
//! high-fidelity tier; the lookup tier below is always available).
//!
//! # Two tiers
//!
//! * **Lookup tier** ([`to_unicode`]) — always compiled, zero deps. Rewrites a
//!   label's `$...$` regions to inline Unicode text: Greek letters, operators,
//!   super/subscripts, `\frac`→`a/b`, `\sqrt`→`√(…)`. It never emits a stray
//!   `\` or `$`. This is the baseline for every backend and the *only* path
//!   the terminal backend can use (it renders to a character grid).
//!
//! * **Typst tier** (`#[cfg(feature = "math")]`) — compiles the **whole label**
//!   (text + math) with the Typst typesetter and returns an SVG fragment
//!   ([`render_label_svg`]) or pixmap ([`render_label_pixmap`]) that the
//!   SVG/PNG/PDF backends embed. Real 2-D typography: math italic, stacked
//!   fractions, radicals with vinculum, large operators with limits.
//!
//! The two share `$`-detection ([`contains_math`], [`split_segments`]) and the
//! command→symbol table ([`command_to_unicode`]). They diverge only on
//! *structure*: the lookup tier flattens `\frac`/`\sqrt`/super-sub to inline
//! forms; the typst tier hands that structure to Typst.
//!
//! Typst math is **not** LaTeX: a multi-letter run like `mc` is one identifier
//! (`E = mc^2` fails — write `E = m c^2`). On a typst-tier compile failure the
//! backend falls back to the lookup tier and warns once per distinct label.

#[cfg(feature = "math")]
use std::sync::{Mutex, OnceLock};

// ─────────────────────────── shared: detection ────────────────────────────

/// One segment of a label string: literal text or a math region (the body of
/// a `$...$`, without the dollar signs).
pub enum Segment<'a> {
    Text(&'a str),
    Math(&'a str),
}

/// Cheap pre-check: does the string contain at least one `$...$` region?
/// Requires two unescaped `$`. Avoids the segment-split cost for plain labels.
pub fn contains_math(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut count = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() && bytes[i + 1] == b'$' {
            i += 2;
            continue;
        }
        if bytes[i] == b'$' {
            count += 1;
            if count >= 2 {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// Split a label on `$...$` regions, honoring `\$` as a literal dollar.
/// An unclosed `$` makes the remainder a literal text segment.
pub fn split_segments(s: &str) -> Vec<Segment<'_>> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut cursor = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() && bytes[i + 1] == b'$' {
            i += 2;
            continue;
        }
        if bytes[i] == b'$' {
            if cursor < i {
                out.push(Segment::Text(&s[cursor..i]));
            }
            let math_start = i + 1;
            let mut j = math_start;
            while j < bytes.len() {
                if bytes[j] == b'\\' && j + 1 < bytes.len() {
                    j += 2;
                    continue;
                }
                if bytes[j] == b'$' {
                    break;
                }
                j += 1;
            }
            if j < bytes.len() {
                out.push(Segment::Math(&s[math_start..j]));
                i = j + 1;
                cursor = i;
            } else {
                out.push(Segment::Text(&s[i..]));
                cursor = bytes.len();
                break;
            }
        } else {
            i += 1;
        }
    }
    if cursor < bytes.len() {
        out.push(Segment::Text(&s[cursor..]));
    }
    out
}

/// Map a LaTeX-style command name (no leading `\`) to a Unicode symbol.
/// Shared by both tiers — the typst tier feeds these straight to Typst, which
/// renders the Unicode math symbols natively.
pub fn command_to_unicode(name: &str) -> Option<char> {
    Some(match name {
        // Greek lowercase
        "alpha" => 'α',
        "beta" => 'β',
        "gamma" => 'γ',
        "delta" => 'δ',
        "epsilon" | "varepsilon" => 'ε',
        "zeta" => 'ζ',
        "eta" => 'η',
        "theta" => 'θ',
        "iota" => 'ι',
        "kappa" => 'κ',
        "lambda" => 'λ',
        "mu" => 'μ',
        "nu" => 'ν',
        "xi" => 'ξ',
        "pi" => 'π',
        "rho" => 'ρ',
        "sigma" => 'σ',
        "tau" => 'τ',
        "upsilon" => 'υ',
        "phi" | "varphi" => 'φ',
        "chi" => 'χ',
        "psi" => 'ψ',
        "omega" => 'ω',
        // Greek uppercase
        "Gamma" => 'Γ',
        "Delta" => 'Δ',
        "Theta" => 'Θ',
        "Lambda" => 'Λ',
        "Xi" => 'Ξ',
        "Pi" => 'Π',
        "Sigma" => 'Σ',
        "Phi" => 'Φ',
        "Psi" => 'Ψ',
        "Omega" => 'Ω',
        // Operators / relations
        "cdot" => '·',
        "times" => '×',
        "div" => '÷',
        "pm" => '±',
        "mp" => '∓',
        "leq" | "le" => '≤',
        "geq" | "ge" => '≥',
        "neq" | "ne" => '≠',
        "approx" => '≈',
        "equiv" => '≡',
        "sim" => '∼',
        "propto" => '∝',
        "ll" => '≪',
        "gg" => '≫',
        // Symbols
        "infty" => '∞',
        "partial" => '∂',
        "nabla" => '∇',
        "degree" => '°',
        "angle" => '∠',
        "forall" => '∀',
        "exists" => '∃',
        "in" => '∈',
        "notin" => '∉',
        "subset" => '⊂',
        "cup" => '∪',
        "cap" => '∩',
        "ldots" => '…',
        "cdots" => '⋯',
        // Large operators
        "sum" => '∑',
        "prod" => '∏',
        "int" => '∫',
        // Arrows
        "to" | "rightarrow" => '→',
        "leftarrow" => '←',
        "Rightarrow" => '⇒',
        "Leftarrow" => '⇐',
        "leftrightarrow" => '↔',
        _ => return None,
    })
}

// ─────────────────────────── lookup tier ───────────────────────────────────

/// Rewrite a label's `$...$` math regions to inline Unicode text, leaving
/// surrounding text untouched. The result is plain text every backend can
/// render directly. See the module docs for the supported set.
///
/// Guarantees: the output never contains a `\` introduced by a math command
/// or a `$` math delimiter.
pub fn to_unicode(label: &str) -> String {
    let mut out = String::with_capacity(label.len());
    for seg in split_segments(label) {
        match seg {
            Segment::Text(t) => out.push_str(t),
            Segment::Math(body) => clean_math(body, &mut out),
        }
    }
    out
}

/// Lower a single `$...$` body to inline Unicode, appending to `out`.
fn clean_math(body: &str, out: &mut String) {
    let bytes = body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '\\' {
            let (name, next) = read_command(body, i + 1);
            match name {
                "frac" => {
                    if let Some(((n, d), end)) = take_two_groups(body, next) {
                        push_frac_inline(n, d, out);
                        i = end;
                        continue;
                    }
                }
                "sqrt" => {
                    let (index, after_idx) = read_optional_bracket(body, next);
                    if let Some((arg, end)) = take_group(body, after_idx) {
                        if let Some(idx) = index {
                            // Index as superscript before the radical, if it
                            // maps cleanly; else fall back to inline form.
                            if let Some(sup) = all_super(idx) {
                                out.push_str(&sup);
                            } else {
                                out.push_str(idx);
                            }
                        }
                        out.push('√');
                        let mut inner = String::new();
                        clean_math(arg, &mut inner);
                        wrap_inline(&inner, out);
                        i = end;
                        continue;
                    }
                }
                _ => {
                    if let Some(u) = command_to_unicode(name) {
                        out.push(u);
                        i = next;
                        continue;
                    }
                    // Unknown command: drop it (its `{arg}` is emitted by the
                    // brace rules below as cleaned content).
                    if !name.is_empty() {
                        i = next;
                        continue;
                    }
                }
            }
            // Stray backslash (e.g. before a non-letter) — drop it.
            i += 1;
            continue;
        }
        if c == '^' || c == '_' {
            if let Some((grp, end)) = read_script_group(body, i + 1) {
                let sub = substitute_commands(grp);
                let mapped = if c == '^' {
                    all_super(&sub)
                } else {
                    all_sub(&sub)
                };
                match mapped {
                    Some(uni) => out.push_str(&uni),
                    None => {
                        // All-or-nothing: keep a clean caret/underscore form.
                        out.push(c);
                        out.push('(');
                        out.push_str(&sub);
                        out.push(')');
                    }
                }
                i = end;
                continue;
            }
        }
        if c == '{' || c == '}' {
            // Stray grouping braces — strip.
            i += 1;
            continue;
        }
        out.push(c);
        i += 1;
    }
}

/// Substitute `\cmd` → Unicode within a (super/sub/sqrt) group, with no
/// structural handling. Unknown commands are dropped; braces stripped.
fn substitute_commands(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '\\' {
            let (name, next) = read_command(s, i + 1);
            if let Some(u) = command_to_unicode(name) {
                out.push(u);
            }
            i = if next > i + 1 { next } else { i + 1 };
            continue;
        }
        if c == '{' || c == '}' {
            i += 1;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// `\frac{a}{b}` → `a/b`, parenthesising multi-character parts.
fn push_frac_inline(num: &str, den: &str, out: &mut String) {
    let n = {
        let mut s = String::new();
        clean_math(num, &mut s);
        s
    };
    let d = {
        let mut s = String::new();
        clean_math(den, &mut s);
        s
    };
    wrap_inline(&n, out);
    out.push('/');
    wrap_inline(&d, out);
}

/// Append `s`, wrapping in parens when it's more than one grapheme so inline
/// fractions/radicals stay unambiguous (`1/2` but `(a+b)/c`, `√(x+y)`).
fn wrap_inline(s: &str, out: &mut String) {
    if s.chars().count() <= 1 {
        out.push_str(s);
    } else {
        out.push('(');
        out.push_str(s);
        out.push(')');
    }
}

/// Map every char of `s` to its Unicode superscript, or `None` if any lacks
/// one (all-or-nothing).
fn all_super(s: &str) -> Option<String> {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        out.push(super_char(c)?);
    }
    Some(out)
}

fn all_sub(s: &str) -> Option<String> {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        out.push(sub_char(c)?);
    }
    Some(out)
}

fn super_char(c: char) -> Option<char> {
    Some(match c {
        '0' => '⁰',
        '1' => '¹',
        '2' => '²',
        '3' => '³',
        '4' => '⁴',
        '5' => '⁵',
        '6' => '⁶',
        '7' => '⁷',
        '8' => '⁸',
        '9' => '⁹',
        '+' => '⁺',
        '-' => '⁻',
        '=' => '⁼',
        '(' => '⁽',
        ')' => '⁾',
        'a' => 'ᵃ',
        'b' => 'ᵇ',
        'c' => 'ᶜ',
        'd' => 'ᵈ',
        'e' => 'ᵉ',
        'f' => 'ᶠ',
        'g' => 'ᵍ',
        'h' => 'ʰ',
        'i' => 'ⁱ',
        'j' => 'ʲ',
        'k' => 'ᵏ',
        'l' => 'ˡ',
        'm' => 'ᵐ',
        'n' => 'ⁿ',
        'o' => 'ᵒ',
        'p' => 'ᵖ',
        'r' => 'ʳ',
        's' => 'ˢ',
        't' => 'ᵗ',
        'u' => 'ᵘ',
        'v' => 'ᵛ',
        'w' => 'ʷ',
        'x' => 'ˣ',
        'y' => 'ʸ',
        'z' => 'ᶻ',
        _ => return None,
    })
}

fn sub_char(c: char) -> Option<char> {
    Some(match c {
        '0' => '₀',
        '1' => '₁',
        '2' => '₂',
        '3' => '₃',
        '4' => '₄',
        '5' => '₅',
        '6' => '₆',
        '7' => '₇',
        '8' => '₈',
        '9' => '₉',
        '+' => '₊',
        '-' => '₋',
        '=' => '₌',
        '(' => '₍',
        ')' => '₎',
        'a' => 'ₐ',
        'e' => 'ₑ',
        'h' => 'ₕ',
        'i' => 'ᵢ',
        'j' => 'ⱼ',
        'k' => 'ₖ',
        'l' => 'ₗ',
        'm' => 'ₘ',
        'n' => 'ₙ',
        'o' => 'ₒ',
        'p' => 'ₚ',
        'r' => 'ᵣ',
        's' => 'ₛ',
        't' => 'ₜ',
        'u' => 'ᵤ',
        'v' => 'ᵥ',
        'x' => 'ₓ',
        _ => return None,
    })
}

// ── small parsing helpers (shared by both tiers) ──

/// Read an alphabetic command name starting at `start`; returns (name, index
/// just past it). Empty name if `start` isn't a letter.
fn read_command(s: &str, start: usize) -> (&str, usize) {
    let bytes = s.as_bytes();
    let mut end = start;
    while end < bytes.len() && (bytes[end] as char).is_ascii_alphabetic() {
        end += 1;
    }
    (&s[start..end], end)
}

/// If `pos` is at `{`, return (inner, index past `}`).
fn take_group(s: &str, pos: usize) -> Option<(&str, usize)> {
    let bytes = s.as_bytes();
    if pos >= bytes.len() || bytes[pos] != b'{' {
        return None;
    }
    let mut depth = 1;
    let mut i = pos + 1;
    let inner_start = i;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((&s[inner_start..i], i + 1));
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Read two consecutive `{..}{..}` groups.
fn take_two_groups(s: &str, pos: usize) -> Option<((&str, &str), usize)> {
    let (a, after_a) = take_group(s, pos)?;
    let (b, after_b) = take_group(s, after_a)?;
    Some(((a, b), after_b))
}

/// If `pos` is at `[`, return (inner, index past `]`).
fn read_optional_bracket(s: &str, pos: usize) -> (Option<&str>, usize) {
    let bytes = s.as_bytes();
    if pos < bytes.len() && bytes[pos] == b'[' {
        if let Some(close) = s[pos + 1..].find(']') {
            return (Some(&s[pos + 1..pos + 1 + close]), pos + 1 + close + 1);
        }
    }
    (None, pos)
}

/// Read a `^`/`_` operand: a `{group}`, a braceless `\command`, or a single
/// following char.
fn read_script_group(s: &str, pos: usize) -> Option<(&str, usize)> {
    let bytes = s.as_bytes();
    if pos >= bytes.len() {
        return None;
    }
    if bytes[pos] == b'{' {
        return take_group(s, pos);
    }
    // Braceless command operand, e.g. `x^\alpha` — grab the whole `\name` so it
    // isn't truncated to a lone `\` (which would leave `alpha` as literal text).
    if bytes[pos] == b'\\' {
        let (name, end) = read_command(s, pos + 1);
        if !name.is_empty() {
            return Some((&s[pos..end], end));
        }
    }
    // Single char (one UTF-8 scalar).
    let ch_len = s[pos..].chars().next()?.len_utf8();
    Some((&s[pos..pos + ch_len], pos + ch_len))
}

// ─────────────────────────── typst tier ────────────────────────────────────

#[cfg(feature = "math")]
mod typst_tier {
    use super::*;
    use crate::render::color::Color;
    use typst::diag::{FileError, FileResult, Warned};
    use typst::foundations::{Bytes, Datetime};
    use typst::syntax::{FileId, Source};
    use typst::text::{Font, FontBook};
    use typst::utils::LazyHash;
    use typst::{Library, LibraryExt, World};
    use typst_library::layout::PagedDocument;

    /// A whole label rendered to an SVG fragment for embedding.
    #[derive(Debug, Clone)]
    pub struct MathSvg {
        /// Inner SVG markup (between the outer `<svg>…</svg>`), with
        /// `xlink:href` rewritten to `href`.
        pub inner_svg: String,
        pub width_pt: f64,
        pub height_pt: f64,
        /// Top-of-bbox → baseline, in pt, for vertical alignment.
        pub baseline_offset_pt: f64,
    }

    /// A whole label rendered to an RGBA pixmap for the raster backend.
    #[derive(Debug, Clone)]
    pub struct MathPixmap {
        pub rgba: Vec<u8>,
        pub width_px: u32,
        pub height_px: u32,
        pub baseline_offset_px: f64,
    }

    /// Render the whole label (text + `$...$`) to an SVG fragment. `None` on
    /// compile failure (caller falls back to the lookup tier + warns).
    pub fn render_label_svg(label: &str, size_pt: f64, color: Option<&Color>) -> Option<MathSvg> {
        let doc = compile(label, size_pt, color)?;
        let page = doc.pages.first()?;
        Some(MathSvg {
            inner_svg: extract_inner(&typst_svg::svg(page)),
            width_pt: page.frame.width().to_pt(),
            height_pt: page.frame.height().to_pt(),
            baseline_offset_pt: page.frame.baseline().to_pt(),
        })
    }

    /// Render the whole label to an RGBA pixmap at `pixels_per_pt`.
    pub fn render_label_pixmap(
        label: &str,
        size_pt: f64,
        color: Option<&Color>,
        pixels_per_pt: f32,
    ) -> Option<MathPixmap> {
        let doc = compile(label, size_pt, color)?;
        let page = doc.pages.first()?;
        let baseline_pt = page.frame.baseline().to_pt();
        let pixmap = typst_render::render(page, pixels_per_pt);
        Some(MathPixmap {
            width_px: pixmap.width(),
            height_px: pixmap.height(),
            rgba: pixmap.data().to_vec(),
            baseline_offset_px: baseline_pt * pixels_per_pt as f64,
        })
    }

    fn compile(label: &str, size_pt: f64, color: Option<&Color>) -> Option<PagedDocument> {
        let world = KuvaWorld::new(build_source(label, size_pt, color));
        let Warned { output, .. } = typst::compile::<PagedDocument>(&world);
        match output {
            Ok(doc) => Some(doc),
            Err(errs) => {
                if warn_once(label) {
                    let detail = errs
                        .first()
                        .map(|d| {
                            let mut s = d.message.to_string();
                            if let Some(h) = d.hints.first() {
                                s.push_str(" (");
                                s.push_str(h);
                                s.push(')');
                            }
                            s
                        })
                        .unwrap_or_else(|| "unknown error".into());
                    eprintln!(
                        "kuva: could not typeset math in label {label:?}: {detail}. \
                         Note: this is Typst math syntax, not LaTeX — \
                         falling back to plain Unicode."
                    );
                }
                None
            }
        }
    }

    /// Build a minimal Typst document for the whole label: text segments are
    /// escaped markup, math segments become native Typst `$...$`.
    fn build_source(label: &str, size_pt: f64, color: Option<&Color>) -> String {
        let mut markup = String::with_capacity(label.len() + 16);
        for seg in split_segments(label) {
            match seg {
                Segment::Text(t) => super::escape_typst_markup(t, &mut markup),
                Segment::Math(body) => {
                    markup.push('$');
                    markup.push_str(&to_typst_math(body));
                    markup.push('$');
                }
            }
        }
        let fill = color.and_then(typst_fill).unwrap_or_default();
        // A small margin: `height: auto` sizes the page to the text's logical
        // box, which can exclude descenders and italic overhang — with `0pt`
        // margin those glyph edges get clipped at the pixmap boundary. `0.3em`
        // (relative to the font) leaves room without visible padding, and the
        // reported baseline/width include it so positioning stays correct.
        format!(
            "#set page(width: auto, height: auto, margin: 0.3em, fill: none)\n\
             #set text(font: \"DejaVu Sans\", size: {size_pt}pt{fill})\n\
             {markup}"
        )
    }

    /// `Some(", fill: rgb(\"#rrggbb\")")` for a concrete color, else `None`
    /// (default black).
    fn typst_fill(c: &Color) -> Option<String> {
        match c {
            Color::Rgb(..) => Some(format!(", fill: rgb(\"{}\")", c.to_svg_string())),
            Color::Css(s) if s.starts_with('#') => Some(format!(", fill: rgb(\"{s}\")")),
            _ => None,
        }
    }

    /// Extract the inner SVG and rewrite `xlink:href`→`href` so usvg (PDF) and
    /// strict viewers accept the fragment without an `xmlns:xlink` decl.
    ///
    /// Finds the end of the opening `<svg …>` tag via the first `>`. This is
    /// safe for `typst-svg` output (its `<svg>` attribute values never contain
    /// a literal `>`), but is not a general SVG parser.
    fn extract_inner(full: &str) -> String {
        let open_end = match full.find('>') {
            Some(i) => i + 1,
            None => return String::new(),
        };
        let close = full.rfind("</svg>").unwrap_or(full.len());
        full[open_end..close].replace("xlink:href", "href")
    }

    /// Returns `true` the first time a given expression is seen, so each
    /// distinct failing label warns at most once per process (not globally
    /// once). The set is small in practice — one entry per unique bad label.
    fn warn_once(key: &str) -> bool {
        static SEEN: OnceLock<Mutex<std::collections::HashSet<String>>> = OnceLock::new();
        let seen = SEEN.get_or_init(|| Mutex::new(std::collections::HashSet::new()));
        match seen.lock() {
            Ok(mut set) => set.insert(key.to_string()),
            Err(_) => true,
        }
    }

    // ── fonts / World ──

    fn fonts() -> &'static Vec<Font> {
        static FONTS: OnceLock<Vec<Font>> = OnceLock::new();
        FONTS.get_or_init(|| {
            let mut v = Vec::new();
            for bytes in [crate::fonts::dejavu_sans(), crate::fonts::newcm_math()] {
                for f in Font::iter(Bytes::new(bytes)) {
                    v.push(f);
                }
            }
            v
        })
    }

    fn library() -> &'static LazyHash<Library> {
        static LIB: OnceLock<LazyHash<Library>> = OnceLock::new();
        LIB.get_or_init(|| LazyHash::new(Library::default()))
    }

    fn font_book() -> &'static LazyHash<FontBook> {
        static BOOK: OnceLock<LazyHash<FontBook>> = OnceLock::new();
        BOOK.get_or_init(|| {
            let mut book = FontBook::new();
            for f in fonts() {
                book.push(f.info().clone());
            }
            LazyHash::new(book)
        })
    }

    struct KuvaWorld {
        main: FileId,
        source: Source,
    }

    impl KuvaWorld {
        fn new(src: String) -> Self {
            let main = FileId::new_fake(typst::syntax::VirtualPath::new("/main.typ"));
            Self {
                main,
                source: Source::new(main, src),
            }
        }
    }

    impl World for KuvaWorld {
        fn library(&self) -> &LazyHash<Library> {
            library()
        }
        fn book(&self) -> &LazyHash<FontBook> {
            font_book()
        }
        fn main(&self) -> FileId {
            self.main
        }
        fn source(&self, id: FileId) -> FileResult<Source> {
            if id == self.main {
                Ok(self.source.clone())
            } else {
                Err(FileError::NotFound(id.vpath().as_rooted_path().into()))
            }
        }
        fn file(&self, id: FileId) -> FileResult<Bytes> {
            Err(FileError::NotFound(id.vpath().as_rooted_path().into()))
        }
        fn font(&self, index: usize) -> Option<Font> {
            fonts().get(index).cloned()
        }
        fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
            Datetime::from_ymd(2024, 1, 1)
        }
    }
}

#[cfg(feature = "math")]
pub use typst_tier::{render_label_pixmap, render_label_svg, MathPixmap, MathSvg};

/// Translate a `$...$` body from LaTeX-ish syntax to Typst math syntax:
/// commands → Unicode symbols (Typst renders them natively), `\frac{a}{b}` →
/// `frac(a, b)`, `\sqrt{x}` → `sqrt(x)` / `\sqrt[n]{x}` → `root(n, x)`,
/// `^{…}`/`_{…}` → `^(…)`/`_(…)`. Available without the `math` feature so it
/// can be unit-tested deterministically.
pub fn to_typst_math(body: &str) -> String {
    let bytes = body.as_bytes();
    let mut out = String::with_capacity(body.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '\\' {
            let (name, next) = read_command(body, i + 1);
            match name {
                "frac" => {
                    if let Some(((n, d), end)) = take_two_groups(body, next) {
                        out.push_str("frac(");
                        out.push_str(&to_typst_math(n));
                        out.push_str(", ");
                        out.push_str(&to_typst_math(d));
                        out.push(')');
                        i = end;
                        continue;
                    }
                }
                "sqrt" => {
                    let (index, after_idx) = read_optional_bracket(body, next);
                    if let Some((arg, end)) = take_group(body, after_idx) {
                        match index {
                            Some(idx) => {
                                out.push_str("root(");
                                out.push_str(&to_typst_math(idx));
                                out.push_str(", ");
                                out.push_str(&to_typst_math(arg));
                                out.push(')');
                            }
                            None => {
                                out.push_str("sqrt(");
                                out.push_str(&to_typst_math(arg));
                                out.push(')');
                            }
                        }
                        i = end;
                        continue;
                    }
                }
                _ => {
                    if let Some(u) = command_to_unicode(name) {
                        out.push(u);
                        i = next;
                        continue;
                    }
                    if !name.is_empty() {
                        // Unknown command: drop the backslash, keep the name so
                        // Typst can resolve it if it happens to be valid there.
                        out.push_str(name);
                        i = next;
                        continue;
                    }
                }
            }
            i += 1;
            continue;
        }
        if c == '^' || c == '_' {
            if let Some((grp, end)) = read_script_group(body, i + 1) {
                out.push(c);
                out.push('(');
                out.push_str(&to_typst_math(grp));
                out.push(')');
                i = end;
                continue;
            }
        }
        if c == '{' {
            out.push('(');
            i += 1;
            continue;
        }
        if c == '}' {
            out.push(')');
            i += 1;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Escape Typst markup specials in literal label text so it renders verbatim.
/// Shared by the `math` tier (whole-label source) and the `typst` markup
/// backend. Covers the markup-significant characters: `#`, `$`, `*`, `_`,
/// `` ` ``, `<`, `>`, `@`, `\`, `"`, and the content-block delimiters `[` `]`
/// (a label like `signal [dB]` would otherwise lose its brackets).
pub(crate) fn escape_typst_markup(s: &str, out: &mut String) {
    for c in s.chars() {
        if matches!(
            c,
            '#' | '$' | '*' | '_' | '`' | '<' | '>' | '@' | '\\' | '"' | '[' | ']'
        ) {
            out.push('\\');
        }
        out.push(c);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_math() {
        assert!(contains_math("a $x$ b"));
        assert!(!contains_math("a $ b"));
        assert!(!contains_math("price \\$5"));
    }

    #[test]
    fn greek_and_operators() {
        assert_eq!(to_unicode("$\\sigma$"), "σ");
        assert_eq!(to_unicode("$\\alpha + \\beta$"), "α + β");
        assert_eq!(to_unicode("$a \\leq b \\cdot c$"), "a ≤ b · c");
        assert_eq!(to_unicode("$\\Omega$"), "Ω");
        assert_eq!(to_unicode("$\\infty$"), "∞");
    }

    #[test]
    fn superscripts_and_subscripts() {
        assert_eq!(to_unicode("$x^2$"), "x²");
        assert_eq!(to_unicode("$x_i$"), "xᵢ");
        assert_eq!(to_unicode("$x^{2n}$"), "x²ⁿ");
        assert_eq!(to_unicode("$x_{i+1}$"), "xᵢ₊₁");
    }

    #[test]
    fn superscript_all_or_nothing_fallback() {
        // 'q' has no Unicode superscript, so the whole group falls back.
        assert_eq!(to_unicode("$x^{2q}$"), "x^(2q)");
        // Uppercase generally has none either.
        assert_eq!(to_unicode("$x^{A}$"), "x^(A)");
    }

    #[test]
    fn fractions_inline() {
        assert_eq!(to_unicode("$\\frac{1}{2}$"), "1/2");
        assert_eq!(to_unicode("$\\frac{a+b}{c}$"), "(a+b)/c");
        assert_eq!(to_unicode("$\\frac{\\sqrt{a}}{b}$"), "(√a)/b");
    }

    #[test]
    fn sqrt_inline() {
        assert_eq!(to_unicode("$\\sqrt{x}$"), "√x");
        assert_eq!(to_unicode("$\\sqrt{x+y}$"), "√(x+y)");
        assert_eq!(to_unicode("$\\sqrt{x^2+y^2}$"), "√(x²+y²)");
    }

    #[test]
    fn nested_chain_stays_linear() {
        assert_eq!(
            to_unicode("$\\frac{\\sqrt{a}}{b^2} + \\sum_{i=1}^{n} x_i$"),
            "(√a)/(b²) + ∑ᵢ₌₁ⁿ xᵢ"
        );
    }

    #[test]
    fn no_backslash_or_dollar_in_output() {
        let out = to_unicode("$\\frac{\\unknown{a}}{\\sqrt{b}}$");
        assert!(!out.contains('\\'), "got {out}");
        assert!(!out.contains('$'), "got {out}");
    }

    #[test]
    fn text_around_math_preserved() {
        assert_eq!(
            to_unicode("Variance, $\\sigma^2$ (units)"),
            "Variance, σ² (units)"
        );
    }

    #[test]
    fn escaped_dollar_is_literal() {
        assert_eq!(to_unicode("price \\$5 each"), "price \\$5 each");
    }

    #[test]
    fn typst_translation() {
        assert_eq!(to_typst_math("\\sigma^2"), "σ^(2)");
        assert_eq!(to_typst_math("\\frac{a}{b}"), "frac(a, b)");
        assert_eq!(to_typst_math("\\sqrt{x}"), "sqrt(x)");
        assert_eq!(to_typst_math("\\sqrt[3]{x}"), "root(3, x)");
        assert_eq!(to_typst_math("x_{i+1}"), "x_(i+1)");
    }

    // ── detection / segmentation edge cases ──

    #[test]
    fn detection_edges() {
        assert!(contains_math("$$")); // two dollars, even if empty
        assert!(contains_math("a $x$ b $y$ c"));
        assert!(!contains_math("")); // empty
        assert!(!contains_math("no math here"));
        assert!(!contains_math("\\$5 and \\$6")); // both escaped
    }

    #[test]
    fn empty_and_unclosed_math() {
        assert_eq!(to_unicode("$$"), ""); // empty region
        assert_eq!(to_unicode("a $ b"), "a $ b"); // unclosed → literal
        assert_eq!(to_unicode("$\\alpha"), "$\\alpha"); // unclosed → literal
    }

    #[test]
    fn multiple_regions() {
        assert_eq!(to_unicode("$\\alpha$ and $\\beta$"), "α and β");
        assert_eq!(to_unicode("$x^2$ vs $y_1$"), "x² vs y₁");
    }

    #[test]
    fn full_greek_sample() {
        assert_eq!(to_unicode("$\\theta$"), "θ");
        assert_eq!(to_unicode("$\\lambda$"), "λ");
        assert_eq!(to_unicode("$\\mu$"), "μ");
        assert_eq!(to_unicode("$\\varphi$"), "φ");
        assert_eq!(to_unicode("$\\Delta$"), "Δ");
        assert_eq!(to_unicode("$\\Sigma$"), "Σ");
        assert_eq!(to_unicode("$\\Psi$"), "Ψ");
    }

    #[test]
    fn operator_sample() {
        assert_eq!(to_unicode("$a \\times b$"), "a × b");
        assert_eq!(to_unicode("$a \\div b$"), "a ÷ b");
        assert_eq!(to_unicode("$x \\neq y$"), "x ≠ y");
        assert_eq!(to_unicode("$x \\approx y$"), "x ≈ y");
        assert_eq!(to_unicode("$x \\to \\infty$"), "x → ∞");
        assert_eq!(to_unicode("$\\partial f$"), "∂ f");
        assert_eq!(to_unicode("$x \\in S$"), "x ∈ S");
        assert_eq!(to_unicode("$90\\degree$"), "90°");
    }

    #[test]
    fn unknown_command_and_stray_braces() {
        assert_eq!(to_unicode("$\\foo{x}$"), "x"); // drop cmd, keep cleaned arg
        assert_eq!(to_unicode("$\\foo$"), ""); // bare unknown dropped
        assert_eq!(to_unicode("${x}$"), "x"); // stray braces stripped
    }

    #[test]
    fn subscript_then_superscript() {
        assert_eq!(to_unicode("$x_i^2$"), "xᵢ²");
    }

    #[test]
    fn braceless_command_as_script_operand() {
        // `x^\alpha` must grab the whole `\alpha`, not a lone `\` (which left
        // `alpha` as literal text). α has no Unicode superscript → clean fallback.
        assert_eq!(to_unicode("$x^\\alpha$"), "x^(α)");
        assert_eq!(to_unicode("$x_\\beta$"), "x_(β)");
        assert_eq!(to_typst_math("x^\\alpha"), "x^(α)");
    }

    #[test]
    fn subscript_all_or_nothing_fallback() {
        // 'b' has no Unicode subscript, so the group falls back.
        assert_eq!(to_unicode("$x_{bc}$"), "x_(bc)");
    }

    #[test]
    fn sqrt_with_index() {
        assert_eq!(to_unicode("$\\sqrt[3]{x}$"), "³√x"); // cube root index
        assert_eq!(to_unicode("$\\sqrt[n]{x}$"), "ⁿ√x"); // n has a superscript
    }

    #[test]
    fn quadratic_formula_full_chain() {
        assert_eq!(
            to_unicode("$\\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}$"),
            "(-b ± √(b² - 4ac))/(2a)"
        );
    }

    #[test]
    fn typst_translation_more() {
        assert_eq!(to_typst_math("\\alpha + \\beta"), "α + β");
        assert_eq!(to_typst_math("\\frac{\\alpha}{2}"), "frac(α, 2)");
        assert_eq!(to_typst_math("\\sqrt[3]{x+1}"), "root(3, x+1)");
        assert_eq!(to_typst_math("x_i"), "x_(i)");
        assert_eq!(to_typst_math("a \\cdot b"), "a · b");
        assert_eq!(to_typst_math("{x}"), "(x)"); // braces → parens
        assert_eq!(to_typst_math("\\foo"), "foo"); // unknown: drop backslash
    }
}
