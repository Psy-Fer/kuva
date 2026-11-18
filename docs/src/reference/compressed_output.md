# Compressed Output & HTML

Figures made of many rectangles (brick plots over hundreds of reads, large heatmaps,
clustermaps) produce big SVG files. kuva has three tools for keeping them small:

1. **Rect batching** (on by default) writes plain rectangles compactly, so the SVG itself is
   about half the size.
2. **`.svgz` output** gzips the SVG, usually 10 to 50 times smaller again.
3. **HTML helpers** put gzipped figures in a web page that decompresses them in the browser,
   so you get the small file size and it still opens anywhere, including straight from disk.

As a guide, a brick plot of 1,154 CIGAR bars is 92.5 KB as plain `<rect>`s, 40.9 KB with rect
batching, and 2.0 KB gzipped.

## Rect batching

Consecutive rectangles with no stroke, opacity or tooltip are written as one `<path>` per fill
colour instead of one `<rect>` per rectangle. Each rectangle stays its own closed shape inside
the path, so the thin gaps between bricks or cells are kept, and the image is the same:

- Rectangles of different colours that overlap must keep their paint order, so the run is
  split at that point and batching carries on after it.
- Rectangles of the same solid colour may overlap; painting them once as one shape looks the
  same. (A translucent colour would look lighter, so its overlaps split the run too.)

It is on by default for every SVG kuva writes. Turn it off if something downstream needs each
rectangle as its own element, for example to select or restyle single cells in Inkscape or
with CSS:

```rust,no_run
use kuva::backend::svg::SvgBackend;
# let scene = kuva::render::render::Scene::new(100.0, 100.0);

let svg = SvgBackend::new().with_rect_batching(false).render_scene(&scene);
```

On the CLI, pass `--no-rect-batching`.

## `.svgz` output

Requires the `svgz` feature (included in `full`).

```rust,no_run
use kuva::backend::svg::SvgBackend;
# let scene = kuva::render::render::Scene::new(100.0, 100.0);

let bytes = SvgBackend::new().render_scene_svgz(&scene);
std::fs::write("figure.svgz", bytes).unwrap();
```

```bash
kuva heatmap matrix.tsv -o figure.svgz
```

Image viewers, Inkscape and Firefox open a `.svgz` file directly. **Chrome does not open a
local `.svgz` file**: it only decompresses SVG that a web server sends with
`Content-Encoding: gzip`, and a `file://` path or an `<img src="figure.svgz">` fails. That makes
`.svgz` good for archiving and for serving from a configured web server, but not for handing
someone a file to double-click. For that, use HTML.

## HTML pages

The `kuva::html` module (also behind the `svgz` feature) puts gzipped figures in a web page.
The page carries each figure compressed and decompresses it in the browser with the built-in
`DecompressionStream` API. This works in Chrome 80+, Firefox 113+ and Safari 16.4+, opened from
disk or from a server, with no libraries.

| Function | Gives you |
|---|---|
| `html::figure(&svgz, name)` | One figure as an HTML snippet: an `<img>`, a "Download SVG" link and the compressed data, inside `<figure class="kuva-figure">`. |
| `html::figure_with_caption(&svgz, name, caption_html)` | The same with your own caption HTML (or none, if empty). |
| `html::download_link(name, label)` | A download button for a custom caption. |
| `html::escape(text)` | Escape text for a caption or your own page. |
| `html::loader_script()` | The `<script>` that decompresses and shows every figure on the page. Include it once, after the figures. |
| `html::standalone_page(title, &svgz, name)` | A complete self-contained `.html` file holding one figure. |

`name` is the image's alt text and the download's file name (`name.svg`). Each figure is shown
through an `<img>`, so figures cannot interfere with each other or with your page's styles, and
right-click "Save image as" saves the full SVG. The figure's size is read from the SVG and its
space reserved straight away, so the page doesn't jump as figures load, and figures decompress
only as they come near the screen. A page of many large figures stays fast.

On the CLI, `-o figure.html` writes a standalone page. Its title is `--title` if given,
otherwise the file name.

### One page per figure

