# visus CLI

`visus` is the command-line front-end for the visus plotting library. It reads tabular data from a TSV or CSV file (or stdin) and writes an SVG — or PNG/PDF with the right feature flag — to a file or stdout.

```
visus <SUBCOMMAND> [FILE] [OPTIONS]
```

---

## Building

```bash
cargo build --bin visus                  # SVG output only
cargo build --bin visus --features png   # adds PNG output via resvg
cargo build --bin visus --features pdf   # adds PDF output via svg2pdf
cargo build --bin visus --features full  # both PNG and PDF
```

After building, the binary is at `target/debug/visus` (or `target/release/visus` with `--release`).

---

## Input

Every subcommand takes an optional positional `FILE` argument. If omitted or `-`, data is read from **stdin**.

```bash
# from file
visus scatter data.tsv

# from stdin
cat data.tsv | visus scatter

# explicit stdin
visus scatter - < data.tsv
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
visus scatter data.tsv --x 0 --y 1          # by index
visus scatter data.tsv --x time --y value   # by name (requires header)
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
| `-o`, `--output <FILE>` | stdout (SVG) | Output file path |
| `--title <TEXT>` | — | Title displayed above the chart |
| `--width <PX>` | `800` | Canvas width in pixels |
| `--height <PX>` | `500` | Canvas height in pixels |
| `--theme <NAME>` | `light` | Theme: `light`, `dark`, `solarized`, `minimal` |
| `--palette <NAME>` | `category10` | Color palette for multi-series plots |
| `--cvd-palette <NAME>` | — | Colour-vision-deficiency palette: `deuteranopia`, `protanopia`, `tritanopia`. Overrides `--palette`. |
| `--background <COLOR>` | *(theme default)* | SVG background color (any CSS color string) |

### Axes *(most subcommands)*

| Flag | Default | Description |
|---|---|---|
| `--x-label <TEXT>` | — | X-axis label |
| `--y-label <TEXT>` | — | Y-axis label |
| `--ticks <N>` | `5` | Hint for number of tick marks |
| `--no-grid` | off | Disable background grid |

### Log scale *(scatter, line, histogram, hist2d)*

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

- [scatter](#scatter)
- [line](#line)
- [bar](#bar)
- [histogram](#histogram)
- [box](#box)
- [violin](#violin)
- [pie](#pie)
- [strip](#strip)
- [waterfall](#waterfall)
- [stacked-area](#stacked-area)
- [volcano](#volcano)
- [manhattan](#manhattan)
- [candlestick](#candlestick)
- [heatmap](#heatmap)
- [hist2d](#hist2d)
- [contour](#contour)
- [dot](#dot)
- [upset](#upset)
- [chord](#chord)
- [sankey](#sankey)
- [phylo](#phylo)
- [synteny](#synteny)

---

## scatter

Scatter plot of (x, y) point pairs. Supports multi-series coloring, trend lines, and log scale.

**Input:** any tabular file with two numeric columns.

| Flag | Default | Description |
|---|---|---|
| `--x <COL>` | `0` | X-axis column |
| `--y <COL>` | `1` | Y-axis column |
| `--color-by <COL>` | — | Group by this column; each group gets a distinct color |
| `--color <CSS>` | `steelblue` | Point color (single-series only) |
| `--size <PX>` | `3.0` | Point radius in pixels |
| `--trend` | off | Overlay a linear trend line |
| `--equation` | off | Annotate with regression equation (requires `--trend`) |
| `--correlation` | off | Annotate with Pearson R² (requires `--trend`) |
| `--legend` | off | Show legend |

```bash
visus scatter measurements.tsv --x time --y value --color steelblue

visus scatter measurements.tsv --x time --y value \
    --color-by group --legend --title "Expression over time"

visus scatter measurements.tsv --x time --y value \
    --trend --equation --correlation --log-y
