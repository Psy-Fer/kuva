# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.6] — 2026-04-01

- **`DicePlot`** — new plot type: a grid of cells where each cell shows up to 6 dots in a canonical die-face layout. Ports rendering logic from the [ggdiceplot](https://github.com/maflot/ggdiceplot) R package (v1.2.0). Three input modes: categorical (`with_records`), continuous tile (`with_points`), and per-dot continuous (`with_dot_data`) for ZEBRA-style domino plots. Pip sizing uses the ggdiceplot 1.2.0 tight-packing algorithm with `pip_scale = 0.75` and offset shrinkage. Legend support: spatial-position legend (mini die faces), categorical colour legend, and size legend sections. Column-major grid positions match `make_offsets()`.
- **Custom X/Theta-Tick-Labels for `PolarPlot`** — Re-uses `with_x_tick_format()` for theta axis on `PolarPlot`. Introduces new default `TickFormat::Degree` for `PolarPlot`, so default behavior is unchanged. 
### Fixed

- **docs.rs build** — `doom` feature build script no longer attempts to write to the source directory or access the network when building on docs.rs. Empty placeholder files are written to `OUT_DIR` instead so `include_bytes!` compiles cleanly.

---

## [0.1.5] — 2026-04-01

### Added

- **SVG interactivity v1** (`--interactive` / `Layout::with_interactive()`) — opt-in, self-contained browser interactivity embedded directly in the SVG output with no external dependencies. Degrades silently to a static SVG in PNG/PDF/terminal/Inkscape contexts. Features: hover tooltips, click-to-pin (sticky highlight; Escape to clear), search + dim (text input dims non-matching elements), coordinate readout (cursor x/y in data space shown on hover), and legend toggle (click a legend entry to show/hide the corresponding series). Wired for scatter, line, bar, strip, and volcano plots in this release; remaining renderers deferred to v0.2.
- **`kuva doom`** (`--features cli,doom`) — generates a fully self-contained, offline-playable DOOM SVG (~15 MB). Open in any browser and play with keyboard controls. The Chocolate Doom engine (GPL v2) and shareware WAD are base64-encoded directly into the SVG at build time; no server, no network requests, no external files needed. Easter egg feature, separate from the plotting library.
- **`kuva bar --color-by <COL>`** — grouped bar chart mode. Groups rows by the specified column and creates one colored series per unique value using the active palette, with an automatic legend. When each x-label maps to exactly one series (e.g. `--color-by` equals `--label-col`), falls back to simple per-bar coloring instead of a grouped layout.
- **`kuva strip --legend`** — assigns palette colors per group and shows a legend. Combines with `--interactive` for legend toggle.
- **`PolarPlot` negative radius / `r_min` support** (`--r-min`) — `PolarPlot::with_r_min(f64)` sets the value mapped to the plot centre (default: 0). Points below `r_min` clamp to centre; ring labels show actual r values. CLI: `kuva polar --r-min <F>`. Closes #54.
- **Custom X/Theta-Tick-Labels for `PolarPlot`** — `with_x_tick_format()` now applies to the theta axis on polar plots. New `TickFormat::Degree` default keeps existing behaviour unchanged.
- **`Layout::with_polar_r_label_angle(deg)`** — override the angle at which r-axis ring labels are drawn (default: midpoint between spokes).
- **`ForestPlot`** — forest plot for meta-analysis: point estimates with confidence intervals on a categorical Y-axis, vertical dashed null-effect reference line, optional weight-scaled markers. CLI: `kuva forest data.tsv --label-col study --estimate-col estimate --ci-lower-col lower --ci-upper-col upper`.
- **`Figure::with_twin_y_plots(cell, primary, secondary)`** — twin-Y panels now work inside multi-panel `Figure` grid layouts. Auto-layout via `Layout::auto_from_twin_y_plots`; shared legend collection includes both primary and secondary plots.
- **Fine-grained axis and grid line controls** — new `Layout` builder methods: `with_axis_line_width(f)`, `with_tick_width(f)`, `with_tick_length(f)`, `with_grid_line_width(f)`. All propagate through `ComputedLayout`; grid lines now drawn before axis borders (z-order fix). CLI: `--tick-length`, `--tick-width`, `--grid-stroke`.
- **SVG clip-path support** — data elements are now clipped to the plot area, preventing points and lines from rendering outside the axis borders. Implemented via `Primitive::ClipStart`/`ClipEnd`; ignored by terminal and raster backends. Closes #53.
- **`Histogram2D::with_log_count()`** — log₁₀-scaled colour axis for 2D histograms. Colorbar tick marks are placed at actual count values (0, 1, 10, 100 …) and labelled accordingly. CLI: `kuva hist2d --log-count`. Also adds `Layout::with_colorbar_tick_format(TickFormat)` / CLI `--colorbar-tick-format`.
- **`SankeyPlot` flow labels** — `with_flow_labels()` annotates each ribbon with its flow value; `with_flow_label_units(s)` adds a unit string; `with_flow_label_decimals(n)` controls precision; `with_flow_label_percents()` shows percentages of total flow. CLI: `kuva sankey --flow-labels [--flow-label-percents]`.
- **`kuva heatmap` long-format input** — accepts a value column (`--value-col`) from long-format data (row, column, value triples) with optional per-cell aggregation (`--agg-fn mean|sum|min|max|count`). Wide-format matrix input unchanged.
- **`kuva bar --agg <FUNC>`** — aggregate a numeric value column by a label column using `mean`, `median`, `sum`, `min`, or `max` before plotting. Complements `--count-by` for summarising long-format data.
- **`kuva volcano` and `kuva manhattan` — `--pvalue-col-is-log`** — accept a pre-computed −log₁₀(p) column directly; internally un-transforms via 10^(−v) before passing raw p-values to the plot struct.
- **Colorbar title moved to side** — colorbar titles now render rotated on the left side of the colorbar rather than above it, matching common convention and preventing overlap with axis labels.
- **Per-subcommand CLI documentation** — all CLI subcommands now have dedicated documentation pages at `docs/src/cli/<subcommand>.md`. Closes #36.
- **`BrickPlot` bladerunner stitched STRIGAR format** — `with_strigars` now handles bladerunner's multi-candidate stitched format: `|` separates candidates, each with its own local letter namespace. Inter-candidate gaps appear as `N@` (large gap, N nucleotides wide, no motif entry) or `@:seq` / `1@` (small gap, rendered at `len(seq)` nt). Gap bricks render as light grey. Canonical-rotation normalisation operates across all candidates, so ACCCTA / TAACCC / CCCTAA in different candidates are automatically assigned the same global letter and colour.
- **`BrickPlot::with_start_positions(iter)`** — per-read genomic start coordinates. Pass the reference position where each read begins; kuva shifts rows on the shared x-axis so repeat regions align visually. Equivalent to `with_x_offsets` with negated values but expresses intent clearly.
- **`BrickPlot::with_x_origin(f)`** — sets the reference coordinate that maps to x = 0 on the axis. Applied on top of (and independently from) any per-row offsets; use alongside `with_start_positions` to anchor a biologically meaningful position such as a repeat start to the axis origin.
- **`BrickPlot` per-read offsets in strigar mode** — `with_x_offsets` and `with_start_positions` now apply in strigar mode (previously forced to zero).
- **`BrickPlot` bounds fix with offsets** — x-axis range is now computed per-row using actual row widths and offsets in both DNA and strigar modes, preventing reads from being clipped at the right edge when start positions push them beyond the widest unshifted row.

### Fixed
- **Manhattan chromosome labels not visible** — labels were previously emitted inside the SVG clip-path group, placing them below the clip boundary (the data area) and making them invisible. Labels are now drawn after `ClipEnd` so they render outside the clip region.
- **Multi-panel figure axis ranges** — manually set `with_x_axis_min/max` / `with_y_axis_min/max` values were silently dropped when used inside `Figure` panels. Now correctly forwarded through `clone_layout`. Closes #43.
- **Terminal y-axis label** — `--y-label` text is now rendered vertically (one character per row) in `--terminal` mode instead of horizontally, preventing overlap with the plot area.
- **Terminal legend swatches** — circle-based legend swatches (scatter, density, volcano, manhattan, etc.) now show their actual series color instead of being masked by the legend background in `--terminal` mode.
- **Polar r-label / theta-label overlap** — r-axis ring labels are now positioned at the midpoint angle between spokes instead of directly on the 0° spoke.
- **Histogram zero-height bins** — zero-count bins are now skipped before emitting a `Rect` primitive, eliminating SVG zero-height rect warnings. Closes #51.
- **Density plot boundary leakage** — replaced post-hoc KDE clipping with boundary reflection (ggplot2-style): ghost points mirrored at user-specified bounds restore lost kernel mass so the curve terminates smoothly. `with_x_lo(f)` / `with_x_hi(f)` allow one-sided bounds; `with_x_range(lo, hi)` is kept as a shorthand. Closes #47.
- **Density plot normalization** — `bounds()` sample count now matches the renderer; `--x-min` / `--x-max` CLI flags correctly restrict the KDE evaluation range. Closes #37.
- **Histogram2D with real data** — CLI now accepts `--x-min`/`--x-max`/`--y-min`/`--y-max` to control the binning range; off-by-one at the upper edge clamped to the last bin. Closes #39.
- **Histogram x-axis label truncation** — right margin now accounts for the last tick label's half-width so labels are never clipped. Closes #46.
- **Heatmap / PDF cell limit** — a hard 1 M cell limit in `kuva heatmap` now emits a clear error with the cell count and concrete aggregation suggestions instead of silently producing a broken PDF. Closes #38.

---

## [0.1.4] — 2026-03-12

### Added

- **Twin-Y documentation** — new `docs/src/plots/twin_y.md` covering `render_twin_y`, `auto_from_twin_y_plots`, axis labels, log y2 scale, mixed plot types, palette auto-assignment, and manual range overrides; four SVG examples including a GC bias QC chart.
- **Per-point colors on `StripPlot`** — `with_colored_group(label, iter_of_(value, color)_pairs)` adds a group where each point carries its own color. Colors are matched by position; points beyond the color list fall back to the group/uniform color. Useful when each observation belongs to a distinct category (e.g. motif type) and needs to be visually distinguished within a single column.
- **`PolarPlot`** — polar coordinate scatter/line plot with configurable radial/angular grid, compass (θ=0 north, CW) or math (θ=0 east, CCW) conventions. Supports multiple labeled series, r-max override, r-value labels, spoke angle labels. CLI: `kuva polar --r <COL> --theta <COL> [--color-by <COL>] [--mode scatter|line] [--r-max <F>] [--theta-divisions <N>] [--theta-start <DEG>]`. Closes #25.
- **`TernaryPlot`** — ternary/simplex scatter plot with barycentric coordinate system and equilateral triangle geometry. Auto-normalize with `with_normalize(true)`, configurable grid lines (dashed), percentage tick labels on each edge, bold corner labels, and multi-group coloring. CLI: `kuva ternary --a <COL> --b <COL> --c <COL> [--color-by <COL>] [--a-label <S>] [--b-label <S>] [--c-label <S>] [--normalize] [--grid-lines <N>]`. Closes #8.
- **`RidgelinePlot`** — ridgeline (joyplot) plot with stacked KDE density curves, one per group. Groups are labelled on the y-axis; the x-axis is the continuous data range. Supports `.with_group(label, data)`, `.with_group_color(label, data, color)`, `.with_groups(iter)`, `.with_filled(bool)`, `.with_opacity(f64)`, `.with_overlap(f64)`, `.with_bandwidth(f64)`, `.with_kde_samples(usize)`, `.with_stroke_width(f64)`, `.with_normalize(bool)`, `.with_legend(bool)`, and `.with_line_dash(s)`. CLI: `kuva ridgeline --value <COL> [--group-by <COL>] [--overlap <F>] [--filled] [--bandwidth <F>]`.
- **`DensityPlot`** — kernel density estimate curve over a single numeric column. Gaussian KDE via Silverman's rule (or manual bandwidth), normalised to a proper probability density function (integral ≈ 1). Supports `.with_filled(bool)`, `.with_opacity(f64)`, `.with_bandwidth(f64)`, `.with_kde_samples(usize)`, `.with_stroke_width(f64)`, `.with_line_dash(s)`, `.with_legend(s)`, and `from_curve(x, y)` for pre-computed curves. Multi-group plots use one `DensityPlot` per group with `render_multiple` + palette. CLI: `kuva density --value <COL> [--color-by <COL>] [--filled] [--bandwidth <F>]`. Closes #15.
- **`Histogram::from_bins(edges, counts)`** — create a histogram from precomputed bin edges and counts rather than raw values. `edges` must have length `counts.len() + 1`; counts are `f64` to support fractional values (density estimates, normalised outputs from R/numpy). Closes #24.
- **`LegendPosition` expanded** — the 7 old variants are replaced by 20 new ones grouped by placement zone. All names are now prefixed with `Inside` or `Outside`:
  - *Inside* (overlaid on the data area, 8 px inset): `InsideTopRight`, `InsideTopLeft`, `InsideBottomRight`, `InsideBottomLeft`, `InsideTopCenter`, `InsideBottomCenter`
  - *Outside right margin*: `OutsideRightTop` *(new default)*, `OutsideRightMiddle`, `OutsideRightBottom`
  - *Outside left margin*: `OutsideLeftTop`, `OutsideLeftMiddle`, `OutsideLeftBottom`
  - *Outside top margin*: `OutsideTopLeft`, `OutsideTopCenter`, `OutsideTopRight`
  - *Outside bottom margin*: `OutsideBottomLeft`, `OutsideBottomCenter`, `OutsideBottomRight`
  - `Custom(f64, f64)` — absolute SVG canvas pixel coordinates (what `with_legend_at` now sets internally)
  - `DataCoords(f64, f64)` — data-space coordinates mapped through `map_x`/`map_y` at render time
- **`Layout::with_legend_box(bool)`** — suppress the legend background and border rects; entries and swatches still render
- **`Layout::with_legend_title(s)`** — renders a bold title row above all legend entries
- **`Layout::with_legend_group(title, entries)`** — adds a labelled group of entries; multiple calls stack and take priority over `with_legend_entries`
- **`Layout::with_legend_at_data(x, y)`** — places the legend at data-space coordinates (`DataCoords` variant); no right-margin reserved
- **`LegendGroup` struct** — `{ title: String, entries: Vec<LegendEntry> }`; exported from `kuva::plot`
- **`Layout::with_legend_width(px)`** / **`with_legend_height(px)`** — override auto-computed legend box dimensions
- **`Layout::with_scale(f)`** — uniform scale factor for all plot chrome: font sizes, margins, tick mark lengths, stroke widths, legend padding/swatch geometry, and annotation arrow sizes. Canvas `width`/`height` are unaffected. CLI: `--scale` on all subcommands.
- **Fine-grained tick and gridline control** ([#13](https://github.com/Psy-Fer/kuva/issues/13)) — `Layout::with_x_axis_min/max`, `with_y_axis_min/max`, `with_x_tick_step`, `with_y_tick_step`, `with_minor_ticks(n)`, `with_show_minor_grid(bool)`; minor ticks are 3 px marks; minor gridlines use 0.5 stroke-width. CLI: `--x-min`, `--x-max`, `--y-min`, `--y-max`, `--x-tick-step`, `--y-tick-step`, `--minor-ticks`, `--minor-grid`.
- **Per-point colors on `ScatterPlot` and per-group colors on `StripPlot`** — `ScatterPlot::with_colors(iter)` indexed per point; `StripPlot::with_group_colors(iter)` indexed per group. Both fall back to the uniform `color` field for out-of-range indices. `ScatterPlot::bounds()` now returns `None` on empty data rather than panicking.
- **Per-group colors on `ViolinPlot` and `BoxPlot`** — `with_group_colors(iter)` added to both, mirroring `StripPlot`. All elements of a box group (box, whiskers, caps) share the group color. CLI: `--group-colors` (comma-separated) on `kuva violin` and `kuva box`.
- **Circle marker opacity + stroke** — `Primitive::Circle` and `Primitive::CircleBatch` now carry `fill_opacity: Option<f64>`, `stroke: Option<Color>`, and `stroke_width: Option<f64>`. Builder methods `with_marker_opacity(f64)` and `with_marker_stroke_width(f64)` added to `ScatterPlot`, `StripPlot`, `PolarPlot` (per-series), and `TernaryPlot`.
- **`Color` type** (`render::color`) — 3-variant enum (`Rgb/None/Css`) replacing `String` for fill/stroke in the render pipeline; `Color::Rgb(u8,u8,u8)` is 4 bytes inline with zero heap allocation; `From<&str>` parses hex, `rgb()`, `"none"`, and 50+ named CSS colors.
- **`CircleBatch` and `RectBatch`** — SoA (struct-of-arrays) `Primitive` variants with contiguous coordinate arrays for scatter and heatmap; all backends support them.
- **Benchmark suite** — `benches/render.rs`, `benches/svg.rs`, `benches/kde.rs` with Criterion; `docs/src/benchmarks.md` with tables and run instructions.

### Changed

- `Layout::with_legend_at(x, y)` now sets `legend_position = Custom(x, y)`; `legend_xy` field removed
- Margin calculation in `ComputedLayout::from_layout` is position-aware: `Inside*`, `Custom`, and `DataCoords` add no margin; `Outside*` variants expand the appropriate edge
- `render_legend_at` signature extended with `groups`, `title`, and `show_box` parameters
- Legend width auto-sizing character multiplier increased from 7.0 → 8.5 px/char
- `Primitive::Path` now uses `Box<PathData>` — shrinks enum from ~128 to ~88 bytes per element
- SVG output uses hex colors for named CSS colors (e.g. `fill="red"` → `fill="#ff0000"`)
- **SVG serialization 50–70% faster** — replaced all `format!()` calls in `SvgBackend` with direct `push_str()`/`write!()`; eliminates per-primitive heap allocations in hot loops
- **Float formatting via `ryu`** — 2–5× faster float→string conversion; coordinates rounded to 2 decimal places; whole numbers omit the decimal point
- **Single-pass XML escaping** — `write_escaped()` scans text content once; no allocation when input has no special characters
- **`PngBackend` font database cached** — system fonts loaded once via `OnceLock`; eliminates 100ms+ overhead on repeated PNG renders
- **`Scene` pre-allocated** — `Scene::new()` accepts an estimated primitive count and calls `Vec::with_capacity()`
- **KDE truncated kernel** — `simple_kde` windows evaluations to `[x ± 4bw]` via binary search; ~8× faster at 100k samples
- **Manhattan pre-bucketing** — SNPs bucketed into `HashMap<&str, Vec<usize>>` before span loop; ~22× faster at 1M SNPs
- **Heatmap single-pass** — two nested loops merged into one; intermediate `flat: Vec<f64>` allocation eliminated

### Fixed

- **`render_twin_y` now supports `Plot::Density`** — `DensityPlot` was silently dropped in both the primary and secondary match arms; it is now routed to `add_density` with the correct computed layout for each axis.
- **Legend overhaul** — background/border rects can now be suppressed via `with_legend_box(false)`; y-axis label x-position computed dynamically from actual tick label widths rather than a fixed offset; `margin_left` now uses actual tick string generation instead of a 6-char heuristic
- **`BrickPlot` strigar color/legend ordering** — deterministic sort replaces `HashMap` iteration order; output is now byte-identical across runs
- **Rotated x-axis tick labels** — `margin_left`/`margin_right` now account for horizontal projection of rotated labels; `TextAnchor::Start` used for positive rotation angles. Affects bar, waterfall, candlestick, and dot plots.
- **Terminal legend swatch alignment** — `LegendShape::Line` swatches now write to `char_grid` so they take priority over legend background; `LegendShape::Rect` snaps to `height × 0.75` so swatches land in the same row as their label at all terminal sizes
- **Terminal legend entry spacing** — legend entries step by exact whole-cell multiples (`round(18 / cell_h).max(1) * cell_h`); eliminates fractional-row misalignment across all terminal sizes and subcommands
- **Terminal phylo leaf label row** — removed `+ 4.0` SVG baseline offset on leaf labels for Left/Right orientations
- **`ridgeline` example** — output now written to `docs/src/assets/ridgeline/` instead of the repo root

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
