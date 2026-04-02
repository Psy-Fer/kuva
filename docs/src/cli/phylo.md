# kuva phylo

Phylogenetic tree from a Newick string or edge-list TSV.

**Input (default):** edge-list TSV with parent, child, and branch-length columns.

**Input (alternative):** pass `--newick` with a Newick string; the file argument is not used.

| Flag | Default | Description |
|---|---|---|
| `--newick <STR>` | — | Newick string (overrides file input) |
| `--parent-col <COL>` | `0` | Parent node column |
| `--child-col <COL>` | `1` | Child node column |
| `--length-col <COL>` | `2` | Branch length column |
| `--orientation <DIR>` | `left` | `left`, `right`, `top`, `bottom` |
| `--branch-style <STYLE>` | `rectangular` | `rectangular`, `slanted`, `circular` |
| `--phylogram` | off | Scale branches by length |
| `--legend <LABEL>` | — | Show legend |

```bash
# from edge-list TSV
kuva phylo phylo.tsv \
    --parent-col parent --child-col child --length-col length

# from Newick string
kuva phylo --newick "((A:0.1,B:0.2):0.3,C:0.4);" --branch-style circular

# phylogram, top orientation
kuva phylo phylo.tsv \
    --parent-col parent --child-col child --length-col length \
    --phylogram --orientation top
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
