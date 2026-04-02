# kuva line

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
kuva line measurements.tsv --x time --y value --color-by group --legend

kuva line measurements.tsv --x time --y value --fill --color "rgba(70,130,180,0.4)"
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
