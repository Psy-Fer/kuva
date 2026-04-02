# kuva sankey

Sankey / alluvial flow diagram.

**Input:** three columns — source node, target node, flow value.

| Flag | Default | Description |
|---|---|---|
| `--source-col <COL>` | `0` | Source node column |
| `--target-col <COL>` | `1` | Target node column |
| `--value-col <COL>` | `2` | Flow value column |
| `--link-gradient` | off | Fill each link with a gradient from source node colour to target node colour |
| `--opacity <F>` | `0.5` | Link opacity |
| `--legend <LABEL>` | — | Show legend |

```bash
kuva sankey sankey.tsv \
    --source-col source --target-col target --value-col value

kuva sankey sankey.tsv \
    --source-col source --target-col target --value-col value \
    --link-gradient --legend "read flow"
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
