# kuva heatmap

Color-encoded matrix heatmap.

**Input (wide format):** first column is the row label, remaining columns are numeric values. The header row (if present) supplies column labels.

```
gene    Sample_01  Sample_02  Sample_03 …
 TP53    0.25       -1.78       1.58     …
BRCA1   0.23        0.48       1.06     …
```

**Input (long format):** use `--long-format` to pass `(row, col, value)` triples instead. Missing combinations are filled with `0`. Column order defaults to 0/1/2; override with `--row-col`, `--col-col`, `--value-col`.

```
species     week  abundance
Firmicutes  1     352
Firmicutes  2     381
Bacteroidetes  1  262
```

| Flag | Default | Description |
|---|---|---|
| `--colormap <NAME>` | `viridis` | Color map: `viridis`, `inferno`, `grayscale` |
| `--values` | off | Print numeric values in each cell |
| `--legend <LABEL>` | — | Show color bar with this label |
| `--long-format` | off | Accept `(row, col, value)` triples instead of a wide matrix |
| `--row-col <COL>` | `0` | Row-label column (with `--long-format`) |
| `--col-col <COL>` | `1` | Column-label column (with `--long-format`) |
| `--value-col <COL>` | `2` | Value column (with `--long-format`) |

```bash
# wide matrix
kuva heatmap heatmap.tsv

kuva heatmap heatmap.tsv --colormap inferno --values --legend "z-score"

# long-format: species × week abundance table
kuva heatmap data.tsv --long-format \
    --row-col species --col-col week --value-col abundance \
    --title "Abundance by Species and Week"

# long-format from a counts table with named columns
kuva heatmap counts.tsv --long-format \
    --row-col gene --col-col sample --value-col tpm \
    --legend "TPM" --colormap inferno
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
