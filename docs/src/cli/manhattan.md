# kuva manhattan

Manhattan plot for GWAS results.

**Input:** chromosome, (optional) base-pair position, and p-value columns.

Two layout modes:
- **Sequential** *(default)*: chromosomes are sorted and SNPs receive consecutive integer x-positions. Position column is not used.
- **Base-pair** (`--genome-build`): SNP x-coordinates are resolved from chromosome sizes in a reference build.

| Flag | Default | Description |
|---|---|---|
| `--chr-col <COL>` | `0` | Chromosome column |
| `--pos-col <COL>` | `1` | Base-pair position column (bp mode only) |
| `--pvalue-col <COL>` | `2` | p-value column |
| `--pvalue-col-is-log` | off | p-value column already contains −log₁₀(p); un-transform before plotting |
| `--genome-build <BUILD>` | — | Enable bp mode: `hg19`, `hg38`, or `t2t` |
| `--genome-wide <F>` | `7.301` | Genome-wide threshold (−log₁₀ scale) |
| `--suggestive <F>` | `5.0` | Suggestive threshold (−log₁₀ scale) |
| `--top-n <N>` | `0` | Label N most-significant points above genome-wide threshold |
| `--point-size <PX>` | `2.5` | Point radius |
| `--color-a <CSS>` | `steelblue` | Even-chromosome color |
| `--color-b <CSS>` | `#5aadcb` | Odd-chromosome color |
| `--legend` | off | Show threshold legend |

```bash
# sequential mode (no position column needed)
kuva manhattan gene_stats.tsv --chr-col chr --pvalue-col pvalue --top-n 5

# base-pair mode
kuva manhattan gwas.tsv \
    --chr-col chr --pos-col pos --pvalue-col pvalue \
    --genome-build hg38 --top-n 10 --legend

# when p-value column already holds -log10(p)
kuva manhattan gwas.tsv --chr-col chr --pvalue-col neg_log10_p --pvalue-col-is-log
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
