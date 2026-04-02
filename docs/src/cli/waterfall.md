# kuva waterfall

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
kuva waterfall waterfall.tsv --label-col process --value-col log2fc \
    --connectors --values

# mark two rows as summary bars
kuva waterfall income.tsv \
    --total "Gross profit" --total "Net income" \
    --connectors --values
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
