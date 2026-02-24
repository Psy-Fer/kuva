# 2D Histogram

A 2D histogram (density map) bins scatter points `(x, y)` into a rectangular grid and colors each cell by its count. The colorbar labeled **"Count"** is added to the right margin automatically. Use it to visualize the joint distribution of two continuous variables.

**Import path:** `visus::plot::Histogram2D`, `visus::plot::histogram2d::ColorMap`

---

## Basic usage

Pass `(x, y)` scatter points along with explicit axis ranges and bin counts to `.with_data()`. Points outside the specified ranges are silently discarded.

```rust,no_run
use visus::plot::Histogram2D;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

// (x, y) scatter points — e.g. from a 2D measurement
let data: Vec<(f64, f64)> = vec![];  // ...your data here

let hist = Histogram2D::new()
    .with_data(data, (0.0, 30.0), (0.0, 30.0), 30, 30);

let plots = vec![Plot::Histogram2d(hist)];
let layout = Layout::auto_from_plots(&plots)
    .with_title("2D Histogram — Viridis")
    .with_x_label("X")
    .with_y_label("Y");

let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
std::fs::write("hist2d.svg", svg).unwrap();
```

<img src="../assets/histogram2d/basic.svg" alt="2D histogram — single Gaussian cluster, Viridis colormap" width="560">

A single bivariate Gaussian cluster binned into a 30×30 grid. The Viridis colorbar on the right shows the count scale from zero (dark blue) to the maximum (yellow).

---

## Correlation annotation

`.with_correlation()` computes the Pearson r coefficient from the raw scatter points and prints it in the top-right corner.

```rust,no_run
# use visus::plot::Histogram2D;
# use visus::render::plots::Plot;
let hist = Histogram2D::new()
    .with_data(data, (0.0, 20.0), (0.0, 20.0), 25, 25)
    .with_correlation();
```

<img src="../assets/histogram2d/correlation.svg" alt="2D histogram with Pearson r = 0.85 annotation" width="560">

The diagonal density ridge reflects a strong positive correlation (r ≈ 0.85). The coefficient is computed from all input points, including those clipped outside the plot range.

---

## Bimodal data — Inferno colormap

`ColorMap::Inferno` maps low counts to near-black and high counts to bright yellow. It is effective for high-contrast visualization of structured or multi-modal data.

```rust,no_run
use visus::plot::Histogram2D;
use visus::plot::histogram2d::ColorMap;
# use visus::render::plots::Plot;

let hist = Histogram2D::new()
    .with_data(data, (0.0, 30.0), (0.0, 30.0), 30, 30)
    .with_color_map(ColorMap::Inferno);
```

<img src="../assets/histogram2d/bimodal.svg" alt="2D histogram — bimodal distribution, Inferno colormap" width="560">

Two Gaussian clusters in opposite corners of the grid, visible as bright islands against the dark background. Empty bins are not drawn, preserving the black background of Inferno.

---

## Bin resolution — Grayscale

Bin count controls the trade-off between noise and detail. `ColorMap::Grayscale` maps zero to white and the maximum to black, useful for printing or publication figures.

<img src="../assets/histogram2d/bins_coarse.svg" alt="2D histogram — 10×10 coarse bins, Grayscale" width="420">
<img src="../assets/histogram2d/bins_fine.svg" alt="2D histogram — 50×50 fine bins, Grayscale" width="420">

Left: 10×10 bins smooth the distribution but lose detail. Right: 50×50 bins reveal the Gaussian shape but individual cells become noisy at lower sample counts.

---

## Range convention

The x and y extents passed to `with_data` should **start at `0.0`**. Internally the layout maps bin indices `0..bins_x` and `0..bins_y` onto the plot canvas; the render function converts each bin's physical coordinate using the same mapping. Starting the range at zero keeps both coordinate systems aligned.

| Range | `bins_x` | Bin width | Works? |
|-------|----------|-----------|--------|
| `(0.0, 30.0)` | `30` | 1.0 | Yes |
| `(0.0, 20.0)` | `25` | 0.8 | Yes |
| `(5.0, 25.0)` | `20` | 1.0 | Off-axis — avoid |

---

## Colormaps

| `ColorMap` variant | Description |
|--------------------|-------------|
| `ColorMap::Viridis` | Blue → green → yellow. Perceptually uniform, colorblind-safe. **(default)** |
| `ColorMap::Inferno` | Black → orange → yellow. High contrast. |
| `ColorMap::Grayscale` | White → black. Print-friendly. |
| `ColorMap::Custom(f)` | User-supplied `Arc<dyn Fn(f64) -> String>`. |

---

## API reference

| Method | Description |
|--------|-------------|
| `Histogram2D::new()` | Create with defaults (10×10 bins, Viridis) |
| `.with_data(data, x_range, y_range, bins_x, bins_y)` | Load `(x, y)` points and bin them |
| `.with_color_map(cmap)` | Set the colormap (default `ColorMap::Viridis`) |
| `.with_correlation()` | Print Pearson r in the top-right corner |
