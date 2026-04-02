# kuva pie

Pie or donut chart.

**Input:** label column + numeric value column.

| Flag | Default | Description |
|---|---|---|
| `--label-col <COL>` | `0` | Label column |
| `--value-col <COL>` | `1` | Value column |
| `--count-by <COL>` | — | Count occurrences per unique value in this column (ignores `--value-col`) |
| `--color-col <COL>` | — | Optional CSS color column |
| `--donut` | off | Render as a donut (hollow center) |
| `--inner-radius <PX>` | `80` | Donut hole radius in pixels |
| `--percent` | off | Append percentage to slice labels |
| `--label-position <MODE>` | *(auto)* | `inside`, `outside`, or `none` |
| `--legend` | off | Show legend |

```bash
kuva pie pie.tsv --label-col feature --value-col percentage --percent

kuva pie pie.tsv --label-col feature --value-col percentage \
    --donut --legend --label-position outside

# count occurrences of each group
kuva pie scatter.tsv --count-by group --percent --legend
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
