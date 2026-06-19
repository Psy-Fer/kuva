# kuva density

Kernel density estimate of a single numeric column. Produces a smooth probability density curve; optionally fills the area underneath. Multi-group plots use one curve per group with palette colors.

**Input:** a tabular file with at least one numeric column. When `--color-by` is used, an additional categorical column drives the grouping.

| Flag | Default | Description |
|---|---|---|
| `--value <COL>` | `0` | Column of numeric values to estimate |
| `--y <COL>[,<COL>…]` | — | Comma-separated columns; plots one curve per column (column name = legend label). Mutually exclusive with `--color-by` when 2+ columns given |
| `--color-by <COL>` | — | Group by this column; one curve per unique value |
| `--filled` | off | Fill the area under each density curve |
| `--bandwidth <F>` | *(Silverman)* | KDE bandwidth override |
| `--x-min <F>` | — | Lower bound for KDE evaluation; boundary reflection applied at this edge |
| `--x-max <F>` | — | Upper bound for KDE evaluation; boundary reflection applied at this edge |

Either flag can be used independently. Use `--x-min 0 --x-max 1` for data bounded to `[0, 1]` (e.g. identity scores, β-values, allele frequencies). Use `--x-min 0` alone for data that cannot be negative but has no known upper cap.

```bash
kuva density samples.tsv --value expression \
    --x-label "Expression" --y-label "Density" --title "Expression distribution"

kuva density samples.tsv --value expression --color-by group --filled \
    --title "Expression by group"

# Identity scores bounded to [0, 1] — prevents KDE from extending into impossible values
kuva density scores.tsv --value score --x-min 0 --x-max 1

# Counts that cannot be negative but have no upper cap
kuva density counts.tsv --value count --x-min 0

# multi-column: one curve per column
kuva density data.tsv --y col_a,col_b,col_c --filled
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
