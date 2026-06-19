# kuva histogram

Frequency histogram from one or more numeric columns.

**Input:** one or more numeric columns per row.

| Flag | Default | Description |
|---|---|---|
| `--value-col <COL>` | `0` | Value column (single-column mode) |
| `--y <COL>[,<COL>…]` | — | Comma-separated columns; overlays one histogram per column over a shared x-range (overrides `--value-col`) |
| `--color <CSS>` | `steelblue` | Bar fill color (single-column mode) |
| `--bins <N>` | `10` | Number of bins |
| `--normalize` | off | Normalize to probability density (area = 1) |
| `--legend` | off | Show a legend entry for each series (multi-column mode) |

```bash
kuva histogram histogram.tsv --value-col value --bins 30

kuva histogram histogram.tsv --bins 20 --normalize \
    --title "Expression distribution" --y-label "Density"

# overlay distributions of two columns
kuva histogram data.tsv --y col_a,col_b --bins 20 --legend
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
