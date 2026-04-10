# kuva qq

Q-Q (quantile-quantile) plot in two modes:

- **Normal mode** (default) — sample quantiles vs standard-normal theoretical quantiles with a robust Q1–Q3 reference line. Use for normality checks and comparing distribution shapes.
- **Genomic mode** (`--genomic`) — −log₁₀(observed p) vs −log₁₀(expected p). Use for GWAS p-value calibration. Input values must be raw p-values in (0, 1].

**Input:** a tabular file with at least one numeric column. When `--color-by` is used, an additional categorical column groups the data.

| Flag | Default | Description |
|---|---|---|
| `--value <COL>` | `0` | Column of values (raw data or p-values) |
| `--color-by <COL>` | — | Group by this column; one set of points per unique value |
| `--genomic` | off | Genomic mode: input values are p-values in (0, 1] |
| `--ci-band` | off | 95 % pointwise CI band around the reference diagonal |
| `--lambda` | off | Annotate λ (genomic inflation factor); genomic mode only |
| `--no-reference-line` | — | Hide the reference line |
| `--marker-size <F>` | `3.0` | Marker radius in pixels |
| `--fill-opacity <F>` | — | Marker fill opacity (0–1) |

```bash
# Normal Q-Q
kuva qq data.tsv --value score --title "Normal Q-Q"

# Multi-group
kuva qq data.tsv --value score --color-by group

# Genomic Q-Q
kuva qq gwas.tsv --value pvalue --genomic \
    --x-label "Expected -log10(p)" --y-label "Observed -log10(p)"

# With CI band and lambda annotation
kuva qq gwas.tsv --value pvalue --genomic --ci-band --lambda
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
