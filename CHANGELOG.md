# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **`PolarPlot`** — polar coordinate scatter/line plot with configurable radial/angular grid, compass (θ=0 north, CW) or math (θ=0 east, CCW) conventions. Supports multiple labeled series, r-max override, r-value labels, spoke angle labels. CLI: `kuva polar --r <COL> --theta <COL> [--color-by <COL>] [--mode scatter|line] [--r-max <F>] [--theta-divisions <N>] [--theta-start <DEG>]`. Closes #25.
- **`TernaryPlot`** — ternary/simplex scatter plot with barycentric coordinate system and equilateral triangle geometry. Auto-normalize with `with_normalize(true)`, configurable grid lines (dashed), percentage tick labels on each edge, bold corner labels, and multi-group coloring. CLI: `kuva ternary --a <COL> --b <COL> --c <COL> [--color-by <COL>] [--a-label <S>] [--b-label <S>] [--c-label <S>] [--normalize] [--grid-lines <N>]`. Closes #8.
- **`RidgelinePlot`** — ridgeline (joyplot) plot with stacked KDE density curves, one per group. Groups are labelled on the y-axis; the x-axis is the continuous data range. Supports `.with_group(label, data)`, `.with_group_color(label, data, color)`, `.with_groups(iter)`, `.with_filled(bool)`, `.with_opacity(f64)`, `.with_overlap(f64)`, `.with_bandwidth(f64)`, `.with_kde_samples(usize)`, `.with_stroke_width(f64)`, `.with_normalize(bool)`, `.with_legend(bool)`, and `.with_line_dash(s)`. CLI: `kuva ridgeline --value <COL> [--group-by <COL>] [--overlap <F>] [--filled] [--bandwidth <F>]`.
- **`DensityPlot`** — kernel density estimate curve over a single numeric column. Gaussian KDE via Silverman's rule (or manual bandwidth), normalised to a proper probability density function (integral ≈ 1). Supports `.with_filled(bool)`, `.with_opacity(f64)`, `.with_bandwidth(f64)`, `.with_kde_samples(usize)`, `.with_stroke_width(f64)`, `.with_line_dash(s)`, `.with_legend(s)`, and `from_curve(x, y)` for pre-computed curves. Multi-group plots use one `DensityPlot` per group with `render_multiple` + palette. CLI: `kuva density --value <COL> [--color-by <COL>] [--filled] [--bandwidth <F>]`.
- **Circle marker opacity + stroke** — `Primitive::Circle` and `Primitive::CircleBatch` now carry `fill_opacity: Option<f64>`, `stroke: Option<Color>`, and `stroke_width: Option<f64>`. SVG backend emits `fill-opacity`, `stroke`, and `stroke-width` attributes when `Some`; PNG raster backend applies alpha scaling and stroke rendering; terminal backend ignores new fields. Builder methods `with_marker_opacity(f64)` and `with_marker_stroke_width(f64)` added to `ScatterPlot`, `StripPlot`, `PolarPlot` (per-series), and `TernaryPlot`. Enables solid (default), semi-transparent + border, and hollow open-circle marker styles.
- `Color` type (`render::color`) — 3-variant enum (Rgb/None/Css) replacing `String` for fill/stroke in the render pipeline; `Color::Rgb(u8,u8,u8)` is 4 bytes inline with zero heap allocation; `From<&str>` parses hex, rgb(), "none", and 50+ named CSS colors
- `CircleBatch` and `RectBatch` — SoA (struct-of-arrays) `Primitive` variants with contiguous coordinate arrays for scatter and heatmap; all backends support them

### Fixed

