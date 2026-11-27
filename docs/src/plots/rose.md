# Nightingale Rose Chart

A **Nightingale rose** (coxcomb chart) is a polar bar chart where each sector's **area** or **radius** is proportional to its data value.  It was famously used by Florence Nightingale to visualise causes of soldier mortality.

## Basic usage

```rust,no_run
use kuva::plot::rose::RosePlot;
use kuva::render::{plots::Plot, layout::Layout, render::render_rose};
use kuva::backend::svg::SvgBackend;

let plot = RosePlot::new()
    .with_slice("Jan", 30.0)
    .with_slice("Feb", 20.0)
    .with_slice("Mar", 45.0)
    .with_slice("Apr", 38.0);

let svg = SvgBackend.render_scene(&render_rose(plot, Layout::default()));
std::fs::write("rose.svg", svg).unwrap();
```

Or bulk-add slices:

```rust,no_run
use kuva::plot::rose::RosePlot;

let plot = RosePlot::new().with_slices([
    ("Jan", 30.0), ("Feb", 20.0), ("Mar", 45.0), ("Apr", 38.0),
]);
```

## Auto-binning bearing data

Pass raw compass bearings (0–360°) and a bin count:

```rust,no_run
use kuva::plot::rose::RosePlot;

let bearings = vec![10.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0, 355.0];
let plot = RosePlot::new()
    .with_bearing_data(bearings, 8)  // 8 compass octants
    .with_compass_labels();          // N, NE, E, SE, ...
```

## Stacked mode

Multiple series stacked within each sector:

```rust,no_run
use kuva::plot::rose::RosePlot;

let plot = RosePlot::new()
    .with_x_labels(["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"])
    .with_stack("Preventable", vec![12.0, 11.0, 14.0, 10.0, 9.0, 7.0, 6.0, 5.0, 8.0, 10.0, 13.0, 15.0])
    .with_stack("Wounds",      vec![ 3.0,  4.0,  2.0,  3.0, 2.0, 2.0, 1.0, 1.0, 2.0,  3.0,  3.0,  4.0])
    .with_legend("Cause of death");
```

## Grouped mode

Each series occupies its own sub-wedge within each sector:

```rust,no_run
use kuva::plot::rose::{RosePlot, RoseMode};

let plot = RosePlot::new()
    .with_mode(RoseMode::Grouped)
    .with_x_labels(["Q1", "Q2", "Q3", "Q4"])
    .with_group("Product A", vec![20.0, 35.0, 25.0, 40.0])
    .with_group("Product B", vec![15.0, 22.0, 30.0, 28.0])
    .with_legend("Sales");
```

## Encoding modes

| Mode | Formula | Use case |
|------|---------|----------|
| `Area` (default) | `r = sqrt(base² + frac*(max²-base²))` | Perceptually accurate — areas proportional to values |
| `Radius` | `r = base + frac*(max_r-base)` | Radius proportional to values (overestimates large sectors) |

```rust,no_run
use kuva::plot::rose::{RosePlot, RoseEncoding};

let plot = RosePlot::new()
    .with_encoding(RoseEncoding::Radius)
    .with_slices([("A", 10.0), ("B", 30.0), ("C", 60.0)]);
```

## Compass labels

Replace numeric labels with cardinal/intercardinal directions:

```rust,no_run
use kuva::plot::rose::{RosePlot, compass_labels_for_n};

// Automatic from sector count (works for 4, 8, 16 sectors)
let plot = RosePlot::new()
    .with_bearing_data(some_bearings, 8)
    .with_compass_labels();

// Or set manually
let labels = compass_labels_for_n(4);  // ["N", "E", "S", "W"]
```

## Inner radius / donut

```rust,no_run
use kuva::plot::rose::RosePlot;

let plot = RosePlot::new()
    .with_inner_radius(0.3)   // 30% of max_r is hollow
    .with_slices([("A", 40.0), ("B", 60.0), ("C", 30.0)]);
```

## Builder reference

