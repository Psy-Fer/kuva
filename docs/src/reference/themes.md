# Themes

Themes control the colours of all plot chrome — background, axes, grid lines, tick marks, text, and legend. Plot data colours are not affected by themes; those come from the color passed to each plot or from a `Palette`.

Four built-in themes are available. The default is `light`.

---

## Applying a theme

### Rust API

```rust
use kuva::render::layout::Layout;
use kuva::render::theme::Theme;

let layout = Layout::auto_from_plots(&plots)
    .with_theme(Theme::dark());
```

### CLI

```bash
kuva scatter data.tsv --x x --y y --theme dark
kuva bar data.tsv --label-col gene --value-col count --theme minimal
```

Available CLI values: `light`, `dark`, `minimal`, `solarized`.

---

## Built-in themes

### `light` (default)

White background, black axes and text, light gray grid lines.

| Property | Value |
|----------|-------|
| Background | `white` |
| Axes / ticks / text | `black` |
| Grid | `#ccc` |
| Legend background | `white` |
| Legend border | `black` |
| Font | system sans-serif |
| Grid shown | yes |

### `dark`

Dark charcoal background, light gray text and axes.

| Property | Value |
|----------|-------|
| Background | `#1e1e1e` |
| Axes / ticks | `#cccccc` |
| Text | `#e0e0e0` |
| Grid | `#444444` |
| Legend background | `#2d2d2d` |
| Legend border | `#666666` |
| Font | system sans-serif |
| Grid shown | yes |

### `minimal`

White background, no grid, serif font, no legend border. Suited for publication figures where grid lines add visual noise.

| Property | Value |
|----------|-------|
| Background | `white` |
| Axes / ticks / text | `black` |
| Grid | `#e0e0e0` |
| Legend border | none |
| Font | `serif` |
| Grid shown | **no** |

### `solarized`

Warm cream background based on Ethan Schoonover's Solarized palette.

| Property | Value |
|----------|-------|
| Background | `#fdf6e3` |
| Axes / ticks | `#586e75` |
| Text | `#657b83` |
| Grid | `#eee8d5` |
| Legend background | `#fdf6e3` |
| Legend border | `#93a1a1` |
| Font | system sans-serif |
| Grid shown | yes |

---

## Custom themes

Build a `Theme` struct directly to set any combination of properties:

```rust
use kuva::render::theme::Theme;

let theme = Theme {
    background: "#0d1117".into(),   // GitHub dark background
    axis_color: "#8b949e".into(),
    grid_color: "#21262d".into(),
    tick_color: "#8b949e".into(),
    text_color: "#c9d1d9".into(),
    legend_bg: "#161b22".into(),
    legend_border: "#30363d".into(),
    pie_leader: "#8b949e".into(),
    box_median: "#0d1117".into(),
    violin_border: "#8b949e".into(),
    colorbar_border: "#8b949e".into(),
    font_family: None,
    show_grid: true,
};

let layout = Layout::auto_from_plots(&plots).with_theme(theme);
```
