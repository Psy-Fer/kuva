//! Rendering for [`crate::plot::logo::SequenceLogo`].
//!
//! The SVG/PNG/PDF path emits stacked glyph outlines: bold contours
//! extracted with `ttf-parser`, converted to SVG path `d` strings,
//! and mapped onto each (position × probability-fraction) cell.  Letters
//! therefore stretch non-uniformly on the y-axis, matching the
//! Schneider / Stephens convention.
//!
//! The terminal path (active when `layout.term_rows` is set) emits one
//! colored letter per character cell instead — a column of stacked cells
//! per position, cell count proportional to each letter's contribution —
//! because a char grid cannot stretch a glyph.

use std::collections::HashMap;
use std::sync::OnceLock;

use ttf_parser::{Face, GlyphId, OutlineBuilder};

use crate::plot::logo::SequenceLogo;
use crate::render::color::Color;
use crate::render::layout::{ComputedLayout, Layout};
use crate::render::render::{PathData, Primitive, Scene, TextAnchor};
use crate::render::text_metrics::{ascent, line_height, FontStyle};

/// Cached glyph outlines for one typeface, keyed by character.
struct GlyphCache {
    outlines: HashMap<char, GlyphOutline>,
}

struct GlyphOutline {
    /// Path `d` string in font-em units (y-up).
    path_em: String,
    /// X-range covered by the outline (font-em units).
    ///
    /// Using the outline bbox rather than the advance width strips the
    /// font's side bearings, so a stretched glyph fills its column edge
    /// to edge — the canonical WebLogo look.
    x_min_em: f64,
    x_max_em: f64,
    /// Y-range covered by the outline (font-em units, y-up).
    y_min_em: f64,
    y_max_em: f64,
}

fn build_cache(bytes: &[u8]) -> GlyphCache {
    let face = Face::parse(bytes, 0).expect("bundled logo font TTF failed to parse");
    // Precompute A-Z, a-z, digits, and gap/wildcard letters so any common
    // alphabet is covered in one pass.
    let candidates: Vec<char> = ('A'..='Z')
        .chain('a'..='z')
        .chain('0'..='9')
        .chain(['-', '.', '*', '?'])
        .collect();
    let outlines = candidates
        .into_iter()
        .filter_map(|c| build_glyph(&face, c).map(|o| (c, o)))
        .collect();
    GlyphCache { outlines }
}

fn glyph_cache() -> &'static GlyphCache {
    static CACHE: OnceLock<GlyphCache> = OnceLock::new();
    CACHE.get_or_init(|| build_cache(crate::fonts::logo_font()))
}

fn build_glyph(face: &Face, c: char) -> Option<GlyphOutline> {
    let glyph_id: GlyphId = face.glyph_index(c)?;
    let mut builder = PathBuilder::default();
    let bbox = face.outline_glyph(glyph_id, &mut builder)?;
    Some(GlyphOutline {
        path_em: builder.path,
        x_min_em: bbox.x_min as f64,
        x_max_em: bbox.x_max as f64,
        y_min_em: bbox.y_min as f64,
        y_max_em: bbox.y_max as f64,
    })
}

/// Builds an SVG path `d` string from a glyph outline, emitting only
/// `M` / `L` / `C` / `Z`.
///
/// TrueType outlines are quadratic, but kuva's rasteriser understands only
/// `m l h v c a z` — a `Q` command is silently dropped, which makes every
/// curved letter (C, G, O, S…) vanish from PNG output while straight-line
/// letters (A, T) survive. Every quadratic is therefore degree-elevated to
/// its exact cubic equivalent here, so all backends see the same commands
/// and render identically.
#[derive(Default)]
struct PathBuilder {
    path: String,
    /// Current pen position — the quadratic's implicit start point.
    cx: f32,
    cy: f32,
    /// Start of the current subpath, restored by `close`.
    start: (f32, f32),
}