```

---

## line

Line plot. Identical column flags to scatter; adds line-style options.

**Input:** any tabular file with two numeric columns, sorted by x.

| Flag | Default | Description |
|---|---|---|
| `--x <COL>` | `0` | X-axis column |
| `--y <COL>` | `1` | Y-axis column |
| `--color-by <COL>` | — | Multi-series grouping |
| `--color <CSS>` | `steelblue` | Line color (single-series) |
| `--stroke-width <PX>` | `2.0` | Line stroke width |
| `--dashed` | off | Dashed line style |
| `--dotted` | off | Dotted line style |
| `--fill` | off | Fill area under the line |
| `--legend` | off | Show legend |

```bash
visus line measurements.tsv --x time --y value --color-by group --legend

visus line measurements.tsv --x time --y value --fill --color "rgba(70,130,180,0.4)"
```

---

## bar

Bar chart from label/value pairs.

**Input:** first column labels, second column numeric values.

| Flag | Default | Description |
|---|---|---|
| `--label-col <COL>` | `0` | Label column |
| `--value-col <COL>` | `1` | Value column |
| `--color <CSS>` | `steelblue` | Bar fill color |
| `--bar-width <F>` | `0.8` | Bar width as a fraction of the slot |

```bash
visus bar bar.tsv --label-col category --value-col count --color "#4682b4"

visus bar bar.tsv --x-label "Pathway" --y-label "Gene count" \
    -o pathways.svg
```

---

## histogram

Frequency histogram from a single numeric column.

**Input:** one numeric column per row.

| Flag | Default | Description |
|---|---|---|
| `--value-col <COL>` | `0` | Value column |
| `--color <CSS>` | `steelblue` | Bar fill color |
| `--bins <N>` | `10` | Number of bins |
| `--normalize` | off | Normalize to probability density (area = 1) |

```bash
visus histogram histogram.tsv --value-col value --bins 30

visus histogram histogram.tsv --bins 20 --normalize \
    --title "Expression distribution" --y-label "Density"
```

---

## box

Box-and-whisker plot. Groups are taken from one column; values from another.

**Input:** two columns — group label and numeric value, one observation per row.

| Flag | Default | Description |
|---|---|---|
| `--group-col <COL>` | `0` | Group label column |
| `--value-col <COL>` | `1` | Numeric value column |
| `--color <CSS>` | `steelblue` | Box fill color |
| `--overlay-points` | off | Overlay individual points as a jittered strip |
| `--overlay-swarm` | off | Overlay individual points as a non-overlapping beeswarm |

```bash
visus box samples.tsv --group-col group --value-col expression

visus box samples.tsv --group-col group --value-col expression \
    --overlay-swarm --color "rgba(70,130,180,0.6)"
```

---

## violin

Kernel-density violin plot. Same input format as `box`.

**Input:** two columns — group label and numeric value, one observation per row.

| Flag | Default | Description |
|---|---|---|
| `--group-col <COL>` | `0` | Group label column |
| `--value-col <COL>` | `1` | Numeric value column |
| `--color <CSS>` | `steelblue` | Violin fill color |
| `--bandwidth <F>` | *(Silverman)* | KDE bandwidth |
| `--overlay-points` | off | Overlay individual points as a jittered strip |
| `--overlay-swarm` | off | Overlay individual points as a non-overlapping beeswarm |

```bash
visus violin samples.tsv --group-col group --value-col expression

visus violin samples.tsv --group-col group --value-col expression \
    --overlay-swarm --bandwidth 0.3
```

---

## pie

Pie or donut chart.

**Input:** label column + numeric value column.

| Flag | Default | Description |
|---|---|---|
| `--label-col <COL>` | `0` | Label column |
| `--value-col <COL>` | `1` | Value column |
| `--color-col <COL>` | — | Optional CSS color column |
| `--donut` | off | Render as a donut (hollow center) |
| `--inner-radius <PX>` | `80` | Donut hole radius in pixels |
| `--percent` | off | Append percentage to slice labels |
| `--label-position <MODE>` | *(auto)* | `inside`, `outside`, or `none` |
| `--legend` | off | Show legend |

```bash
visus pie pie.tsv --label-col feature --value-col percentage --percent

