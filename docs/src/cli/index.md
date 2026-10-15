# kuva CLI

`kuva` is the command-line front-end for the kuva plotting library. It reads tabular data from a TSV or CSV file (or stdin) and writes an SVG — or PNG/PDF with the right feature flag — to a file or stdout.

```
kuva <SUBCOMMAND> [FILE] [OPTIONS]
```

---

## Installation

### Step 1 — install Rust

If you don't have Rust installed, get it via [rustup](https://rustup.rs):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the on-screen prompts (the defaults are fine). Then either restart your shell or run:

```bash
source ~/.cargo/env
```

Verify with `cargo --version`. You only need to do this once.

kuva itself only needs Rust 1.87. The one exception is PDF output: `--features pdf`/`full` pulls in `krilla`, which requires Rust >= 1.92. If `rustup show` reports an older version and you plan to use `--features full` or `pdf`, run `rustup update` first.

### Step 2 — install kuva

**From crates.io** (recommended once a release is published):

```bash
cargo install kuva --features cli          # SVG output
cargo install kuva --features cli,full     # SVG + PNG + PDF
```

**From a local clone** (install to `~/.cargo/bin/` and put it on your `$PATH`):

```bash
git clone https://github.com/Psy-Fer/kuva && cd kuva

cargo install --path . --features cli          # SVG output
cargo install --path . --features cli,full     # SVG + PNG + PDF
```

After either method, `kuva` is available anywhere in your shell — no need to reference `./target/release/kuva` or modify `$PATH` manually. Confirm with:

```bash
kuva --help
```

### Building without installing

If you only want to build and run from the repo without installing:

```bash
cargo build --release --bin kuva --features cli,full
./target/release/kuva --help
```

---

## Input

Every subcommand takes an optional positional `FILE` argument. If omitted or `-`, data is read from **stdin**.

```bash
# from file
kuva scatter data.tsv

# from stdin
cat data.tsv | kuva scatter

# explicit stdin
kuva scatter - < data.tsv
```

### Delimiter detection

| Priority | Rule |
|---|---|
| 1 | `--delimiter` flag |
| 2 | File extension: `.csv` → `,`, `.tsv`/`.txt` → tab |
| 3 | Sniff first line: whichever of tab or comma appears more often |

### Header detection

The first row is treated as a header when either its first field fails to parse as a number, or some column holds a non-numeric label sitting atop an otherwise all-numeric column (so a leading numeric key column no longer hides the header, e.g. `5,data` above `0,1` / `1,2`).

Two flags override the auto-detection (they are mutually exclusive):

- `--header`: force the first row to be a header even if it looks like data (useful for all-numeric column names such as years).
- `--no-header`: force the first row to be data even if it looks like a header.

### Column selection

Columns are selected by **0-based integer index** or **header name**:

```bash
kuva scatter data.tsv --x 0 --y 1          # by index
kuva scatter data.tsv --x time --y value   # by name (requires header)
```

An all-numeric column name (such as a year) works too. A bare numeric argument is read as an index first; if that index is out of range and the file has a header column literally named by that number, it selects that column instead:

```bash
kuva scatter expression.tsv --x 2023 --y 2024 --header   # columns named "2023" / "2024"
```

In-range indices are never reinterpreted, so `--x 0` always means the first column. This bare-number fallback only resolves names whose numeric value exceeds the column count (years and similar).

To remove the ambiguity entirely, prefix the value:

| Prefix | Meaning | Example |
|--------|---------|---------|
| `name:` or `col:` | Always a column name, even an all-digit one | `--x name:2024`, `--group-col col:1` |
| `idx:` or `index:` or `#` | Always a 0-based index, never resolved to a name | `--x idx:2`, `--y #3` |
| *(none)* | Bare token: numeric = index-first (with the fallback above), otherwise a name | `--x time`, `--y 4` |

