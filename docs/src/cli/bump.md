# kuva bump

Render a bump chart from a tabular file.

## Input format

Three columns: series name, time/condition label, rank (or raw value with `--raw-value`).

```
series  time   rank
Alpha   2021   1
Alpha   2022   3
Beta    2021   2
Beta    2022   1
Gamma   2021   3
Gamma   2022   2
```

## Usage

```
kuva bump [OPTIONS] [INPUT]
```

## Data columns

| Flag | Default | Description |
|------|---------|-------------|
| `--series <COL>` | `0` | Series name column (name or 0-based index). |
| `--time <COL>` | `1` | Time / condition label column. |
| `--rank <COL>` | `2` | Rank column (pre-ranked data). |
| `--raw-value` | off | Treat the rank column as a raw value and auto-compute ranks per time point. |
| `--rank-ascending` | off | With `--raw-value`: lower value → better (lower) rank number. |
| `--tie-break <MODE>` | `average` | Tie-breaking for auto-ranking: `average`, `min`, `max`, `stable`. |

## Appearance

| Flag | Default | Description |
|------|---------|-------------|
| `--curve <STYLE>` | `sigmoid` | Line style: `sigmoid` or `straight`. |
| `--rank-labels` | off | Draw the rank number inside each dot. |
| `--no-series-labels` | off | Hide the series name labels at the left/right edges. |
| `--dot-radius <F>` | `6.0` | Dot radius in pixels. |
| `--stroke-width <F>` | `2.5` | Line stroke width in pixels. |
| `--highlight <NAME>` | — | Highlight one series by name; all others are muted. |
| `--no-legend` | off | Hide the legend. |

## Examples

```bash
# Basic pre-ranked data
kuva bump data.tsv --series series --time year --rank rank -o bump.svg

# Auto-rank from scores (higher = better)
kuva bump scores.tsv --series team --time season --rank score --raw-value -o bump.svg

# Lower score is better (e.g. race times)
kuva bump times.tsv --series athlete --time race --rank time \
    --raw-value --rank-ascending -o bump.svg

# Highlight one series
kuva bump data.tsv --highlight "Alpha" -o bump.svg

# Sigmoid curves with rank labels inside dots
kuva bump data.tsv --curve sigmoid --rank-labels -o bump.svg
```
