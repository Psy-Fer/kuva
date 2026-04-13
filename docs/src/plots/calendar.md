# Calendar Heatmap

A **calendar heatmap** (GitHub contribution graph style) displays daily data values in a grid of week columns × 7 day rows.  Multiple years or arbitrary date ranges can be stacked vertically.  Cell color encodes the aggregated value for that day.

## Basic usage — event counting

```rust,no_run
use kuva::plot::calendar::CalendarPlot;
use kuva::render::{plots::Plot, layout::Layout, render::render_calendar};
use kuva::backend::svg::SvgBackend;

let plot = CalendarPlot::new()
    .with_events(vec!["2024-03-15", "2024-03-15", "2024-03-16"])
    .with_year(2024)
    .with_legend_label("commits");

let svg = SvgBackend.render_scene(&render_calendar(plot, Layout::default()));
std::fs::write("calendar.svg", svg).unwrap();
```

## Numeric data (sum / mean / max)

```rust,no_run
use kuva::plot::calendar::{CalendarPlot, CalendarAgg};

let plot = CalendarPlot::new()
    .with_data(vec![
        ("2024-01-05", 120.0),
        ("2024-01-05",  80.0),  // two values on same day → aggregated
        ("2024-06-20", 350.0),
    ])
    .with_aggregation(CalendarAgg::Sum)
    .with_year(2024)
    .with_legend_label("sales (USD)");
```

## Multiple years

```rust,no_run
use kuva::plot::calendar::CalendarPlot;

let plot = CalendarPlot::new()
    .with_data(data)          // data spans 2023–2024
    .with_years([2023, 2024]) // one row per year; auto-detected if omitted
    .with_legend_label("downloads");
```

If neither `with_year` nor `with_years` is called, years are **auto-detected** from the data dates.

## Custom date ranges (financial year, rolling window, …)

### Single named period

```rust,no_run
use kuva::plot::calendar::CalendarPlot;

let plot = CalendarPlot::new()
    .with_data(data)
    .with_period("FY2023/24", "2023-07-01", "2024-06-30");
```

### Multiple named periods

```rust,no_run
use kuva::plot::calendar::CalendarPlot;

let plot = CalendarPlot::new()
    .with_data(data)
    .with_periods([
        ("FY2022/23", "2022-07-01", "2023-06-30"),
        ("FY2023/24", "2023-07-01", "2024-06-30"),
        ("FY2024/25", "2024-07-01", "2025-06-30"),
    ]);
```

A period can also span more than a year — each period becomes one calendar row.

## Aggregation modes

| Variant | Behaviour |
|---------|-----------|
| `Count` (default) | Number of data points on each day |
| `Sum` | Sum of all values for each day |
| `Mean` | Average of all values for each day |
| `Max` | Maximum value for each day |

```rust,no_run
use kuva::plot::calendar::{CalendarPlot, CalendarAgg};

let plot = CalendarPlot::new()
    .with_aggregation(CalendarAgg::Mean);
```

## Week start

```rust,no_run
use kuva::plot::calendar::{CalendarPlot, WeekStart};

// GitHub-style: Sunday at the top
let plot = CalendarPlot::new()
    .with_week_start(WeekStart::Sunday);

// ISO default: Monday at the top
let plot = CalendarPlot::new()
    .with_week_start(WeekStart::Monday);
```

## Color customization

### Changing the color map

The default colormap is a light-green → dark-green gradient with sqrt-gamma that mimics GitHub's contribution graph.  Any [`ColorMap`](../reference/colormap.md) variant can be used instead:

```rust,no_run
use kuva::plot::calendar::CalendarPlot;
use kuva::plot::ColorMap;

// Viridis
let plot = CalendarPlot::new()
    .with_color_map(ColorMap::Viridis);

// YellowOrangeRed (ColorBrewer)
let plot = CalendarPlot::new()
    .with_color_map(ColorMap::YellowOrangeRed);
```

### Custom color function

```rust,no_run
use std::sync::Arc;
use kuva::plot::calendar::CalendarPlot;
use kuva::plot::ColorMap;

let plot = CalendarPlot::new()
    .with_color_map(ColorMap::Custom(Arc::new(|t: f64| {
        // Ice-blue to red heat map
        let r = (t * 220.0) as u8;
        let b = ((1.0 - t) * 220.0) as u8;
        format!("rgb({r},30,{b})")
    })));
```

### Missing-day and zero-value colors

```rust,no_run
use kuva::plot::calendar::CalendarPlot;

let plot = CalendarPlot::new()
    .with_missing_color("#f0f0f0")   // days absent from the dataset
    .with_zero_color("#e8e8e8");     // days present with value == 0
                                     // (falls back to missing_color if unset)
```

### Explicit color scale range

```rust,no_run
use kuva::plot::calendar::CalendarPlot;

let plot = CalendarPlot::new()
    .with_value_range(0.0, 100.0);  // clamp scale regardless of data max
```

## Builder reference

| Method | Default | Description |
|--------|---------|-------------|
| `with_data(iter)` | — | Add `(date, value)` pairs; date format `"YYYY-MM-DD"` |
| `with_events(iter)` | — | Add bare date strings; each occurrence counts as 1.0 |
| `with_aggregation(agg)` | `Count` | `CalendarAgg::Count/Sum/Mean/Max` |
| `with_year(y)` | auto | Display a single full calendar year |
| `with_years(iter)` | auto | Display multiple full calendar years, one row each |
| `with_period(label, start, end)` | — | Display a single named date range |
| `with_periods(iter)` | — | Display multiple named date ranges |
| `with_date_range(start, end)` | — | Unnamed single date range (label from start year) |
| `with_week_start(ws)` | `Monday` | `WeekStart::Monday` (ISO) or `Sunday` (GitHub) |
| `with_color_map(cmap)` | GitHub green | [`ColorMap`] variant for the value → color mapping |
| `with_missing_color(color)` | `"#ebedf0"` | CSS color for days absent from the dataset |
| `with_zero_color(color)` | `None` | CSS color for days with value exactly 0; falls back to `missing_color` |
| `with_value_range(min, max)` | auto | Explicit color scale endpoints |
| `with_month_labels(bool)` | `true` | Show abbreviated month names above the grid |
| `with_day_labels(bool)` | `true` | Show Mon/Wed/Fri labels on the left |
| `with_cell_size(px)` | `13.0` | Size of each day cell in pixels |
| `with_cell_gap(px)` | `2.0` | Gap between cells in pixels |
| `with_legend(bool)` | `true` | Show the colorbar legend |
| `with_legend_label(label)` | `None` | Label beneath the colorbar |
