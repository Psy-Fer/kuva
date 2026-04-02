# kuva violin

Kernel-density violin plot. Same input format as `box`.

**Input:** two columns — group label and numeric value, one observation per row.

| Flag | Default | Description |
|---|---|---|
| `--group-col <COL>` | `0` | Group label column |
| `--value-col <COL>` | `1` | Numeric value column |
| `--color <CSS>` | `steelblue` | Violin fill color (uniform, all groups) |
| `--group-colors <CSS,...>` | — | Per-group colors, comma-separated; falls back to `--color` for unlisted groups |
| `--bandwidth <F>` | *(Silverman)* | KDE bandwidth |
| `--overlay-points` | off | Overlay individual points as a jittered strip |
| `--overlay-swarm` | off | Overlay individual points as a non-overlapping beeswarm |

```bash
kuva violin samples.tsv --group-col group --value-col expression

kuva violin samples.tsv --group-col group --value-col expression \
    --overlay-swarm --bandwidth 0.3

kuva violin samples.tsv --group-col group --value-col expression \
    --group-colors "steelblue,tomato,seagreen,goldenrod,mediumpurple"
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
