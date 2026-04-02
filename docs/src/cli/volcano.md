# kuva volcano

Volcano plot for differential expression results.

**Input:** three columns — gene name, log₂ fold change, raw p-value.

| Flag | Default | Description |
|---|---|---|
| `--name-col <COL>` | `0` | Gene/feature name column |
| `--x-col <COL>` | `1` | log₂FC column |
| `--y-col <COL>` | `2` | p-value column (raw, not −log₁₀) |
| `--pvalue-col-is-log` | off | p-value column already contains −log₁₀(p); un-transform before plotting |
| `--fc-cutoff <F>` | `1.0` | \|log₂FC\| threshold |
| `--p-cutoff <F>` | `0.05` | p-value significance threshold |
| `--top-n <N>` | `0` | Label the N most-significant points |
| `--color-up <CSS>` | `firebrick` | Up-regulated point color |
| `--color-down <CSS>` | `steelblue` | Down-regulated point color |
| `--color-ns <CSS>` | `#aaaaaa` | Not-significant point color |
| `--point-size <PX>` | `3.0` | Point radius |
| `--legend` | off | Show Up / Down / NS legend |

```bash
kuva volcano gene_stats.tsv \
    --name-col gene --x-col log2fc --y-col pvalue \
    --top-n 20 --legend

kuva volcano gene_stats.tsv \
    --name-col gene --x-col log2fc --y-col pvalue \
    --fc-cutoff 2.0 --p-cutoff 0.01 --top-n 10

# when p-value column already holds -log10(p)
kuva volcano results.tsv --name-col gene --x-col log2fc --y-col neg_log10_p \
    --pvalue-col-is-log
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
