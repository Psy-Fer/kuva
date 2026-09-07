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
//! `pdf` feature ([`crate::render::math`]), which links the typst compiler
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
//! # Path and clip support
//!
//! - `Path` primitives (SVG path data: arrowheads, chord ribbons, sankey
//!   flows, venn outlines) are lowered to CETZ `merge-path` calls — lines
//!   and cubic Béziers, with quadratics promoted and arcs converted via the
//!   standard endpoint→center parameterisation.
//! - Clip regions (`ClipStart`/`ClipEnd`) split the output into stacked
//!   `#place`d canvases; a clipped chunk is wrapped in Typst's
//!   `box(clip: true)` at the clip rectangle.
//!
//! # Limitations
//!
//! - Batched primitives (`CircleBatch`, `RectBatch`) unroll to one CETZ call
//!   per element. Acceptable for typical plot sizes; can be optimized later.
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

        let h = scene.height;

        // Split the element stream into chunks at clip boundaries. A scene
        // without clip regions stays a single chunk and renders as one plain
        // canvas (the common case). With clips, each chunk becomes its own
        // `#place`d canvas so that a clipped chunk can be wrapped in a
        // `box(clip: true)` — Typst's clipping primitive — while later
        // chunks still stack above it in paint order.
        let chunks = split_clip_chunks(&scene.elements);
        let single = chunks.len() == 1 && chunks[0].0.is_none();

        // An empty scene still emits one (empty) canvas so the document is a
        // valid, non-degenerate Typst file.
        if single && chunks[0].1.is_empty() {
            open_canvas(&mut out, scene.width, h);
            out.push_str("})\n");
            return out;
        }

        for (clip, elems) in &chunks {
            if elems.is_empty() {
                continue;
            }
            match clip {
                None => {
                    if !single {
                        out.push_str("#place(top + left, dx: 0pt, dy: 0pt)[");
                    }
                    open_canvas(&mut out, scene.width, h);
                    for p in elems {
                        emit_primitive(&mut out, p, h);
                    }
                    out.push_str("})");
                    if !single {
                        out.push(']');
                    }
                    out.push('\n');
                }
                Some(r) => {
                    // Clip box at the region's page position; inside it, the
                    // canvas keeps page-absolute coordinates and is shifted
                    // back by the box offset so geometry lands where the SVG
                    // backend's <clipPath> would put it.
                    let _ = write!(
                        out,
                        "#place(top + left, dx: {}pt, dy: {}pt)[#box(width: {}pt, height: {}pt, clip: true)[#place(top + left, dx: {}pt, dy: {}pt)[",
                        r.x, r.y, r.width, r.height, -r.x, -r.y
                    );
                    open_canvas(&mut out, scene.width, h);
                    for p in elems {
                        emit_primitive(&mut out, p, h);
                    }
                    out.push_str("})]]]\n");
                }
            }
        }
        out
    }
}

/// Rectangular clip region carried between `ClipStart`/`ClipEnd`.
struct ClipRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// Split the primitive stream into `(clip, elements)` chunks at clip
/// boundaries. Kuva emits flat (non-nested) clip regions — the plot-area
/// clip around data marks; nested `ClipStart`s are intersected defensively.
fn split_clip_chunks(elements: &[Primitive]) -> Vec<(Option<ClipRect>, Vec<&Primitive>)> {
    let mut chunks: Vec<(Option<ClipRect>, Vec<&Primitive>)> = vec![(None, Vec::new())];
    let mut stack: Vec<ClipRect> = Vec::new();
    for p in elements {
        match p {
            Primitive::ClipStart {
                x,
                y,
                width,
                height,
                id: _,
            } => {
                let r = match stack.last() {
                    // Nested clip: intersect with the enclosing rect.
                    Some(outer) => {
                        let x0 = x.max(outer.x);
                        let y0 = y.max(outer.y);
                        let x1 = (x + width).min(outer.x + outer.width);
                        let y1 = (y + height).min(outer.y + outer.height);
                        ClipRect {
                            x: x0,
                            y: y0,
                            width: (x1 - x0).max(0.0),
                            height: (y1 - y0).max(0.0),
                        }
                    }
                    None => ClipRect {
                        x: *x,
                        y: *y,
                        width: *width,
                        height: *height,
                    },
                };
                chunks.push((
                    Some(ClipRect {
                        x: r.x,
                        y: r.y,
                        width: r.width,
                        height: r.height,
                    }),
                    Vec::new(),
                ));
                stack.push(r);
            }
            Primitive::ClipEnd => {
                stack.pop();
                match stack.last() {
                    None => chunks.push((None, Vec::new())),
                    Some(outer) => chunks.push((
                        Some(ClipRect {
                            x: outer.x,
                            y: outer.y,
                            width: outer.width,
                            height: outer.height,
                        }),
                        Vec::new(),
                    )),
                }
            }
            other => chunks.last_mut().unwrap().1.push(other),
        }
    }
    chunks
}

