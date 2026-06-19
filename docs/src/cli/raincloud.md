# kuva raincloud

Raincloud plot — combines a half-violin KDE cloud, box-and-whisker, and jittered raw points in one panel per group.

**Input:** one row per observation with group and value columns.

| Flag | Default | Description |
|---|---|---|
| `--group-col <COL>` | `0` | Group label column |
| `--value-col <COL>` | `1` | Numeric value column |
| `--color <CSS>` | — | Color for single-group plots |
| `--bandwidth <F>` | auto | KDE bandwidth (Silverman's rule by default) |
| `--no-cloud` | off | Hide the half-violin KDE |
| `--no-box` | off | Hide the box-and-whisker |
| `--no-rain` | off | Hide the jittered raw points |
| `--flip` | off | Mirror cloud to the opposite side |
| `--horizontal` | off | Render groups on the Y-axis, values on the X-axis |
| `--legend <LABEL>` | — | Add legend entries (one per group) |

```bash
kuva raincloud data.tsv --group-col group --value-col score

kuva raincloud data.tsv --group-col condition --value-col response \
    --no-rain --legend "Condition" --title "Treatment Response"

# horizontal layout
kuva raincloud data.tsv --group-col group --value-col score --horizontal
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes.*
