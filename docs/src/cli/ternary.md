# kuva ternary

Ternary (simplex) scatter plot with barycentric coordinate system.

**Input:** TSV/CSV with three columns for the A, B, C components of each point.

| Flag | Default | Description |
|---|---|---|
| `--a <COL>` | `0` | Column for the top-vertex (A) component |
| `--b <COL>` | `1` | Column for the bottom-left (B) component |
| `--c <COL>` | `2` | Column for the bottom-right (C) component |
| `--color-by <COL>` | — | Group by column for colored series |
| `--a-label <S>` | `A` | Label for the top (A) vertex |
| `--b-label <S>` | `B` | Label for the bottom-left (B) vertex |
| `--c-label <S>` | `C` | Label for the bottom-right (C) vertex |
| `--normalize` | off | Normalize each row so a+b+c=1 |
| `--grid-lines <N>` | `5` | Grid lines per axis |
| `--legend` | off | Show legend |

```bash
kuva ternary ternary.tsv --a a --b b --c c --title "Ternary Plot"

kuva ternary ternary.tsv --a a --b b --c c --color-by group \
    --a-label "Silicon" --b-label "Oxygen" --c-label "Carbon" \
    --title "Mineral Composition"
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
