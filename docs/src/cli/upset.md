# kuva upset

UpSet plot for set-intersection analysis.

**Input:** binary (0/1) matrix — one column per set, one row per element. Column headers become set names.

```
GWAS_hit  eQTL  Splicing_QTL  Methylation_QTL  Conservation  ClinVar
1         0     0             1                1             1
0         0     1             1                1             0
```

| Flag | Default | Description |
|---|---|---|
| `--sort <MODE>` | `frequency` | Sort intersections: `frequency`, `degree`, `natural` |
| `--max-visible <N>` | — | Show only the top N intersections |

```bash
kuva upset upset.tsv

kuva upset upset.tsv --sort degree --max-visible 15
```

> **Terminal output:** not yet supported. `kuva upset --terminal` prints a message and exits cleanly; use `-o file.svg` instead.

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
