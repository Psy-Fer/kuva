# kuva chord

Chord diagram for pairwise flow data.

**Input:** square N×N matrix — first column is the row label (ignored for layout), header row supplies node names.

```
region        Cortex  Hippocampus  Amygdala …
Cortex        0       320          13       …
Hippocampus   320     0            210      …
```

| Flag | Default | Description |
|---|---|---|
| `--gap <DEG>` | `2.0` | Gap between arcs in degrees |
| `--opacity <F>` | `0.7` | Ribbon opacity |
| `--legend <LABEL>` | — | Show legend |

```bash
kuva chord chord.tsv

kuva chord chord.tsv --gap 3.0 --opacity 0.5 --legend "connectivity"
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
