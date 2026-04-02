# kuva stacked-area

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
kuva stacked-area stacked_area.tsv \
    --x-col week --group-col species --y-col abundance

kuva stacked-area stacked_area.tsv \
    --x-col week --group-col species --y-col abundance \
    --normalize --y-label "Relative abundance (%)"
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
