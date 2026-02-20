# visus

A lightweight scientific plotting library in Rust. Zero heavy dependencies — just build your plot, render to SVG, done.

## Features

- **Builder pattern API** — chainable methods for constructing plots
- **Generic numeric inputs** — accepts `i32`, `u32`, `f32`, `f64` seamlessly via `Into<f64>`
- **SVG output** — clean, scalable vector graphics
- **Multi-plot support** — overlay multiple plots on shared axes with automatic legends
- **Auto-layout** — automatic axis scaling, tick generation, and margin computation
- **Built-in statistics** — linear regression, KDE, percentiles, Pearson correlation

## Plot Types

| Type | Description |
|------|-------------|
| Scatter | Points with optional trend lines, error bars, equation/correlation display |
| Line | Connected line segments with configurable stroke |
| Bar | Single or grouped bars with categorical x-axis |
| Histogram | Binned frequency distribution with optional normalization |
| 2D Histogram | Bivariate density with colormaps and correlation |
| Box | Quartiles, median, whiskers (1.5×IQR) |
| Violin | Kernel density estimation shape |
| Pie | Slices with labels, supports donut charts via inner radius |
| Series | Index-based 1D data with line, point, or both styles |
| Heatmap | 2D matrix with colormaps (Viridis, Inferno, Grayscale, custom) |
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

## TODO

### Plot types
- [ ] Stretched bar plots
- [ ] Stacked bar plots
- [ ] Area / filled line plots
- [ ] Step plots
- [ ] Error band / shaded confidence interval plots
- [ ] Contour plots
- [ ] Bubble plots (scatter with variable size per point)
- [ ] Waterfall charts

### Layout & axes
- [ ] Subplot grid / multi-panel figures (subplots side by side)
- [ ] Logarithmic axis scales
- [ ] Secondary Y-axis (twin axes)
- [ ] Date/time axis support
- [ ] Custom tick formatting (e.g. percentages, scientific notation)
- [ ] Configurable legend positioning (currently hardcoded top-right)
- [ ] Colorbar / continuous color legend for heatmaps

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

### Annotations & interactivity
- [ ] Text annotations with arrows
- [ ] Horizontal / vertical reference lines
- [ ] Shaded regions

## License

MIT
