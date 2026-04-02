# kuva contour

Contour plot from scattered (x, y, z) triplets.

**Input:** three columns — x coordinate, y coordinate, scalar value.

| Flag | Default | Description |
|---|---|---|
| `--x <COL>` | `0` | X column |
| `--y <COL>` | `1` | Y column |
| `--z <COL>` | `2` | Scalar value column |
| `--levels <N>` | `8` | Number of contour levels |
| `--filled` | off | Fill between contour levels |
| `--colormap <NAME>` | `viridis` | Color map (filled mode) |
| `--line-color <CSS>` | — | Line color (unfilled mode) |
| `--legend <LABEL>` | — | Show legend entry |

```bash
kuva contour contour.tsv --x x --y y --z density

kuva contour contour.tsv --x x --y y --z density \
    --filled --levels 12 --colormap inferno
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
