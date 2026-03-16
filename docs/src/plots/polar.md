# Polar Plot

A polar coordinate plot renders data in (r, θ) space — radial distance and angle — projected onto a circular canvas with a configurable grid.

By default **kuva** uses compass convention: θ=0 at north (top), increasing clockwise. To use math convention (θ=0 at east, increasing CCW), combine `.with_theta_start(90.0)` and `.with_clockwise(false)`.

![Polar plot — cardioid and reference circle](../assets/polar/basic.svg)

## Rust API

```rust
use kuva::plot::polar::{PolarMode, PolarPlot};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::backend::svg::SvgBackend;

// Cardioid: r = 1 + cos(θ)
let n = 72;
let theta: Vec<f64> = (0..n).map(|i| i as f64 * 360.0 / n as f64).collect();
let r: Vec<f64> = theta.iter().map(|&t| 1.0 + t.to_radians().cos()).collect();

let plot = PolarPlot::new()
    .with_series_labeled(r, theta, "Cardioid", PolarMode::Line)
    .with_r_max(2.1)
    .with_r_grid_lines(4)
    .with_theta_divisions(12)
    .with_legend(true);

let plots = vec![Plot::Polar(plot)];
let layout = Layout::auto_from_plots(&plots).with_title("Cardioid");
let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
```

### Scatter vs Line mode

```rust
// Scatter (default): each (r, θ) point is a circle
let plot = PolarPlot::new().with_series(r, theta);

// Line: points connected by a path
let plot = PolarPlot::new().with_series_line(r, theta);

// Labeled series (used for legends)
let plot = PolarPlot::new()
    .with_series_labeled(r, theta, "Wind speed", PolarMode::Scatter);
```

### Conventions

```rust
// Compass convention (default): 0° = north, clockwise
let compass = PolarPlot::new()
    .with_theta_start(0.0)
    .with_clockwise(true);

// Math convention: 0° = east, CCW
let math = PolarPlot::new()
    .with_theta_start(90.0)
    .with_clockwise(false);
```

### Marker opacity and stroke (scatter mode)

Control fill transparency and an optional outline on scatter-mode points. Settings are per-series and must be called immediately after the series they apply to.

500 observations with two dominant directions (NE at 45° and SW at 225°). With solid markers each directional cluster collapses into an opaque wedge, hiding the internal spread. At `opacity = 0.2` the denser core of each cluster is visibly darker than its fringe, and the thin `0.7 px` stroke keeps individual observations readable.

```rust,no_run
use kuva::plot::polar::PolarPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

// (populate r_vals and t_vals with 500 (r, theta_degrees) observations)
# let (r_vals, t_vals): (Vec<f64>, Vec<f64>) = (vec![], vec![]);
let plot = PolarPlot::new()
    .with_series(r_vals, t_vals)
    .with_color("steelblue")
    .with_marker_opacity(0.2)
    .with_marker_stroke_width(0.7)
    .with_r_max(1.2)
    .with_theta_divisions(24);

let plots = vec![Plot::Polar(plot)];
let layout = Layout::auto_from_plots(&plots)
    .with_title("Directional scatter — semi-transparent markers (500 pts)");

let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
```

<img src="../assets/polar/marker_density.svg" alt="Polar scatter with two directional clusters and semi-transparent markers" width="560">

These builders have no effect on `PolarMode::Line` series.

### Grid control

```rust
let plot = PolarPlot::new()
    .with_r_grid_lines(5)        // 5 concentric rings
    .with_theta_divisions(8)     // 8 spokes (every 45°)
    .with_r_labels(true)         // show r value on each ring
    .with_grid(true);            // show grid (default)
```

### Builder reference

| Method | Default | Description |
|---|---|---|
| `.with_series(r, theta)` | — | Add scatter series |
| `.with_series_line(r, theta)` | — | Add line series |
| `.with_series_labeled(r, theta, label, mode)` | — | Add labeled series |
| `.with_r_max(f64)` | auto | Set maximum radial extent |
| `.with_theta_start(deg)` | `0.0` | Where θ=0 appears (CW from north) |
| `.with_clockwise(bool)` | `true` | Direction of increasing θ |
| `.with_r_grid_lines(n)` | `4` | Number of concentric grid circles |
| `.with_theta_divisions(n)` | `12` | Number of angular spokes |
| `.with_grid(bool)` | `true` | Show/hide grid |
| `.with_r_labels(bool)` | `true` | Show/hide r-value labels |
| `.with_legend(bool)` | `false` | Show legend for labeled series |
| `.with_color(s)` | — | Set fill color of the last added series |
| `.with_marker_opacity(f)` | solid | Fill alpha for scatter markers of the last series (`0.0`–`1.0`) |
| `.with_marker_stroke_width(w)` | none | Outline stroke for scatter markers of the last series |

## CLI

```bash
# Basic scatter
kuva polar data.tsv --r r --theta theta --title "Polar Plot"

# Line mode, multiple series via color-by
kuva polar data.tsv --r r --theta theta --color-by group --mode line

# Custom r-max and angular divisions
kuva polar data.tsv --r r --theta theta --r-max 5.0 --theta-divisions 8
```

### CLI flags

| Flag | Default | Description |
|---|---|---|
| `--r <COL>` | `0` | Radial value column |
| `--theta <COL>` | `1` | Angle column (degrees) |
| `--color-by <COL>` | — | One series per unique group value |
| `--mode <MODE>` | `scatter` | `scatter` or `line` |
| `--r-max <F>` | auto | Maximum radial extent |
| `--theta-divisions <N>` | `12` | Angular grid spokes |
| `--theta-start <DEG>` | `0.0` | Where θ=0 appears (CW from north) |
| `--legend` | off | Show legend |
