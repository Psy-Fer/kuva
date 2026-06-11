//! Typst output backend (feature `typst`).
//!
//! Emits a [Typst](https://typst.app) document that draws the [`Scene`] using
//! the CETZ package for vector graphics. The output is plain text — kuva does
//! not embed a Typst compiler. Users run `typst compile fig.typ` themselves
//! to produce PDF/SVG/PNG; this keeps kuva's dependency footprint at zero
//! for this feature.
//!
//! # Why a separate backend?
//!
//! This backend emits Typst *markup* (`.typ`), which the user compiles
//! themselves with `typst compile`. It's the right choice when the plot is
//! destined for a larger Typst document, or when the user wants to hand-edit
//! the output. It has zero compiled dependencies — pure string emission.
//!
//! For rendering math *without* an external toolchain, see the separate
//! `math` feature ([`crate::render::math`]), which links the typst compiler
//! as a library and embeds rendered `$...$` regions directly into kuva's
//! SVG/PNG/PDF output. Both routes use Typst's typesetter; they differ only
//! in whether you run the compiler yourself.
//!
//! # Output shape
//!
//! ```typst
//! #set page(width: 800pt, height: 600pt, margin: 0pt, fill: white)
//! #import "@preview/cetz:0.5.2"
//! #cetz.canvas({
//!     import cetz.draw: *
//!     // primitives go here
//!     line((0pt, 0pt), (100pt, 100pt), stroke: 1pt + black)
//!     circle((50pt, 50pt), radius: 3pt, fill: blue)
//!     content((100pt, 200pt), text(size: 14pt)[some label])
//! })
//! ```
//!
//! # Coordinate system
//!
//! Kuva's `Scene` uses SVG-style coordinates: origin at top-left, y grows
//! downward. CETZ uses mathematical coordinates: origin at bottom-left, y
//! grows upward. The backend flips y on emission so the output looks the
//! same as the SVG.
//!
//! # Limitations (T0 scaffold — to be lifted iteratively)
//!
//! - `Path` primitive (arbitrary SVG path data) is not yet translated — kuva
//!   uses Paths sparingly (mostly arrowheads and curved bands) so this is a
//!   moderate gap to close later.
//! - Batched primitives (`CircleBatch`, `RectBatch`) unroll to one CETZ call
//!   per element. Acceptable for typical plot sizes; can be optimized later.
//! - Clip regions (`ClipStart`/`ClipEnd`) are silently dropped.
//! - Interactive features (tooltips, scripts) are not applicable to a
//!   typesetting target and are dropped.

use std::fmt::Write;

use crate::render::color::Color;
use crate::render::render::{Primitive, Scene, TextAnchor, TextSpan};

/// Typst output backend.
///
/// Construct with [`TypstBackend::default`] and call
/// [`TypstBackend::render_scene`] to get the Typst source as a `String`.
pub struct TypstBackend {
    /// CETZ version to import. Pinned to a known-good version so the output
    /// keeps working when Typst's package index gains new releases.
    cetz_version: &'static str,
}

impl Default for TypstBackend {
    fn default() -> Self {
        Self {
            cetz_version: "0.5.2",
        }
    }
}

impl TypstBackend {
    /// Render a [`Scene`] to a Typst source string. The returned string is a
    /// complete `.typ` document — write it to a file and run
    /// `typst compile that_file.typ` to produce PDF/SVG/PNG.
    pub fn render_scene(&self, scene: &Scene) -> String {
        let mut out = String::with_capacity(8192);

        // Preamble: page setup, no margin so coordinates align with kuva's
        // pixel space exactly.
        let _ = write!(
            out,
            "#set page(width: {}pt, height: {}pt, margin: 0pt",
            scene.width, scene.height
        );
        if let Some(bg) = &scene.background_color {
            let _ = write!(out, ", fill: {}", typst_color_named(bg));
        }
        out.push_str(")\n");

        // Default font for text.
        if let Some(ff) = &scene.font_family {
            let _ = writeln!(out, "#set text(font: \"{}\")", primary_font(ff));
        }

        // Pull in CETZ for the drawing primitives.
        let _ = writeln!(out, "#import \"@preview/cetz:{}\"", self.cetz_version);

        // Open canvas. `length: 1pt` makes coordinates 1:1 with point units.
        out.push_str("#cetz.canvas(length: 1pt, {\n  import cetz.draw: *\n");

        let h = scene.height;
        for p in &scene.elements {
            emit_primitive(&mut out, p, h);
        }

        out.push_str("})\n");
        out
    }
}

