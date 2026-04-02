# kuva forest

Forest plot — point estimates with confidence intervals for meta-analysis.

**Input:** one row per study with columns for label, estimate, CI lower, CI upper, and optionally weight.

| Flag | Default | Description |
|---|---|---|
| `--label-col <COL>` | `0` | Study label column |
| `--estimate-col <COL>` | `1` | Point estimate column |
| `--ci-lower-col <COL>` | `2` | CI lower-bound column |
| `--ci-upper-col <COL>` | `3` | CI upper-bound column |
| `--weight-col <COL>` | — | Optional weight column (scales marker radius) |
| `--color <CSS>` | `steelblue` | Point and whisker color |
| `--marker-size <PX>` | `6.0` | Base marker half-width |
| `--whisker-width <PX>` | `1.5` | Whisker stroke width |
| `--null-value <F>` | `0.0` | Null-effect reference value |
| `--no-null-line` | off | Disable the dashed null reference line |
| `--cap-size <PX>` | `0` | Whisker end-cap half-height (0 = no caps) |

```bash
kuva forest data.tsv --label-col study --estimate-col estimate \
    --ci-lower-col lower --ci-upper-col upper

kuva forest data.tsv --label-col study --estimate-col estimate \
    --ci-lower-col lower --ci-upper-col upper --weight-col weight
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
