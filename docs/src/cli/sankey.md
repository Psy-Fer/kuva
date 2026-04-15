# kuva sankey

Sankey / alluvial flow diagram.

`kuva sankey` supports two input modes:

- Edge-list Sankey input: source node, target node, flow value.
- Wide alluvium input: one ordered `--axis-col` per stage, plus an optional `--value-col`.

| Flag | Default | Description |
|---|---|---|
| `--axis-col <COL>` | repeatable | Ordered alluvium axis columns; switches the parser into wide alluvium mode |
| `--source-col <COL>` | `0` | Source node column |
| `--target-col <COL>` | `1` | Target node column |
| `--value-col <COL>` | `2` | Flow value column |
| `--link-gradient` | off | Fill each link with a gradient from source node colour to target node colour |
| `--opacity <F>` | `0.5` | Link opacity |
| `--legend <LABEL>` | — | Show legend |
| `--node-order <MODE>` | `input` | Node ordering within columns: `input`, `crossings`, or `neighbornet` |
| `--node-order-seed <N>` | `42` | RNG seed for crossing-reduction ordering |
| `--coloring <MODE>` | `label` | Node coloring mode: `label` or `left` |
| `--flow-labels` | off | Show absolute flow values on ribbons |
| `--flow-percent` | off | Show each ribbon as a percent of source outflow |
| `--flow-label-format <FMT>` | `auto` | Flow label number format: `auto`, `sci`, `integer`, `fixed2` |
| `--flow-label-unit <UNIT>` | — | Unit suffix appended to absolute flow labels |
| `--flow-label-min-height <F>` | `8.0` | Minimum ribbon height required to render a label |

## Edge-list mode

Use classic source-target-value input when your flow table is already stored as edges:

```bash
kuva sankey sankey.tsv \
    --source-col source --target-col target --value-col value

kuva sankey sankey.tsv \
    --source-col source --target-col target --value-col value \
    --link-gradient --legend "read flow"
```

## Alluvium mode

Use repeated `--axis-col` flags for ordered categorical stages. `kuva` will build full alluvia, aggregate adjacent links, and optionally apply wompwomp-style ordering and left-to-right color propagation:

```bash
kuva sankey alluvium.tsv \
    --axis-col tissue --axis-col cluster --axis-col sex \
    --value-col count \
    --node-order crossings \
    --node-order-seed 42 \
    --coloring left \
    --title "Ordered Alluvium"

kuva sankey alluvium.tsv \
    --axis-col tissue --axis-col cluster --axis-col sex \
    --value-col count \
    --node-order neighbornet \
    --coloring label \
    --title "NeighborNet Alluvium"
```

`--node-order crossings` uses the default weighted crossing-reduction path. `--node-order neighbornet` switches to the neighbornet backend used by wompwomp. `--coloring left` propagates colors from dominant parents left-to-right; `--coloring label` assigns one palette color per visible label.

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
