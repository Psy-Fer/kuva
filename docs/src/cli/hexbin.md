# kuva hexbin

Hexagonal-bin density plot from two numeric columns.

**Input:** two numeric columns (x and y); optionally a third column for z aggregation.

| Flag | Default | Description |
|---|---|---|
| `--x <COL>` | `0` | X-axis column (name or index) |
| `--y <COL>` | `1` | Y-axis column (name or index) |
| `--z <COL>` | — | Third variable column for aggregation-based coloring |
| `--reduce <FUNC>` | `count` | Aggregation for z: `count`, `mean`, `sum`, `median`, `min`, `max` |
| `--n-bins <N>` | `20` | Number of hex columns across the x-axis |
| `--log-color` | off | Log₁₀ color scale — compresses high-count peaks |
| `--min-count <N>` | `1` | Suppress bins with fewer than N points |
| `--normalize` | off | Divide counts by total points (fractional density) |
| `--flat-top` | off | Flat-top hex orientation instead of pointy-top |
| `--stroke <COLOR>` | — | Hex outline color (CSS string, e.g. `"#333333"`) |
| `--colormap <NAME>` | `viridis` | Color map: `viridis`, `inferno`, `turbo`, `grayscale` |
| `--no-colorbar` | off | Hide the colorbar |

```bash
# Basic density plot from two columns
kuva hexbin data.tsv --x x --y y

# More bins for finer structure
kuva hexbin data.tsv --x x --y y --n-bins 40

# Log scale — reveals sparse structure around a dense core
kuva hexbin data.tsv --x x --y y --log-color

# Suppress peripheral noise — only render bins with ≥5 points
kuva hexbin data.tsv --x x --y y --min-count 5

# Fractional density (0–1 scale)
kuva hexbin data.tsv --x x --y y --normalize

# Color by mean of a third variable
kuva hexbin data.tsv --x x --y y --z value --reduce mean

# Flat-top orientation with Inferno colormap
kuva hexbin data.tsv --x x --y y --flat-top --colormap inferno

# Add hex outlines
kuva hexbin data.tsv --x x --y y --stroke "#444444"
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
