use crate::plot::{FillPattern, MarkerShape};
use crate::plot::line::LineStyle;
use crate::render::render::Scene;

/// Grey shade sequence for BW mode.
///
/// Alternates light/dark so adjacent groups have maximum contrast
/// when placed next to each other (e.g. bars in a grouped bar chart).
const BW_GREYS: &[&str] = &[
    "#f0f0f0", // 0 — very light
    "#383838", // 1 — very dark
    "#b0b0b0", // 2 — medium-light
    "#181818", // 3 — near-black
    "#787878", // 4 — medium
];

/// Pattern sequence for BW mode — each variant is visually distinct.
const BW_PATTERNS: &[FillPattern] = &[
    FillPattern::DiagonalForward,
    FillPattern::Horizontal,
    FillPattern::Crosshatch,
    FillPattern::Vertical,
    FillPattern::Dots,
    FillPattern::DiagonalBack,
    FillPattern::DiagonalCrosshatch,
];

/// Dash sequence for BW line plots.
const BW_DASHES: &[LineStyle] = &[
    LineStyle::Solid,
    LineStyle::Dashed,
    LineStyle::Dotted,
    LineStyle::DashDot,
];

/// Marker shape sequence for BW scatter/point plots.
const BW_SHAPES: &[MarkerShape] = &[
    MarkerShape::Circle,
    MarkerShape::Square,
    MarkerShape::Triangle,
    MarkerShape::Diamond,
    MarkerShape::Cross,
    MarkerShape::Plus,
];

/// Returns the grey shade and fill pattern for group index `i` in BW mode.
///
/// Grey shades and patterns cycle independently (5 greys × 7 patterns = 35
/// unique combinations before any repeat) so even large group counts remain
/// distinguishable.
pub fn bw_fill(i: usize) -> (&'static str, FillPattern) {
    let grey = BW_GREYS[i % BW_GREYS.len()];
    let pat  = BW_PATTERNS[i % BW_PATTERNS.len()];
    (grey, pat)
}

/// Returns the [`LineStyle`] for group index `i` in BW mode.
pub fn bw_dash(i: usize) -> LineStyle {
    BW_DASHES[i % BW_DASHES.len()].clone()
}

/// Returns the [`MarkerShape`] for group index `i` in BW mode.
pub fn bw_shape(i: usize) -> MarkerShape {
    BW_SHAPES[i % BW_SHAPES.len()]
}

/// Registers a pattern's `<pattern>` element in `scene.defs` (idempotent)
/// and returns the CSS fill string `"url(#<id>)"` to use on a primitive.
///
/// For [`FillPattern::Solid`] this is a no-op and returns an empty string.
pub fn register_pattern(scene: &mut Scene, pattern: FillPattern) -> String {
    if !pattern.is_patterned() {
        return String::new();
    }
    let id = pattern.id();
    if !scene.defs.iter().any(|d| d.contains(id)) {
        scene.defs.push(pattern.svg_def().to_string());
    }
    format!("url(#{id})")
}
