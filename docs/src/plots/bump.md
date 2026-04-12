# Bump Chart

A bump chart shows how the **rank** of each series changes across discrete time points or conditions.  Lines connect consecutive ranks; the best rank (1) appears at the top.

## Basic usage (pre-ranked)

```rust,no_run
use kuva::plot::bump::BumpPlot;
use kuva::render::{plots::Plot, layout::Layout, render::render_multiple};
use kuva::backend::svg::SvgBackend;

let plot = BumpPlot::new()
    .with_series("Alpha", vec![1, 3, 2, 1])
    .with_series("Beta",  vec![2, 1, 1, 3])
    .with_series("Gamma", vec![3, 2, 3, 2])
    .with_x_labels(["2021", "2022", "2023", "2024"]);

let plots = vec![Plot::Bump(plot)];
let layout = Layout::auto_from_plots(&plots).with_title("Rank over time");
let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
std::fs::write("bump.svg", svg).unwrap();
```

## Auto-ranking from raw values

Instead of supplying pre-computed ranks you can provide raw values; kuva ranks them per time point automatically.

```rust,no_run
use kuva::plot::bump::BumpPlot;

let plot = BumpPlot::new()
    .with_raw_series("A", vec![95.0, 80.0, 88.0])
    .with_raw_series("B", vec![80.0, 95.0, 72.0])
    .with_raw_series("C", vec![70.0, 85.0, 95.0])
    .with_x_labels(["Q1", "Q2", "Q3"]);
```

By default, **higher value = rank 1** (better).  Pass `.with_rank_ascending(true)` to flip this so lower value = rank 1.

## Builder reference

| Method | Default | Description |
|--------|---------|-------------|
| `.with_series(name, ranks)` | — | Add a pre-ranked series (integer or float ranks). |
| `.with_ranked_series(name, ranks)` | — | Pre-ranked series that allows `None` gaps. |
| `.with_raw_series(name, values)` | — | Raw values; ranks computed automatically. |
| `.with_raw_series_opt(name, values)` | — | Raw values with optional gaps (`None` breaks the line). |
| `.with_x_labels(labels)` | — | Labels for each time point / condition on the x-axis. |
| `.with_curve_style(style)` | `Sigmoid` | Line style between rank points: `Sigmoid` or `Straight`. |
| `.with_show_rank_labels(bool)` | `false` | Draw the rank number inside each dot. |
| `.with_show_series_labels(bool)` | `true` | Draw series name labels at the left and right edges. |
| `.with_dot_radius(f64)` | `6.0` | Dot radius in pixels. |
| `.with_stroke_width(f64)` | `2.5` | Line stroke width in pixels. |
| `.with_highlight(name)` | `None` | Highlight one series; all others are muted to 20 % opacity. |
| `.with_legend(bool)` | `true` | Show / hide the legend. |
| `.with_rank_ascending(bool)` | `false` | If `true`, lower raw value → better (lower) rank number. |
| `.with_tie_break(mode)` | `Average` | Tie-breaking for auto-ranking: `Average`, `Min`, `Max`, `Stable`. |

## Highlight mode

Highlighting one series draws it with a thicker stroke and bolder endpoint labels; all others are rendered at reduced opacity and with muted grey labels.

```rust,no_run
let plot = BumpPlot::new()
    .with_series("Alpha", vec![1, 3, 2, 1])
    .with_series("Beta",  vec![2, 1, 1, 3])
    .with_series("Gamma", vec![3, 2, 3, 2])
    .with_highlight("Alpha");
```

## Missing time points

Supply `None` entries via `.with_ranked_series` or `.with_raw_series_opt` to produce line breaks at absent time points:

```rust,no_run
let plot = BumpPlot::new()
    .with_ranked_series("Alpha", vec![Some(1.0), None, Some(2.0), Some(1.0)])
    .with_x_labels(["A", "B", "C", "D"]);
```

## Tie-breaking modes

| Mode | Behavior |
|------|----------|
| `Average` (default) | Tied series share the average of the occupied rank positions (e.g. 2.5, 2.5). |
| `Min` | All tied series receive the best (minimum) rank number. |
| `Max` | All tied series receive the worst (maximum) rank number. |
| `Stable` | Tied series retain their insertion order. |