- **Rotated x-axis tick labels no longer clip canvas edges** — `margin_left`/`margin_right` now account for the horizontal projection of rotated category labels. Negative rotation (e.g. `-45°`) uses `TextAnchor::End`; the first category label extends leftward by `label_px × cos(|angle|)` and `margin_left` is expanded if the existing y-axis margin is insufficient. Positive rotation uses `TextAnchor::Start`; the last category label extends rightward and `margin_right` is expanded symmetrically. Previously `TextAnchor::End` was hardcoded for all rotation angles regardless of sign; `axis.rs` now correctly uses `TextAnchor::Start` for positive angles. Affects bar, waterfall, candlestick, and dot plots (all of which use `-45°` rotation by default).
- **Terminal legend swatch alignment** — `LegendShape::Line` swatches (density, line plots, reference-line entries) were hidden behind the legend background rect in terminal output because horizontal lines went to the lower-priority `line_char_bits` layer. Short lines (≤8 cells wide) now write to `char_grid` so they take priority over the background. A 4.2 px y-offset is added before the row mapping so the swatch lands in the same character row as its label.
- **Terminal `LegendShape::Rect` row alignment** — legend colour swatches for all rect-style plots (stacked area, histogram, box, violin, etc.) could appear one row above their text label when a cell boundary fell between `swatch_cy` and `text_baseline`. The small-rect height snap now uses `height × 0.75` (the lower third of the rect) instead of the centre (`height × 0.5`), placing the swatch in the same character row as the label at all terminal sizes.

### Changed

- `Primitive::Path` now uses `Box<PathData>` — shrinks enum from ~128 to ~88 bytes per element
- SVG output uses hex colors for named CSS colors (e.g. `fill="red"` → `fill="#ff0000"`) — test assertions updated accordingly
- **SVG serialization 50–70% faster** — replaced all `format!()` calls in `SvgBackend` with direct `push_str()`/`write!()` into the output buffer; eliminates per-primitive heap allocations in hot loops
- **Float formatting via `ryu`** — coordinate values now use `ryu` (the same crate as `serde_json`) for 2–5× faster float→string conversion; coordinates are rounded to 2 decimal places; whole numbers omit the decimal point (e.g. `"72"` not `"72.0"`)
- **Single-pass XML escaping** — `write_escaped()` scans text content once instead of five chained `.replace()` calls; no allocation when input has no special characters
- **`PngBackend` font database cached** — system fonts are loaded once via `OnceLock` and shared across all render calls; eliminates a 100ms+ overhead on repeated PNG renders
- **`Scene` pre-allocated** — `Scene::new()` now accepts an estimated primitive count and calls `Vec::with_capacity()`; reduces reallocation count for large plots

---

## [0.1.4] — 2026-03-10

### Added

- **`LegendPosition` expanded** — the 7 old variants are replaced by 20 new ones grouped by placement zone. All names are now prefixed with `Inside` or `Outside`:
  - *Inside* (overlaid on the data area, 8 px inset): `InsideTopRight`, `InsideTopLeft`, `InsideBottomRight`, `InsideBottomLeft`, `InsideTopCenter`, `InsideBottomCenter`
  - *Outside right margin*: `OutsideRightTop` *(new default)*, `OutsideRightMiddle`, `OutsideRightBottom`
  - *Outside left margin*: `OutsideLeftTop`, `OutsideLeftMiddle`, `OutsideLeftBottom`
  - *Outside top margin*: `OutsideTopLeft`, `OutsideTopCenter`, `OutsideTopRight`
  - *Outside bottom margin*: `OutsideBottomLeft`, `OutsideBottomCenter`, `OutsideBottomRight`
  - `Custom(f64, f64)` — absolute SVG canvas pixel coordinates (what `with_legend_at` now sets internally)
  - `DataCoords(f64, f64)` — data-space coordinates mapped through `map_x`/`map_y` at render time
