# Violin Plot

A violin plot estimates the probability density of each group using kernel density estimation (KDE) and renders it as a symmetric shape — widest where data is most dense. Unlike box plots, violins reveal multi-modal and skewed distributions that a five-number summary would obscure.

**Import path:** `kuva::plot::ViolinPlot`

---

## Basic usage

Add one group per category with `.with_group(label, values)`. Groups are rendered left-to-right in the order they are added.

```rust,no_run
use kuva::plot::ViolinPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

let plot = ViolinPlot::new()
    .with_group("Normal",  normal_data)   // unimodal
    .with_group("Bimodal", bimodal_data)  // two peaks — invisible in a box plot
    .with_group("Skewed",  skewed_data)   // long tail
    .with_color("steelblue")
    .with_width(30.0);

let plots = vec![Plot::Violin(plot)];
let layout = Layout::auto_from_plots(&plots)
    .with_title("Violin Plot")
    .with_y_label("Value");

let scene = render_multiple(plots, layout);
let svg = SvgBackend.render_scene(&scene);
std::fs::write("violin.svg", svg).unwrap();
```

<img src="../assets/violin/basic.svg" alt="Basic violin plot" width="560">

The bimodal group shows two distinct bulges — structure that a box plot would represent as a single median and IQR, losing all information about the separation.

---

## Violin width

`.with_width(px)` sets the maximum half-width of each violin in pixels. The default is `30.0`. Unlike bar width, this is an absolute pixel value, not a fraction of the category slot.

```rust,no_run
# use kuva::plot::ViolinPlot;
let plot = ViolinPlot::new()
    .with_group("A", data)
    .with_width(20.0);   // narrower violins
```

---

## KDE bandwidth

Bandwidth controls how smooth the density estimate is. The default uses Silverman's rule-of-thumb, which works well for unimodal, roughly normal data. Set it manually with `.with_bandwidth(h)` when the automatic choice is too smooth (hides modes) or too rough (noisy).

```rust,no_run
use kuva::plot::ViolinPlot;
# use kuva::render::plots::Plot;

// Too narrow — jagged, noisy
let plot = ViolinPlot::new().with_group("", data.clone()).with_bandwidth(0.15);

// Automatic — Silverman's rule (default, no call needed)
let plot = ViolinPlot::new().with_group("", data.clone());

// Too wide — over-smoothed, modes blend together
let plot = ViolinPlot::new().with_group("", data.clone()).with_bandwidth(2.0);
```

<table>
<tr>
<td><img src="../assets/violin/bandwidth_narrow.svg" alt="Narrow bandwidth" width="186"></td>
<td><img src="../assets/violin/bandwidth_auto.svg" alt="Auto bandwidth" width="186"></td>
<td><img src="../assets/violin/bandwidth_wide.svg" alt="Wide bandwidth" width="186"></td>
</tr>
<tr>
<td align="center"><code>h = 0.15</code> (too narrow)</td>
<td align="center">Auto — Silverman</td>
<td align="center"><code>h = 2.0</code> (too wide)</td>
</tr>
</table>

`.with_kde_samples(n)` sets the number of points at which the density is evaluated (default `200`). Increase it for a smoother rendered curve; the default is adequate for most datasets.

---

## Point overlays

Adding individual points on top of the violin makes the sample size visible and helps readers judge the reliability of the density estimate.

### Beeswarm overlay

`.with_swarm_overlay()` spreads points horizontally to avoid overlap. Recommended for smaller datasets (roughly N < 200 per group).

```rust,no_run
use kuva::plot::ViolinPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

let plot = ViolinPlot::new()
    .with_group("Normal",  normal_data)
    .with_group("Bimodal", bimodal_data)
    .with_group("Skewed",  skewed_data)
    .with_color("steelblue")
    .with_width(30.0)
    .with_swarm_overlay()
    .with_overlay_color("rgba(0,0,0,0.35)")
    .with_overlay_size(2.5);

let plots = vec![Plot::Violin(plot)];
let layout = Layout::auto_from_plots(&plots)
    .with_title("Violin + Swarm Overlay")
    .with_y_label("Value");

let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
```

<img src="../assets/violin/swarm_overlay.svg" alt="Violin with swarm overlay" width="560">

### Jittered strip

`.with_strip(jitter)` places points with random horizontal offsets. More appropriate for large datasets where beeswarm layout becomes slow.

```rust,no_run
# use kuva::plot::ViolinPlot;
let plot = ViolinPlot::new()
    .with_group("A", data)
    .with_color("steelblue")
    .with_strip(0.15)                       // jitter spread in data units
    .with_overlay_color("rgba(0,0,0,0.4)") // semi-transparent recommended
    .with_overlay_size(3.0);
```

---

## Per-group colors

Color each group independently within a single `ViolinPlot` using `.with_group_colors()`. Colors are matched to groups by position — the first color applies to the first group added, and so on. The uniform `.with_color()` value is used as a fallback for any group without an entry.

```rust,no_run
use kuva::plot::ViolinPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

let plot = ViolinPlot::new()
    .with_group("Normal",  normal_samples(0.0, 1.0, 300, 1))
    .with_group("Bimodal", bimodal_samples(-2.0, 2.0, 0.6, 300, 2))
    .with_group("Skewed",  skewed_samples(300, 3))
    .with_group_colors(["steelblue", "tomato", "seagreen"])
    .with_width(30.0);

let plots = vec![Plot::Violin(plot)];
let layout = Layout::auto_from_plots(&plots)
    .with_title("Violin Plot — Per-group Colors")
    .with_y_label("Value");

let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
```

