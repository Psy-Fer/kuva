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

If the first field of the first row fails to parse as a number, the row is treated as a header. Override with `--no-header`.

### Column selection

Columns are selected by **0-based integer index** or **header name**:

```bash
kuva scatter data.tsv --x 0 --y 1          # by index
kuva scatter data.tsv --x time --y value   # by name (requires header)
```

---

## Output

| Flag | Effect |
|---|---|
| *(omitted)* | SVG to stdout |
| `-o out.svg` | SVG to file |
| `-o out.png` | PNG (requires `--features png`) |
| `-o out.pdf` | PDF (requires `--features pdf`) |

Format is inferred from the file extension. Any unrecognised extension is treated as SVG.

---

## Shared flags

These flags are available on every subcommand.

### Output & appearance

| Flag | Default | Description |
|---|---|---|
| `-o`, `--output <FILE>` | stdout (SVG) | Output file path (mutually exclusive with `--terminal`) |
| `--title <TEXT>` | — | Title displayed above the chart |
| `--width <PX>` | `800` | Canvas width in pixels |
| `--height <PX>` | `500` | Canvas height in pixels |
| `--theme <NAME>` | `light` | Theme: `light`, `dark`, `solarized`, `minimal` |
| `--palette <NAME>` | `category10` | Color palette for multi-series plots |
| `--cvd-palette <NAME>` | — | Colour-vision-deficiency palette: `deuteranopia`, `protanopia`, `tritanopia`. Overrides `--palette`. |
| `--background <COLOR>` | *(theme default)* | SVG background color (any CSS color string) |

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
- **Save button** — top-right button serialises the current SVG DOM (including any pinned/dimmed state). *Note: the download is not yet functional; this will be fixed in v0.2.*

Supported in this release: `scatter`, `line`, `bar`, `strip`, `volcano`. All other subcommands accept `--interactive` and load the UI chrome (coordinate readout, search box) but do not yet have per-point hover/search — remaining renderers will be wired in v0.2.

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

### Input

| Flag | Description |
|---|---|
| `--no-header` | Treat first row as data, not a header |
| `-d`, `--delimiter <CHAR>` | Override field delimiter |

---

## Subcommands

| Subcommand | Description |
|---|---|
| [scatter](./scatter.md) | Scatter plot of (x, y) point pairs |
| [line](./line.md) | Line plot |
| [bar](./bar.md) | Bar chart from label/value pairs |
| [histogram](./histogram.md) | Frequency histogram from a single numeric column |
| [density](./density.md) | Kernel density estimate curve |
| [ridgeline](./ridgeline.md) | Stacked KDE density curves, one per group |
| [box](./box.md) | Box-and-whisker plot |
| [violin](./violin.md) | Kernel-density violin plot |
| [pie](./pie.md) | Pie or donut chart |
| [forest](./forest.md) | Forest plot — point estimates with confidence intervals |
| [strip](./strip.md) | Strip / jitter plot |
| [waterfall](./waterfall.md) | Waterfall / bridge chart |
| [stacked-area](./stacked_area.md) | Stacked area chart |
| [volcano](./volcano.md) | Volcano plot for differential expression |
| [manhattan](./manhattan.md) | Manhattan plot for GWAS results |
| [candlestick](./candlestick.md) | OHLC candlestick chart |
| [heatmap](./heatmap.md) | Color-encoded matrix heatmap |
| [hist2d](./hist2d.md) | Two-dimensional histogram |
| [contour](./contour.md) | Contour plot from scattered (x, y, z) triplets |
| [dot](./dot.md) | Dot plot (size + color at categorical positions) |
| [upset](./upset.md) | UpSet plot for set-intersection analysis |
| [chord](./chord.md) | Chord diagram for pairwise flow data |
| [sankey](./sankey.md) | Sankey / alluvial flow diagram |
| [phylo](./phylo.md) | Phylogenetic tree |
| [synteny](./synteny.md) | Synteny / genomic alignment ribbon plot |
| [polar](./polar.md) | Polar coordinate scatter/line plot |
| [ternary](./ternary.md) | Ternary (simplex) scatter plot |

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
