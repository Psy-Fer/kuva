//! Embed a typst-rendered label (whole-label SVG fragment) into the SVG
//! backend's output. Feature `math`.
//!
//! With whole-label rendering there is no per-segment layout: the entire
//! label was typeset by Typst into one fragment with a known width and
//! baseline. We only place it — anchor horizontally by its width, align its
//! baseline to the label's `y`, and wrap a rotation if requested.

use std::fmt::Write;

use crate::render::math::MathSvg;
use crate::render::render::TextAnchor;

/// Append the positioned math fragment to `svg`. `uid` must be unique per
/// embedded label within the document — it namespaces the fragment's element
/// IDs (see below).
pub fn embed_label(
    svg: &mut String,
    x: f64,
    y: f64,
    anchor: TextAnchor,
    rotate: Option<f64>,
    math: &MathSvg,
    uid: usize,
) {
    let anchor_shift = match anchor {
        TextAnchor::Start => 0.0,
        TextAnchor::Middle => -math.width_pt / 2.0,
        TextAnchor::End => -math.width_pt,
    };
    let tx = x + anchor_shift;
    let ty = y - math.baseline_offset_pt;

    // Keep a rotated label inside the viewBox: a tall math label (e.g. a
    // rotated y-axis title with a superscript) can have a cross-extent wider
    // than the reserved margin and would otherwise clip at the left/top edge.
    // Nudge it inward by the amount its rotated bbox crosses the origin.
    let (shift_x, shift_y) = match rotate {
        Some(angle) => {
            let rad = angle * std::f64::consts::PI / 180.0;
            let (c, s) = (rad.cos(), rad.sin());
            let corners = [
                (tx, ty),
                (tx + math.width_pt, ty),
                (tx + math.width_pt, ty + math.height_pt),
                (tx, ty + math.height_pt),
            ];
            let (mut min_x, mut min_y) = (f64::INFINITY, f64::INFINITY);
            for (px, py) in corners {
                let rx = x + c * (px - x) - s * (py - y);
                let ry = y + s * (px - x) + c * (py - y);
                min_x = min_x.min(rx);
                min_y = min_y.min(ry);
            }
            ((-min_x).max(0.0), (-min_y).max(0.0))
        }
        None => (0.0, 0.0),
    };
    let nudge = shift_x > 0.0 || shift_y > 0.0;

    if nudge {
        let _ = write!(svg, r#"<g transform="translate({shift_x},{shift_y})">"#);
    }
    if let Some(angle) = rotate {
        // Rotate about the label's anchor point (x, y), matching how the SVG
        // backend rotates plain <text>.
        let _ = write!(svg, r#"<g transform="rotate({angle},{x},{y})">"#);
    }
    // Each Typst fragment carries its own `id="glyph"` defs and content-hashed
    // `id="g…"` glyph symbols. With more than one math label per plot those IDs
    // collide across fragments → duplicate-ID (invalid) SVG. Namespace every ID
    // and local href target with a per-label tag so each fragment is unique.
    let tag = format!("m{uid}-");
    let inner = math
        .inner_svg
        .replace("id=\"", &format!("id=\"{tag}"))
        .replace("href=\"#", &format!("href=\"#{tag}"));
    let _ = write!(svg, r#"<g transform="translate({tx},{ty})">{inner}</g>"#);
    if rotate.is_some() {
        svg.push_str("</g>");
    }
    if nudge {
        svg.push_str("</g>");
    }
}