<img src="../assets/violin/group_colors.svg" alt="Violin plot with per-group colors" width="560">

A partial list is also valid — groups beyond the list length fall back to the uniform `.with_color()` value:

```rust,no_run
# use kuva::plot::ViolinPlot;
let plot = ViolinPlot::new()
    .with_group("Normal",  vec![/* values */])
    .with_group("Bimodal", vec![/* values */])
    .with_group("Skewed",  vec![/* values */])
    .with_color("steelblue")        // fallback for groups 1 and 2
    .with_group_colors(["tomato"]); // only group 0 gets this color
```

> **Legend note:** the legend entry uses the uniform `.with_color()` value. For a fully labeled per-group legend, create one `ViolinPlot` per group and attach `.with_legend()` to each, then use a `Layout::with_palette()` to auto-assign colors.

---

## Horizontal mode

`.with_horizontal(true)` rotates the chart so categories appear on the Y-axis and values on the X-axis.

```rust,no_run
use kuva::plot::ViolinPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::backend::svg::SvgBackend;

let plot = ViolinPlot::new()
    .with_group("Normal",  normal_data)
    .with_group("Bimodal", bimodal_data)
    .with_group("Skewed",  skewed_data)
    .with_group_colors(["steelblue", "tomato", "seagreen"])
    .with_width(0.8)
    .with_horizontal(true);

let plots = vec![Plot::Violin(plot)];
let layout = Layout::auto_from_plots(&plots)
    .with_title("Horizontal Violin Plot")
    .with_x_label("Value");

let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
```

<img src="../assets/violin/horizontal.svg" alt="Horizontal violin plot" width="560">

---

## API reference

| Method | Description |
|--------|-------------|
| `ViolinPlot::new()` | Create a violin plot with defaults |
| `.with_group(label, values)` | Add a group; accepts any `Into<f64>` iterable |
| `.with_color(s)` | Violin fill color (CSS color string) |
| `.with_group_colors(iter)` | Per-group fill colors; falls back to `.with_color` for out-of-range indices |
| `.with_width(px)` | Maximum half-width of each violin in pixels (default `30.0`) |
| `.with_legend(s)` | Attach a legend label |
| `.with_bandwidth(h)` | KDE bandwidth; omit for Silverman's rule (recommended default) |
| `.with_kde_samples(n)` | KDE evaluation points (default `200`) |
| `.with_strip(jitter)` | Overlay jittered strip; `jitter` is spread width in data units |
| `.with_swarm_overlay()` | Overlay beeswarm points (spread to avoid overlap) |
| `.with_overlay_color(s)` | Overlay point color (default `"rgba(0,0,0,0.45)"`) |
| `.with_overlay_size(r)` | Overlay point radius in pixels (default `3.0`) |
| `.with_horizontal(bool)` | Rotate chart: categories on Y-axis, values on X-axis (default `false`) |

**See also:** [Box Plot](./boxplot.md) for a simpler quantile summary, [Raincloud Plot](./raincloud.md) for violin + box + points combined, [Ridgeline Plot](./ridgeline.md) for stacking many groups' densities.

---

## CLI

Kernel-density violin plot. Same input format as `box`.

**Input:** two columns — group label and numeric value, one observation per row.

| Flag | Default | Description |
|---|---|---|
| `--group-col <COL>` | `0` | Group label column |
| `--value-col <COL>` | `1` | Numeric value column |
| `--y <COL>[,<COL>…]` | — | Comma-separated columns; each column becomes a separate group (column name = group label). Overrides `--group-col` + `--value-col` when 2+ columns given |
| `--color <CSS>` | `steelblue` | Violin fill color (uniform, all groups) |
| `--group-colors <CSS,...>` | — | Per-group colors, comma-separated; falls back to `--color` for unlisted groups |
| `--bandwidth <F>` | *(Silverman)* | KDE bandwidth |
| `--overlay-points` | off | Overlay individual points as a jittered strip |
| `--overlay-swarm` | off | Overlay individual points as a non-overlapping beeswarm |
| `--horizontal` | off | Render groups on the Y-axis, values on the X-axis |
| `--signif <A,B,P>` | — | Significance bracket between groups A and B with p-value P, labelled with stars by default. A and B are group labels, or `#i` for a zero-based group index; a non-numeric P is drawn verbatim. Repeatable; brackets stack above the data and the value axis grows to fit. See [Significance Brackets](../reference/significance.md) |
| `--signif-format <STYLE>` | `stars` | Bracket label style: `stars` (`*`–`****`, `ns`), `p` (`p = 0.023`), or `both` |

```bash
kuva violin samples.tsv --group-col group --value-col expression

kuva violin samples.tsv --group-col group --value-col expression \
    --overlay-swarm --bandwidth 0.3

kuva violin samples.tsv --group-col group --value-col expression \
    --group-colors "steelblue,tomato,seagreen,goldenrod,mediumpurple"

# multi-column: each numeric column is a group
kuva violin data.tsv --y col_a,col_b,col_c

# horizontal layout
kuva violin samples.tsv --group-col group --value-col expression --horizontal
```

---

*See also: [Shared flags](../cli/index.md#shared-flags) — output, appearance, axes, log scale.*
