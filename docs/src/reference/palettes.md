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

Available CLI values: `wong`, `okabe_ito`, `tol_bright`, `tol_muted`, `tol_light`, `ibm`, `category10`, `pastel`, `bold`, plus the LTC palettes below: `paloma`, `maya`, `dora`, `ploen`, `olga`, `mterese`, `franscoise`, `fernande`, `sylvie`, `expevo`, `minou`, `alger`, `seafarer`, `luminaries`, `casa-natal` (an optional `ltc-` prefix is accepted, e.g. `ltc-maya`).

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

### LTC palettes

Curated qualitative palettes from [loukesio/ltc-color-palettes](https://github.com/loukesio/ltc-color-palettes) (MIT licensed), named after Picasso's muses and a few places. They are hand-picked for small-cardinality categorical series where the ColorBrewer/Tableau defaults feel clinical. Hex values are shown so you can lift them directly.

**`Palette::paloma()`** (5 colours): soft coral, sage, and butter

<div style="display:flex;height:26px;width:100%;max-width:520px;border-radius:5px;overflow:hidden;border:1px solid rgba(0,0,0,.12)"><span style="flex:1;background:#83AF9B"></span><span style="flex:1;background:#C8C8A9"></span><span style="flex:1;background:#F8DA8A"></span><span style="flex:1;background:#F7BF95"></span><span style="flex:1;background:#FE8CA1"></span></div>

`#83AF9B` `#C8C8A9` `#F8DA8A` `#F7BF95` `#FE8CA1`

**`Palette::maya()`** (5 colours): navy, sky, ice, coral, ink

<div style="display:flex;height:26px;width:100%;max-width:520px;border-radius:5px;overflow:hidden;border:1px solid rgba(0,0,0,.12)"><span style="flex:1;background:#3D5A80"></span><span style="flex:1;background:#98C1D9"></span><span style="flex:1;background:#E0FBFC"></span><span style="flex:1;background:#EE6C4D"></span><span style="flex:1;background:#293241"></span></div>

`#3D5A80` `#98C1D9` `#E0FBFC` `#EE6C4D` `#293241`

**`Palette::dora()`** (5 colours): teal, plum, and warm reds

<div style="display:flex;height:26px;width:100%;max-width:520px;border-radius:5px;overflow:hidden;border:1px solid rgba(0,0,0,.12)"><span style="flex:1;background:#52777A"></span><span style="flex:1;background:#542437"></span><span style="flex:1;background:#C02942"></span><span style="flex:1;background:#D95B43"></span><span style="flex:1;background:#ECD078"></span></div>

`#52777A` `#542437` `#C02942` `#D95B43` `#ECD078`

**`Palette::ploen()`** (5 colours): slate blues, mauve, and peach

<div style="display:flex;height:26px;width:100%;max-width:520px;border-radius:5px;overflow:hidden;border:1px solid rgba(0,0,0,.12)"><span style="flex:1;background:#3F5671"></span><span style="flex:1;background:#83A1C3"></span><span style="flex:1;background:#CEB5C8"></span><span style="flex:1;background:#FAC898"></span><span style="flex:1;background:#B17776"></span></div>

`#3F5671` `#83A1C3` `#CEB5C8` `#FAC898` `#B17776`

**`Palette::olga()`** (5 colours): pastel green, teal, gold, apricot, mauve

<div style="display:flex;height:26px;width:100%;max-width:520px;border-radius:5px;overflow:hidden;border:1px solid rgba(0,0,0,.12)"><span style="flex:1;background:#C9E3C2"></span><span style="flex:1;background:#8BC8CB"></span><span style="flex:1;background:#ECCD80"></span><span style="flex:1;background:#F5AB70"></span><span style="flex:1;background:#9C87A1"></span></div>

`#C9E3C2` `#8BC8CB` `#ECCD80` `#F5AB70` `#9C87A1`

**`Palette::mterese()`** (5 colours): cream, apricot, pink, and cornflower

<div style="display:flex;height:26px;width:100%;max-width:520px;border-radius:5px;overflow:hidden;border:1px solid rgba(0,0,0,.12)"><span style="flex:1;background:#F7DDAA"></span><span style="flex:1;background:#FAC3AD"></span><span style="flex:1;background:#F897A1"></span><span style="flex:1;background:#9298BA"></span><span style="flex:1;background:#9CBEED"></span></div>

`#F7DDAA` `#FAC3AD` `#F897A1` `#9298BA` `#9CBEED`

**`Palette::franscoise()`** (5 colours): blues, mauve, and warm reds

<div style="display:flex;height:26px;width:100%;max-width:520px;border-radius:5px;overflow:hidden;border:1px solid rgba(0,0,0,.12)"><span style="flex:1;background:#5980B1"></span><span style="flex:1;background:#B96A8D"></span><span style="flex:1;background:#A55062"></span><span style="flex:1;background:#E05256"></span><span style="flex:1;background:#E9A986"></span></div>

`#5980B1` `#B96A8D` `#A55062` `#E05256` `#E9A986`

**`Palette::fernande()`** (4 colours): coral, yellow, green, blue

<div style="display:flex;height:26px;width:100%;max-width:520px;border-radius:5px;overflow:hidden;border:1px solid rgba(0,0,0,.12)"><span style="flex:1;background:#FF7676"></span><span style="flex:1;background:#F9D662"></span><span style="flex:1;background:#7CAB7D"></span><span style="flex:1;background:#75B7D1"></span></div>

`#FF7676` `#F9D662` `#7CAB7D` `#75B7D1`

**`Palette::sylvie()`** (5 colours): gold, coral, lavender, teal, pink

<div style="display:flex;height:26px;width:100%;max-width:520px;border-radius:5px;overflow:hidden;border:1px solid rgba(0,0,0,.12)"><span style="flex:1;background:#E8B961"></span><span style="flex:1;background:#E88170"></span><span style="flex:1;background:#C6BDE8"></span><span style="flex:1;background:#5DB7C4"></span><span style="flex:1;background:#FD95BC"></span></div>

`#E8B961` `#E88170` `#C6BDE8` `#5DB7C4` `#FD95BC`

**`Palette::expevo()`** (6 colours): high-contrast orange, gold, teal, plum, navy, grey

<div style="display:flex;height:26px;width:100%;max-width:520px;border-radius:5px;overflow:hidden;border:1px solid rgba(0,0,0,.12)"><span style="flex:1;background:#FC4E07"></span><span style="flex:1;background:#E7B800"></span><span style="flex:1;background:#00AFBB"></span><span style="flex:1;background:#8B4769"></span><span style="flex:1;background:#1D457F"></span><span style="flex:1;background:#808080"></span></div>

`#FC4E07` `#E7B800` `#00AFBB` `#8B4769` `#1D457F` `#808080`

**`Palette::minou()`** (6 colours): teal, red, gold, green, ink, slate

<div style="display:flex;height:26px;width:100%;max-width:520px;border-radius:5px;overflow:hidden;border:1px solid rgba(0,0,0,.12)"><span style="flex:1;background:#00798C"></span><span style="flex:1;background:#D1495B"></span><span style="flex:1;background:#EDAE49"></span><span style="flex:1;background:#66A182"></span><span style="flex:1;background:#2E4057"></span><span style="flex:1;background:#8D96A3"></span></div>

`#00798C` `#D1495B` `#EDAE49` `#66A182` `#2E4057` `#8D96A3`

**`Palette::alger()`** (5 colours): black, teal, sage, gold, red

<div style="display:flex;height:26px;width:100%;max-width:520px;border-radius:5px;overflow:hidden;border:1px solid rgba(0,0,0,.12)"><span style="flex:1;background:#000000"></span><span style="flex:1;background:#1A5B5B"></span><span style="flex:1;background:#ACC8BE"></span><span style="flex:1;background:#F4AB5C"></span><span style="flex:1;background:#D1422F"></span></div>

`#000000` `#1A5B5B` `#ACC8BE` `#F4AB5C` `#D1422F`

**`Palette::seafarer()`** (5 colours): deep teal, cream, sage, moss, sand

<div style="display:flex;height:26px;width:100%;max-width:520px;border-radius:5px;overflow:hidden;border:1px solid rgba(0,0,0,.12)"><span style="flex:1;background:#013D5A"></span><span style="flex:1;background:#FCF3E3"></span><span style="flex:1;background:#BDD3CE"></span><span style="flex:1;background:#708C69"></span><span style="flex:1;background:#E4A25B"></span></div>

`#013D5A` `#FCF3E3` `#BDD3CE` `#708C69` `#E4A25B`

**`Palette::luminaries()`** (6 colours): orange, teal, charcoal, ivory, gold, mist

<div style="display:flex;height:26px;width:100%;max-width:520px;border-radius:5px;overflow:hidden;border:1px solid rgba(0,0,0,.12)"><span style="flex:1;background:#FF5B04"></span><span style="flex:1;background:#075056"></span><span style="flex:1;background:#233038"></span><span style="flex:1;background:#FDF6E3"></span><span style="flex:1;background:#F4D47C"></span><span style="flex:1;background:#D3DBDD"></span></div>

`#FF5B04` `#075056` `#233038` `#FDF6E3` `#F4D47C` `#D3DBDD`

**`Palette::casa_natal()`** (9 colours): vibrant nine-colour set for larger categorical scales

<div style="display:flex;height:26px;width:100%;max-width:520px;border-radius:5px;overflow:hidden;border:1px solid rgba(0,0,0,.12)"><span style="flex:1;background:#245E55"></span><span style="flex:1;background:#ED773C"></span><span style="flex:1;background:#808BC5"></span><span style="flex:1;background:#C63F3E"></span><span style="flex:1;background:#EAC119"></span><span style="flex:1;background:#EAA7C7"></span><span style="flex:1;background:#9ED6DF"></span><span style="flex:1;background:#1D1D1B"></span><span style="flex:1;background:#EAE4DA"></span></div>

`#245E55` `#ED773C` `#808BC5` `#C63F3E` `#EAC119` `#EAA7C7` `#9ED6DF` `#1D1D1B` `#EAE4DA`

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
| `Palette::paloma()` … `Palette::casa_natal()` | LTC qualitative palettes (see the LTC section above) |
| `Palette::custom(name, colors)` | User-defined palette |
| `pal[i]` | Color at index `i`; wraps with modulo |
| `pal.len()` | Number of colors |
| `pal.colors()` | Slice of all color strings |
| `pal.iter()` | Cycling iterator (never returns `None`) |