/// Open a `cetz.canvas` block. The invisible full-page rect anchors the
/// canvas's bounding box to the page extent, so drawings keep their absolute
/// positions even when nothing touches the page corners.
fn open_canvas(out: &mut String, w: f64, h: f64) {
    let _ = write!(
        out,
        "#cetz.canvas(length: 1pt, {{\n  import cetz.draw: *\n  rect((0, 0), ({w}, {h}), stroke: none)\n"
    );
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

        Primitive::Path(data) => emit_path(out, data, scene_h),

        // Grouping carries no visual state of its own; clipping is handled
        // structurally in `render_scene` (chunks + `box(clip: true)`), so
        // both marker pairs are no-ops at the per-primitive level.
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

// ── Path primitive ────────────────────────────────────────────────────────────
//
// Kuva's `PathData.d` is SVG path data, but from a closed vocabulary: the
// render layer only ever emits absolute `M`, `L`, `C`, `Q`, `A` (circular,
// unrotated), and `Z` — see `build_path` and the arrowhead/chord/sankey
// emitters. CETZ has no SVG-path element, so each subpath is lowered to a
// `merge-path` of `line`/`bezier` calls: quadratics are promoted to cubics
// and arcs are converted (endpoint → center parameterisation, then split
// into ≤90° cubic approximations with the standard `4/3·tan(Δ/4)` factor).

/// One parsed segment of SVG path data (absolute coordinates).
enum PathSeg {
    Move(f64, f64),
    LineTo(f64, f64),
    Cubic([f64; 6]),
    Quad([f64; 4]),
    /// rx, ry, large-arc flag, sweep flag, end x, end y (x-rotation is always
    /// 0 in kuva's output and is ignored).
    Arc(f64, f64, bool, bool, f64, f64),
    Close,
}

/// Parse kuva-emitted SVG path data. Unknown/relative commands end parsing
/// gracefully (returns what was read so far) rather than panicking — the
/// backend then draws the prefix, which is still better than dropping the
/// whole path.
fn parse_svg_path(d: &str) -> Vec<PathSeg> {
    let b = d.as_bytes();
    let mut i = 0usize;
    let mut cmd = 0u8;
    let mut nums: Vec<f64> = Vec::with_capacity(8);
    let mut segs = Vec::new();
    let mut first_pair_done = false;
    while i < b.len() {
        let c = b[i];
        if c.is_ascii_alphabetic() {
            match c {
                b'M' | b'L' | b'C' | b'Q' | b'A' => {
                    cmd = c;
                    first_pair_done = false;
                    nums.clear();
                }
                b'Z' | b'z' => segs.push(PathSeg::Close),
                _ => return segs, // relative/unsupported command: stop here
            }
            i += 1;
            continue;
        }
        if c == b' ' || c == b',' || c == b'\t' || c == b'\n' {
            i += 1;
            continue;
        }
        // Read one float.
        let start = i;
        i += 1;
        while i < b.len() {
            let ch = b[i];
            let numeric = ch.is_ascii_digit() || ch == b'.' || ch == b'e' || ch == b'E';
            let signed_exp = (ch == b'-' || ch == b'+') && (b[i - 1] == b'e' || b[i - 1] == b'E');
            if numeric || signed_exp {
                i += 1;
            } else {
                break;
            }
        }
        match d[start..i].parse::<f64>() {
            Ok(v) => nums.push(v),
            Err(_) => return segs,
        }
        let need = match cmd {
            b'M' | b'L' => 2,
            b'C' => 6,
            b'Q' => 4,
            b'A' => 7,
            _ => return segs, // number before any command
        };
        if nums.len() == need {
            match cmd {
                b'M' => {
                    if first_pair_done {
                        // Extra pairs after an M are implicit line-tos.
                        segs.push(PathSeg::LineTo(nums[0], nums[1]));
                    } else {
                        segs.push(PathSeg::Move(nums[0], nums[1]));
                        first_pair_done = true;
                    }
                }
                b'L' => segs.push(PathSeg::LineTo(nums[0], nums[1])),
                b'C' => segs.push(PathSeg::Cubic([
                    nums[0], nums[1], nums[2], nums[3], nums[4], nums[5],
                ])),
                b'Q' => segs.push(PathSeg::Quad([nums[0], nums[1], nums[2], nums[3]])),
                b'A' => segs.push(PathSeg::Arc(
                    nums[0],
                    nums[1],
                    nums[3] != 0.0,
                    nums[4] != 0.0,
                    nums[5],
                    nums[6],
                )),
                _ => {}
            }
            nums.clear();
        }
    }
    segs
}

/// Convert one SVG arc (endpoint parameterisation, x-rotation 0) starting at
/// `(x1, y1)` into cubic Bézier segments, appended as `[c1x, c1y, c2x, c2y,
/// x, y]` tuples. Follows the SVG spec's F.6.5/F.6.6 conversion.
#[allow(clippy::too_many_arguments)]
fn arc_to_cubics(
    x1: f64,
    y1: f64,
    rx: f64,
    ry: f64,
    large: bool,
    sweep: bool,
    x2: f64,
    y2: f64,
    out: &mut Vec<[f64; 6]>,
) {
    let (mut rx, mut ry) = (rx.abs(), ry.abs());
    if rx == 0.0 || ry == 0.0 || (x1 == x2 && y1 == y2) {
        out.push([x1, y1, x2, y2, x2, y2]); // degenerate: straight line
        return;
    }
    let x1p = (x1 - x2) / 2.0;
    let y1p = (y1 - y2) / 2.0;
    // Scale radii up if the endpoints can't be connected at the given size.
    let lambda = (x1p * x1p) / (rx * rx) + (y1p * y1p) / (ry * ry);
    if lambda > 1.0 {
        let s = lambda.sqrt();
        rx *= s;
        ry *= s;
    }
    let num = (rx * rx) * (ry * ry) - (rx * rx) * (y1p * y1p) - (ry * ry) * (x1p * x1p);
    let den = (rx * rx) * (y1p * y1p) + (ry * ry) * (x1p * x1p);
    let mut co = (num.max(0.0) / den).sqrt();
    if large == sweep {
        co = -co;
    }
    let cxp = co * rx * y1p / ry;
    let cyp = -co * ry * x1p / rx;
    let cx = cxp + (x1 + x2) / 2.0;
    let cy = cyp + (y1 + y2) / 2.0;

    let ang = |ux: f64, uy: f64, vx: f64, vy: f64| (ux * vy - uy * vx).atan2(ux * vx + uy * vy);
    let theta1 = ang(1.0, 0.0, (x1p - cxp) / rx, (y1p - cyp) / ry);
    let mut dtheta = ang(
        (x1p - cxp) / rx,
        (y1p - cyp) / ry,
        (-x1p - cxp) / rx,
        (-y1p - cyp) / ry,
    );
    if !sweep && dtheta > 0.0 {
        dtheta -= std::f64::consts::TAU;
    } else if sweep && dtheta < 0.0 {
        dtheta += std::f64::consts::TAU;
    }

    let n = (dtheta.abs() / std::f64::consts::FRAC_PI_2).ceil().max(1.0) as usize;
    let step = dtheta / n as f64;
    let k = 4.0 / 3.0 * (step / 4.0).tan();
    let point = |a: f64| (cx + rx * a.cos(), cy + ry * a.sin());
    let deriv = |a: f64| (-rx * a.sin(), ry * a.cos());
    let mut a1 = theta1;
    for _ in 0..n {
        let a2 = a1 + step;
        let (px1, py1) = point(a1);
        let (px2, py2) = point(a2);
        let (dx1, dy1) = deriv(a1);
        let (dx2, dy2) = deriv(a2);
        out.push([
            px1 + k * dx1,
            py1 + k * dy1,
            px2 - k * dx2,
            py2 - k * dy2,
            px2,
            py2,
        ]);
        a1 = a2;
    }
}

/// Emit a `Primitive::Path` as one CETZ `merge-path` per subpath.
fn emit_path(out: &mut String, data: &crate::render::render::PathData, scene_h: f64) {
    let segs = parse_svg_path(&data.d);
    if segs.is_empty() {
        return;
    }

    // Style arguments shared by every subpath.
    let mut style = String::new();
    match &data.fill {
        Some(f) => {
            let _ = write!(style, "fill: {}", paint(f, data.opacity));
        }
        None => style.push_str("fill: none"),
    }
    if data.stroke_width > 0.0 && !matches!(data.stroke, Color::None) {
        match &data.stroke_dasharray {
            Some(dash) => {
                let _ = write!(
                    style,
                    ", stroke: (thickness: {}pt, paint: {}, dash: ({}))",
                    data.stroke_width,
                    paint(&data.stroke, data.opacity),
                    dash_lengths(dash)
                );
            }
            None => {
                let _ = write!(
                    style,
                    ", stroke: {}pt + {}",
                    data.stroke_width,
                    paint(&data.stroke, data.opacity)
                );
            }
        }
    } else {
        style.push_str(", stroke: none");
    }

    // Walk the segments, flushing one merge-path per subpath.
    let mut body = String::new();
    let mut cur = (0.0f64, 0.0f64);
    let mut start = cur;
    let mut closed = false;
    let flush = |body: &mut String, closed: bool, out: &mut String| {
        if body.is_empty() {
            return;
        }
        let _ = writeln!(out, "  merge-path({style}, close: {closed}, {{");
        out.push_str(body);
        out.push_str("  })\n");
        body.clear();
    };
    let mut cubics: Vec<[f64; 6]> = Vec::new();
    for seg in &segs {
        match seg {
            PathSeg::Move(x, y) => {
                flush(&mut body, closed, out);
                closed = false;
                cur = (*x, *y);
                start = cur;
            }
            PathSeg::LineTo(x, y) => {
                let _ = writeln!(
                    body,
                    "    line(({}, {}), ({}, {}))",
                    r2(cur.0),
                    r2(flip_y(cur.1, scene_h)),
                    r2(*x),
                    r2(flip_y(*y, scene_h))
                );
                cur = (*x, *y);
            }
            PathSeg::Cubic([c1x, c1y, c2x, c2y, x, y]) => {
                write_bezier(
                    &mut body,
                    cur,
                    (*c1x, *c1y),
                    (*c2x, *c2y),
                    (*x, *y),
                    scene_h,
                );
                cur = (*x, *y);
            }
            PathSeg::Quad([qx, qy, x, y]) => {
                // Promote to cubic: c1 = p + 2/3 (q - p), c2 = e + 2/3 (q - e).
                let c1 = (
                    cur.0 + 2.0 / 3.0 * (qx - cur.0),
                    cur.1 + 2.0 / 3.0 * (qy - cur.1),
                );
                let c2 = (x + 2.0 / 3.0 * (qx - x), y + 2.0 / 3.0 * (qy - y));
                write_bezier(&mut body, cur, c1, c2, (*x, *y), scene_h);
                cur = (*x, *y);
            }
            PathSeg::Arc(rx, ry, large, sweep, x, y) => {
                cubics.clear();
                arc_to_cubics(cur.0, cur.1, *rx, *ry, *large, *sweep, *x, *y, &mut cubics);
                for [c1x, c1y, c2x, c2y, ex, ey] in &cubics {
                    write_bezier(
                        &mut body,
                        cur,
                        (*c1x, *c1y),
                        (*c2x, *c2y),
                        (*ex, *ey),
                        scene_h,
                    );
                    cur = (*ex, *ey);
                }
            }
            PathSeg::Close => {
                closed = true;
                cur = start;
            }
        }
    }
    flush(&mut body, closed, out);
}

/// Write one CETZ cubic `bezier(start, end, ctrl1, ctrl2)` call with y-flip.
fn write_bezier(
    body: &mut String,
    from: (f64, f64),
    c1: (f64, f64),
    c2: (f64, f64),
    to: (f64, f64),
    scene_h: f64,
) {
    let _ = writeln!(
        body,
        "    bezier(({}, {}), ({}, {}), ({}, {}), ({}, {}))",
        r2(from.0),
        r2(flip_y(from.1, scene_h)),
        r2(to.0),
        r2(flip_y(to.1, scene_h)),
        r2(c1.0),
        r2(flip_y(c1.1, scene_h)),
        r2(c2.0),
        r2(flip_y(c2.1, scene_h))
    );
}

/// A Typst paint expression for a color with optional SVG-style opacity.
fn paint(c: &Color, opacity: Option<f64>) -> String {
    let base = typst_color(c);
    match opacity {
        Some(o) if o < 1.0 && base != "none" => {
            format!("{}.transparentize({}%)", base, r2((1.0 - o) * 100.0))
        }
        _ => base,
    }
}

/// Convert an SVG `stroke-dasharray` value (`"4 3"`, `"2,2"`) to a Typst
/// dash-pattern tuple body (`"4pt, 3pt"`).
fn dash_lengths(dasharray: &str) -> String {
    let parts: Vec<String> = dasharray
        .split([' ', ','])
        .filter(|s| !s.is_empty())
        .map(|s| format!("{s}pt"))
        .collect();
    // A one-element SVG dasharray means equal on/off runs; Typst tuples need
    // two entries to mean the same thing.
    if parts.len() == 1 {
        format!("{}, {}", parts[0], parts[0])
    } else {
        parts.join(", ")
    }
}

/// Round to 2 decimals for compact output (matches the SVG backend's floats).
#[inline]
fn r2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
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
