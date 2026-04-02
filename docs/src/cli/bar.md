# kuva bar

Bar chart from label/value pairs.

**Input:** first column labels, second column numeric values.

| Flag | Default | Description |
|---|---|---|
| `--label-col <COL>` | `0` | Label column |
| `--value-col <COL>` | `1` | Value column |
| `--count-by <COL>` | — | Count occurrences per unique value in this column (ignores `--value-col`) |
| `--agg <FUNC>` | — | Aggregate `--value-col` by `--label-col`: `mean`, `median`, `sum`, `min`, `max` |
| `--color <CSS>` | `steelblue` | Bar fill color |
| `--bar-width <F>` | `0.8` | Bar width as a fraction of the slot |
| `--color-by <COL>` | — | Group rows by this column and color each series by palette, producing a grouped bar chart with an automatic legend |

### Grouped bar chart (`--color-by`)

`--color-by` pivots the data into a grouped bar chart — one colored sub-bar per unique value in the specified column, using the active palette. When every x-label maps to exactly one series value (e.g. `--color-by` on the same column as `--label-col`), kuva falls back to simple per-bar coloring so bars stay centered under their tick labels.

```bash
kuva bar bar.tsv --label-col category --value-col count --color "#4682b4"

kuva bar bar.tsv --x-label "Pathway" --y-label "Gene count" \
    -o pathways.svg

# count occurrences of each group
kuva bar scatter.tsv --count-by group --y-label "Count"

# aggregate: total abundance per species from long-format data
kuva bar data.tsv --label-col species --value-col abundance --agg sum

# mean expression per gene across samples
kuva bar expr.tsv --label-col gene --value-col tpm --agg mean \
    --y-label "Mean TPM"

# grouped bar chart: one bar per species per condition
kuva bar data.tsv --label-col species --value-col abundance \
    --color-by condition -o grouped.svg

# interactive grouped bar with legend toggle
kuva bar data.tsv --label-col species --value-col abundance \
    --color-by condition --interactive -o grouped_interactive.svg
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
