# kuva scatter

Scatter plot of (x, y) point pairs. Supports multi-series coloring, trend lines, and log scale.

**Input:** any tabular file with two numeric columns.

| Flag | Default | Description |
|---|---|---|
| `--x <COL>` | `0` | X-axis column |
| `--y <COL>` | `1` | Y-axis column |
| `--color-by <COL>` | — | Group by this column; each group gets a distinct color |
| `--color <CSS>` | `steelblue` | Point color (single-series only) |
| `--size <PX>` | `3.0` | Point radius in pixels |
| `--trend` | off | Overlay a linear trend line |
| `--equation` | off | Annotate with regression equation (requires `--trend`) |
| `--correlation` | off | Annotate with Pearson R² (requires `--trend`) |
| `--legend` | off | Show legend |

```bash
kuva scatter measurements.tsv --x time --y value --color steelblue

kuva scatter measurements.tsv --x time --y value \
    --color-by group --legend --title "Expression over time"

kuva scatter measurements.tsv --x time --y value \
    --trend --equation --correlation --log-y
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
