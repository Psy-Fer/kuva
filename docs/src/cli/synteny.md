# kuva synteny

Synteny / genomic alignment ribbon plot.

**Input:** two files:
- **Sequences file** *(positional)*: TSV with sequence name and length columns.
- **Blocks file** (`--blocks-file`): TSV with columns `seq1, start1, end1, seq2, start2, end2, strand`.

```
# sequences.tsv
name    length
Chr1A   2800000
Chr1B   2650000

# blocks.tsv
seq1   start1  end1    seq2   start2  end2    strand
Chr1A  56000   137237  Chr1B  63958   143705  +
Chr1A  150674  271188  Chr1B  165366  303075  -
```

| Flag | Default | Description |
|---|---|---|
| `--blocks-file <FILE>` | *(required)* | Blocks TSV file |
| `--bar-height <PX>` | `18.0` | Sequence bar height in pixels |
| `--opacity <F>` | `0.65` | Block ribbon opacity |
| `--proportional` | off | Scale bar widths proportionally to sequence length |
| `--legend <LABEL>` | — | Show legend |

```bash
kuva synteny synteny_seqs.tsv --blocks-file synteny_blocks.tsv

kuva synteny synteny_seqs.tsv --blocks-file synteny_blocks.tsv \
    --proportional --legend "synteny blocks"
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
