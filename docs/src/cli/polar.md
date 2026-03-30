# kuva polar

Polar coordinate scatter/line plot. Compass convention by default (θ=0 at north, increasing clockwise).

**Input:** TSV/CSV with columns for radial value `r` and angle `theta` (degrees).

| Flag | Default | Description |
|---|---|---|
| `--r <COL>` | `0` | Column containing radial values |
| `--theta <COL>` | `1` | Column containing angle values (degrees) |
| `--color-by <COL>` | — | Group by column — one series per unique value |
| `--mode <MODE>` | `scatter` | Plot mode: `scatter` or `line` |
| `--r-max <F>` | auto | Maximum radial extent |
| `--r-min <F>` | `0` | Value mapped to the plot centre; use a negative value for dB-scale data |
| `--theta-divisions <N>` | `12` | Angular spoke divisions (12 = every 30°) |
| `--theta-start <DEG>` | `0.0` | Where θ=0 appears, degrees CW from north |
| `--legend` | off | Show legend |

```bash
kuva polar polar.tsv --r r --theta theta --title "Polar Plot"

kuva polar polar.tsv --r r --theta theta --color-by group --mode line \
    --title "Wind Rose"

# dB-scale antenna pattern: r values range from -20 to 0 dBi
kuva polar pattern.tsv --r gain_dbi --theta angle --mode line \
    --r-min -20 --r-max 0 --title "Antenna Pattern (dBi)"
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
