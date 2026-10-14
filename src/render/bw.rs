use crate::plot::line::LineStyle;
use crate::plot::{ColorMap, FillPattern, MarkerShape};
use crate::render::render::Scene;
use std::sync::Arc;

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

/// Lowest point of the `Grayscale` ramp used for *continuous* data in BW mode.
///
/// The ramp itself starts at pure `#ffffff`, which is invisible against kuva's
/// white canvas. For a dense grid (heatmap, clustermap) that is merely faint,
/// but for a plot that omits empty cells (hexbin, calendar) it is ambiguous:
/// a minimum-value cell reads as *no data* rather than *least data*. Starting
/// at `#f0f0f0` keeps the lightest cells visible while leaving essentially the
/// whole tonal range intact, and matches the floor `BW_GREYS` already applies
/// to categorical fills for exactly the same reason.
const BW_CONTINUOUS_FLOOR: f64 = 0.12;

/// The colormap continuous data uses in BW mode.
///
/// Every BW override routes through this, fills and colorbars alike, so the bar
/// always describes the ramp the cells were actually drawn with.
pub fn bw_colormap() -> ColorMap {
    ColorMap::Custom(Arc::new(|t: f64| {
        let t = t.clamp(0.0, 1.0);
        ColorMap::Grayscale.map(BW_CONTINUOUS_FLOOR + t * (1.0 - BW_CONTINUOUS_FLOOR))
    }))
}

/// Returns the grey shade and fill pattern for group index `i` in BW mode.
///
/// Grey shades and patterns cycle independently (5 greys × 7 patterns = 35
/// unique combinations before any repeat) so even large group counts remain
/// distinguishable.
pub fn bw_fill(i: usize) -> (&'static str, FillPattern) {
    let grey = BW_GREYS[i % BW_GREYS.len()];
    let pat = BW_PATTERNS[i % BW_PATTERNS.len()];
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