visus pie pie.tsv --label-col feature --value-col percentage \
    --donut --legend --label-position outside
```

---

## strip

Strip / jitter plot — individual points along a categorical axis.

**Input:** group label column + numeric value column, one observation per row.

| Flag | Default | Description |
|---|---|---|
| `--group-col <COL>` | `0` | Group label column |
| `--value-col <COL>` | `1` | Numeric value column |
| `--color <CSS>` | `steelblue` | Point color |
| `--point-size <PX>` | `4.0` | Point radius in pixels |
| `--swarm` | off | Beeswarm (non-overlapping) layout |
| `--center` | off | All points at group center (no spread) |

Default layout when neither `--swarm` nor `--center` is given: random jitter (±30 % of slot width).

```bash
visus strip samples.tsv --group-col group --value-col expression

visus strip samples.tsv --group-col group --value-col expression --swarm
```

---

## waterfall

Waterfall / bridge chart showing a running total built from incremental bars.

**Input:** label column + numeric value column. Mark subtotal/total bars with `--total`.

| Flag | Default | Description |
|---|---|---|
| `--label-col <COL>` | `0` | Label column |
| `--value-col <COL>` | `1` | Value column |
| `--total <LABEL>` | — | Mark this label as a summary bar (repeatable) |
| `--connectors` | off | Draw dashed connector lines between bars |
| `--values` | off | Print numeric values on each bar |
| `--color-pos <CSS>` | green | Positive delta bar color |
| `--color-neg <CSS>` | red | Negative delta bar color |
| `--color-total <CSS>` | `steelblue` | Total/subtotal bar color |

```bash
visus waterfall waterfall.tsv --label-col process --value-col log2fc \
    --connectors --values

# mark two rows as summary bars
visus waterfall income.tsv \
    --total "Gross profit" --total "Net income" \
    --connectors --values
```

---

## stacked-area

Stacked area chart in long format.

**Input:** three columns — x value, group label, y value — one observation per row. Rows are grouped by the group column; within each group the x/y pairs are collected in order.

| Flag | Default | Description |
|---|---|---|
| `--x-col <COL>` | `0` | X-axis column |
| `--group-col <COL>` | `1` | Series group column |
| `--y-col <COL>` | `2` | Y-axis column |
| `--normalize` | off | Normalize each x-position to 100 % |
| `--fill-opacity <F>` | `0.7` | Fill opacity for each band |

```bash
visus stacked-area stacked_area.tsv \
    --x-col week --group-col species --y-col abundance

visus stacked-area stacked_area.tsv \
    --x-col week --group-col species --y-col abundance \
    --normalize --y-label "Relative abundance (%)"
```

---

## volcano

Volcano plot for differential expression results.

**Input:** three columns — gene name, log₂ fold change, raw p-value.

| Flag | Default | Description |
|---|---|---|
| `--name-col <COL>` | `0` | Gene/feature name column |
| `--x-col <COL>` | `1` | log₂FC column |
| `--y-col <COL>` | `2` | p-value column (raw, not −log₁₀) |
| `--fc-cutoff <F>` | `1.0` | \|log₂FC\| threshold |
| `--p-cutoff <F>` | `0.05` | p-value significance threshold |
| `--top-n <N>` | `0` | Label the N most-significant points |
| `--color-up <CSS>` | `firebrick` | Up-regulated point color |
| `--color-down <CSS>` | `steelblue` | Down-regulated point color |
| `--color-ns <CSS>` | `#aaaaaa` | Not-significant point color |
| `--point-size <PX>` | `3.0` | Point radius |
| `--legend` | off | Show Up / Down / NS legend |

