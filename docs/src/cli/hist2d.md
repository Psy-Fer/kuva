# kuva hist2d

Two-dimensional histogram (density grid) from two numeric columns.

**Input:** two numeric columns.

| Flag | Default | Description |
|---|---|---|
| `--x <COL>` | `0` | X-axis column |
| `--y <COL>` | `1` | Y-axis column |
| `--bins-x <N>` | `10` | Number of bins on the X axis |
| `--bins-y <N>` | `10` | Number of bins on the Y axis |
| `--colormap <NAME>` | `viridis` | Color map: `viridis`, `inferno`, `turbo`, `grayscale` |
| `--correlation` | off | Overlay Pearson correlation coefficient |
| `--log-count` | off | Log-scale the color mapping via `log₁₀(count+1)`. Useful when a dense core dominates the color scale and hides structure in surrounding low-density regions. Colorbar label updates to "log₁₀(Count + 1)" with tick marks at actual count values (1, 10, 100, …). |
| `--colorbar-tick-format <FMT>` | `auto` | Colorbar tick label format: `auto`, `sci`, `integer`, `fixed2`. `auto` renders integers as-is and switches to scientific notation when counts reach 10 000. |

```bash
# Basic density grid
kuva hist2d measurements.tsv --x time --y value

# Fine-grained bins with correlation annotation
kuva hist2d measurements.tsv --x time --y value \
    --bins-x 30 --bins-y 30 --colormap turbo --correlation

# Log color scale — reveals sparse structure around a dense core
kuva hist2d data.tsv --x x --y y --bins-x 30 --bins-y 30 --log-count

# Force scientific notation on the colorbar (e.g. for very large counts)
kuva hist2d data.tsv --x x --y y --colorbar-tick-format sci
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
