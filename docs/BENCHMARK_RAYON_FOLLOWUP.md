# Benchmark: PR #30 without rayon vs dev (baseline)

Per maintainer request: benchmark this PR **without rayon** against the current dev branch to confirm speedup.

## Summary

PR #30 (architecture optimizations: ryu, write_coord, etc.) **without rayon** shows substantial speedup over dev across all tested sizes. Rayon was removed from both scatter and heatmap; the architecture improvements alone deliver the gains.

## Scatter (scene_and_svg)

| Size   | dev (baseline) | PR30 no-rayon | Δ      |
|--------|----------------|---------------|--------|
| 100    | ~20.3 µs       | ~17.1 µs      | **-16%** |
| 1 000  | ~157 µs        | ~106 µs       | **-32%** |
| 10 000 | ~1.44 ms       | ~1.06 ms      | **-26%** |
| 100 000| ~21.1 ms       | ~10.7 ms      | **-49%** |
| 1 000 000 | ~205 ms     | ~130 ms       | **-37%** |

## Heatmap (no_values)

| Grid    | dev (baseline) | PR30 no-rayon | Δ      |
|---------|----------------|---------------|--------|
| 10×10   | ~36.3 µs       | ~33.9 µs      | **-9%**  |
| 50×50   | ~692 µs        | ~620 µs       | **-10%** |
| 100×100 | ~2.65 ms       | ~2.36 ms      | **-11%** |
| 200×200 | ~10.79 ms      | ~9.76 ms      | **-10%** |
| 500×500 | ~65.3 ms       | ~58.8 ms      | **-10%** |

## Conclusion

- **Scatter**: Rayon hurt at every size (per maintainer's earlier analysis). Removing it improves performance; PR30 architecture gains are 16–49% over dev.
- **Heatmap**: Removing rayon still improves heatmap by ~10% across sizes. The architecture optimizations outweigh any benefit rayon provided at large grids.
- **Recommendation**: Drop rayon entirely. Zero mandatory dependency, simpler code, better performance for typical workloads.