- **`Layout::with_legend_box(bool)`** — suppress the legend background and border rects; entries and swatches still render (fixes: legend box could not previously be hidden)
- **`Layout::with_legend_title(s)`** — renders a bold title row above all legend entries
- **`Layout::with_legend_group(title, entries)`** — adds a labelled group of entries; multiple calls stack and take priority over `with_legend_entries`
- **`Layout::with_legend_at_data(x, y)`** — places the legend at data-space coordinates (`DataCoords` variant); no right-margin reserved
- **`LegendGroup` struct** — `{ title: String, entries: Vec<LegendEntry> }`; exported from `kuva::plot`
- **`Layout::with_legend_width(px)`** — override the auto-computed legend box width; escape hatch when labels are wider than the heuristic estimate
- **`Layout::with_legend_height(px)`** — override the auto-computed legend box height; escape hatch for manually controlled legend sizing
- **`Layout::with_scale(f)`** — uniform scale factor (default `1.0`) for all plot chrome: font sizes, margins, tick mark lengths, stroke widths, legend padding/swatch geometry, and annotation arrow sizes. Canvas `width`/`height` are unaffected. `TextAnnotation::font_size` and `ReferenceLine::stroke_width` are user-set and are not auto-scaled; scale them manually when needed.
- **`--scale` CLI flag** — exposes `with_scale` on all subcommands via `BaseArgs`

### Changed

- `Layout::with_legend_at(x, y)` now sets `legend_position = Custom(x, y)` instead of writing a separate `legend_xy` field; behaviour is unchanged
- `Layout` field `legend_xy: Option<(f64, f64)>` removed — superseded by `Custom(x, y)` in the enum
- Margin calculation in `ComputedLayout::from_layout` is now position-aware: `Inside*`, `Custom`, and `DataCoords` variants add no margin; the 12 `Outside*` variants expand the appropriate edge (right/left/top/bottom)
- `render_legend_at` signature extended with `groups`, `title`, and `show_box` parameters (used by `Figure` shared-legend rendering)
- Legend width auto-sizing character multiplier increased from 7.0 → 8.5 px/char; reduces overflow for wide-glyph labels (e.g. "Unmapped" on Sankey plots)

### Fixed

- Legend background and border rects could not be suppressed — now controlled via `with_legend_box(false)`
- Y axis label x-position was a fixed pixel offset from the canvas left edge, ignoring actual tick label widths — now computed dynamically as `margin_left − 8 − y_tick_label_px − 5 − label_size/2`, placing the label consistently close to the axis regardless of tick label width; clamped to 8 px from canvas edge
- `margin_left` computation replaced the 6-char heuristic with actual tick string generation (`generate_ticks` / `generate_ticks_log` / `generate_ticks_with_step` + `TickFormat::format`); `ComputedLayout` carries the result as `y_tick_label_px` for use in `axis.rs`

---

## [0.1.3] — 2026-03-04

### Added

- `SvgBackend` is now a proper struct with `with_pretty(bool)` — `SvgBackend::new().with_pretty(true)` emits one element per line with 2-space indentation and group-depth tracking; compact output is unchanged and remains the default; a backward-compat `const SvgBackend` shim keeps all existing call sites compiling without modification
- `impl Default for SvgBackend` added (fixes `new_without_default` Clippy lint)

### Changed

- Default font family is now `"DejaVu Sans, Liberation Sans, Arial, sans-serif"` (previously fell back to the browser/renderer default); propagated through `ComputedLayout` and `Figure::render` via a shared `DEFAULT_FONT_FAMILY` constant
- `title_size` default increased from 16 → 18 px
- `tick_size` default increased from 10 → 12 px; margins auto-expand from `tick_size` so no text is clipped
- CLI `--width` / `--height` flags are now optional with no default; canvas size is auto-computed from plot content when omitted, allowing pie outside-label widening and other layout-sensitive plots to size themselves correctly; explicit `--width`/`--height` still takes precedence

### Fixed

