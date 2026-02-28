# Color Palettes

A `Palette` is a named, ordered list of colors that auto-cycles across plots. Palettes are used to assign consistent, visually distinct colors to multiple series without specifying each one manually.

---

## Using a palette

### Auto-cycle across plots

Pass a palette to `Layout::with_palette()` and kuva assigns colors in order to each plot that does not already have an explicit color set:

```rust
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;

let layout = Layout::auto_from_plots(&plots)
    .with_palette(Palette::wong());
```

### Manual indexing

Index directly into a palette with `[]`. Indexing wraps with modulo, so `pal[n]` is always valid regardless of palette size:

```rust
let pal = Palette::tol_bright();
let color_a = &pal[0];  // "#4477AA"
let color_b = &pal[1];  // "#EE6677"
let color_c = &pal[7];  // wraps: same as pal[0]
```

### CLI

```bash
kuva scatter data.tsv --x x --y y --palette wong
kuva line data.tsv --x-col time --y-col value --color-by group --palette tol_muted
```

Available CLI values: `wong`, `okabe_ito`, `tol_bright`, `tol_muted`, `tol_light`, `ibm`, `category10`, `pastel`, `bold`.

For convenience, `--cvd-palette TYPE` selects a colorblind-safe palette by condition name: `deuteranopia`, `protanopia`, `tritanopia`.

---

## Built-in palettes

### Colorblind-safe

| Constructor | N | Colors |
|-------------|---|--------|
| `Palette::wong()` | 8 | `#E69F00` `#56B4E9` `#009E73` `#F0E442` `#0072B2` `#D55E00` `#CC79A7` `#000000` |
| `Palette::okabe_ito()` | 8 | Same as Wong — widely known by both names |
| `Palette::tol_bright()` | 7 | `#4477AA` `#EE6677` `#228833` `#CCBB44` `#66CCEE` `#AA3377` `#BBBBBB` |
| `Palette::tol_muted()` | 10 | `#CC6677` `#332288` `#DDCC77` `#117733` `#88CCEE` `#882255` `#44AA99` `#999933` `#AA4499` `#DDDDDD` |
| `Palette::tol_light()` | 9 | `#77AADD` `#EE8866` `#EEDD88` `#FFAABB` `#99DDFF` `#44BB99` `#BBCC33` `#AAAA00` `#DDDDDD` |
| `Palette::ibm()` | 5 | `#648FFF` `#785EF0` `#DC267F` `#FE6100` `#FFB000` |

**Recommendations:**
- `wong` / `okabe_ito` — best general choice; safe for deuteranopia and protanopia (~7% of males)
- `tol_bright` — safe for tritanopia; good for presentations
- `tol_muted` — 10 colors for larger datasets; safe for all common CVD types
- `ibm` — compact 5-color set from the IBM Design Language

### Colorblind condition aliases

These are convenience constructors that return an appropriate palette for a specific condition:

| Constructor | Returns | Safe for |
|-------------|---------|----------|
| `Palette::deuteranopia()` | Wong | Red-green (~6% of males) |
| `Palette::protanopia()` | Wong | Red-green (~1% of males) |
| `Palette::tritanopia()` | Tol Bright | Blue-yellow (rare) |

### General-purpose

| Constructor | N | Colors |
|-------------|---|--------|
| `Palette::category10()` | 10 | `#1f77b4` `#ff7f0e` `#2ca02c` `#d62728` `#9467bd` `#8c564b` `#e377c2` `#7f7f7f` `#bcbd22` `#17becf` |
| `Palette::pastel()` | 10 | `#aec7e8` `#ffbb78` `#98df8a` `#ff9896` `#c5b0d5` `#c49c94` `#f7b6d2` `#c7c7c7` `#dbdb8d` `#9edae5` |
| `Palette::bold()` | 10 | `#e41a1c` `#377eb8` `#4daf4a` `#984ea3` `#ff7f00` `#a65628` `#f781bf` `#999999` `#66c2a5` `#fc8d62` |

`category10` is the default when no palette is set.

---

## Custom palettes

```rust
use kuva::render::palette::Palette;

let pal = Palette::custom(
    "my_palette",
    vec!["#264653".into(), "#2a9d8f".into(), "#e9c46a".into(),
         "#f4a261".into(), "#e76f51".into()],
);

let layout = Layout::auto_from_plots(&plots).with_palette(pal);
```

---

## API reference

| Method | Description |
|--------|-------------|
| `Palette::wong()` | Bang Wong 8-color colorblind-safe palette |
| `Palette::okabe_ito()` | Alias for Wong |
| `Palette::tol_bright()` | Paul Tol qualitative bright, 7 colors |
| `Palette::tol_muted()` | Paul Tol qualitative muted, 10 colors |
| `Palette::tol_light()` | Paul Tol qualitative light, 9 colors |
| `Palette::ibm()` | IBM Design Language, 5 colors |
| `Palette::deuteranopia()` | Alias for Wong |
| `Palette::protanopia()` | Alias for Wong |
| `Palette::tritanopia()` | Alias for Tol Bright |
| `Palette::category10()` | Tableau/D3 Category10, 10 colors **(default)** |
| `Palette::pastel()` | Pastel variant of Category10, 10 colors |
| `Palette::bold()` | High-saturation vivid, 10 colors |
| `Palette::custom(name, colors)` | User-defined palette |
| `pal[i]` | Color at index `i`; wraps with modulo |
| `pal.len()` | Number of colors |
| `pal.colors()` | Slice of all color strings |
| `pal.iter()` | Cycling iterator (never returns `None`) |