`name:` is the way to select a column whose name is a small number that would otherwise be read as an index (e.g. a column literally named `1` in a 5-column file: `--x name:1`). `idx:` forces a pure index, so an out-of-range `idx:` errors rather than falling back to a name. Prefixes work on every column argument (`--x`, `--y`, `--value-col`, `--group-col`, `--color-by`, `--axes`, …) in every subcommand.

#### Ranges and globs (multi-column arguments)

Multi-column arguments (`--y`, `--value-cols`, `--samples`, `--axes`) also accept **index ranges** and **name globs**, which expand to several columns at once:

| Form | Selects | Example |
|------|---------|---------|
| `a..b` | Indices `a` up to (but not including) `b` | `--y 1..4` -> columns 1, 2, 3 |
| `a..=b` | Indices `a` through `b` inclusive | `--y 1..=3` -> columns 1, 2, 3 |
| `a..` | Index `a` to the last column | `--y 3..` |
| `..b` | The first column up to `b` (exclusive) | `--y ..4` |
| `..` | Every column | `--y ..` |
| `pat*` / `*pat` / `*pat*` | Header names matching the glob (`*` = any run of characters) | `--y sum_*`, `--y *_A` |

You can mix all forms in one comma-separated list (duplicates are removed, order preserved):

```bash
kuva line data.tsv --x date --y "y1,sum_*"      # y1 plus every sum_* column
kuva line data.tsv --x 0 --y 3..                 # column 3 to the last
kuva box  data.tsv --y "a*"                       # a single glob is a valid multi-column selection
```

A single glob or range on its own still routes into the plot's multi-column (grouped/overlaid) path. Globs need a header row; a glob that matches nothing is an error. Ranges are index-based. (For `--value-cols`/`--axes`, which take space-separated values rather than commas, pass each selector as its own value: `--value-cols a1 b*`.)

### Missing values *(scatter, line, histogram, bar, parallel)*

Empty cells, cells holding a missing-value token (`NA`, `NaN`, `null`, `N/A`, `.` by default,
case-insensitive), and non-finite values (`inf`, `-inf`) are treated as missing rather than
aborting the command (non-finite values would otherwise corrupt axis ranges). A cell that is
neither empty, a token, nor a number is still an error, so selecting the wrong column is caught.

- `--na-strategy <drop|zero|error>`: what to do with a missing value in a selected numeric column.
  `drop` (default) removes the row and prints a one-line note; `zero` replaces it with 0; `error`
  restores the old strict behaviour.
- `--na-values <tokens>`: comma-separated tokens to treat as missing, replacing the default set.
  Empty cells are always missing. Accepts negative sentinels, e.g. `--na-values -999,NA`.
- `--clamp-min <N>` / `--clamp-max <N>`: clip numeric values to a bound. Finite values outside the
  bound are clipped, and crucially `+inf` becomes the max and `-inf` becomes the min, so an infinite
  value can be capped at a real number rather than dropped. A common case is capping an infinite
  `-log10(p)` (from a p-value of exactly 0): `--clamp-max 300`. If only one side is set, an infinity
  on the other side stays missing.

```bash
kuva scatter data.tsv --x time --y value                 # drops rows with missing x or y
kuva histogram data.tsv --value-col v --na-strategy zero # missing -> 0
kuva scatter data.tsv --x pos --y logp --clamp-max 300   # +inf -> 300 instead of dropped
```

### Parquet input

Every subcommand that reads tabular data also accepts **`.parquet`** files, not just scatter or any single subcommand: parquet support lives in the shared input layer every subcommand goes through, so it applies uniformly across all of them.

```bash
kuva scatter data.parquet --x x --y y -o plot.svg
cat data.parquet | kuva histogram --value-col value    # also detected via stdin
```

Requires building with the `parquet` feature (`cargo build --features cli,parquet`, or `cli,full,parquet` for every backend). Without it, a `.parquet` file is read as plain text and will fail to parse.

Detection is automatic, no flag needed:

| Input | How it's detected |
|---|---|
| File path | `.parquet` extension (case-insensitive) |
| stdin | Magic bytes (`PAR1` header) sniffed from the piped data |

