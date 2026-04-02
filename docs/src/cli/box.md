# kuva box

Box-and-whisker plot. Groups are taken from one column; values from another.

**Input:** two columns — group label and numeric value, one observation per row.

| Flag | Default | Description |
|---|---|---|
| `--group-col <COL>` | `0` | Group label column |
| `--value-col <COL>` | `1` | Numeric value column |
| `--color <CSS>` | `steelblue` | Box fill color (uniform, all groups) |
| `--group-colors <CSS,...>` | — | Per-group colors, comma-separated; falls back to `--color` for unlisted groups |
| `--overlay-points` | off | Overlay individual points as a jittered strip |
| `--overlay-swarm` | off | Overlay individual points as a non-overlapping beeswarm |

```bash
kuva box samples.tsv --group-col group --value-col expression

kuva box samples.tsv --group-col group --value-col expression \
    --overlay-swarm --color "rgba(70,130,180,0.6)"

kuva box samples.tsv --group-col group --value-col expression \
    --group-colors "steelblue,tomato,seagreen,goldenrod,mediumpurple"
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