impl OutlineBuilder for PathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        use std::fmt::Write;
        let _ = write!(self.path, "M{:.2} {:.2}", x, y);
        self.cx = x;
        self.cy = y;
        self.start = (x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        use std::fmt::Write;
        let _ = write!(self.path, "L{:.2} {:.2}", x, y);
        self.cx = x;
        self.cy = y;
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        use std::fmt::Write;
        // Exact degree elevation of a quadratic (P0, P1, P2) to a cubic:
        //   C1 = P0 + ⅔(P1 − P0)      C2 = P2 + ⅔(P1 − P2)
        // The curve is identical, not an approximation.
        const T: f32 = 2.0 / 3.0;
        let c1x = self.cx + T * (x1 - self.cx);
        let c1y = self.cy + T * (y1 - self.cy);
        let c2x = x + T * (x1 - x);
        let c2y = y + T * (y1 - y);
        let _ = write!(
            self.path,
            "C{:.2} {:.2} {:.2} {:.2} {:.2} {:.2}",
            c1x, c1y, c2x, c2y, x, y
        );
        self.cx = x;
        self.cy = y;
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        use std::fmt::Write;
        let _ = write!(
            self.path,
            "C{:.2} {:.2} {:.2} {:.2} {:.2} {:.2}",
            x1, y1, x2, y2, x, y
        );
        self.cx = x;
        self.cy = y;
    }
    fn close(&mut self) {
        self.path.push('Z');
        self.cx = self.start.0;
        self.cy = self.start.1;
    }
}

/// Em-space bounding box of a glyph outline.
#[derive(Clone, Copy)]
struct EmBox {
    x_min: f64,
    y_min: f64,
    x_max: f64,
    y_max: f64,
}

/// Pixel-space target rectangle.
#[derive(Clone, Copy)]
struct PxRect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

/// Transform the em-space path `d` string into a target pixel rectangle.
///
/// The font's y-axis is up (typical TTF); the plot's y-axis is down.  The
/// affine maps the glyph's em-space bounding box `em` into pixel-space
/// rectangle `dst`, flipping y in the process.
fn transform_path(path_em: &str, em: EmBox, dst: PxRect) -> String {
    let x_min_em = em.x_min;
    let y_min_em = em.y_min;
    let x_max_em = em.x_max;
    let y_max_em = em.y_max;
    let px = dst.x;
    let py = dst.y;
    let pw = dst.w;
    let ph = dst.h;
    let dx = (x_max_em - x_min_em).max(1.0);
    let dy = (y_max_em - y_min_em).max(1.0);
    let sx = pw / dx;
    let sy = ph / dy;
    let mut out = String::with_capacity(path_em.len());
    let mut i = 0;
    let bytes = path_em.as_bytes();
    let mut nums: Vec<f64> = Vec::with_capacity(6);
    // Simple SVG-path lexer: commands are ASCII letters; numbers are
    // whitespace-separated floats.  Output re-encodes each command with the
    // transformed coordinates.
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c.is_ascii_alphabetic() {
            out.push(c);
            i += 1;
            nums.clear();
            while i < bytes.len() {
                // Skip separators.
                while i < bytes.len() && matches!(bytes[i] as char, ' ' | ',' | '\t') {
                    i += 1;
                }
                if i >= bytes.len() {
                    break;
                }
                let nb = bytes[i] as char;
                if nb.is_ascii_alphabetic() {
                    break;
                }
                // Parse a float.
                let start = i;
                if bytes[i] == b'-' || bytes[i] == b'+' {
                    i += 1;
                }
                while i < bytes.len()
                    && (bytes[i].is_ascii_digit() || bytes[i] == b'.' || bytes[i] == b'-')
                {
                    // In practice em-space numbers here have at most one '.' and no
                    // exponents; a leading '-' after digits would be a separator,
                    // so treat that as end-of-number.
                    if bytes[i] == b'-' && i != start {
                        break;
                    }
                    i += 1;
                }
                let s = &path_em[start..i];
                if let Ok(v) = s.parse::<f64>() {
                    nums.push(v);
                }
            }
            // Emit transformed pairs.
            for pair in nums.chunks_exact(2) {
                let ex = pair[0];
                let ey = pair[1];
                let ox = px + (ex - x_min_em) * sx;
                let oy = py + ph - (ey - y_min_em) * sy;
                use std::fmt::Write;
                let _ = write!(out, "{:.2} {:.2} ", ox, oy);
            }
        } else {
            i += 1;
        }
    }
    out
}

/// Horizontal breathing room per column, as a fraction of column width.
///
/// Letters otherwise fill their column edge to edge — the classic WebLogo
/// look — and neighbouring columns would touch.
const COLUMN_INSET: f64 = 0.03;

/// Display positions in draw order, paired with the source position each
/// draws from.  Reverse-complement walks the matrix backwards.
fn source_indices(n: usize, reverse: bool) -> Vec<usize> {
    if reverse {
        (0..n).rev().collect()
    } else {
        (0..n).collect()
    }
}