Column selection (`--x`, `--y`, `--value-col`, etc.) works identically to CSV/TSV, by index or header name. Under the hood, only the requested columns are decoded from disk (a projected Arrow read), so memory and time scale with the columns you actually select rather than the full schema, useful for wide parquet files with many unused columns.

`--header`, `--no-header`, and `--delimiter` are ignored for parquet input (with a warning) since parquet is self-describing: it always carries its own schema and column names, so there's no header row to skip and no delimiter to guess.

---

## Output

| Flag | Effect |
|---|---|
| *(omitted)* | SVG to stdout |
| `-o out.svg` | SVG to file |
| `-o out.png` | PNG (requires `--features png`) |
| `-o out.pdf` | PDF (requires `--features pdf`; needs Rust >= 1.92 to build — higher than kuva's own MSRV, see [Installation](#installation)) |

Format is inferred from the file extension case-insensitively. Output paths must end in `.svg`, `.png`, or `.pdf`; other or missing extensions are rejected before input is read.

---

## Shared flags

These flags are available on every subcommand.

### Output & appearance

| Flag | Default | Description |
|---|---|---|
| `-o`, `--output <FILE>` | stdout (SVG) | `.svg`, `.png`, or `.pdf` output path, case-insensitive (mutually exclusive with `--terminal`) |
| `--title <TEXT>` | — | Title displayed above the chart |
| `--subtitle <TEXT>` | — | Secondary line under the title, smaller and muted ([Reference → Layout](../reference/layout.md)) |
| `--width <PX>` | `800` | Canvas width in pixels |
| `--height <PX>` | `500` | Canvas height in pixels |
| `--theme <NAME>` | `light` | Theme: `light`, `dark`, `solarized`, `minimal` |
| `--palette <NAME>` | `category10` | Color palette for multi-series plots |
| `--cvd-palette <NAME>` | — | Colour-vision-deficiency palette: `deuteranopia`, `protanopia`, `tritanopia`. Overrides `--palette`. |
| `--bw` | off | [Black & white / accessibility mode](../reference/bw_mode.md) — replaces colors with grey shades, hatch patterns, dash styles, and marker shapes so the plot stays legible in greyscale |
| `--background <COLOR>` | *(theme default)* | SVG background color (any CSS color string) |

### Fonts

| Flag | Default | Description |
|---|---|---|
| `--embed-font` | off | Embed DejaVu Sans directly in SVG output (mutually exclusive with `--terminal`) |

By default, SVG output references fonts by name and relies on the viewer to resolve them. This works fine in browsers and on any system where DejaVu Sans, Verdana, Liberation Sans, or Arial is installed. In environments with no system fonts — headless servers, containers, CI pipelines — text may be missing or fall back to an unexpected face.

`--embed-font` bakes DejaVu Sans as a base64 `@font-face` block into the SVG `<style>` element, making the file fully self-contained at the cost of roughly 1 MB of extra size. PNG and PDF output is unaffected: those backends always have the font available regardless of this flag.

```bash
# Self-contained SVG for use with rsvg-convert or similar tools in containers
kuva scatter data.tsv --x x --y y --embed-font -o plot.svg

# Pipe into rsvg-convert in a minimal container
kuva scatter data.tsv --x x --y y --embed-font | rsvg-convert -o plot.png
```

---

### SVG interactivity

| Flag | Default | Description |
|---|---|---|
| `--interactive` | off | Embed browser interactivity in SVG output (ignored for PNG/PDF/terminal) |

When `--interactive` is set the output SVG contains a self-contained `<script>` block with no external dependencies. Features:

- **Hover tooltip** — hovering a data point shows its label and value.
- **Click to pin** — click a point to keep its highlight; click again or press **Escape** to clear all pins.
- **Search** — type in the search box (top-left of the plot area) to dim non-matching points. **Escape** clears.
- **Coordinate readout** — mouse position inside the plot area is shown in data-space coordinates.
- **Legend toggle** — click a legend entry to show/hide that series.
- **Save button** — top-right button serialises the current SVG DOM (including any pinned/dimmed state). *Note: the download is not yet functional.*

Supported in this release: `scatter`, `line`, `bar`, `strip`, `volcano`. All other subcommands accept `--interactive` and load the UI chrome (coordinate readout, search box) but do not yet have per-point hover/search — full renderer coverage is planned for a future release.

```bash
kuva scatter data.tsv --x x --y y --color-by group --legend --interactive -o plot.svg
kuva volcano hits.tsv --gene gene --log2fc log2fc --pvalue pvalue --legend --interactive -o volcano.svg
```

### Terminal output

| Flag | Default | Description |
|---|---|---|
| `--terminal` | off | Render directly in the terminal using Unicode braille and block characters; mutually exclusive with `-o` |
| `--term-width <N>` | *(auto)* | Terminal width in columns (overrides auto-detect) |
| `--term-height <N>` | *(auto)* | Terminal height in rows (overrides auto-detect) |

Terminal output uses Unicode braille dots (U+2800–U+28FF) for scatter points and continuous curves, full-block characters (`█`) for bar and histogram fills, and ANSI 24-bit colour. Terminal dimensions are auto-detected from the current tty; pass `--term-width` and `--term-height` to override (useful in scripts or when piping).

```bash
# Scatter plot directly in terminal
kuva scatter data.tsv --x x --y y --terminal

# Explicit dimensions
kuva bar counts.tsv --label-col gene --value-col count --terminal --term-width 120 --term-height 40

# Manhattan plot on a remote server
cat gwas.tsv | kuva manhattan --chr-col chr --pvalue-col pvalue --terminal
```

> **Note:** Terminal output is not yet supported for `upset`. Running `kuva upset --terminal` prints a message and exits cleanly; use `-o file.svg` instead.

### Axes *(most subcommands)*

| Flag | Default | Description |
|---|---|---|
| `--x-label <TEXT>` | — | X-axis label |
| `--y-label <TEXT>` | — | Y-axis label |
| `--ticks <N>` | `5` | Hint for number of tick marks |
| `--no-grid` | off | Disable background grid |

### Log scale *(scatter, line, histogram, density, hist2d)*

| Flag | Description |
|---|---|
| `--log-x` | Logarithmic X axis |
| `--log-y` | Logarithmic Y axis |

### Date/time X axis *(scatter, line)*

| Flag | Default | Description |
|---|---|---|
| `--x-date-format <FMT>` | — | Parse the X column as a date/time using this `strftime`-style format (e.g. `%Y-%m-%d`, `%m/%d/%Y %H:%M`) instead of a plain number. Formats with no time component parse as midnight UTC. |
| `--x-date-unit <UNIT>` | auto | Tick spacing unit: `years`, `months`, `weeks`, `days`, `hours`, or `minutes`. Omit for auto mode, which inspects the data range and picks one. Ignored unless `--x-date-format` is set. |
| `--x-date-tick-format <FMT>` | *(per-unit default)* | Tick label format, overriding the unit's default (see table below). Ignored in auto mode. |
| `--x-date-tick-step <N>` | `1` | Draw one tick every `N` units instead of every 1. |

Default tick format per unit (used when `--x-date-tick-format` is omitted):

| Unit | Default format | Example |
|---|---|---|
| `years` | `%Y` | `2024` |
| `months` | `%b %Y` | `Jan 2024` |
| `weeks` | `%b %d` | `Jan 15` |
| `days` | `%Y-%m-%d` | `2024-01-15` |
| `hours` | `%H:%M` | `14:30` |
| `minutes` | `%H:%M` | `14:30` |

```bash
# Auto mode: format and unit picked from the data range
kuva line prices.tsv --x date --y close --x-date-format "%Y-%m-%d"

# Explicit unit and tick format
kuva scatter prices.tsv --x date --y close \
    --x-date-format "%Y-%m-%d" --x-date-unit months --x-date-tick-format "%b %y"

# One tick every 2 weeks
kuva line prices.tsv --x date --y close \
    --x-date-format "%Y-%m-%d" --x-date-unit weeks --x-date-tick-step 2
```

See [Reference → Date & Time Axes](../reference/datetime.md) for the underlying `DateTimeAxis` API.

### Secondary Y axis *(twin-y)*

| Flag | Description |
|---|---|
| `--y2-label <TEXT>` | Label for the secondary (right) Y axis |
| `--y2-min <F>` | Fix the secondary Y axis lower bound; overrides auto-range |
| `--y2-max <F>` | Fix the secondary Y axis upper bound; overrides auto-range |
| `--log-y2` | Log-scale the secondary Y axis |
| `--y2-tick-format <FORMAT>` | Tick label format for the secondary Y axis: auto (default), int, sci, percent, or fixed:N |

### Input

| Flag | Description |
|---|---|
| `--header` | Force first row as a header (overrides auto-detection) |
| `--no-header` | Treat first row as data, not a header |
| `-d`, `--delimiter <CHAR>` | Override field delimiter |

---

## Subcommands

All 57 subcommands, grouped the same way plot pages are grouped in the sidebar. Each link goes straight to that subcommand's **CLI** section at the bottom of its library-equivalent page, right next to the Rust API it wraps — so full per-flag documentation, usage examples, and the builder reference live together on one page instead of two.

### Distributions

| Subcommand | Description |
|---|---|
| [histogram](../plots/histogram.md#cli) | Frequency histogram from one or more numeric columns |
| [hist2d](../plots/histogram2d.md#cli) | Two-dimensional histogram (density grid) from two numeric columns |
| [density](../plots/density.md#cli) | Kernel density estimate curve |
| [ridgeline](../plots/ridgeline.md#cli) | Stacked KDE density curves, one per group |
| [ecdf](../plots/ecdf.md#cli) | Empirical cumulative distribution function |
| [qq](../plots/qq.md#cli) | Q-Q (quantile-quantile) plot |
| [box](../plots/boxplot.md#cli) | Box-and-whisker plot |
| [violin](../plots/violin.md#cli) | Kernel-density violin plot |
| [strip](../plots/strip.md#cli) | Strip / jitter plot |
| [raincloud](../plots/raincloud.md#cli) | Half-violin KDE cloud, box, and jittered points combined |
| [hexbin](../plots/hexbin.md#cli) | Hexagonal-bin density plot from two numeric columns |
| [heatmap](../plots/heatmap.md#cli) | Color-encoded matrix heatmap |

### Relationships & correlation

| Subcommand | Description |
|---|---|
| [scatter](../plots/scatter.md#cli) | Scatter plot of (x, y) point pairs |
| [line](../plots/line.md#cli) | Line plot |
| [contour](../plots/contour.md#cli) | Contour plot from scattered (x, y, z) triplets |
| [parallel](../plots/parallel.md#cli) | Parallel coordinates, one axis per variable |
| [polar](../plots/polar.md#cli) | Polar coordinate scatter/line plot |
| [ternary](../plots/ternary.md#cli) | Ternary (simplex) scatter plot |
| [quiver](../plots/quiver.md#cli) | 2-D vector field rendered as arrows |

### Categorical & comparison

| Subcommand | Description |
|---|---|
| [bar](../plots/bar.md#cli) | Bar chart from label/value pairs |
| [pie](../plots/pie.md#cli) | Pie or donut chart |
| [waffle](../plots/waffle.md#cli) | Proportional grid of filled cells |
| [funnel](../plots/funnel.md#cli) | Stage-by-stage attrition funnel |
| [pareto](../plots/pareto.md#cli) | Bars sorted descending, plus a cumulative-percentage line |
| [pyramid](../plots/pyramid.md#cli) | Population pyramid (back-to-back horizontal bars) |
| [lollipop](../plots/lollipop.md#cli) | Dot-and-stem alternative to bar charts |
| [slope](../plots/slope.md#cli) | Paired before/after comparisons |
| [dot](../plots/dotplot.md#cli) | Dot plot (size + color at categorical positions) |
| [mosaic](../plots/mosaic.md#cli) | Mosaic / Marimekko two-way contingency table |
| [venn](../plots/venn.md#cli) | Venn diagram, 2 to 4 overlapping sets |
| [upset](../plots/upset.md#cli) | UpSet plot for set-intersection analysis |
| [radar](../plots/radar.md#cli) | Radar / spider chart |
| [rose](../plots/rose.md#cli) | Nightingale rose (coxcomb) chart |

### Time series

| Subcommand | Description |
|---|---|
| [stacked-area](../plots/stacked_area.md#cli) | Stacked area chart |
| [streamgraph](../plots/streamgraph.md#cli) | Flowing stacked area with a displaced baseline |
| [candlestick](../plots/candlestick.md#cli) | OHLC candlestick chart |
| [waterfall](../plots/waterfall.md#cli) | Running total built from incremental bars |
| [horizon](../plots/horizon.md#cli) | Folded stacked time series for many series in limited height |
| [calendar](../plots/calendar.md#cli) | GitHub-style daily contribution grid |
| [gantt](../plots/gantt.md#cli) | Task bars with milestones and a "now" line |
| [bump](../plots/bump.md#cli) | Rank changes over time |

### Statistical & model evaluation

| Subcommand | Description |
|---|---|
| [roc](../plots/roc.md#cli) | ROC curve for binary classifiers |
| [pr](../plots/pr.md#cli) | Precision-recall curve |
| [survival](../plots/survival.md#cli) | Kaplan-Meier survival curve |
| [forest](../plots/forest.md#cli) | Point estimates with confidence intervals |

### Hierarchical & network

| Subcommand | Description |
|---|---|
| [treemap](../plots/treemap.md#cli) | Tile a rectangle proportionally to values |
| [sunburst](../plots/sunburst.md#cli) | Radial hierarchy chart |
| [network](../plots/network.md#cli) | Graph diagram from an edge list or adjacency matrix |
| [sankey](../plots/sankey.md#cli) | Sankey / alluvial flow diagram |
| [chord](../plots/chord.md#cli) | Chord diagram for pairwise flow data |
| [phylo](../plots/phylo.md#cli) | Phylogenetic tree from a Newick string or edge-list |

### Genomics & bioinformatics

| Subcommand | Description |
|---|---|
| [manhattan](../plots/manhattan.md#cli) | Manhattan plot for GWAS results |
| [volcano](../plots/volcano.md#cli) | Volcano plot for differential expression |
| [synteny](../plots/synteny.md#cli) | Genomic alignment ribbon plot |

### 3D

| Subcommand | Description |
|---|---|
| [scatter3d](../plots/scatter3d.md#cli) | 3D scatter plot with orthographic projection |
| [surface3d](../plots/surface3d.md#cli) | 3D surface mesh with depth-sorted rendering |

### Composite & Utility

| Subcommand | Description |
|---|---|
| [twin-y](../plots/twin_y.md#cli) | Two series sharing an x-axis with independent primary/secondary y-scales |

---

## Tips

**Pipe to a viewer:**
```bash
kuva scatter data.tsv | display            # ImageMagick
kuva scatter data.tsv | inkscape --pipe    # Inkscape
```

**Quick PNG without a file:**
```bash
kuva scatter data.tsv -o /tmp/out.png      # requires --features png
```

**Themed dark output:**
```bash
kuva manhattan gwas.tsv --chr-col chr --pvalue-col pvalue \
    --theme dark --background "#1a1a2e" -o manhattan_dark.svg
```

**Colour-vision-deficiency palette:**
```bash
kuva scatter data.tsv --x time --y value --color-by group \
    --cvd-palette deuteranopia
```
