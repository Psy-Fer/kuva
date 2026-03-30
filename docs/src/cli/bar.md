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
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
