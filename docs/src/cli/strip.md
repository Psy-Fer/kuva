# kuva strip

Strip / jitter plot — individual points along a categorical axis.

**Input:** group label column + numeric value column, one observation per row.

| Flag | Default | Description |
|---|---|---|
| `--group-col <COL>` | `0` | Group label column |
| `--value-col <COL>` | `1` | Numeric value column |
| `--color <CSS>` | `steelblue` | Point color |
| `--point-size <PX>` | `4.0` | Point radius in pixels |
| `--swarm` | off | Beeswarm (non-overlapping) layout |
| `--center` | off | All points at group center (no spread) |
| `--legend` | off | Color groups by palette and show a legend |

Default layout when neither `--swarm` nor `--center` is given: random jitter (±30 % of slot width).

`--legend` assigns a distinct palette color to each group and adds a legend. Combine with `--interactive` to enable legend toggle (click a legend entry to show/hide that group).

```bash
kuva strip samples.tsv --group-col group --value-col expression

kuva strip samples.tsv --group-col group --value-col expression --swarm

# colored groups with legend
kuva strip samples.tsv --group-col group --value-col expression \
    --legend -o strip_legend.svg

# interactive: hover, search, legend toggle
kuva strip samples.tsv --group-col group --value-col expression \
    --legend --interactive -o strip_interactive.svg
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