```rust,no_run
use kuva::backend::svg::SvgBackend;
# let scene = kuva::render::render::Scene::new(100.0, 100.0);

let svgz = SvgBackend::new().render_scene_svgz(&scene);
let page = kuva::html::standalone_page("RFC1 reads", &svgz, "RFC1");
std::fs::write("RFC1.html", page).unwrap();
```

### A summary page linking to per-figure pages

Write one standalone page per figure, then an index page of your own that links to them.
Plain relative links work from disk, so the whole folder can be zipped and shared.

```rust,no_run
use kuva::backend::svg::SvgBackend;
# let scenes: Vec<(String, kuva::render::render::Scene)> = vec![];

let mut rows = String::new();
for (gene, scene) in &scenes {
    let svgz = SvgBackend::new().render_scene_svgz(scene);
    std::fs::write(format!("{gene}.html"), kuva::html::standalone_page(gene, &svgz, gene)).unwrap();
    rows.push_str(&format!("<li><a href=\"{gene}.html\">{gene}</a></li>\n"));
}
let index = format!(
    "<!DOCTYPE html>\n<html><head><meta charset=\"utf-8\"><title>Sample summary</title></head>\n\
     <body><h1>Sample summary</h1>\n<ul>\n{rows}</ul>\n</body></html>\n"
);
std::fs::write("index.html", index).unwrap();
```

Escape any text you put in your own page with `html::escape`. `figure` and `standalone_page`
already escape the title and name they are given.

### Many figures in one page, opened on demand

Put each figure in a `<details>` element. A figure inside a closed `<details>` is not
decompressed until it is opened, so the page loads quickly however many figures it holds.

```rust,no_run
use kuva::backend::svg::SvgBackend;
# let scenes: Vec<(String, kuva::render::render::Scene)> = vec![];

let mut body = String::from("<h1>All loci</h1>\n");
for (gene, scene) in &scenes {
    let svgz = SvgBackend::new().render_scene_svgz(scene);
    body.push_str(&format!("<details><summary>{gene}</summary>\n"));
    body.push_str(&kuva::html::figure(&svgz, gene));
    body.push_str("</details>\n");
}
body.push_str(kuva::html::loader_script());
let page = format!(
    "<!DOCTYPE html>\n<html><head><meta charset=\"utf-8\"><title>All loci</title>\n\
     </head>\n<body>\n{body}</body></html>\n"
);
std::fs::write("loci.html", page).unwrap();
```

To decompress a figure from your own script, for example when you show a hidden tab, call
`kuvaInflate(figureElement)`.

### Custom captions

`figure` writes its own caption with a "Download SVG" link. To put your own content there
(a locus name, flags, your own buttons), use `figure_with_caption`. Any link with class
`kuva-download` inside the figure gets the decompressed SVG once it loads, so a custom caption
can still offer the download:

```rust,no_run
use kuva::html;
# let svgz: Vec<u8> = vec![];

let locus = "RFC1 <AAGGG>n"; // from input data, so escape it
let caption = format!(
    "<strong>{}</strong> pathogenic expansion {}",
    html::escape(locus),
    html::download_link(locus, "Save SVG"),
);
let snippet = html::figure_with_caption(&svgz, locus, &caption);
```

The caption is inserted as is, so it can hold markup. Escape any text you put in it with
`html::escape`, especially names or notes that came from input data; otherwise a `<` in that
text becomes markup in the page. `download_link` escapes its own arguments.

### Styling

The snippet uses these classes, so you can style it from your page:

- `figure.kuva-figure`: the wrapper.
- `.kuva-frame`: the box that reserves the figure's space. It is already sized to the figure
  and scales down to fit narrow screens.
- `.kuva-figure img`: the figure itself.
- `a.kuva-download`: the download link.
- `p.kuva-error`: shown in place of a figure that can't be displayed.

Each figure also has a `data-kuva-state` attribute: `loading`, then `ready`; or `error` if its
data is damaged, or `unsupported` in a browser without `DecompressionStream`.

## Choosing a format

| You want | Use |
|---|---|
| A file for papers, slides or editing | `.svg` (or `.pdf` / `.png`) |
| To archive many figures compactly | `.svgz` |
| To share figures that open anywhere, including from disk | `.html` |
| To serve figures from a web server | `.svgz` with `Content-Encoding: gzip`, or `.html` |