- **Brick plot legend order** — strigar motif legend entries are now sorted by global letter (A → Z) so the most-frequent motif always appears first
- **Sankey z-order** — node labels are now emitted after ribbons rather than before them; labels are no longer painted over by coloured ribbon bands
- **UpSet count labels** — intersection size labels above bars are suppressed when the column is too narrow to fit the number without overlapping an adjacent label
- **Pie outside label / legend overlap** — canvas widening for outside labels was blocked when the CLI forced `layout.width = Some(800)`; fixed by making `BaseArgs.width`/`height` `Option<f64>` so the widening condition fires correctly when the user has not explicitly set a size
- **Manhattan `--top-n`** — top-N point labels were filtered by the genome-wide significance threshold before selection, producing no labels when no points exceeded it; labels now pick the top-N most significant points unconditionally
- **Phylo circular whitespace** — replaced the conservative `hpad = edge_pad + label_pad` padding with a direct minimum-clearance formula (`max_r = min(pw/2 − edge_pad − label_gap − chars×7, ph/2 − edge_pad − 7)`); on an 800×800 canvas with 23-character leaf labels the tree radius increases from 94 px to 194 px

---

## [0.1.2] — 2026-03-02

### Added

- `Figure::with_figure_size(w, h)` — specify total figure dimensions and have cell sizes auto-computed to fit, accounting for padding, spacing, title height, and shared legend area

### Fixed

- Clippy warnings resolved: `type_complexity` in `TerminalBackend` (extracted `type Rgb = (u8, u8, u8)`), `manual_is_multiple_of` in `render_utils`, and `needless_range_loop` suppressed on intentional triangular matrix loops in chord rendering
- `test_missing_feature_error` / `test_missing_feature_pdf` marked `#[ignore]` — these tests check a compile-time feature gate and were producing false-positive failures when a stale binary built with `--features full` was present on disk
- CI Clippy step now runs with `-D warnings` — all warnings are errors

---

## [0.1.1] — 2026-03-01

### Added

- `kuva::prelude::*` — single-import module re-exporting all plot structs, `Plot`, `Layout`, `Figure`, `Theme`, `Palette`, render helpers, backends, annotations, and datetime utilities
- `Into<Plot>` for all 25 plot structs — write `plot.into()` instead of `Plot::Scatter(plot)`
- `render_to_svg(plots, layout) -> String` — full pipeline in one call
- `render_to_png(plots, layout, scale) -> Result<Vec<u8>, String>` — one-call PNG output (feature `png`)
- `render_to_pdf(plots, layout) -> Result<Vec<u8>, String>` — one-call PDF output (feature `pdf`)
- GitHub Actions workflow to deploy the mdBook documentation to GitHub Pages on every push to `main`

### Fixed

- Unresolved intra-doc links (`Rect`, `Text`, `Line`) in `backend::terminal` module doc

---

## [0.1.0] — 2026-02-28

Initial release of kuva.

### Added

**Plot types (25)**
- `ScatterPlot` — x/y scatter with optional trend line, Pearson correlation, error bars, confidence bands, bubble sizing, and colour-by grouping
- `LinePlot` — connected line plots with optional area fill, step mode, and line style (solid/dashed/dotted/dash-dot)
- `BarPlot` — vertical bar charts with optional grouping and stacking
- `Histogram` — single-variable frequency histogram with optional normalisation and log scale
- `Histogram2D` — 2D density histogram with configurable colourmap
- `BoxPlot` — box-and-whisker with optional strip/swarm overlay
- `ViolinPlot` — KDE violin with optional strip/swarm overlay and configurable bandwidth
- `PiePlot` — pie/donut chart with inside and outside label modes, percentages, and minimum label fraction threshold
- `SeriesPlot` — multi-series line chart sharing a common x axis
- `Heatmap` — matrix heatmap with configurable colourmap and optional value labels
- `BrickPlot` — per-read sequencing alignment visualisation with STRIGAR string support
- `BandPlot` — line with shaded confidence band
- `WaterfallPlot` — waterfall chart with delta/total bar kinds, connectors, value labels, and sign-based colouring
- `StripPlot` — strip/jitter plot with jitter, swarm, and centre modes
- `VolcanoPlot` — log2 fold-change vs −log10(p-value) with threshold lines, up/down/NS colouring, and gene labels
- `ManhattanPlot` — genome-wide association plot with per-chromosome colouring, gene labels, and hg19/hg38/T2T base-pair coordinate mode
- `DotPlot` — size + colour encoding on a categorical grid with stacked size legend and colour bar
- `UpSetPlot` — UpSet intersection diagram with bitmask input, sort modes, and set-size bars
- `StackedAreaPlot` — stacked area chart with absolute and 100%-normalised modes
- `CandlestickPlot` — OHLC candlestick chart with optional volume panel and datetime x axis
- `ContourPlot` — contour plot from scattered or grid data using marching squares and IDW interpolation; filled and line modes
- `ChordPlot` — chord diagram from an N×N flow matrix with per-node colours and Bézier ribbons
- `SankeyPlot` — Sankey diagram with auto column assignment, tapered Bézier ribbons, and source/gradient/per-link colour modes
- `PhyloTree` — phylogenetic tree from Newick string, edge list, distance matrix (UPGMA), or linkage matrix; rectangular/slanted/circular branch styles; Left/Right/Top/Bottom orientation; clade colouring; bootstrap support values
- `SyntenyPlot` — pairwise genomic synteny diagram with named sequences, forward/inverted blocks, Bézier ribbons, per-sequence or shared scale, and block colouring

