# visus

A lightweight scientific plotting library in Rust. Zero heavy dependencies — just build your plot, render to SVG, done.

## Features

- **Builder pattern API** — chainable methods for constructing plots
- **Generic numeric inputs** — accepts `i32`, `u32`, `f32`, `f64` seamlessly via `Into<f64>`
- **SVG output** — clean, scalable vector graphics
- **Multi-plot support** — overlay multiple plots on shared axes with automatic legends
- **Subplot figures** — grid layouts with shared axes, merged cells, panel labels, and shared legends
- **Auto-layout** — automatic axis scaling, tick generation, and margin computation
- **Log-scale axes** — logarithmic scaling for data spanning orders of magnitude, with 1-2-5 tick marks
- **Annotations** — text labels with arrows, reference lines (horizontal/vertical), and shaded regions
- **Error bars & bands** — symmetric/asymmetric error bars on scatter and line plots, confidence interval bands
- **Built-in statistics** — linear regression, KDE, percentiles, Pearson correlation

## Plot Types

| Type | Description |
|------|-------------|
| Scatter | Points with optional trend lines, error bars, bands, equation/correlation display |
| Line | Connected line segments with error bars, bands, and configurable stroke |
| Bar | Single or grouped bars with categorical x-axis |
| Histogram | Binned frequency distribution with optional normalization |
| 2D Histogram | Bivariate density with colormaps and correlation |
| Box | Quartiles, median, whiskers (1.5×IQR) |
| Violin | Kernel density estimation shape |
| Pie | Slices with labels (inside/outside/auto), percentages, leader lines, donut charts, per-slice legend |
| Series | Index-based 1D data with line, point, or both styles |
| Heatmap | 2D matrix with colormaps (Viridis, Inferno, Grayscale, custom) |
| Band | Filled area between upper and lower curves (confidence intervals) |
| Brick | Character-level sequence visualization (DNA/RNA templates) |

## Quick Start

```rust
use visus::plot::ScatterPlot;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_scatter;
use visus::render::layout::Layout;

let plot = ScatterPlot::new()
    .with_data(vec![(1.0, 5.0), (4.5, 3.5), (5.0, 8.7)])
    .with_color("blue")
    .with_size(5.0);

let layout = Layout::new((0.0, 10.0), (0.0, 10.0))
    .with_title("My Plot")
    .with_x_label("X")
    .with_y_label("Y");

let scene = render_scatter(&plot, layout).with_background(Some("white"));
let svg = SvgBackend.render_scene(&scene);
std::fs::write("plot.svg", svg).unwrap();
```

## Multi-Plot Example

Overlay multiple plot types on the same axes with automatic legends:

```rust
use visus::plot::{ScatterPlot, LinePlot};
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;
use visus::backend::svg::SvgBackend;

let line = LinePlot::new()
    .with_data((0..100).map(|x| (x as f64 / 10.0, (x as f64 / 10.0).sin())).collect::<Vec<_>>())
    .with_color("blue")
    .with_legend("sine");

let points = ScatterPlot::new()
    .with_data(vec![(1.57, 1.0), (3.14, 0.0), (4.71, -1.0)])
    .with_color("red")
    .with_legend("peaks");

let plots = vec![Plot::Line(line), Plot::Scatter(points)];

let layout = Layout::auto_from_plots(&plots)
    .with_title("Sine Wave")
    .with_x_label("Radians")
    .with_y_label("Amplitude");

let scene = render_multiple(plots, layout).with_background(Some("white"));
let svg = SvgBackend.render_scene(&scene);
```

## Grouped Bar Chart Example

```rust
use visus::plot::BarPlot;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

let bar = BarPlot::new()
    .with_group("Laptop", vec![(3.2, "tomato"), (7.8, "skyblue")])
    .with_group("Server", vec![(5.8, "tomato"), (9.1, "skyblue")])
    .with_legend(vec!["blow5", "pod5"]);

let plots = vec![Plot::Bar(bar)];
let layout = Layout::auto_from_plots(&plots)
    .with_title("Software Performance")
    .with_y_label("Time (s)");

let scene = render_multiple(plots, layout);
```