/// Render one sequence logo into `scene`.
///
/// Only the letter stack is drawn here, inside the plot-area clip that
/// `render_multiple` wraps every plot in. The consensus row sits below the
/// x-axis and is emitted separately via [`add_consensus_row`] after the clip
/// closes — inside it, the row is silently clipped away.
pub fn add_sequence_logo(
    sl: &SequenceLogo,
    scene: &mut Scene,
    computed: &ComputedLayout,
    layout: &Layout,
) {
    let n = sl.n_positions();
    if n == 0 {
        return;
    }

    if let Some(rows) = layout.term_rows {
        add_terminal_stack(sl, scene, computed, rows as usize);
    } else {
        add_glyph_stack(sl, scene, computed);
    }
}

/// Vector path rendering — SVG / PNG / PDF.
fn add_glyph_stack(sl: &SequenceLogo, scene: &mut Scene, computed: &ComputedLayout) {
    let cache = glyph_cache();
    let n = sl.n_positions();

    for (pos_idx, source_idx) in source_indices(n, sl.reverse_complement)
        .into_iter()
        .enumerate()
    {
        let stack = sl.stack(source_idx);
        if stack.is_empty() {
            continue;
        }

        // Column in physical coords: centered at pos+1 (positions are
        // 1-indexed internally, --start shifts axis labels only).
        let col_left = computed.map_x(pos_idx as f64 + 0.5);
        let col_right = computed.map_x(pos_idx as f64 + 1.5);
        let col_w = (col_right - col_left).abs();

        let mut cursor = 0.0_f64;
        for (letter, height) in stack.iter().filter(|(_, h)| *h > 0.0) {
            let display_letter = if sl.reverse_complement {
                sl.alphabet.complement(*letter)
            } else {
                *letter
            };

            let bottom = cursor;
            let top = cursor + *height;
            cursor = top;

            let y_top = computed.map_y(top);
            let y_bot = computed.map_y(bottom);
            let cell_h = (y_bot - y_top).abs();
            // A degenerate layout (e.g. one built from an empty plot list)
            // yields non-finite coordinates; skip rather than writing NaN
            // into the path data.
            if !cell_h.is_finite() || !y_top.is_finite() || !col_left.is_finite() || cell_h < 0.5 {
                continue;
            }

            let color = sl.palette.color_of(display_letter);
            let color_str = format!("rgb({},{},{})", color[0], color[1], color[2]);

            if let Some(outline) = cache.outlines.get(&display_letter) {
                let em = EmBox {
                    x_min: outline.x_min_em,
                    y_min: outline.y_min_em,
                    x_max: outline.x_max_em,
                    y_max: outline.y_max_em,
                };
                // Each letter fills its column, less a hair of breathing room
                // so neighbouring columns don't touch.
                let inset = col_w * COLUMN_INSET;
                let dst = PxRect {
                    x: col_left + inset,
                    y: y_top,
                    w: col_w - 2.0 * inset,
                    h: cell_h,
                };
                let d = transform_path(&outline.path_em, em, dst);
                scene.add(Primitive::Path(Box::new(PathData {
                    d,
                    fill: Some(Color::from(color_str.as_str())),
                    stroke: Color::from("none"),
                    stroke_width: 0.0,
                    opacity: None,
                    stroke_dasharray: None,
                })));
            } else {
                // Fallback for characters missing from the font: a filled
                // rect + centered letter.
                scene.add(Primitive::Rect {
                    x: col_left,
                    y: y_top,
                    width: col_w,
                    height: cell_h,
                    fill: Color::from(color_str.as_str()),
                    stroke: None,
                    stroke_width: None,
                    opacity: Some(0.85),
                });
                scene.add(Primitive::Text {
                    x: col_left + col_w * 0.5,
                    y: y_top + cell_h * 0.75,
                    content: display_letter.to_string(),
                    size: cell_h.min(col_w * 1.2).max(6.0) as u32,
                    anchor: TextAnchor::Middle,
                    rotate: None,
                    bold: true,
                    color: Some(Color::from("white")),
                });
            }
        }
    }
}

