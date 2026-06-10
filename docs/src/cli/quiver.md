# kuva quiver

Quiver plot — 2-D vector field rendered as arrows. Each row is one arrow with a tail at `(x, y)` and a vector `(u, v)`.

**Input:** one row per arrow with four numeric columns: tail x, tail y, u-component, v-component.

| Flag | Default | Description |
|---|---|---|
| `--x-col <COL>` | `0` | Tail X column (0-based index or header name) |
| `--y-col <COL>` | `1` | Tail Y column |
| `--u-col <COL>` | `2` | Vector x-component column |
| `--v-col <COL>` | `3` | Vector y-component column |
| `--color <CSS>` | `steelblue` | Arrow color (ignored when `--colormap` is set) |
| `--arrow-scale <F>` | — | Pin the multiplier applied to `(u, v)` before axis mapping. Disables auto-scaling |
| `--auto-scale <F>` | `0.9` | Fraction of the nearest-neighbor distance used for the longest arrow (the auto-scale heuristic sizes arrows to ~one grid cell). Auto-scaling is on by default; pass this flag only to change the fraction. Mutually exclusive with `--arrow-scale` |
| `--shaft-width <PX>` | `1.2` | Shaft stroke width |
| `--head-length <PX>` | *proportional* | Pin head length (pixels). Default: 28% of shaft, clamped to 4–14 px |
| `--head-width <PX>` | *proportional* | Pin head half-width (pixels) |
| `--tight-bounds` | off | Derive axis bounds from arrow tails only (arrows may overflow) |
| `--colormap <NAME>` | — | Color arrows by magnitude. Triggers automatic colorbar rendering |
| `--colorbar-label <TXT>` | — | Label shown next to the colorbar |
| `--pivot <MODE>` | `tail` | Where `(x, y)` sits on each arrow: `tail`, `middle`, `tip` |
| `--legend <TXT>` | — | Legend entry label |

```bash
# Zero config — auto-scaling picks a sensible arrow length.
kuva quiver field.tsv \
    --title "Velocity Field" --x-label "x" --y-label "y"

# Color by magnitude with a viridis colorbar
kuva quiver field.tsv --colormap viridis --colorbar-label "Speed"

# Tighter field (longer arrows fill the plot) + tails-only axis bounds.
kuva quiver field.tsv --auto-scale 0.95 --tight-bounds

# Pick columns by name
kuva quiver field.tsv \
    --x-col lon --y-col lat --u-col wind_u --v-col wind_v
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes.*