// ── Per-primitive emission ────────────────────────────────────────────────────

fn emit_primitive(out: &mut String, p: &Primitive, scene_h: f64) {
    match p {
        Primitive::Circle {
            cx,
            cy,
            r,
            fill,
            fill_opacity: _,
            stroke,
            stroke_width,
        } => {
            // CETZ coordinates are bare numbers (multiplied by canvas `length:`).
            let _ = write!(
                out,
                "  circle(({}, {}), radius: {}pt, fill: {}",
                cx,
                flip_y(*cy, scene_h),
                r,
                typst_color(fill)
            );
            if let Some(s) = stroke {
                let _ = write!(
                    out,
                    ", stroke: {}pt + {}",
                    stroke_width.unwrap_or(1.0),
                    typst_color(s)
                );
            } else {
                out.push_str(", stroke: none");
            }
            out.push_str(")\n");
        }

        Primitive::Line {
            x1,
            y1,
            x2,
            y2,
            stroke,
            stroke_width,
            stroke_dasharray,
        } => {
            let _ = write!(
                out,
                "  line(({}, {}), ({}, {}), stroke: {}pt + {}",
                x1,
                flip_y(*y1, scene_h),
                x2,
                flip_y(*y2, scene_h),
                stroke_width,
                typst_color(stroke)
            );
            if stroke_dasharray.is_some() {
                out.push_str(" + (dash: \"dashed\")");
            }
            out.push_str(")\n");
        }

        Primitive::Rect {
            x,
            y,
            width,
            height,
            fill,
            stroke,
            stroke_width,
            opacity: _,
        } => {
            let bottom_y = flip_y(*y + *height, scene_h);
            let top_y = flip_y(*y, scene_h);
            let _ = write!(
                out,
                "  rect(({}, {}), ({}, {}), fill: {}",
                x,
                bottom_y,
                *x + *width,
                top_y,
                typst_color(fill)
            );
            if let Some(s) = stroke {
                let _ = write!(
                    out,
                    ", stroke: {}pt + {}",
                    stroke_width.unwrap_or(1.0),
                    typst_color(s)
                );
            } else {
                out.push_str(", stroke: none");
            }
            out.push_str(")\n");
        }

        Primitive::Text {
            x,
            y,
            content,
            size,
            anchor,
            rotate,
            bold,
            color,
        } => {
            emit_text(
                out,
                *x,
                *y,
                content,
                *size,
                *anchor,
                *rotate,
                *bold,
                false,
                color.as_ref(),
                scene_h,
            );
        }

        Primitive::RichText {
            x,
            y,
            spans,
            size,
            anchor,
            color,
        } => {
            // Flatten spans into a single content with per-span styling.
            // For the prototype, concatenate into one Typst markup string
            // using #emph/#strong/#underline as appropriate.
            let mut flat = String::new();
            for s in spans {
                write_styled_span(&mut flat, s);
            }
            emit_typst_content(
                out,
                *x,
                *y,
                &flat,
                *size,
                *anchor,
                None,
                false,
                false,
                color.as_ref(),
                scene_h,
                true,
            );
        }

        Primitive::CircleBatch {
            cx,
            cy,
            r,
            fill,
            fill_opacity: _,
            stroke,
            stroke_width,
        } => {
            for (x, y) in cx.iter().zip(cy.iter()) {
                let _ = write!(
                    out,
                    "  circle(({}, {}), radius: {}pt, fill: {}",
                    x,
                    flip_y(*y, scene_h),
                    r,
                    typst_color(fill)
                );
                if let Some(s) = stroke {
                    let _ = write!(
                        out,
                        ", stroke: {}pt + {}",
                        stroke_width.unwrap_or(1.0),
                        typst_color(s)
                    );
                } else {
                    out.push_str(", stroke: none");
                }
                out.push_str(")\n");
            }
        }

        Primitive::RectBatch { x, y, w, h, fills } => {
            for (((xi, yi), wi), (hi, fi)) in x
                .iter()
                .zip(y.iter())
                .zip(w.iter())
                .zip(h.iter().zip(fills.iter()))
            {
                let bottom_y = flip_y(*yi + *hi, scene_h);
                let top_y = flip_y(*yi, scene_h);
                let _ = writeln!(
                    out,
                    "  rect(({}, {}), ({}, {}), fill: {}, stroke: none)",
                    xi,
                    bottom_y,
                    *xi + *wi,
                    top_y,
                    typst_color(fi)
                );
            }
        }

        Primitive::PolyLine {
            points,
            stroke,
            stroke_width,
            stroke_dasharray: _,
        } => {
            if points.len() < 2 {
                return;
            }
            out.push_str("  line(");
            for (i, (px, py)) in points.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                let _ = write!(out, "({}, {})", px, flip_y(*py, scene_h));
            }
            let _ = writeln!(
                out,
                ", stroke: {}pt + {})",
                stroke_width,
                typst_color(stroke)
            );
        }

        // TODO: Path primitive — kuva's Path uses SVG path data which would
        // need conversion to CETZ's bezier()/path()/merge-path() syntax.
        // Deferred to a follow-up; most kuva plots don't rely on Path.
        Primitive::Path(_) => {
            out.push_str("  // TODO: Primitive::Path not yet supported in Typst backend\n");
        }

        // Grouping and clipping are dropped — Typst has different concepts
        // for these and they're not strictly necessary for static output.
        Primitive::GroupStart { .. }
        | Primitive::GroupEnd
        | Primitive::ClipStart { .. }
        | Primitive::ClipEnd => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_text(
    out: &mut String,
    x: f64,
    y: f64,
    content: &str,
    size: u32,
    anchor: TextAnchor,
    rotate: Option<f64>,
    bold: bool,
    italic: bool,
    color: Option<&Color>,
    scene_h: f64,
) {
    // Detect `$...$` regions in the label and emit them as native Typst math
    // (Typst's own typesetter handles the rendering, with real math fonts).
    // Plain text between math regions is escaped so Typst markup specials
    // don't trigger.
    use crate::render::math::{contains_math, split_segments, to_typst_math, Segment};
    let mut markup = String::with_capacity(content.len() + 8);
    if contains_math(content) {
        for seg in split_segments(content) {
            match seg {
                Segment::Text(s) => write_typst_escaped(&mut markup, s),
                Segment::Math(body) => {
                    markup.push('$');
                    markup.push_str(&to_typst_math(body));
                    markup.push('$');
                }
            }
        }
        emit_typst_content(
            out, x, y, &markup, size, anchor, rotate, bold, italic, color, scene_h, true,
        );
    } else {
        write_typst_escaped(&mut markup, content);
        emit_typst_content(
            out, x, y, &markup, size, anchor, rotate, bold, italic, color, scene_h, false,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_typst_content(
    out: &mut String,
    x: f64,
    y: f64,
    content_markup: &str,
    size: u32,
    anchor: TextAnchor,
    rotate: Option<f64>,
    bold: bool,
    italic: bool,
    color: Option<&Color>,
    scene_h: f64,
    content_is_markup: bool,
) {
    // CETZ: content((x, y), anchor: "...", [markup])
    let anchor_str = match anchor {
        TextAnchor::Start => "west",
        TextAnchor::Middle => "center",
        TextAnchor::End => "east",
    };

    let _ = write!(out, "  content(({}, {}), ", x, flip_y(y, scene_h));

    // Build inline text styling.
    let mut style = String::new();
    let _ = write!(style, "size: {}pt", size);
    if bold {
        style.push_str(", weight: \"bold\"");
    }
    if italic {
        style.push_str(", style: \"italic\"");
    }
    if let Some(c) = color {
        let _ = write!(style, ", fill: {}", typst_color(c));
    }

    if content_is_markup {
        let _ = write!(out, "[#text({})[{}]]", style, content_markup);
    } else {
        let _ = write!(out, "text({})[{}]", style, content_markup);
    }
    let _ = write!(out, ", anchor: \"{}\"", anchor_str);
    if let Some(deg) = rotate {
        // CETZ rotation is counter-clockwise positive; SVG rotation is
        // clockwise positive. Flip the sign.
        let _ = write!(out, ", angle: {}deg", -deg);
    }
    out.push_str(")\n");
}

fn write_styled_span(out: &mut String, span: &TextSpan) {
    let mut s = String::new();
    write_typst_escaped(&mut s, &span.text);
    if span.bold {
        out.push_str("#strong[");
    }
    if span.italic {
        out.push_str("#emph[");
    }
    if span.underline {
        out.push_str("#underline[");
    }
    out.push_str(&s);
    if span.underline {
        out.push(']');
    }
    if span.italic {
        out.push(']');
    }
    if span.bold {
        out.push(']');
    }
}

/// Escape Typst markup specials in plain text content. Delegates to the shared
/// escaper so the markup backend and the `math` tier stay in sync (the previous
/// local version missed `_`, `*`, `` ` ``, `<`, `>` — a label like `a_b` or
/// `*x*` was mis-typeset as subscript/emphasis).
fn write_typst_escaped(out: &mut String, s: &str) {
    crate::render::math::escape_typst_markup(s, out);
}

// ── Color & font helpers ──────────────────────────────────────────────────────

/// Convert a kuva [`Color`] to a Typst color literal: `rgb("#aabbcc")` or
/// `rgb("#aabbccdd")` for alpha.
fn typst_color(c: &Color) -> String {
    match c {
        Color::Rgb(r, g, b) => format!("rgb(\"#{:02x}{:02x}{:02x}\")", r, g, b),
        Color::None => "none".to_string(),
        Color::Css(s) => typst_color_named(s),
    }
}

/// Convert an arbitrary CSS color string to a Typst color expression.
///
/// Typst's `rgb()` constructor accepts hex strings only; bare named colors
/// are identifiers in Typst's standard library. For unrecognised names we
/// fall back to `black` rather than emit a parse error.
fn typst_color_named(s: &str) -> String {
    if s == "none" || s.is_empty() {
        return "none".to_string();
    }
    if s.starts_with('#') {
        return format!("rgb(\"{}\")", s);
    }
    // Typst's standard named colors (matches the CSS basic palette).
    match s.to_lowercase().as_str() {
        "black" | "gray" | "silver" | "white" | "navy" | "blue" | "aqua" | "teal" | "eastern"
        | "purple" | "fuchsia" | "maroon" | "red" | "orange" | "yellow" | "olive" | "green"
        | "lime" => s.to_lowercase(),
        _ => "black".to_string(),
    }
}

/// Extract the primary font family name from a comma-separated CSS-style
/// `font-family` value: `"DejaVu Sans, Liberation Sans, Arial"` → `"DejaVu Sans"`.
fn primary_font(s: &str) -> String {
    s.split(',')
        .next()
        .unwrap_or(s)
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

/// Flip a y coordinate from SVG-space (top-left origin, y down) to CETZ-space
/// (bottom-left origin, y up).
#[inline]
fn flip_y(y: f64, scene_h: f64) -> f64 {
    scene_h - y
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::color::Color;
    use crate::render::render::{Primitive, Scene, TextAnchor};

    fn empty_scene(w: f64, h: f64) -> Scene {
        Scene {
            width: w,
            height: h,
            background_color: Some("white".into()),
            text_color: None,
            font_family: Some("DejaVu Sans, Arial".into()),
            elements: Vec::new(),
            defs: Vec::new(),
            has_tooltips: false,
            interactive: false,
            axis_meta: None,
            scripts: Vec::new(),
        }
    }

    #[test]
    fn empty_scene_emits_valid_preamble() {
        let s = empty_scene(800.0, 600.0);
        let typst = TypstBackend::default().render_scene(&s);
        assert!(typst.contains("#set page(width: 800pt, height: 600pt"));
        assert!(typst.contains("margin: 0pt"));
        assert!(typst.contains("#import \"@preview/cetz:"));
        assert!(typst.contains("#cetz.canvas("));
        assert!(typst.contains("})\n"));
    }

    #[test]
    fn circle_primitive_emits_cetz_circle() {
        let mut s = empty_scene(100.0, 100.0);
        s.elements.push(Primitive::Circle {
            cx: 50.0,
            cy: 50.0,
            r: 5.0,
            fill: Color::Rgb(0, 0, 255),
            fill_opacity: None,
            stroke: None,
            stroke_width: None,
        });
        let typst = TypstBackend::default().render_scene(&s);
        // y flipped: 100 - 50 = 50 (symmetric case).
        assert!(typst.contains("circle((50, 50), radius: 5pt"));
        assert!(typst.contains("fill: rgb(\"#0000ff\")"));
        assert!(typst.contains("stroke: none"));
    }

    #[test]
    fn line_primitive_emits_cetz_line_with_y_flip() {
        let mut s = empty_scene(200.0, 100.0);
        s.elements.push(Primitive::Line {
            x1: 0.0,
            y1: 10.0,
            x2: 200.0,
            y2: 10.0,
            stroke: Color::Rgb(0, 0, 0),
            stroke_width: 1.0,
            stroke_dasharray: None,
        });
        let typst = TypstBackend::default().render_scene(&s);
        // y=10 → flipped = 100-10 = 90.
        assert!(typst.contains("line((0, 90), (200, 90)"));
        assert!(typst.contains("stroke: 1pt + rgb(\"#000000\")"));
    }

    #[test]
    fn text_primitive_emits_cetz_content() {
        let mut s = empty_scene(200.0, 100.0);
        s.elements.push(Primitive::Text {
            x: 50.0,
            y: 80.0,
            content: "hello".into(),
            size: 14,
            anchor: TextAnchor::Middle,
            rotate: None,
            bold: false,
            color: None,
        });
        let typst = TypstBackend::default().render_scene(&s);
        assert!(typst.contains("content((50, 20)"));
        assert!(typst.contains("text(size: 14pt)[hello]"));
        assert!(typst.contains("anchor: \"center\""));
    }

    #[test]
    fn text_with_typst_special_chars_is_escaped() {
        let mut s = empty_scene(200.0, 100.0);
        s.elements.push(Primitive::Text {
            x: 10.0,
            y: 10.0,
            content: "Price: $5 [USD] #1".into(),
            size: 12,
            anchor: TextAnchor::Start,
            rotate: None,
            bold: false,
            color: None,
        });
        let typst = TypstBackend::default().render_scene(&s);
        // $, [, ], #, and @ must be backslash-escaped to render literally.
        assert!(typst.contains("Price: \\$5 \\[USD\\] \\#1"));
    }

    #[test]
    fn rect_primitive_emits_cetz_rect() {
        let mut s = empty_scene(200.0, 100.0);
        s.elements.push(Primitive::Rect {
            x: 10.0,
            y: 20.0,
            width: 50.0,
            height: 30.0,
            fill: Color::Rgb(255, 0, 0),
            stroke: None,
            stroke_width: None,
            opacity: None,
        });
        let typst = TypstBackend::default().render_scene(&s);
        // y=20, h=30; top=flip(20)=80, bottom=flip(50)=50.
        assert!(typst.contains("rect((10, 50), (60, 80)"));
        assert!(typst.contains("fill: rgb(\"#ff0000\")"));
    }

    #[test]
    fn rotation_sign_is_flipped() {
        // SVG rotation is clockwise positive; CETZ is counter-clockwise.
        let mut s = empty_scene(100.0, 100.0);
        s.elements.push(Primitive::Text {
            x: 50.0,
            y: 50.0,
            content: "rot".into(),
            size: 12,
            anchor: TextAnchor::Middle,
            rotate: Some(-90.0),
            bold: false,
            color: None,
        });
        let typst = TypstBackend::default().render_scene(&s);
        assert!(typst.contains("angle: 90deg"));
    }
}