## Log-Scale Example

```rust
use visus::plot::ScatterPlot;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;
use visus::backend::svg::SvgBackend;

let scatter = ScatterPlot::new()
    .with_data(vec![
        (1.0, 0.001), (10.0, 0.1), (100.0, 10.0),
        (1000.0, 1000.0), (10000.0, 10000.0),
    ])
    .with_color("teal");

let plots = vec![Plot::Scatter(scatter)];

let layout = Layout::auto_from_plots(&plots)
    .with_log_scale()          // both axes log, or use .with_log_x() / .with_log_y()
    .with_title("Log-Scale Scatter")
    .with_x_label("X (log)")
    .with_y_label("Y (log)");

let scene = render_multiple(plots, layout);
let svg = SvgBackend.render_scene(&scene);
```

## Subplot / Figure Layout Example

Arrange multiple independent subplots in a grid with shared axes, merged cells, and panel labels:

```rust
use visus::plot::{ScatterPlot, LinePlot};
use visus::render::figure::Figure;
use visus::render::layout::Layout;
use visus::render::plots::Plot;
use visus::backend::svg::SvgBackend;

let scatter = ScatterPlot::new()
    .with_data(vec![(1.0, 2.0), (3.0, 5.0), (5.0, 3.0)])
    .with_color("blue");

let line = LinePlot::new()
    .with_data(vec![(0.0, 0.0), (2.0, 4.0), (4.0, 3.0)])
    .with_color("red");

let plots = vec![
    vec![Plot::Scatter(scatter)],
    vec![Plot::Line(line)],
];

let layouts = vec![
    Layout::auto_from_plots(&plots[0]).with_title("Scatter"),
    Layout::auto_from_plots(&plots[1]).with_title("Line"),
];

let figure = Figure::new(1, 2)
    .with_title("My Figure")
    .with_plots(plots)
    .with_layouts(layouts)
    .with_shared_y_all()
    .with_labels();

let scene = figure.render();
let svg = SvgBackend.render_scene(&scene);
```

Features:
- **Grid layout** — `Figure::new(rows, cols)` creates an `rows x cols` grid
- **Merged cells** — `.with_structure(vec![vec![0,1], vec![2], vec![3]])` for spanning
- **Shared axes** — `.with_shared_y_all()`, `.with_shared_x_all()`, per-row/column/slice variants
- **Panel labels** — `.with_labels()` adds bold A, B, C labels (also numeric, lowercase, or custom)
- **Shared legends** — `.with_shared_legend()` or `.with_shared_legend_bottom()` for figure-wide legends
- **Figure title** — `.with_title()` adds a centered title above all subplots
- **Configurable sizing** — `.with_cell_size(w, h)`, `.with_spacing(px)`, `.with_padding(px)`

## TODO

### Plot types
- [ ] Stacked bar plots
- [ ] Area / filled line plots
- [ ] Step plots
- [ ] Contour plots
- [ ] Bubble plots (scatter with variable size per point)
- [ ] Waterfall charts

### Layout & axes
- [ ] Secondary Y-axis (twin axes)
- [ ] Date/time axis support
- [ ] Custom tick formatting (e.g. percentages, scientific notation)

### Styling
- [ ] Line styles (dashed, dotted, dash-dot)
- [ ] Marker shapes (square, triangle, diamond, cross)
- [ ] Configurable font sizes and font families
- [ ] Theme support (dark mode, publication-ready, etc.)
- [ ] Custom color palettes / color cycles

### Backends & output
- [ ] PNG rasterization
- [ ] PDF output
- [ ] CLI binary: `cat data.txt | visus --histogram -o hist.svg`

## License

MIT
