# kuva treemap

Treemap — tile a rectangle proportionally to values, with optional hierarchical grouping.

**Input:** at minimum a label column and a value column; optionally a parent column for two-level hierarchy.

| Flag | Default | Description |
|---|---|---|
| `--label <COL>` | `0` | Label column (name or index) |
| `--value <COL>` | `1` | Value column (name or index) |
| `--parent <COL>` | — | Parent column → 2-level hierarchy |
| `--color-by <MODE>` | `parent` | Color mode: `parent`, `value`, `explicit` |
| `--color-col <COL>` | — | Color values (`value` mode) or CSS color strings (`explicit` mode) |
| `--colormap <NAME>` | `viridis` | Colormap: `viridis`, `inferno`, `turbo`, `grayscale` |
| `--layout <NAME>` | `squarify` | Layout: `squarify`, `slicedice`, `binary` |
| `--padding <F>` | `4.0` | Padding px between parent border and children |
| `--colorbar` | off | Show colorbar in value mode |
| `--colorbar-label <S>` | — | Colorbar label |
| `--no-tooltips` | off | Suppress SVG hover tooltips |
| `--max-depth <N>` | — | Maximum depth to render |

```bash
# Flat treemap from two columns
kuva treemap data.tsv --label name --value size

# Two-level: group rows by parent column
kuva treemap data.tsv --label gene --value count --parent pathway

# Color leaves by a third column (e.g. p-value)
kuva treemap data.tsv --label term --value count --color-by value --color-col pvalue --colorbar --colorbar-label "p-value"

# Slice-and-dice layout
kuva treemap data.tsv --label name --value size --layout slicedice

# Suppress tooltips for a clean static SVG
kuva treemap data.tsv --label name --value size --no-tooltips

# Limit to two depth levels
kuva treemap data.tsv --label name --value size --parent group --max-depth 2

# Custom colormap and explicit title
kuva treemap data.tsv --label name --value size --colormap inferno -t "Category breakdown"
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