```bash
visus volcano gene_stats.tsv \
    --name-col gene --x-col log2fc --y-col pvalue \
    --top-n 20 --legend

visus volcano gene_stats.tsv \
    --name-col gene --x-col log2fc --y-col pvalue \
    --fc-cutoff 2.0 --p-cutoff 0.01 --top-n 10
```

---

## manhattan

Manhattan plot for GWAS results.

**Input:** chromosome, (optional) base-pair position, and p-value columns.

Two layout modes:
- **Sequential** *(default)*: chromosomes are sorted and SNPs receive consecutive integer x-positions. Position column is not used.
- **Base-pair** (`--genome-build`): SNP x-coordinates are resolved from chromosome sizes in a reference build.

| Flag | Default | Description |
|---|---|---|
| `--chr-col <COL>` | `0` | Chromosome column |
| `--pos-col <COL>` | `1` | Base-pair position column (bp mode only) |
| `--pvalue-col <COL>` | `2` | p-value column |
| `--genome-build <BUILD>` | — | Enable bp mode: `hg19`, `hg38`, or `t2t` |
| `--genome-wide <F>` | `7.301` | Genome-wide threshold (−log₁₀ scale) |
| `--suggestive <F>` | `5.0` | Suggestive threshold (−log₁₀ scale) |
| `--top-n <N>` | `0` | Label N most-significant points above genome-wide threshold |
| `--point-size <PX>` | `2.5` | Point radius |
| `--color-a <CSS>` | `steelblue` | Even-chromosome color |
| `--color-b <CSS>` | `#5aadcb` | Odd-chromosome color |
| `--legend` | off | Show threshold legend |

```bash
# sequential mode (no position column needed)
visus manhattan gene_stats.tsv --chr-col chr --pvalue-col pvalue --top-n 5

# base-pair mode
visus manhattan gwas.tsv \
    --chr-col chr --pos-col pos --pvalue-col pvalue \
    --genome-build hg38 --top-n 10 --legend
```

---

## candlestick

OHLC candlestick chart.

**Input:** label, open, high, low, close columns (and optionally volume).

| Flag | Default | Description |
|---|---|---|
| `--label-col <COL>` | `0` | Period label column |
| `--open-col <COL>` | `1` | Open price column |
| `--high-col <COL>` | `2` | High price column |
| `--low-col <COL>` | `3` | Low price column |
| `--close-col <COL>` | `4` | Close price column |
| `--volume-col <COL>` | — | Optional volume column |
| `--volume-panel` | off | Show volume bar panel below price chart |
| `--candle-width <F>` | `0.7` | Body width as a fraction of slot |
| `--color-up <CSS>` | green | Bullish candle color |
| `--color-down <CSS>` | red | Bearish candle color |
| `--color-doji <CSS>` | `#888888` | Doji candle color |

```bash
visus candlestick candlestick.tsv \
    --label-col date --open-col open --high-col high \
    --low-col low --close-col close

visus candlestick candlestick.tsv \
    --label-col date --open-col open --high-col high \
    --low-col low --close-col close \
    --volume-col volume --volume-panel
```

---

## heatmap

Color-encoded matrix heatmap.

**Input:** wide-format matrix — first column is the row label, remaining columns are numeric values. The header row (if present) supplies column labels.

```
gene    Sample_01  Sample_02  Sample_03 …
TP53    0.25       -1.78       1.58     …
BRCA1   0.23        0.48       1.06     …
```

| Flag | Default | Description |
|---|---|---|
| `--colormap <NAME>` | `viridis` | Color map: `viridis`, `inferno`, `grayscale` |
| `--values` | off | Print numeric values in each cell |
| `--legend <LABEL>` | — | Show color bar with this label |

```bash
visus heatmap heatmap.tsv

visus heatmap heatmap.tsv --colormap inferno --values --legend "z-score"
```

---

