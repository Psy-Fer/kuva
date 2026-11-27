//! Rendering for [`crate::plot::logo::SequenceLogo`].
//!
//! The SVG/PNG/PDF path emits stacked glyph outlines: bold letter contours
//! baked into source (`render::logo_glyphs`), mapped onto each
//! (position × probability-fraction) cell.  Letters
//! therefore stretch non-uniformly on the y-axis, matching the
//! Schneider / Stephens convention.
//!
//! The terminal path (active when `layout.term_rows` is set) emits one
//! colored letter per character cell instead — a column of stacked cells
//! per position, cell count proportional to each letter's contribution —
//! because a char grid cannot stretch a glyph.

use crate::plot::logo::SequenceLogo;
use crate::render::color::Color;
use crate::render::layout::{ComputedLayout, Layout};
use crate::render::plots::Plot;
use crate::render::render::{PathData, Primitive, Scene, TextAnchor};
use crate::render::text_metrics::FontStyle;

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

            if let Some(outline) = crate::render::logo_glyphs::glyph(display_letter) {
                let em = EmBox {
                    x_min: outline.x_min,
                    y_min: outline.y_min,
                    x_max: outline.x_max,
                    y_max: outline.y_max,
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
                let d = transform_path(outline.path, em, dst);
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

    // The terminal backend draws text in the theme colour unless it sits in a
    // group carrying this marker; logo letters are coloured by residue.
    scene.add(Primitive::GroupStart {
        transform: None,
        title: None,
        extra_attrs: Some(crate::backend::terminal::TERM_TEXT_COLOR_ATTR.to_string()),
    });
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
    scene.add(Primitive::GroupEnd);
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
        computed.fonts.line_height(tick_px, FontStyle::Regular)
    };
    let baseline = computed.height - computed.margin_bottom
        + computed.tick_mark_major
        + computed.fonts.ascent(tick_px, FontStyle::Regular)
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

// ── Canvas sizing and wrapping ──────────────────────────────────────────────

/// Plot-area width per position that an auto-sized logo keeps.
pub(crate) const LOGO_MIN_COL_PX: f64 = 48.0;
/// Longest logo kept on one auto-widened line (~4000 px of plot area).
pub const AUTO_WRAP_MAX_POSITIONS: usize = 80;
/// Positions per row when a long logo wraps automatically.
pub const AUTO_WRAP_PER_LINE: usize = 50;
/// Plot-area height of one wrapped row when the layout sets no height.
const WRAP_ROW_PLOT_H: f64 = 200.0;
/// Gap between the plot area and a wrapped logo's legend.
const WRAP_LEGEND_GAP: f64 = 15.0;

/// Size a sequence logo's canvas at render time, where an unset width still
/// means "automatic" (an explicit `with_width` after `auto_from_plots` is
/// respected). Wraps the logo onto rows when `with_per_line` asks for it, or
/// when it is too long for one auto-widened line; otherwise widens the canvas
/// to keep [`LOGO_MIN_COL_PX`] per position. Terminal output is sized by the
/// character grid and is left alone.
/// Returns the finished scene when the logo wrapped; otherwise adjusts `layout`
/// in place and returns `None` so rendering continues normally.
pub(crate) fn prepare_logo(plots: &[Plot], layout: &mut Layout) -> Option<Scene> {
    let sl = plots.iter().find_map(|p| match p {
        Plot::SequenceLogo(sl) => Some(sl),
        _ => None,
    })?;
    if layout.term_rows.is_some() {
        return None;
    }
    let n = sl.n_positions();
    let per_line = sl.per_line.or_else(|| {
        (layout.width.is_none() && n > AUTO_WRAP_MAX_POSITIONS).then_some(AUTO_WRAP_PER_LINE)
    });
    if let Some(k) = per_line.filter(|&k| k < n) {
        if plots.len() == 1 {
            return Some(render_wrapped(sl, layout, k));
        }
        if sl.per_line.is_some() {
            eprintln!(
                "kuva logo: per-line wrapping applies only when the logo is the only plot; drawing one line"
            );
        }
    }
    if layout.width.is_none() {
        let plot_w = n as f64 * LOGO_MIN_COL_PX;
        if plot_w > 600.0 {
            // Fit the plot area exactly: measure the margins this layout gets.
            let probe = ComputedLayout::from_layout(layout);
            layout.width = Some(probe.margin_left + plot_w + probe.margin_right);
        }
    }
    None
}

/// Draw `sl` as stacked rows of `per_line` positions.
///
/// Each row is rendered by `render_multiple` from a copy of `layout` and placed
/// with a translate group, the way `Figure` places panels. Rows share the left
/// and right margins (so the axes line up), the y range and the tick count;
/// every row keeps `per_line` column slots, so the last, shorter row has the
/// same column width. The title and subtitle go on the first row, the x-axis
/// label on the last, and the legend is drawn once, beside the first row.
fn render_wrapped(sl: &SequenceLogo, layout: &Layout, per_line: usize) -> Scene {
    use crate::render::figure::clone_layout;
    use crate::render::render::{
        collect_legend_entries, render_legend_at_with_fonts, render_multiple,
    };
    use crate::render::render_utils::{auto_nice_range, auto_tick_count};

    let fonts = layout.fonts();
    let legend_width = layout.legend_width_with(&fonts);
    let n = sl.n_positions();
    let n_rows = n.div_ceil(per_line);
    let chunks = sl.chunks(per_line);

    // One legend for the whole logo, beside the first row.
    let legend_entries = if layout.show_legend {
        layout
            .legend_entries
            .clone()
            .unwrap_or_else(|| collect_legend_entries(&[Plot::SequenceLogo(sl.clone())]))
    } else {
        Vec::new()
    };
    let legend_w = if legend_entries.is_empty() {
        0.0
    } else {
        legend_width + WRAP_LEGEND_GAP
    };

    // Frequency mode has no fixed ceiling: pin every row to the whole logo's
    // tallest column so the rows share one scale.
    let y_max = match (sl.units, layout.y_axis_max) {
        (crate::plot::logo::LogoUnits::Frequency, None) => {
            let tallest = sl
                .matrix
                .iter()
                .map(|row| row.iter().sum::<f64>())
                .fold(0.0_f64, f64::max);
            Some(auto_nice_range(0.0, tallest.max(1.0), 5).1)
        }
        _ => layout.y_axis_max,
    };

    let row_layout = |r: usize, chunk: &SequenceLogo| -> Layout {
        let mut l = clone_layout(layout);
        let first = r == 0;
        let last = r + 1 == n_rows;
        l.x_categories = Some(
            (0..chunk.n_positions())
                .map(|i| format!("{}", chunk.start + i as i64))
                .collect(),
        );
        l.x_axis_min = Some(0.5);
        l.x_axis_max = Some(per_line as f64 + 0.5);
        l.x_range = (0.5, per_line as f64 + 0.5);
        if y_max.is_some() {
            l.y_axis_min = Some(0.0);
            l.y_axis_max = y_max;
        }
        if !first {
            l.title = None;
            l.subtitle = None;
        }
        if !last {
            l.x_label = None;
        }
        l.show_legend = false;
        l.legend_entries = None;
        l
    };

    // Probe each row's margins at a nominal size, then fix the shared ones.
    let probes: Vec<ComputedLayout> = chunks
        .iter()
        .enumerate()
        .map(|(r, c)| {
            let mut l = row_layout(r, c);
            l.width = Some(1000.0);
            l.height = Some(400.0);
            ComputedLayout::from_layout(&l)
        })
        .collect();
    let margin_left = probes.iter().map(|p| p.margin_left).fold(0.0, f64::max);
    let margin_right = probes.iter().map(|p| p.margin_right).fold(0.0, f64::max) + legend_w;
    let vertical: Vec<f64> = probes
        .iter()
        .map(|p| p.margin_top + p.margin_bottom)
        .collect();

    let width = layout
        .width
        .unwrap_or(margin_left + per_line as f64 * LOGO_MIN_COL_PX + margin_right);
    let plot_h = match layout.height {
        Some(h) => ((h - vertical.iter().sum::<f64>()) / n_rows as f64).max(60.0),
        None => WRAP_ROW_PLOT_H,
    };
    // Same y ticks on every row, whatever its title/label bands add.
    let ticks = if layout.ticks > 0 {
        layout.ticks
    } else {
        auto_tick_count(plot_h + vertical[n_rows / 2])
    };
    let row_heights: Vec<f64> = vertical.iter().map(|v| v + plot_h).collect();
    let total_h: f64 = row_heights.iter().sum();

    let mut master = Scene::new(width, total_h);

    let mut y = 0.0;
    for (r, chunk) in chunks.into_iter().enumerate() {
        let mut l = row_layout(r, &chunk);
        l.width = Some(width);
        l.height = Some(row_heights[r]);
        l.ticks = ticks;
        l.force_margin_left = Some(margin_left);
        l.force_margin_right = Some(margin_right);
        let row = render_multiple(vec![Plot::SequenceLogo(chunk)], l);
        if r == 0 {
            master.font_family = row.font_family.clone();
            master.background_color = row.background_color.clone();
            master.text_color = row.text_color.clone();
        }
        master.defs.extend(row.defs);
        master.scripts.extend(row.scripts);
        master.interactive |= row.interactive;
        for w in row.warnings {
            master.warn(w.code, w.message);
        }
        master.add(Primitive::GroupStart {
            transform: Some(format!("translate(0,{y})")),
            title: None,
            extra_attrs: None,
        });
        for e in row.elements {
            master.add(e);
        }
        master.add(Primitive::GroupEnd);
        y += row_heights[r];
    }

    if !legend_entries.is_empty() {
        let lx = width - margin_right + WRAP_LEGEND_GAP + 5.0;
        let ly = probes[0].margin_top;
        render_legend_at_with_fonts(
            &legend_entries,
            None,
            layout.legend_title.as_deref(),
            layout.legend_box,
            &mut master,
            lx,
            ly,
            legend_width,
            layout.body_size,
            &layout.theme,
            &fonts,
        );
    }
    master
}