**Rendering**
- SVG output via `SvgBackend` (always available; no system dependencies)
- PNG rasterisation via `PngBackend` (feature: `png`; uses `resvg`, pure Rust)
- Vector PDF output via `PdfBackend` (feature: `pdf`; uses `svg2pdf`, pure Rust)
- `Figure` for multi-plot grid layouts with merged cells, shared axes, panel labels (A/B/C, a/b/c, 1/2/3, or custom), and shared legends
- Secondary y axis (`render_twin_y`)
- Date/time x and y axes with automatic tick granularity (`DateTimeAxis`)
- Log-scale x and y axes with 1-2-5 tick generation
- Custom tick formatting (`TickFormat`: Auto, Fixed, Integer, Sci, Percent, Custom)
- Text annotations with optional arrow at data coordinates
- Reference lines (horizontal/vertical) with optional label and dash pattern
- Shaded regions (horizontal/vertical fills)
- Theme support: Default, Dark, Publication, and custom themes
- Named colour palettes with modulo-wrapping index access: `category10`, `wong`, `okabe_ito`, `tol_bright`, `tol_muted`, `tol_light`, `ibm`, `pastel`, `bold`, and `Palette::custom()`

**CLI binary (`kuva`)**
- 22 subcommands covering all plot types: `scatter`, `line`, `bar`, `histogram`, `box`, `violin`, `pie`, `strip`, `waterfall`, `stacked-area`, `volcano`, `manhattan`, `candlestick`, `heatmap`, `hist2d`, `contour`, `dot`, `upset`, `chord`, `sankey`, `phylo`, `synteny`
- Auto-detects TSV/CSV delimiter; optional `--no-header` and `-d/--delimiter`
- `--color-by` for palette-assigned group series on scatter, line, strip
- `--theme`, `--palette`, `--colourblind` for appearance control
- `--log-x` / `--log-y` on applicable subcommands
- PNG and PDF output when built with the corresponding feature flags
- Hidden `kuva man` subcommand generates a `man(1)` page via `clap_mangen`
- `--terminal` flag renders plots directly in the terminal using Unicode braille (U+2800–U+28FF), full-block (`█`) fills, and ANSI 24-bit colour; ideal for HPC and remote-server workflows with no display; auto-detects terminal dimensions, overrideable with `--term-width` / `--term-height`; supported by all subcommands except `upset`

### Known limitations

- `kuva brick` CLI subcommand is not yet implemented (pending integration with bladerunner)
- Terminal rendering is not yet supported for `upset` (the command prints a message and exits cleanly; use `-o file.svg` instead)
- No Python or other language bindings
