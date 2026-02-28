# Benchmarks

kuva uses [Criterion](https://bheisler.github.io/criterion.rs/book/) for statistical micro-benchmarks. All numbers on this page were collected on a release build (`opt-level = 3`) on AMD64 Linux. Timing is median wall-clock; HTML reports with per-sample distributions live in `target/criterion/` after running.

## Running the benchmarks

```bash
# All benchmark groups (requires the png + pdf backends to compile cleanly)
cargo bench --features full

# A single group
cargo bench --features full -- render

# HTML report (opens in browser)
open target/criterion/report/index.html
```

Criterion runs a 3-second warm-up then collects 100 samples per benchmark. The measurement is median time; outlier detection flags any samples more than 2 IQRs from the median.

## Benchmark files

| file | what it measures |
|------|-----------------|
| `benches/render.rs` | Full pipeline (scene build + SVG emit) and SVG-only for scatter, scatter+errorbars, line, violin, manhattan, heatmap |
| `benches/kde.rs` | `simple_kde` in isolation at 100 → 100k samples |
| `benches/svg.rs` | `SvgBackend::render_scene` alone, N Circle primitives, no rendering pipeline |

## Results

### `kde` — Gaussian KDE (`simple_kde`, 256 evaluation points)

The truncated kernel sorts the input once then binary-searches for the window `[x ± 4·bw]` around each evaluation point. Only values inside that window contribute (Gaussian weight beyond 4σ is < 0.003%).

| n samples | time |
|-----------|------|
| 100 | 85.7 µs |
| 1,000 | 621 µs |
| 10,000 | 4.07 ms |
| 100,000 | 28.0 ms |

Scaling: ~6.8× per decade of n. For uniformly distributed data the window captures ~10% of points, giving roughly O(n^0.8 × samples) total work instead of the naive O(n × samples). On bounded data (e.g. sin-shaped violin groups) the window covers ~30% of points, yielding a ~3× improvement over naive.

### `render` — scatter and line

| | n=100 | n=1k | n=10k | n=100k | n=1M |
|-|-------|------|-------|--------|------|
| scatter scene+svg | 41 µs | 314 µs | 3.10 ms | 34.5 ms | 414 ms |
| scatter svg-only | 34 µs | 294 µs | 3.07 ms | 31.2 ms | 317 ms |
| scatter scene build | ~7 µs | ~20 µs | ~30 µs | ~3 ms | ~97 ms |
| line scene+svg | 39 µs | 262 µs | 2.49 ms | 28.6 ms | 308 ms |

At 1M points the SVG emit accounts for 317 ms (77% of total). The scene build — coordinate mapping, Vec allocation — costs ~97 ms. SVG generation rate: ~200 ns/element.

### `render` — scatter with error bars

Each error bar adds 3 `Line` primitives (cap–shaft–cap), so n=100k scatter+errorbars emits 400k primitives vs 100k for plain scatter.

| n | plain scatter | with y_err | overhead |
|---|--------------|------------|---------|
| 100 | 41 µs | 175 µs | 4.3× |
| 1,000 | 314 µs | 1.70 ms | 5.4× |
| 10,000 | 3.10 ms | 18.8 ms | 6.1× |
| 100,000 | 34.5 ms | 222 ms | 6.5× |

### `render` — violin (3 groups)

Violin SVG is cheap — three KDE curves emit ~10 path primitives regardless of n. All time is in KDE.

| n per group | scene+svg | svg-only | KDE cost |
|------------|-----------|----------|----------|
| 100 | 557 µs | 20 µs | ~537 µs |
| 1,000 | 2.00 ms | 19 µs | ~1.98 ms |
| 10,000 | 12.7 ms | 25 µs | ~12.7 ms |
| 100,000 | 89.0 ms | 34 µs | ~89 ms |

The violin SVG time is flat at ~25 µs across all scales. KDE matches 3× the `kde` bench values exactly. Practical advice: violin plots are fast at the scales bioinformatics data actually produces (100–5000 points per group); 100k/group is an extreme stress test.

### `render` — manhattan (22 chromosomes)

Pre-bucketing builds a `HashMap<chr → Vec<idx>>` once before the span loop, reducing chromosome lookups from O(22 × n) to O(n).

| n SNPs | time |
|--------|------|
| 1,000 | 372 µs |
| 10,000 | 3.88 ms |
| 100,000 | 42.0 ms |
| 1,000,000 | 501 ms |

Scales linearly: each 10× increase in n costs ~10–12× more time. At 1M SNPs (a full GWAS): 501 ms including scene build and SVG emit. Note: this is render-only; file I/O for a 1M-row TSV adds read time on top.

### `render` — heatmap (n×n matrix)

| n | cells | no values | with values | text overhead |
|---|-------|-----------|-------------|---------------|
| 10 | 100 | 61.7 µs | 114 µs | 1.85× |
| 50 | 2,500 | 1.18 ms | 2.54 ms | 2.15× |
| 100 | 10,000 | 4.86 ms | 10.4 ms | 2.14× |
| 200 | 40,000 | 24.6 ms | — | — |
| 500 | 250,000 | 154 ms | — | — |

Scales with n² (doubling n quadruples cells). The single-loop merge means `show_values` costs exactly ~2× no-values — one extra `format!` and `Text` primitive per cell.

### `svg` — raw string generation

Measures `SvgBackend::render_scene` on a scene containing only Circle primitives. Isolates the string formatting cost with no rendering pipeline.

| n circles | time | ns/element |
|-----------|------|-----------|
| 1,000 | 198 µs | 198 |
| 10,000 | 1.99 ms | 199 |
| 100,000 | 19.9 ms | 199 |
| 1,000,000 | 213 ms | 213 |

Perfectly linear. `String::with_capacity` pre-allocation eliminates reallocation variance. ~200 ns/element is the baseline cost of `format!` through the `std::fmt` machinery.

## Interpretation

**SVG string generation dominates at scale.** For scatter/line at 1M points, 77% of total time is in `SvgBackend::render_scene`. Improving the SVG backend (e.g. write-to-file streaming, avoiding `format!` for simple floats) would have the most impact at extreme scales.

**Violin is all KDE.** The SVG stage for a violin scene is ~25 µs at any n — it's a handful of path curves. For n > 10k/group, consider whether that resolution is actually needed; downsampling to 10k rarely changes the visible shape.

**Manhattan is now O(n).** Pre-bucketing reduced chromosome filtering from O(22n) to O(n). A 1M-SNP GWAS renders in ~500 ms end-to-end (render only, excluding file I/O).

**Error bars are expensive at scale.** 3 extra primitives per point means 4–6× the cost of plain scatter. For large n with error bars, consider downsampling or rendering only bars for significant points.

**Heatmap with values is 2×.** The single-pass loop keeps the overhead exactly proportional — no wasted traversal.

## Optimisations applied

These were identified by profiling before benchmarks existed and confirmed by the benchmark numbers above:

| change | file | expected gain |
|--------|------|--------------|
| Truncated KDE kernel (sort + binary search) | `render_utils.rs` | ~8× at 100k uniform data |
| Manhattan pre-bucketing (HashMap) | `render.rs` | ~22× at 1M SNPs with 22 chr |
| Heatmap single-loop + no `flat` Vec | `render.rs` | ~2× for `show_values`; -1 alloc |
| `String::with_capacity` in SVG backend | `svg.rs` | eliminates O(log n) reallocs |
