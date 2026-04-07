---
name: JointPlot notes
description: Architecture and key implementation details for JointPlot (scatter + marginal distributions)
type: project
---

## JointPlot (added 2026-04-07)

- `src/plot/jointplot.rs` — structs `JointPlot`, `JointGroup`, enum `MarginalType` (Histogram/Density)
- **NOT a `Plot` enum variant** — standalone renderer like `render_twin_y`
- `pub fn render_jointplot(jp: JointPlot, layout: Layout) -> Scene` — entry point in `src/render/render.rs` (~line 8011)
- Private helpers: `joint_histogram_bins`, `joint_draw_top_marginal`, `joint_draw_right_marginal`

**Builder methods on `JointPlot`**: `with_xy`, `with_group`, `with_marginal_type`, `with_top_marginal(bool)`, `with_right_marginal(bool)`, `with_marginal_size(f64)`, `with_marginal_gap(f64)`, `with_bins(usize)`, `with_bandwidth(f64)`, `with_marginal_alpha(f64)`, `with_x_label`, `with_y_label`, `with_legend`, `with_marker_size`, `with_marker_opacity`

**Builder methods on `JointGroup`**: `new(x, y)`, `with_label`, `with_color`

**Internal layout (pixel-space composition)**:
- Scatter sub-scene rendered via `render_multiple` sized to `(scatter_canvas_w, scatter_canvas_h)`
- Inserted into master scene via `GroupStart { transform: "translate(0, scatter_offset_y)" }`
- `scatter_offset_y = title_h + top_h + top_gap`
- Marginals drawn directly as `Primitive::Rect`/`Primitive::Path` using `scatter_computed.map_x()`/`map_y()` for alignment
- Top marginal: bars grow upward from `scatter_offset_y` (panel bottom)
- Right marginal: horizontal bars growing rightward from `scatter_canvas_w + right_gap`

**Why sub-scene translation works:**
- scatter_offset_x = 0 (no horizontal shift), so `scatter_computed.map_x()` values are already in master coords
- `scatter_computed.map_y(y) + scatter_offset_y` gives master-scene y coords for right marginal bars
- Separator lines at panel boundaries drawn in master-scene coords

**Tests**: `tests/jointplot.rs` — 9 tests; outputs to `test_outputs/jointplot_*.svg`

**Closes**: TODO item "Scatter + marginal distributions" under margin panels architecture
