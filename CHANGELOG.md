# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

---

## [0.1.7] — 2026-04-29

### Added

- **`TextPlot`** — new plot type for placing formatted, word-wrapped text inside a figure cell alongside data plots. Supports an optional title, body text with inline markdown-style markup (`**bold**`, `*italic*`, `__underline__`), heading lines (`# H1`, `## H2`), horizontal rules (`---`), paragraph spacing (blank lines), optional background fill, configurable border, text alignment (left/center/right), font size, padding, and text color. Rendered via `Primitive::RichText` — a new multi-span primitive that emits `<tspan>` children in SVG (and the raster backend's mini-SVG overlay), giving pixel-accurate inline styling. PNG and PDF inherit the SVG output automatically; the terminal backend flattens spans to plain text. Word wrapping always breaks on full words, never mid-word. Resolves [#72](https://github.com/Psy-Fer/kuva/issues/72).
- **Date/time axis documentation** — new reference page `docs/src/reference/datetime.md` with four worked examples (monthly line, scatter with dates, multi-series, hourly), covering `ymd()`, `ymd_hms()`, `DateTimeAxis` constructors, `.with_step()`, auto mode, and conversion notes for `chrono`, `time`, and `std::time`. `examples/datetime.rs` generates the accompanying SVG assets. Closes [#32](https://github.com/Psy-Fer/kuva/issues/32).
- **`GanttPlot`** — Gantt chart with horizontal task bars, optional group/phase grouping, group header background bands, per-task progress fill, milestone diamonds, "now" reference line, and smart label placement (inside bar when wide enough, to the right otherwise). Tasks with the same group share a color drawn from the category10 palette; per-task color overrides are supported. Builder API: `with_task`, `with_task_group`, `with_task_group_progress`, `with_task_progress`, `with_colored_task`, `with_milestone`, `with_milestone_group`, `with_now_line`, `with_group_order`, `with_bar_height`, `with_milestone_size`, `with_show_labels`, `with_group_bg`. `render_gantt` convenience function. CLI: `kuva gantt` with `--label-col`, `--start-col`, `--end-col`, `--group-col`, `--progress-col`, `--milestone-col`, `--now`, `--bar-height`, `--no-labels`. **V2 features (dependency arrows, lane stacking) deferred to next version.**
- **`NetworkPlot`** — node-edge network / graph diagram with force-directed (Fruchterman-Reingold) and circular layout algorithms. Supports edge-list and adjacency-matrix input, directed/undirected edges, self-loops, weighted edges (stroke width + opacity), per-node color/size/group, and group-based legends. CLI: `kuva network`.
- **`WafflePlot`** — waffle / unit chart displaying categorical proportions as a grid of colored cells. Supports square and circle cell shapes, configurable grid dimensions, fill order, empty-cell color, unit annotation, show-counts / show-percents labels, and legend. CLI: `kuva waffle`.
- **`HorizonPlot`** — compact multi-series time series visualization. Each series occupies a single row; the value range is divided into N equal-width bands folded onto that row with progressively darker shading. Positive and negative deviations use separate color families. Supports configurable band count, row height, baseline, value-max, and optional band-scale annotations. CLI: `kuva horizon`.
- **`PrPlot`** (Precision-Recall Curve) — precision-recall curve with AUC-PR computed via trapezoidal integration. Supports per-group curves from raw `(score, label)` predictions or pre-computed `(recall, precision)` points, optional optimal-F1 threshold marker, optional AUC label in legend, baseline (random classifier) line, and multi-group legend. CLI: `kuva pr`.
- **`PopulationPyramid`** — back-to-back horizontal bar chart for age/gender structure visualization. Supports single and multi-series (Grouped/Overlap modes), percentage normalization, value labels, per-series colors, and configurable bar width and gap. CLI: `kuva pyramid`.
- **Native Sankey/alluvial ordering** — `SankeyPlot` now supports ordered alluvium input via `with_alluvium()`, `with_alluvia()`, and `with_axis_names()`, plus optional weighted crossing reduction and neighbornet ordering through `with_crossing_reduction()`, `with_neighbornet()`, `with_node_order()`, and `with_node_order_seed()`. Node coloring can follow per-label or left-propagated alluvial coloring via `with_node_coloring()` / `with_left_coloring()`. CLI: `kuva sankey --axis-col ... --node-order crossings|neighbornet --coloring label|left`.
- **13 new CLI subcommands**: `slope`, `lollipop`, `raincloud`, `mosaic`, `waffle`, `pyramid`, `roc`, `pr`, `survival`, `horizon`, `parallel`, `venn`, `calendar` — covering all previously library-only plot types.
- **`kuva --version` / `-V`** — `kuva` CLI now reports its version string. Closes [#69](https://github.com/Psy-Fer/kuva/issues/69).
- **`BrickPlot` bladerunner stitched STRIGAR format** — `with_strigars` now handles bladerunner's multi-candidate stitched format: `|` separates candidates; inter-candidate gaps render as `N@` (large gap) or `@:seq`/`1@` (small gap, light grey). Canonical-rotation normalisation operates across all candidates.
- **`BrickPlot::with_start_positions(iter)`** and **`BrickPlot::with_x_origin(f)`** — per-read genomic start coordinates for aligned repeat region display; `with_x_origin` sets the reference coordinate that maps to x = 0.
- **`StripPlot` per-point markers** — `with_group_markers(label, values, markers)` assigns a distinct marker shape per point within a strip group (triangle, square, diamond, cross, star, plus).
- **`Figure` legend position extensions** — `FigureLegendPosition` now includes `OutsideRightMiddle`, `OutsideRightBottom`, `OutsideTopLeft`, `OutsideTopCenter`, `OutsideTopRight`, `OutsideBottomLeft`, `OutsideBottomCenter`, `OutsideBottomRight`, `OutsideLeftTop`, `OutsideLeftMiddle`, `OutsideLeftBottom` in addition to the existing `Right`, `Bottom`, `Custom`.
- **Text wrapping** — long axis labels, titles, and legend items are now automatically wrapped at word boundaries when they exceed the available width.
- **`ColorMap` extended** — added `Cividis`, `Turbo`, `Cubehelix`, `RdYlGn`, `BrBG`, `PuOr`, and `Spectral` palettes.
- **`Layout::with_width_gap(px)`** and **`Layout::with_height_gap(px)`** — explicit per-axis gap overrides for fine-tuning panel spacing in `Figure` grids.
- **`BrickPlot` figure normalisation** — row heights are normalised to a consistent pixel height when `BrickPlot` is placed inside a `Figure` panel.
- **`JointPlot`** — scatter plot with marginal histograms or KDE density curves on the top and right axes; multi-group support via `JointGroup`; `MarginalType` enum (Histogram/Density); 9 tests.
- **`Clustermap`** — hierarchical clustermap: heatmap with UPGMA-computed row and column dendrograms rendered as `PhyloTree` panels; `ClustermapNorm` for row/column z-score normalisation; `AnnotationTrack` for categorical bar annotations on rows or columns; pre-supplied `PhyloTree` override; colorbar. Closes [#59](https://github.com/Psy-Fer/kuva/issues/59).
- **Heatmap row/column reordering** — `Heatmap::with_y_categories(order)` permutes data rows so the first label renders at the top; `with_x_categories(order)` permutes columns. Order is expressed as label names. Closes [#59](https://github.com/Psy-Fer/kuva/issues/59).
- **`Layout::with_equal_aspect()`** — expands the shorter axis so that 1 data unit spans the same number of pixels on both x and y; no-op on log axes and pixel-space plots (polar, ternary, pie, chord, etc.). Guards against degenerate zero-width ranges. 5 tests. Closes [#58](https://github.com/Psy-Fer/kuva/issues/58).
- **CLI `--y` multi-column** — `kuva scatter` and `kuva line` now accept comma-separated column names or indices for `--y` (e.g. `--y A,B,C`); one auto-colored series is created per column; conflicts with `--color-by`. Legend labels use column header names when present, otherwise `col_N`. Partially closes [#57](https://github.com/Psy-Fer/kuva/issues/57).
- **`RocPlot`** — ROC curve with trapezoidal AUC, DeLong 95% CI bands, partial AUC, Youden's J optimal-threshold marker, AUC label in legend, and diagonal chance-level reference line; 14 tests. CLI: `kuva roc`.
- **`LollipopPlot`** — lollipop / mutation landscape chart: vertical stems with dot markers, per-point labels and colors, and protein-domain annotation rectangles via `LollipopDomain`; 15 tests. CLI: `kuva lollipop`.
- **`SurvivalPlot`** (Kaplan–Meier) — right-continuous step-function survival curve with Greenwood 95% CI bands, censoring tick marks, multi-group legend, and optional log-rank p-value annotation; 12 tests. CLI: `kuva survival`.
- **`RaincloudPlot`** — half-violin KDE cloud + box-and-whisker + jittered rain in a single panel; per-group colors; `with_flip`, `with_bandwidth_scale`, and `with_cloud`/`with_box`/`with_rain` toggles; 11 tests. CLI: `kuva raincloud`.
- **`SlopePlot`** (dumbbell plot) — paired before/after values connected by a line; direction-based coloring (up/down/flat); outer-anchored, collision-safe labels; configurable dot size and group colors; 13 tests. CLI: `kuva slope`.
- **`VennPlot`** — Venn diagram for 2–4 sets; classic and proportional circle/ellipse layouts; raw element or pre-computed size input; region labels with per-set colored indicator dots; hybrid inline + leader-line labeling with march-outward anchor placement; `with_leader_lines`, `with_set_indicators`, `with_proportional`, `with_loss`; 21 tests. CLI: `kuva venn`.
- **`ParallelPlot`** — parallel coordinates chart; per-axis independent normalisation; optional shared scale; polyline + cubic-Bézier rendering; group colors + legend; optional mean overlay; axis inversion with orange indicator; adaptive h_inset / tick font scaling to prevent label collision on dense axes; 16 tests. CLI: `kuva parallel`.
- **`MosaicPlot`** — Marimekko / mosaic chart: variable-width columns proportional to column totals with stacked segments; custom y-axis (0–100%); cell percent and/or raw value labels; `with_normalize`, `with_gap`, `with_col_order`, `with_row_order`, `with_group_colors`; 12 tests. CLI: `kuva mosaic`.
- **`StreamgraphPlot`** — stream graph with Catmull-Rom smooth curves; `StreamBaseline` (Wiggle/Symmetric/Zero); `StreamOrder` (InsideOut/ByTotal/Original); inter-stream strokes; inline stream labels at the widest point; 100% normalisation mode; 20 tests.
- **`RadarPlot`** — radar / spider chart: N clockwise axes from the top; filled/stroked polygons; shared or per-axis normalisation; dashed grid rings; vertex dots; legend; 13 tests.
- **`TreemapPlot`** — treemap with squarified/slice-dice/binary layouts; depth-decreasing padding; ByParent/ByValue/Explicit color modes; `with_go_terms()` GO-enrichment convenience builder; SVG hover tooltips; colorbar; 23 tests. CLI: `kuva treemap`.
- **`SunburstPlot`** — sunburst / concentric ring hierarchy using the same `TreemapNode` data model as `TreemapPlot`; donut style via `with_inner_radius`; configurable start angle, ring gap, min label angle, and max depth; ByParent/ByValue/Explicit color modes; colorbar; SVG hover tooltips; 21 tests. CLI: `kuva sunburst`.
- **`BumpPlot`** — bump chart: rank-over-time visualization with sigmoid or straight curves; auto-ranking from raw values; `BumpTieBreak` modes (Average/Min/Max/Stable); highlight one series and mute others; collision-safe endpoint labels; rank labels inside dots; 23 tests. CLI: `kuva bump`.
- **`FunnelPlot`** — funnel chart with vertical/horizontal orientations; trapezoidal connectors; adaptive inside/outside value labels; step-to-step conversion rate annotations; Uniform/ByStage/Gradient color modes; diverging back-to-back mirror mode; 22 tests. CLI: `kuva funnel`.
- **`RosePlot`** — Nightingale rose / coxcomb chart: area-proportional encoding (r ∝ √value) for perceptually accurate wedge sizing; Stacked/Grouped multi-series modes; `with_bearing_data` auto-bins raw directional values into N equal sectors; compass labels; donut hole via `with_inner_radius`; 25 tests. CLI: `kuva rose`.
- **`CalendarPlot`** — calendar heatmap (GitHub-contribution style); full-year and arbitrary date-range periods via `CalendarPeriod`; Count/Sum/Mean/Max aggregation; GitHub-green sqrt-gamma default colormap; month separator L-step paths; colorbar. CLI: `kuva calendar`.
- **Stats box** — `Layout::with_stats_box()` / `with_stats_entry(label, value)` / `with_stats_box_at(position)` / `with_stats_title(s)`: a `LegendPosition`-positioned bordered text box for annotating plots with summary statistics; stacks vertically below the legend when placed at the same position; 7 tests.

### Fixed

- **Fonts in minimal environments** — DejaVu Sans is now bundled inside the crate and loaded into the font database before system fonts in the PNG and PDF backends. Plots rendered in containers or CI pipelines with no installed fonts (e.g. bioconda recipes) will now have correct text rendering instead of blank labels.
- **`SvgBackend::with_embedded_font(true)`** — new builder method that injects a base64-encoded `@font-face` block into the SVG `<style>` element, making the SVG self-contained for environments where system fonts are unavailable (headless servers, `rsvg-convert` in containers, etc.). Off by default to keep SVG file sizes small. CLI: `--embed-font`. Closes [#71](https://github.com/Psy-Fer/kuva/issues/71).
- **`BumpPlot` endpoint label clipping** — endpoint labels on bump charts are now drawn after `ClipEnd` so they are no longer clipped at the plot edge when the final rank is 1 or the series extends to the rightmost column.
- **`StripPlot` legend overflow** — legend box width is now correctly sized from the longest group label when `with_group_colors` is used; previously the box was too narrow and clipped text.
- **`JointPlot` in figure context** — scatter panel, right-marginal histogram, and legend now fit cleanly within the allocated cell. Legend space is carved from `scatter_canvas_w` upfront rather than overflowing into adjacent cells.
- **`JointPlot` axis label centering** — x-label is now centred on the scatter axis (not the full canvas); y-label is centred on the scatter panel height.
- **`JointPlot` label duplication in figure** — x/y labels are suppressed from `add_labels_and_title` when a `JointPlot` is detected in the panel; the custom-positioned labels are the only ones drawn.
- **Clippy warnings (Rust 1.85)** — resolved `collapsible_match`, `useless_conversion` (`into_iter`), `unnecessary_sort_by`, and `clone_on_copy` lints across `dotplot.rs`, `upset.rs`, `candlestick.rs`, `network.rs`, `layout.rs`, `render.rs`, and `src/bin/kuva/rose.rs`.
- **Legend height overflow** — legends with more entries than fit in the canvas height are now capped: visible entries fill the available space (minimum 10 always shown) and a `… (+N more)` line is appended. The canvas right margin is automatically widened to fit the overflow label so it is not clipped by the canvas edge. Affects `BrickPlot` loci with large numbers of distinct repeat motifs (e.g. CANVAS, NIID) as well as any plot using `with_legend_entries` with many entries.

### Changed

- **`Heatmap::with_x_range(lo, hi)` / `with_y_range(lo, hi)`** — set custom axis extents for the heatmap so it can represent a scalar field over a physical domain (e.g. `with_x_range(-10.0, 10.0)`, `with_y_range(-4.0, 4.0)`). Cell positions are mapped linearly across the specified range; default behaviour (`[0.5, cols+0.5]` / `[0.5, rows+0.5]`) is unchanged when neither method is called. 5 tests. Closes [#64](https://github.com/Psy-Fer/kuva/issues/64).
- **`Heatmap::with_cell_size(factor)`** — controls the fraction of each cell's natural size used when drawing the cell rectangle. Default `0.99` preserves the existing thin gap between cells; `1.0` draws cells flush with no visible boundary, which is recommended for large grids where the gap becomes a distracting pattern. Values are clamped to `[0.5, 1.0]`. 3 tests.

- **Unified numerical input types** — all data-entry builder methods now accept `impl Into<f64>` instead of bare `f64`, so `u8`, `u16`, `u32`, `i8`, `i16`, `i32`, and `f32` values can be passed directly without `.into()` or `as f64`. Affects `BarPlot` (`with_bar`, `with_colored_bar`, `with_colored_bars`, `with_bars`, `with_group`), `WaterfallPlot` (`with_delta`, `with_difference`), `ForestPlot` (`with_row`, `with_weighted_row`, `with_colored_row`, `with_weighted_colored_row`, `ForestRow::new`), `MosaicPlot` (`with_cell`, `with_cells`), `RosePlot` (`with_slice`, `with_slices`), `SankeyPlot` (`with_link`, `with_link_colored`, `with_links`, `with_alluvium`), `TernaryPlot` (`with_point`, `with_point_group`, `with_points`), `ParallelPlot` (`with_row`, `with_row_group`), `NetworkPlot` (`with_edge`, `with_edge_color`, `with_edge_label`, `with_edge_styled`, `with_edges`), `LollipopPlot` (`with_points`). Closes [#68](https://github.com/Psy-Fer/kuva/issues/68).

---

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
- **`Scatter3DPlot`** — 3D scatter plot with orthographic projection, depth-sorted painter's algorithm, z-colormap, depth shading, per-point colors/sizes, configurable markers, and matplotlib-style open-box wireframe with grid on all three back walls. CLI: `kuva scatter3d data.tsv --x x --y y --z z`.
- **`Surface3DPlot`** — 3D surface mesh rendered as depth-sorted filled quadrilaterals with z-colormap, wireframe edges, alpha transparency, `with_data_fn()` for function sampling, and `--resolution N` bilinear interpolation upsampling. CLI: `kuva surface3d data.tsv --x x --y y --z z --z-color viridis`.
- **Shared 3D infrastructure** — `Projection3D` orthographic projection module, `View3D` view angles, `Box3DConfig` shared configuration, `draw_3d_box()` reusable box/grid/axes renderer, `DataRanges3D` bounding box, `ColorMap::map_rgb()` zero-allocation colormap path.
- **`parse_colormap`** consolidated into shared `data.rs` helper (was duplicated 6x across CLI modules).
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