/// Character-cell rendering — terminal backend.
///
/// One column of stacked cells per position; each letter occupies the cells
/// whose centers fall inside its pixel band, so cell count is proportional
/// to the letter's contribution.  Tallest letter naturally lands on top,
/// matching the vector rendering.
fn add_terminal_stack(
    sl: &SequenceLogo,
    scene: &mut Scene,
    computed: &ComputedLayout,
    term_rows: usize,
) {
    if term_rows == 0 {
        return;
    }
    let n = sl.n_positions();
    let cell_h = computed.height / term_rows as f64;
    // The terminal Text handler subtracts size*0.35 as a baseline offset;
    // compensate so each letter lands exactly on its cell's row.
    let font_size = 10_u32;
    let baseline = font_size as f64 * 0.35;
    // The row holding the x-axis line belongs to the axis (and its tick
    // labels); a cell whose centre sits just above the line still shares
    // that row, so stop one row short or the bottom of every stack
    // overprints the axis.
    let axis_row = ((computed.height - computed.margin_bottom) / cell_h).floor() as usize;

    for (pos_idx, source_idx) in source_indices(n, sl.reverse_complement)
        .into_iter()
        .enumerate()
    {
        let stack = sl.stack(source_idx);
        if stack.is_empty() {
            continue;
        }
        let x_center = computed.map_x(pos_idx as f64 + 1.0);

        let mut cursor = 0.0_f64;
        for (letter, height) in stack.iter().filter(|(_, h)| *h > 0.0) {
            let display_letter = if sl.reverse_complement {
                sl.alphabet.complement(*letter)
            } else {
                *letter
            };
            let y_top = computed.map_y(cursor + *height);
            let y_bot = computed.map_y(cursor);
            cursor += *height;

            let color = sl.palette.color_of(display_letter);
            let color_str = format!("rgb({},{},{})", color[0], color[1], color[2]);

            // Rows whose cell-center falls inside [y_top, y_bot).
            let r_first = ((y_top / cell_h) - 0.5).ceil().max(0.0) as usize;
            let r_last = (((y_bot / cell_h) - 0.5).floor()).max(0.0) as usize;
            for r in r_first..=r_last.min(term_rows.saturating_sub(1)) {
                if r >= axis_row {
                    continue;
                }
                let cy = (r as f64 + 0.5) * cell_h;
                if cy < y_top || cy >= y_bot {
                    continue;
                }
                scene.add(Primitive::Text {
                    x: x_center,
                    y: cy + baseline,
                    content: display_letter.to_string(),
                    size: font_size,
                    anchor: TextAnchor::Middle,
                    rotate: None,
                    bold: true,
                    color: Some(Color::from(color_str.as_str())),
                });
            }
        }
    }
}

/// Consensus letter centred under each column, one row below the x tick
/// labels. `Layout::below_ticks_rows` (set by `auto_from_plots`) reserves that
/// row in `margin_bottom` as `ComputedLayout::below_ticks_extra`, so the
/// x-axis title lands below it rather than on top of it. Must be called after
/// the plot-area `ClipEnd`.
pub(crate) fn add_consensus_row(sl: &SequenceLogo, scene: &mut Scene, computed: &ComputedLayout) {
    let n = sl.n_positions();
    // Same baseline rule as the tick labels in axis.rs, one reserved row
    // further down. A layout built without `auto_from_plots` reserves nothing;
    // fall back to a text line so the row still clears the tick labels.
    let tick_px = computed.tick_size as f64;
    let row_px = if computed.below_ticks_extra > 0.0 {
        computed.below_ticks_extra
    } else {
        line_height(tick_px, FontStyle::Regular)
    };
    let baseline = computed.height - computed.margin_bottom
        + computed.tick_mark_major
        + ascent(tick_px, FontStyle::Regular)
        + row_px;
    for (pos_idx, source_idx) in source_indices(n, sl.reverse_complement)
        .into_iter()
        .enumerate()
    {
        let Some((letter, prob)) = sl.consensus(source_idx) else {
            continue;
        };
        let letter = if sl.reverse_complement {
            sl.alphabet.complement(letter)
        } else {
            letter
        };
        let display = if prob >= sl.consensus_threshold {
            letter.to_ascii_uppercase()
        } else {
            letter.to_ascii_lowercase()
        };
        let x = computed.map_x(pos_idx as f64 + 1.0);
        scene.add(Primitive::Text {
            x,
            y: baseline,
            content: display.to_string(),
            size: computed.tick_size,
            anchor: TextAnchor::Middle,
            rotate: None,
            bold: prob >= sl.consensus_threshold,
            color: None,
        });
    }
}
