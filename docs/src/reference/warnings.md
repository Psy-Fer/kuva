# Limits and Warnings

Rendering in kuva never fails: every call to `render_multiple` returns a `Scene`. When rendering has to change or leave out something the plots asked for, it records a warning on the scene instead of stopping. Examples are an axis break that can't apply to the axis it was set on, or a size limit that makes a plot use a cheaper method.

## Reading warnings

**Library.** `Scene::warnings` is a list of `RenderWarning`, each with a short stable `code` and a `message` for people:

```rust,no_run
use kuva::prelude::*;
use kuva::render::render::render_multiple;

let bar = BarPlot::new().with_bar("a", 1.0).with_bar("b", 2.0);
let plots = vec![Plot::Bar(bar)];
let layout = Layout::auto_from_plots(&plots).with_y_break(5.0, 50.0);
let scene = render_multiple(plots, layout);
for w in &scene.warnings {
    eprintln!("{}: {}", w.code, w.message);
}
let svg = SvgBackend::new().render_scene(&scene);
```

Match on `code` if your program needs to react to a particular warning; the message wording may change between releases, the codes will not. The same warning from several panels of a `Figure` is listed once.

The one-call helpers (`render_to_svg`, `render_to_png`, `render_to_raster`, `render_to_pdf`, `render_to_pdf_multi`) return only the output, so they print the warnings on stderr, prefixed with `kuva: warning:`. Use `render_multiple` when you want them as data.

**CLI.** Every subcommand prints the scene's warnings on stderr as `warning: <message>`. They never go to stdout, so an SVG piped from stdout stays clean. The exit status is still 0: the figure was written.

## Warnings

| Code | When |
|------|------|
| `axis_break.ignored` | An x or y axis break was requested but can't apply: it needs `lo < hi` strictly inside the range of a linear axis, and isn't supported on log, categorical, equal-aspect, twin-y or candlestick axes. The plot is drawn without the break. See [Axis Breaks](./axis_break.md). |
| `axis_break.points_in_gap` | Some scatter points or line vertices fall inside an axis break. They are drawn as crosses in the gap; the message gives the count. |
| `brackets.overflow` | An explicit axis max leaves too little room for the significance brackets, so the top ones are drawn outside the plot. See [Significance Brackets](./significance.md). |
| `calendar.period_skipped` | A calendar period has a start or end that is not a valid `YYYY-MM-DD` date, or ends before it starts. The period is not drawn; the others are. One warning per period. See [Calendar Heatmap](../plots/calendar.md). |

Some older checks still print straight to stderr instead of recording a warning: heatmap bin edges that don't fit the data, sequence logo input problems, `$...$` labels the math typesetter refuses, and a log axis with no positive data. They will move to `Scene::warnings` over time.