| Method | Default | Description |
|--------|---------|-------------|
| `with_slice(label, value)` | — | Add one sector to the default series |
| `with_slices(iter)` | — | Add multiple `(label, value)` sectors |
| `with_x_labels(iter)` | — | Set all sector labels at once |
| `with_stack(name, values)` | — | Add a stacked series; sets mode=Stacked |
| `with_group(name, values)` | — | Add a grouped series; sets mode=Grouped |
| `with_bearing_data(iter, n)` | — | Bin raw bearings into `n` sectors |
| `with_compass_labels()` | — | Replace labels with compass directions |
| `with_encoding(enc)` | `Area` | `RoseEncoding::Area` or `Radius` |
| `with_mode(mode)` | `Stacked` | `RoseMode::Stacked` or `Grouped` |
| `with_start_angle(deg)` | `0.0` | Degrees clockwise from north for sector 0 |
| `with_clockwise(bool)` | `true` | Direction sectors are laid out |
| `with_inner_radius(f)` | `0.0` | Donut hole fraction (0–0.95) |
| `with_gap(deg)` | `1.0` | Angular gap between sectors in degrees |
| `with_grid(bool)` | `true` | Concentric grid rings |
| `with_grid_lines(n)` | `4` | Number of grid rings |
| `with_spokes(bool)` | `true` | Radial spoke lines |
| `with_show_labels(bool)` | `true` | Sector labels around the perimeter |
| `with_show_values(bool)` | `false` | Value labels at sector tips |
| `with_legend(label)` | `None` | Enable legend |

**See also:** [Polar Plot](./polar.md) for continuous polar scatter/line data, [Bar Chart](./bar.md) for the Cartesian equivalent.

---

## CLI

Render a **Nightingale rose** (coxcomb) chart from a tabular file.

### Input format

Tab- or comma-separated file with at least two columns: a label column and a value column.

```tsv
direction	count
N	25
NE	18
E	12
SE	8
S	10
SW	14
W	20
NW	22
```

For multi-series mode, add a group column and use `--group-by`. Distinct groups times distinct labels may be at most 1,000,000; larger input is an error.

```tsv
direction	speed_class	count
N	low	15
N	high	8
NE	low	22
NE	high	12
```

### Basic examples

```bash
# Single-series rose chart
kuva rose data.tsv --label direction --value count -o rose.svg

# Wind rose from provided example data (stacked low/high speed)
kuva rose examples/data/rose.tsv --label direction \
    --group-by direction --mode stacked -o wind_rose.svg

# With compass direction labels
kuva rose bearings.tsv --value bearing --compass -o compass_rose.svg

# Donut (inner hole)
kuva rose data.tsv --inner-radius 0.3 -o donut_rose.svg
```

### All flags

#### Data selection

| Flag | Description |
|------|-------------|
| `--label <COL>` | Label column (name or 0-based index; default: 0) |
| `--value <COL>` | Value column (name or 0-based index; default: 1) |
| `--group-by <COL>` | Group/series column for multi-series mode |

#### Chart style

| Flag | Default | Description |
|------|---------|-------------|
| `--mode <MODE>` | `stacked` | Multi-series layout: `stacked` or `grouped` |
| `--encoding <ENC>` | `area` | Radius encoding: `area` (accurate) or `radius` |
| `--inner-radius <F>` | `0` | Fraction 0–1; creates a donut hole |
| `--gap <DEG>` | `1` | Angular gap between sectors (degrees) |
| `--start-angle <DEG>` | `0` | Start angle clockwise from north |
| `--no-clockwise` | — | Lay out sectors counterclockwise |
| `--no-grid` | — | Hide concentric grid rings |
| `--grid-lines <N>` | `4` | Number of concentric grid rings |
| `--no-labels` | — | Hide sector labels around the perimeter |
| `--show-values` | — | Show value labels at the tip of each sector |
| `--compass` | — | Replace labels with compass directions (N, NE, E, …) |
| `--legend <LABEL>` | — | Show legend (for multi-series plots) |

### Multi-series example

```bash
kuva rose examples/data/rose.tsv \
    --label direction \
    --value low_speed \
    --group-by direction \
    --legend "Wind speed" \
    --mode stacked \
    --title "Wind Rose" \
    -o wind_rose.svg
```

---

*See also: [Shared flags](../cli/index.md#shared-flags) — output, appearance, axes.*