## hist2d

Two-dimensional histogram (density grid) from two numeric columns.

**Input:** two numeric columns.

| Flag | Default | Description |
|---|---|---|
| `--x <COL>` | `0` | X-axis column |
| `--y <COL>` | `1` | Y-axis column |
| `--bins-x <N>` | `10` | Number of bins on the X axis |
| `--bins-y <N>` | `10` | Number of bins on the Y axis |
| `--colormap <NAME>` | `viridis` | Color map: `viridis`, `inferno`, `grayscale` |
| `--correlation` | off | Overlay Pearson correlation coefficient |

```bash
visus hist2d measurements.tsv --x time --y value

visus hist2d measurements.tsv --x time --y value \
    --bins-x 20 --bins-y 20 --correlation
```

---

## contour

Contour plot from scattered (x, y, z) triplets.

**Input:** three columns — x coordinate, y coordinate, scalar value.

| Flag | Default | Description |
|---|---|---|
| `--x <COL>` | `0` | X column |
| `--y <COL>` | `1` | Y column |
| `--z <COL>` | `2` | Scalar value column |
| `--levels <N>` | `8` | Number of contour levels |
| `--filled` | off | Fill between contour levels |
| `--colormap <NAME>` | `viridis` | Color map (filled mode) |
| `--line-color <CSS>` | — | Line color (unfilled mode) |
| `--legend <LABEL>` | — | Show legend entry |

```bash
visus contour contour.tsv --x x --y y --z density

visus contour contour.tsv --x x --y y --z density \
    --filled --levels 12 --colormap inferno
```

---

## dot

Dot plot encoding two variables (size and color) at categorical (x, y) positions.

**Input:** four columns — x category, y category, size value, color value.

| Flag | Default | Description |
|---|---|---|
| `--x-col <COL>` | `0` | X-category column |
| `--y-col <COL>` | `1` | Y-category column |
| `--size-col <COL>` | `2` | Size-encoding column |
| `--color-col <COL>` | `3` | Color-encoding column |
| `--colormap <NAME>` | `viridis` | Color map |
| `--max-radius <PX>` | `12.0` | Maximum dot radius |
| `--size-legend <LABEL>` | — | Show size legend with this label |
| `--colorbar <LABEL>` | — | Show color bar with this label |

```bash
visus dot dot.tsv \
    --x-col pathway --y-col cell_type \
    --size-col pct_expressed --color-col mean_expr

visus dot dot.tsv \
    --x-col pathway --y-col cell_type \
    --size-col pct_expressed --color-col mean_expr \
    --size-legend "% expressed" --colorbar "mean expr"
```

---

## upset

UpSet plot for set-intersection analysis.

**Input:** binary (0/1) matrix — one column per set, one row per element. Column headers become set names.

```
GWAS_hit  eQTL  Splicing_QTL  Methylation_QTL  Conservation  ClinVar
1         0     0             1                1             1
0         0     1             1                1             0
```

| Flag | Default | Description |
|---|---|---|
| `--sort <MODE>` | `frequency` | Sort intersections: `frequency`, `degree`, `natural` |
| `--max-visible <N>` | — | Show only the top N intersections |

```bash
visus upset upset.tsv

visus upset upset.tsv --sort degree --max-visible 15
```

---

## chord

Chord diagram for pairwise flow data.

**Input:** square N×N matrix — first column is the row label (ignored for layout), header row supplies node names.

```
region        Cortex  Hippocampus  Amygdala …
Cortex        0       320          13       …
Hippocampus   320     0            210      …
```

| Flag | Default | Description |
|---|---|---|
| `--gap <DEG>` | `2.0` | Gap between arcs in degrees |
| `--opacity <F>` | `0.7` | Ribbon opacity |
| `--legend <LABEL>` | — | Show legend |

```bash
visus chord chord.tsv

visus chord chord.tsv --gap 3.0 --opacity 0.5 --legend "connectivity"
```

