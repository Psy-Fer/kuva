# kuva histogram

Frequency histogram from a single numeric column.

**Input:** one numeric column per row.

| Flag | Default | Description |
|---|---|---|
| `--value-col <COL>` | `0` | Value column |
| `--color <CSS>` | `steelblue` | Bar fill color |
| `--bins <N>` | `10` | Number of bins |
| `--normalize` | off | Normalize to probability density (area = 1) |

```bash
kuva histogram histogram.tsv --value-col value --bins 30

kuva histogram histogram.tsv --bins 20 --normalize \
    --title "Expression distribution" --y-label "Density"
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
