# Histogram

A histogram bins a 1-D dataset into equal-width intervals and renders each bin as a vertical bar. It supports explicit ranges, normalization, and overlapping distributions.

**Import path:** `visus::plot::Histogram`

---

## Basic usage

```rust,no_run
use visus::plot::Histogram;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

let data: Vec<f64> = vec![/* your samples */];

let hist = Histogram::new()
    .with_data(data)
    .with_bins(20)
    .with_color("steelblue");

let plots = vec![Plot::Histogram(hist)];
let layout = Layout::auto_from_plots(&plots)
    .with_title("Histogram")
    .with_x_label("Value")
    .with_y_label("Count");

let scene = render_multiple(plots, layout);
let svg = SvgBackend.render_scene(&scene);
std::fs::write("histogram.svg", svg).unwrap();
```

<img src="../assets/histogram/basic.svg" alt="Basic histogram" width="560">

---

## Bin count

`.with_bins(n)` sets the number of equal-width bins. The default is `10`. Fewer bins smooth out noise; more bins reveal finer structure at the cost of per-bin counts.

```rust,no_run
# use visus::plot::Histogram;
// Coarse — few bins, clear shape
let hist = Histogram::new().with_data(data.clone()).with_bins(5);

// Fine — many bins, more detail
let hist = Histogram::new().with_data(data).with_bins(40);
```

<table>
<tr>
<td><img src="../assets/histogram/bins_coarse.svg" alt="5 bins" width="280"></td>
<td><img src="../assets/histogram/bins_fine.svg" alt="40 bins" width="280"></td>
</tr>
</table>

---

## Explicit range

By default the bin edges span the minimum and maximum of the data. Use `.with_range()` to fix the extent — for example, when comparing multiple histograms that must share the same x-axis scale, or to exclude outliers from the binning:

```rust,no_run
use visus::plot::Histogram;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

let hist = Histogram::new()
    .with_data(data)
    .with_bins(20)
    .with_color("steelblue")
    .with_range((0.0, 10.0));  // values outside this range are ignored

let plots = vec![Plot::Histogram(hist)];
let layout = Layout::new((0.0, 10.0), (0.0, 50.0))  // match the fixed range
    .with_x_label("Value")
    .with_y_label("Count");
```

---

## Normalized histogram

`.with_normalize()` rescales bar heights so the tallest bar equals `1.0`. This is peak-normalization — useful for comparing the shape of distributions with different sample sizes. The y-axis shows relative frequency, not counts or probability density.

```rust,no_run
use visus::plot::Histogram;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

let hist = Histogram::new()
    .with_data(data)
    .with_bins(20)
    .with_color("steelblue")
    .with_normalize();

let plots = vec![Plot::Histogram(hist)];
let layout = Layout::auto_from_plots(&plots)
    .with_title("Normalized Histogram")
    .with_x_label("Value")
    .with_y_label("Relative frequency");
```

<img src="../assets/histogram/normalized.svg" alt="Normalized histogram" width="560">

---

## Overlapping distributions

Place multiple `Histogram` structs in the same `Vec<Plot>`. Since bars have no built-in opacity setting, use 8-digit hex colors (`#RRGGBBAA`) to make each series semi-transparent so the overlap is visible:

```rust,no_run
use visus::plot::Histogram;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

// #4682b480 = steelblue at 50% opacity
// #dc143c80 = crimson at 50% opacity
let hist_a = Histogram::new()
    .with_data(group_a)
    .with_bins(20)
    .with_color("#4682b480")
    .with_legend("Group A");

let hist_b = Histogram::new()
    .with_data(group_b)
    .with_bins(20)
    .with_color("#dc143c80")
    .with_legend("Group B");

let plots = vec![Plot::Histogram(hist_a), Plot::Histogram(hist_b)];
let layout = Layout::auto_from_plots(&plots)
    .with_title("Overlapping Distributions")
    .with_x_label("Value")
    .with_y_label("Count");

let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
```

<img src="../assets/histogram/overlapping.svg" alt="Overlapping histograms" width="560">

The `AA` byte in the hex color controls opacity: `ff` = fully opaque, `80` ≈ 50%, `40` ≈ 25%. When all histograms are normalized, comparing shapes across different sample sizes becomes straightforward.

---

## API reference

| Method | Description |
|--------|-------------|
| `Histogram::new()` | Create a histogram with defaults (10 bins, color `"black"`) |
| `.with_data(iter)` | Set input values; accepts any `Into<f64>` numeric type |
| `.with_bins(n)` | Number of equal-width bins (default `10`) |
| `.with_range((min, max))` | Fix bin edges; values outside are ignored |
| `.with_color(s)` | Bar fill color; use 8-digit hex for alpha transparency |
| `.with_normalize()` | Scale heights so peak bar = 1.0 |
| `.with_legend(s)` | Attach a legend label to this series |
