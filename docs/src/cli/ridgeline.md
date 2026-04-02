# kuva ridgeline

Ridgeline plot (joyplot) — stacked KDE density curves, one per group. Groups are taken from one column; values from another.

**Input:** a tabular file with at least one numeric column and an optional group column.

| Flag | Default | Description |
|---|---|---|
| `--value <COL>` | `0` | Column of numeric values |
| `--group-by <COL>` | — | Group by this column; one ridge per unique value |
| `--filled` | on | Fill the area under each ridge curve |
| `--opacity <F>` | `0.7` | Fill opacity |
| `--overlap <F>` | `0.5` | Ridge overlap factor (0 = no overlap, 1 = full cell height) |
| `--bandwidth <F>` | *(Silverman)* | KDE bandwidth override |

```bash
kuva ridgeline samples.tsv --group-by group --value expression \
    --x-label "Expression" --y-label "Group" --title "Expression by group"

kuva ridgeline samples.tsv --group-by group --value expression --overlap 1.0
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
