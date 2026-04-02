# kuva dot

Dot plot encoding two variables (size and color) at categorical (x, y) positions.

**Input:** four columns — x category, y category, size value, color value.

| Flag | Default | Description |
|---|---|---|
| `--x-col <COL>` | `0` | X-category column |
| `--y-col <COL>` | `1` | Y-category column |
| `--size-col <COL>` | `2` | Size-encoding column |
| `--color-col <COL>` | `3` | Color-encoding column |
| `--colormap <NAME>` | `viridis` | Color map |
| `--max-radius <PX>` | `12.0` | Maximum dot radius |
| `--size-legend <LABEL>` | — | Show size legend with this label |
| `--colorbar <LABEL>` | — | Show color bar with this label |

```bash
kuva dot dot.tsv \
    --x-col pathway --y-col cell_type \
    --size-col pct_expressed --color-col mean_expr

kuva dot dot.tsv \
    --x-col pathway --y-col cell_type \
    --size-col pct_expressed --color-col mean_expr \
    --size-legend "% expressed" --colorbar "mean expr"
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