---

## sankey

Sankey / alluvial flow diagram.

**Input:** three columns — source node, target node, flow value.

| Flag | Default | Description |
|---|---|---|
| `--source-col <COL>` | `0` | Source node column |
| `--target-col <COL>` | `1` | Target node column |
| `--value-col <COL>` | `2` | Flow value column |
| `--link-gradient` | off | Fill each link with a gradient from source node colour to target node colour |
| `--opacity <F>` | `0.5` | Link opacity |
| `--legend <LABEL>` | — | Show legend |

```bash
visus sankey sankey.tsv \
    --source-col source --target-col target --value-col value

visus sankey sankey.tsv \
    --source-col source --target-col target --value-col value \
    --link-gradient --legend "read flow"
```

---

## phylo

Phylogenetic tree from a Newick string or edge-list TSV.

**Input (default):** edge-list TSV with parent, child, and branch-length columns.

**Input (alternative):** pass `--newick` with a Newick string; the file argument is not used.

| Flag | Default | Description |
|---|---|---|
| `--newick <STR>` | — | Newick string (overrides file input) |
| `--parent-col <COL>` | `0` | Parent node column |
| `--child-col <COL>` | `1` | Child node column |
| `--length-col <COL>` | `2` | Branch length column |
| `--orientation <DIR>` | `left` | `left`, `right`, `top`, `bottom` |
| `--branch-style <STYLE>` | `rectangular` | `rectangular`, `slanted`, `circular` |
| `--phylogram` | off | Scale branches by length |
| `--legend <LABEL>` | — | Show legend |

```bash
# from edge-list TSV
visus phylo phylo.tsv \
    --parent-col parent --child-col child --length-col length

# from Newick string
visus phylo --newick "((A:0.1,B:0.2):0.3,C:0.4);" --branch-style circular

# phylogram, top orientation
visus phylo phylo.tsv \
    --parent-col parent --child-col child --length-col length \
    --phylogram --orientation top
```

---

## synteny

Synteny / genomic alignment ribbon plot.

**Input:** two files:
- **Sequences file** *(positional)*: TSV with sequence name and length columns.
- **Blocks file** (`--blocks-file`): TSV with columns `seq1, start1, end1, seq2, start2, end2, strand`.

```
# sequences.tsv
name    length
Chr1A   2800000
Chr1B   2650000

# blocks.tsv
seq1   start1  end1    seq2   start2  end2    strand
Chr1A  56000   137237  Chr1B  63958   143705  +
Chr1A  150674  271188  Chr1B  165366  303075  -
```

| Flag | Default | Description |
|---|---|---|
| `--blocks-file <FILE>` | *(required)* | Blocks TSV file |
| `--bar-height <PX>` | `18.0` | Sequence bar height in pixels |
| `--opacity <F>` | `0.65` | Block ribbon opacity |
| `--proportional` | off | Scale bar widths proportionally to sequence length |
| `--legend <LABEL>` | — | Show legend |

```bash
visus synteny synteny_seqs.tsv --blocks-file synteny_blocks.tsv

visus synteny synteny_seqs.tsv --blocks-file synteny_blocks.tsv \
    --proportional --legend "synteny blocks"
```

---

## Tips

**Pipe to a viewer:**
```bash
visus scatter data.tsv | display            # ImageMagick
visus scatter data.tsv | inkscape --pipe    # Inkscape
```

**Quick PNG without a file:**
```bash
visus scatter data.tsv -o /tmp/out.png      # requires --features png
```

**Themed dark output:**
```bash
visus manhattan gwas.tsv --chr-col chr --pvalue-col pvalue \
    --theme dark --background "#1a1a2e" -o manhattan_dark.svg
```

**Colour-vision-deficiency palette:**
```bash
visus scatter data.tsv --x time --y value --color-by group \
    --cvd-palette deuteranopia
```
