# Fonts

kuva draws text in DejaVu Sans unless you choose another font. Every label, tick, legend entry
and title is measured with the font's real character widths, so margins, legend boxes and label
gutters fit the text exactly.

With the `custom_fonts` feature (part of `full`) you can load your own font files and use them
by family name. A loaded font is measured, drawn and embedded exactly: the spacing in the SVG,
PNG and PDF output all comes from that font, not from DejaVu Sans.

## Loading a font

Load each font file once, before rendering, then select its family:

```rust,no_run
use kuva::fonts;
use kuva::render::layout::Layout;
# use kuva::render::plots::Plot;
# fn main() -> Result<(), kuva::fonts::FontError> {
# let plots: Vec<Plot> = vec![];

fonts::register_font_file("fonts/Inter-Regular.ttf")?;
fonts::register_font_file("fonts/Inter-Bold.ttf")?;

let layout = Layout::auto_from_plots(&plots).with_font_family("Inter, sans-serif");
# Ok(())
# }
```

On the CLI, `--font-file` loads a file. Without `--font`, the family of the first file becomes
the font for all text:

```bash
kuva bar data.tsv --label-col gene --value-col count \
    --font-file fonts/Inter-Regular.ttf --font-file fonts/Inter-Bold.ttf
```

| Library | CLI | Loads |
|---|---|---|
| `fonts::register_font_file(path)` | `--font-file PATH` (repeatable) | One file: TTF, OTF, or every face in a TTC/OTC collection |
| `fonts::register_fonts_dir(dir)` | `--font-dir DIR` (repeatable) | Every font file in a directory and its subdirectories |
| `fonts::register_font_bytes(bytes)` | | Font data already in memory, for example fetched by a web page |

Each call returns what it loaded (`FontInfo`: family, weight, italic, width, monospace,
variable, and whether its licence restricts embedding). A directory skips files it can't read
and lists them in `DirReport::skipped`; it is an error only when nothing in it loads. On the
CLI a bad file inside a directory is a warning, and a missing `--font-file` is an error.

Loaded fonts belong to the whole process and are never removed. Loading a file with the same
family, weight, style and width again replaces the earlier one. `fonts::registered_fonts()`
lists what is loaded.

The family name is read from the font file, so `Inter-Bold.ttf` joins the `Inter` family rather
than becoming a family called "Inter Bold".

## Choosing families

| Layout | CLI | Sets the font for |
|---|---|---|
| `with_font_family(list)` | `--font FAMILY` | All text |
| `with_tick_font_family(list)` | `--tick-font FAMILY` | Axis tick labels only |
| `with_mono_font_family(list)` | `--mono-font FAMILY` | `code` spans (backtick text), default DejaVu Sans Mono |

Each takes a CSS `font-family` list such as `"Inter, Helvetica, sans-serif"`. kuva uses the
**first** family: that is the one it measures and draws. The rest are fallbacks for SVG viewers
that don't have the first one.

A family that isn't loaded still works the way it always has: SVG names it, and the viewer uses
it if it is installed. kuva can't measure a font it hasn't loaded, though, so the text is spaced
as DejaVu Sans, and PNG output draws DejaVu Sans. The CLI warns when `--font` names a family that
isn't loaded (generic names such as `serif` and `monospace`, and DejaVu Sans itself, don't warn).

## Bold, italic and weights

kuva uses up to four styles of a family (regular, bold, italic and bold italic) and picks a face
for each with the CSS font-matching rules, the same ones browsers and the PDF backend use:

- **Width first.** A normal-width face is preferred, then the nearest narrower, then the nearest
  wider, so a Condensed file never replaces the regular face.
- **Then style.** Italic text uses an italic (or oblique) face if the family has one.
- **Then weight.** Regular text asks for weight 400, bold for 700, and each takes the nearest
  weight the family has. A family with only Light and Black faces uses Light for normal text
  and Black for bold.

**A missing style is never faked.** If a family has no bold face, bold text uses its nearest face
(usually the regular one) in every output. Browsers would normally draw a fake bold, so SVG output
that uses a loaded font sets `font-synthesis: none` to stop them. That keeps the SVG, PNG and PDF
the same, and matching the layout's measurements. The cost is lost emphasis, so check what a
family is missing:

```rust,no_run
# use kuva::fonts;
if let Some(styles) = fonts::family_styles("Inter") {
    for style in styles.missing() {
        eprintln!("Inter has no {style} face");
    }
}
```

The CLI warns once per run, for example "DejaVu Serif has no italic or bold italic face". Load the
missing font files to fix it.

**Variable fonts** (one file with a weight axis) load only their default style. Use the static
files for other weights; most font downloads include them, often in a `static/` folder. The CLI
warns when it loads a variable font.

## What each output does

| Output | Loaded fonts |
|---|---|
| SVG | Names the family. With embedding on, includes the font files the figure uses (see below) |
| PNG | Draws the loaded font itself |
| PDF | Embeds the loaded font (krilla subsets it, so only the characters used are stored) |
| Terminal | Text is drawn in terminal characters; fonts don't apply |

The PDF backend also loads the system's fonts, so a family that is installed but not loaded
still renders in a PDF, but it is spaced as DejaVu Sans. A loaded font always wins over an
installed font of the same name.

## Embedding fonts in SVG

An SVG normally names its fonts and relies on the viewer to have them. To make a self-contained
file, turn on embedding:

```rust,no_run
use kuva::backend::svg::SvgBackend;
# let scene = kuva::render::render::Scene::new(100.0, 100.0);

let svg = SvgBackend::new().with_embedded_font(true).render_scene(&scene);
```

On the CLI, pass `--embed-font`. This always embeds DejaVu Sans (about 1 MB). Each loaded font
the figure actually draws is added as well: a figure with no bold text embeds only the regular
face. Faces inside a TTC/OTC collection are not embedded, because a web font can't pick one face
out of a collection file; load the single-face files if you need them embedded.

**Licences.** Embedding a font (in SVG or PDF) passes a copy of it on to whoever gets the file.
Make sure the font's licence allows that. Some fonts mark themselves as not embeddable; the CLI
warns when it is about to embed one (with `--embed-font`, or for PDF output), and library users
can check `FontInfo::restricted_embedding`.

## Limits

- **No text shaping.** kuva adds up character widths. Kerning, ligatures and scripts that need
  shaping (Arabic, Devanagari and others) are not handled. Browsers apply kerning, so SVG text can
  come out slightly narrower than the space kept for it.
- **Characters a font doesn't have** keep one em of space. SVG viewers draw them with a fallback
  font; PNG draws the font's placeholder glyph (usually a box). There is no automatic fallback to
  a second font.
- **Character widths cover the Basic Multilingual Plane** (U+0000 to U+FFFF), read from at most the
  first 32 Unicode `cmap` subtables of the font. Ordinary fonts have a handful; the limit keeps a
  damaged or crafted font from making the measurement slow.
- **Collections (TTC/OTC) may hold at most 1,024 faces**, each listed once, and a family name
  may be at most 256 characters (`kuva::fonts::MAX_COLLECTION_FACES`, `MAX_FAMILY_NAME_CHARS`).
  A collection over the limit, or listing one face twice, fails to register with an error; a
  longer name record is ignored.
- **Installed fonts aren't looked up by name.** Load the font file with `register_font_file` or
  `--font-file`.
- Two sizes are decided before a font family can be set, so they are always measured with DejaVu
  Sans: the extra x-axis room for bump-chart series labels, and the frame around a `Clustermap`
  drawn with `with_cell_size`. With another font they can be off by a few pixels.
