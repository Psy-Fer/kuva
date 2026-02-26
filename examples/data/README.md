# Example data

Tab-separated files for testing and demonstrating every `visus` subcommand.
All files have a header row. Re-generate with `python3 generate.py` (requires numpy).

---

## Shared datasets

Several subcommands share the same underlying file to show that the same data
can be visualised in multiple ways.

### `samples.tsv` — 600 rows
Columns: `group`, `expression`
Five treatment groups (Control, Drug_A, Drug_B, Drug_C, Drug_D) with 120
samples each. Drug_B is bimodal; Drug_C is wide/noisy; Drug_D is right-skewed.

```bash
visus box   samples.tsv --group-col group --value-col expression --title "Expression by treatment"
visus violin samples.tsv --group-col group --value-col expression --title "Expression by treatment"
visus strip  samples.tsv --group-col group --value-col expression
visus violin samples.tsv --group-col group --value-col expression --swarm
```

### `measurements.tsv` — 450 rows
Columns: `group`, `time`, `value`
Three experimental conditions (Condition_A/B/C), each with 150 time-points
sampled uniformly over 0–100. Each condition has a distinct trend.

```bash
visus scatter measurements.tsv --x time --y value --color-by group --trend
visus line    measurements.tsv --x time --y value --color-by group
visus hist2d  measurements.tsv --x time --y value
visus contour measurements.tsv --x time --y value --filled
```

### `gene_stats.tsv` — 8 000 rows
Columns: `gene`, `chr`, `pos`, `basemean`, `log2fc`, `pvalue`, `padj`
Simulated differential-expression results. ~5 % of genes are truly DE
(large |log2fc|, low p-value); the rest are null.

```bash
visus volcano  gene_stats.tsv --x log2fc --y pvalue --label-col gene --top-n 20
visus manhattan gene_stats.tsv --chr chr --pos pos --pvalue pvalue --label-col gene --top-n 10
```

---

## Individual datasets

### `bar.tsv` — 20 rows
Columns: `category`, `count`
GO-term enrichment hit counts, sorted descending.

```bash
visus bar bar.tsv --label-col category --value-col count --x-label "GO term" --y-label "Hits"
```

### `histogram.tsv` — 900 rows
Columns: `value`
Bimodal fragment-length distribution (two overlapping normal peaks).

```bash
visus histogram histogram.tsv --value-col value --bins 40 --title "Fragment length distribution"
```

### `pie.tsv` — 8 rows
Columns: `feature`, `percentage`
Genomic feature composition (Exon, Intron, Intergenic, UTRs, Promoter, …).

```bash
visus pie pie.tsv --label-col feature --value-col percentage --percent --label-position outside
```

### `heatmap.tsv` — 30 rows (wide matrix)
Columns: `gene`, `Sample_01` … `Sample_12`
Z-score normalised expression for 30 cancer-related genes across 12 samples
(4 groups of 3: Control, TreatA, TreatB, TreatC).

```bash
visus heatmap heatmap.tsv --row-col gene --title "Expression heatmap"
```

### `waterfall.tsv` — 25 rows
Columns: `process`, `log2fc`
Biological processes sorted by fold change (mix of up- and down-regulated).

```bash
visus waterfall waterfall.tsv --label-col process --value-col log2fc
```

### `stacked_area.tsv` — 312 rows
Columns: `week`, `species`, `abundance`
Long-format microbiome abundances for 6 taxa over 52 weeks. Abundances
sum to ~100 per week.

```bash
visus stacked-area stacked_area.tsv --x week --group species --y abundance
visus stacked-area stacked_area.tsv --x week --group species --y abundance --normalize
```

### `candlestick.tsv` — 200 rows
Columns: `date`, `open`, `high`, `low`, `close`, `volume`
200 trading days of simulated OHLC price data starting 2023-01-02.

```bash
visus candlestick candlestick.tsv --date date --open open --high high --low low --close close --volume volume
```

### `contour.tsv` — 600 rows
Columns: `x`, `y`, `density`
Scattered points with density values from a two-peak 2-D Gaussian mixture.

```bash
visus contour contour.tsv --x x --y y --z density --filled
visus contour contour.tsv --x x --y y --z density --levels 8
```

### `dot.tsv` — 56 rows
Columns: `pathway`, `cell_type`, `mean_expr`, `pct_expressed`
8 metabolic pathways × 7 cell types. Dot size = `pct_expressed`,
dot colour = `mean_expr`.

```bash
visus dot dot.tsv --row pathway --col cell_type --size pct_expressed --color mean_expr
```

### `upset.tsv` — 400 rows
Columns: `GWAS_hit`, `eQTL`, `Splicing_QTL`, `Methylation_QTL`, `Conservation`, `ClinVar`
Binary (0/1) set membership for 400 genomic variants across 6 annotation sets.

```bash
visus upset upset.tsv
```

### `chord.tsv` — 8×8 matrix
Columns: `region`, `Cortex`, `Hippocampus`, … `Hypothalamus`
Neural connectivity strengths between 8 brain regions (symmetric matrix,
diagonal = 0).

```bash
visus chord chord.tsv --legend
```

### `sankey.tsv` — 22 rows
Columns: `source`, `target`, `value`
RNA-seq read fate: raw reads flowing through trimming, alignment, and
feature assignment stages.

```bash
visus sankey sankey.tsv --source source --target target --value value
```

### `phylo.tsv` — 38 rows
Columns: `parent`, `child`, `length`
Edge list for a rooted vertebrate species tree (20 leaf taxa, 19 internal
nodes). Branch lengths are biologically plausible.

```bash
visus phylo phylo.tsv --format edges --parent parent --child child --length length
visus phylo phylo.tsv --format edges --style circular
```

### `synteny_seqs.tsv` + `synteny_blocks.tsv`
`synteny_seqs.tsv` (4 rows): `name`, `length` — four chromosomes (Chr1A, Chr1B, Chr2A, Chr2B).
`synteny_blocks.tsv` (21 rows): `seq1`, `start1`, `end1`, `seq2`, `start2`, `end2`, `strand` — syntenic blocks; ~20 % are inversions (strand = −).

```bash
visus synteny --seqs synteny_seqs.tsv --blocks synteny_blocks.tsv
```

### `reads.tsv` — 350 rows
Columns: `name`, `start`, `end`, `strand`
Simulated short reads on a ~8 000 bp region, with a read-density peak
around 2 000–4 000 bp.

```bash
visus brick reads.tsv --start start --end end --strand strand
```

### `strip` (use `samples.tsv`)
The standalone strip/swarm plot uses the same data as box/violin:

```bash
visus strip samples.tsv --group-col group --value-col expression --swarm
```
