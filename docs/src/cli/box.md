# kuva box

Box-and-whisker plot. Groups are taken from one column; values from another.

**Input:** two columns — group label and numeric value, one observation per row.

| Flag | Default | Description |
|---|---|---|
| `--group-col <COL>` | `0` | Group label column |
| `--value-col <COL>` | `1` | Numeric value column |
| `--y <COL>[,<COL>…]` | — | Comma-separated columns; each column becomes a separate group (column name = group label). Overrides `--group-col` + `--value-col` when 2+ columns given |
| `--color <CSS>` | `steelblue` | Box fill color (uniform, all groups) |
| `--group-colors <CSS,...>` | — | Per-group colors, comma-separated; falls back to `--color` for unlisted groups |
| `--overlay-points` | off | Overlay individual points as a jittered strip |
| `--overlay-swarm` | off | Overlay individual points as a non-overlapping beeswarm |
| `--horizontal` | off | Render groups on the Y-axis, values on the X-axis |

```bash
kuva box samples.tsv --group-col group --value-col expression

kuva box samples.tsv --group-col group --value-col expression \
    --overlay-swarm --color "rgba(70,130,180,0.6)"

kuva box samples.tsv --group-col group --value-col expression \
    --group-colors "steelblue,tomato,seagreen,goldenrod,mediumpurple"

# multi-column: each numeric column is a group
kuva box data.tsv --y col_a,col_b,col_c

# horizontal layout
kuva box samples.tsv --group-col group --value-col expression --horizontal
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
