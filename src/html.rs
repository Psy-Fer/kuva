//! Put compressed figures in a web page.
//!
//! A big SVG (a brick plot over many reads, a large heatmap) is mostly repeated markup, so it
//! gzips 10 to 50 times smaller. Browsers can't show a local `.svgz` file directly, but a page
//! can carry the gzipped figure and decompress it itself with the built-in
//! `DecompressionStream` API. That works from `file://` too, with no server and no libraries
//! (Chrome 80+, Firefox 113+, Safari 16.4+).
//!
//! These helpers produce the pieces; the page around them is yours:
//!
//! - [`figure`]: one figure as an HTML snippet (an `<img>`, a download link and the compressed
//!   data), to drop anywhere in your own page. [`figure_with_caption`] takes your own caption,
//!   and [`download_link`] makes a download button for it.
//! - [`loader_script`]: the `<script>` that decompresses every figure on the page. Include it
//!   once, after the figures. Figures are decompressed as they scroll into view, so a page of
//!   many large figures stays fast, and a figure inside a closed `<details>` is only
//!   decompressed when it is opened.
//! - [`standalone_page`]: a complete self-contained `.html` file holding one figure.
//!
//! Each figure is shown through an `<img>`, so figures can't interfere with each other or with
//! the page, and right-click "Save image as" saves the full SVG. The download link saves it
//! under the name you give.
//!
//! ```rust
//! use kuva::backend::svg::SvgBackend;
//! use kuva::plot::ScatterPlot;
//! use kuva::render::layout::Layout;
//! use kuva::render::plots::Plot;
//! use kuva::render::render::render_multiple;
//!
//! let plots: Vec<Plot> = vec![ScatterPlot::new().with_data(vec![(1.0, 2.0), (3.0, 4.0)]).into()];
//! let layout = Layout::auto_from_plots(&plots);
//! let scene = render_multiple(plots, layout);
//! let svgz = SvgBackend::new().render_scene_svgz(&scene);
//!
//! // One self-contained page:
//! let page = kuva::html::standalone_page("My figure", &svgz, "my_figure");
//! assert!(page.starts_with("<!DOCTYPE html>"));
//!
//! // Or several figures in a page of your own:
//! let body = format!(
//!     "<h1>Results</h1>{}{}{}",
//!     kuva::html::figure(&svgz, "first"),
//!     kuva::html::figure(&svgz, "second"),
//!     kuva::html::loader_script(),
//! );
//! assert_eq!(body.matches("class=\"kuva-figure\"").count(), 2);
//! ```

use crate::encoding::base64_encode;
use crate::render::render_utils::escape_attr;

/// The `type` of the `<script>` element holding a figure's base64 gzip data. Not a type
/// browsers run, so the data is inert until the loader reads it.
const DATA_TYPE: &str = "application/x-kuva-svgz";

/// The loader `<script>`. A plain ES5-style IIFE with no dependencies; the only modern API it
/// needs is `DecompressionStream`, and it says so in the figure if the browser lacks it.
const LOADER: &str = r#"<script>
(function () {
  // Replace a figure that can't be shown with a visible message (class kuva-error), and give
  // back the space reserved for it.
  function show(fig, img, state, message) {
    fig.setAttribute("data-kuva-state", state);
    img.alt = message;
    img.style.display = "none";
    var frame = img.parentNode;
    frame.style.aspectRatio = "auto";
    var note = document.createElement("p");
    note.className = "kuva-error";
    note.textContent = message;
    frame.insertBefore(note, img);
  }
  function inflate(fig) {
    if (fig.getAttribute("data-kuva-state")) return;
    fig.setAttribute("data-kuva-state", "loading");
    var data = fig.querySelector('script[type="application/x-kuva-svgz"]');
    var img = fig.querySelector("img");
    if (!data || !img) return;
    if (typeof DecompressionStream === "undefined") {
      show(fig, img, "unsupported", "This browser cannot show compressed figures (needs Chrome 80+, Firefox 113+ or Safari 16.4+).");
      return;
    }
    function fail() {
      show(fig, img, "error", "This figure could not be shown: its data is damaged.");
    }
    try {
      var bin = atob(data.textContent.trim());
      var bytes = new Uint8Array(bin.length);
      for (var i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
      var svg = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("gzip"));
      new Response(svg).blob().then(function (blob) {
        var url = URL.createObjectURL(new Blob([blob], { type: "image/svg+xml" }));
        img.onerror = fail;
        img.src = url;
        var links = fig.querySelectorAll("a.kuva-download");
        for (var k = 0; k < links.length; k++) links[k].href = url;
        fig.setAttribute("data-kuva-state", "ready");
      }).catch(fail);
    } catch (e) {
      fail();
    }
  }
  function start() {
    var figs = document.querySelectorAll("figure.kuva-figure");
    if (typeof IntersectionObserver === "undefined") {
      for (var i = 0; i < figs.length; i++) inflate(figs[i]);
      return;
    }
    var io = new IntersectionObserver(function (entries) {
      entries.forEach(function (e) {
        if (e.isIntersecting) {
          io.unobserve(e.target);
          inflate(e.target);
        }
      });
    }, { rootMargin: "200px" });
    for (var j = 0; j < figs.length; j++) io.observe(figs[j]);
  }
  window.kuvaInflate = inflate;
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", start);
  else start();
})();
</script>
"#;

/// One compressed figure as an HTML snippet: a `<figure class="kuva-figure">` holding an
/// `<img>`, a caption with a "Download SVG" link (`a.kuva-download`) and the figure's gzip data
/// in base64.
///
/// `svgz` is gzipped SVG, e.g. from
/// [`SvgBackend::render_scene_svgz`](crate::backend::svg::SvgBackend::render_scene_svgz).
/// `name` is the image's alt text and the download's file name (`.svg` is added unless it is
/// already there). The figure stays blank until [`loader_script`] is on the page. For your own
/// caption, use [`figure_with_caption`].
pub fn figure(svgz: &[u8], name: &str) -> String {
    figure_with_caption(svgz, name, &download_link(name, "Download SVG"))
}

/// Like [`figure`], with your own HTML as the caption. An empty caption leaves out the
/// `<figcaption>`.
///
/// The caption is inserted as is, so it can hold markup and buttons. **Escape any text you put
/// in it** with [`escape`], especially anything that came from input data (sample names,
/// locus names, notes), or a `<` in that text becomes markup in the page.
///
/// Every `a.kuva-download` link inside the figure gets the decompressed SVG as its `href`
/// once the figure loads, so a caption can carry its own download button: build it with
/// [`download_link`] or write `<a class="kuva-download" download="name.svg">` yourself.
///
/// ```rust
/// use kuva::html;
/// # let svgz = Vec::new();
/// let locus = "RFC1 <AAGGG>n"; // from input data: escape it
/// let caption = format!(
///     "<strong>{}</strong> {}",
///     html::escape(locus),
///     html::download_link(locus, "Save SVG"), // escapes its own arguments
/// );
/// let snippet = html::figure_with_caption(&svgz, locus, &caption);
/// assert!(snippet.contains("<strong>RFC1 &lt;AAGGG&gt;n</strong>"));
/// ```
pub fn figure_with_caption(svgz: &[u8], name: &str, caption_html: &str) -> String {
    let data = base64_encode(svgz);
    let mut out = String::with_capacity(data.len() + caption_html.len() + 400);
    out.push_str("<figure class=\"kuva-figure\">\n");
    // Reserve the figure's own size before it decompresses, so the page doesn't jump as
    // figures load and lazy loading only starts figures that are really near the screen. This
    // has to be on a wrapper: an `<img>` with no `src` yet is drawn as its alt text, and
    // browsers ignore its width/height until the image arrives. The frame scales down on
    // narrow screens and keeps the aspect ratio.
    let alt = escape_attr(name);
    match svg_size(svgz) {
        Some((w, h)) => {
            let (w, h) = (w.round(), h.round());
            out.push_str(&format!(
                "<div class=\"kuva-frame\" style=\"max-width:{w}px;aspect-ratio:{w} / {h}\">\
                 <img alt=\"{alt}\" width=\"{w}\" height=\"{h}\" \
                 style=\"display:block;width:100%;height:auto\"></div>\n"
            ));
        }
        None => out.push_str(&format!(
            "<div class=\"kuva-frame\"><img alt=\"{alt}\" \
             style=\"display:block;max-width:100%\"></div>\n"
        )),
    }
    if !caption_html.is_empty() {
        out.push_str("<figcaption>");
        out.push_str(caption_html);
        out.push_str("</figcaption>\n");
    }
    out.push_str("<script type=\"");
    out.push_str(DATA_TYPE);
    out.push_str("\">");
    // Base64 has no `<`, so it can never close the script element early.
    out.push_str(&data);
    out.push_str("</script>\n</figure>\n");
    out
}

/// Escape text for HTML: `&`, `<`, `>`, `"` and `'` become entities. Safe for element text and
/// for double- or single-quoted attribute values. Use it on any text you put into a caption for
/// [`figure_with_caption`] or into your own page.
pub fn escape(text: &str) -> String {
    escape_attr(text)
}

/// A link that downloads its figure as `name.svg` (`.svg` is added unless `name` already ends
/// with it), for a caption written with [`figure_with_caption`]. `label` is the link text.
/// Both are escaped.
pub fn download_link(name: &str, label: &str) -> String {
    let file = if name.to_ascii_lowercase().ends_with(".svg") {
        name.to_string()
    } else {
        format!("{name}.svg")
    };
    format!(
        "<a class=\"kuva-download\" download=\"{}\">{}</a>",
        escape_attr(&file),
        escape_attr(label)
    )
}

/// The `width`/`height` of the root `<svg>` element, read from the start of the gzip data
/// without decompressing the rest. `None` if they can't be found.
fn svg_size(svgz: &[u8]) -> Option<(f64, f64)> {
    use std::io::Read;
    let mut head = Vec::with_capacity(2048);
    flate2::read::GzDecoder::new(svgz)
        .take(2048)
        .read_to_end(&mut head)
        .ok()?;
    let head = String::from_utf8_lossy(&head);
    let start = head.find("<svg")?;
    let tag = &head[start..start + head[start..].find('>')?];
    let attr = |name: &str| -> Option<f64> {
        let key = format!(" {name}=\"");
        let from = tag.find(&key)? + key.len();
        let value = &tag[from..from + tag[from..].find('"')?];
        value
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite() && *v > 0.0)
    };
    Some((attr("width")?, attr("height")?))
}

/// The `<script>` element that decompresses and shows every [`figure`] on the page. Include it
/// once per page, after the figures (e.g. just before `</body>`).
///
/// Figures are decompressed as they come within 200 px of the viewport, so a long page only
/// pays for what is looked at. To decompress one yourself (for example when a hidden panel is
/// shown), call `kuvaInflate(figureElement)` from your own script.
///
/// Each figure's `data-kuva-state` is `loading`, then `ready`; or `error` if its data is
/// damaged, or `unsupported` in a browser without `DecompressionStream`. On either failure the
/// image is replaced by a `<p class="kuva-error">` explaining why.
pub fn loader_script() -> &'static str {
    LOADER
}

/// A complete, self-contained HTML page showing one compressed figure, with `title` as the
/// page title. Write it to a `.html` file and open it in any current browser, including
/// straight from disk.
pub fn standalone_page(title: &str, svgz: &[u8], name: &str) -> String {
    let fig = figure(svgz, name);
    let mut out = String::with_capacity(fig.len() + LOADER.len() + 600);
    out.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    out.push_str("<title>");
    out.push_str(&escape_attr(title));
    out.push_str("</title>\n<style>\n");
    out.push_str(
        "body { margin: 16px; font-family: system-ui, sans-serif; background: #fff; }\n\
         .kuva-figure { margin: 0; }\n\
         .kuva-figure figcaption { margin-top: 8px; font-size: 14px; }\n",
    );
    out.push_str("</style>\n</head>\n<body>\n");
    out.push_str(&fig);
    out.push_str(loader_script());
    out.push_str("</body>\n</html>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn gunzip(bytes: &[u8]) -> String {
        let mut s = String::new();
        flate2::read::GzDecoder::new(bytes)
            .read_to_string(&mut s)
            .unwrap();
        s
    }

    /// Minimal base64 decoder for the round-trip test.
    fn unbase64(s: &str) -> Vec<u8> {
        let val = |c: u8| match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            _ => 63,
        };
        let bytes: Vec<u8> = s.bytes().filter(|&c| c != b'=').collect();
        let mut out = Vec::new();
        for chunk in bytes.chunks(4) {
            let n = chunk
                .iter()
                .enumerate()
                .fold(0u32, |n, (i, &c)| n | (val(c) as u32) << (18 - 6 * i));
            out.push((n >> 16) as u8);
            if chunk.len() > 2 {
                out.push((n >> 8) as u8);
            }
            if chunk.len() > 3 {
                out.push(n as u8);
            }
        }
        out
    }

    #[test]
    fn figure_round_trips_the_svg() {
        let svg =
            "<svg xmlns=\"http://www.w3.org/2000/svg\"><rect width=\"1\" height=\"1\"/></svg>";
        let fig = figure(&crate::encoding::gzip(svg.as_bytes()), "gene A");
        let start = fig.find("svgz\">").unwrap() + "svgz\">".len();
        let end = fig[start..].find("</script>").unwrap() + start;
        assert_eq!(gunzip(&unbase64(&fig[start..end])), svg);
        assert!(fig.contains("download=\"gene A.svg\""));
        assert!(fig.contains("alt=\"gene A\""));
    }

    #[test]
    fn names_are_escaped() {
        let fig = figure(b"", "a\"><script>x</script>");
        assert!(!fig.contains("<script>x"));
        let page = standalone_page("</title><b>", b"", "n.svg");
        assert!(!page.contains("</title><b>"));
        // An explicit .svg is not doubled.
        assert!(page.contains("download=\"n.svg\""));
    }

    #[test]
    fn standalone_page_has_one_figure_and_the_loader() {
        let page = standalone_page("T", &crate::encoding::gzip(b"<svg/>"), "f");
        assert_eq!(page.matches("class=\"kuva-figure\"").count(), 1);
        assert_eq!(page.matches("DecompressionStream").count(), 2);
        assert!(page.ends_with("</html>\n"));
    }

    /// The `<img>` carries the SVG's own size, read from the gzip data, so the page reserves
    /// the space before the figure decompresses.
    #[test]
    fn figure_img_has_the_svg_size() {
        let svg =
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="820.4" height="520"><g/></svg>"#;
        let fig = figure(&crate::encoding::gzip(svg.as_bytes()), "f");
        assert!(
            fig.contains(
                r#"<div class="kuva-frame" style="max-width:820px;aspect-ratio:820 / 520">"#
            ),
            "{fig}"
        );
        assert!(fig.contains(r#"width="820" height="520""#), "{fig}");
        // Unreadable data: no size, still a figure.
        let fig = figure(b"not gzip", "f");
        assert!(!fig.contains("aspect-ratio"), "{fig}");
        assert!(fig.contains("<img alt=\"f\""));
    }

    #[test]
    fn custom_caption_and_download_link() {
        let gz = crate::encoding::gzip(b"<svg/>");
        let caption = format!("<b>RFC1</b> {}", download_link("RFC1 <x>", "Save & go"));
        let fig = figure_with_caption(&gz, "RFC1", &caption);
        assert!(fig.contains("<figcaption><b>RFC1</b> <a class=\"kuva-download\" download=\"RFC1 &lt;x&gt;.svg\">Save &amp; go</a></figcaption>"), "{fig}");
        let bare = figure_with_caption(&gz, "RFC1", "");
        assert!(!bare.contains("figcaption"), "{bare}");
        assert!(!bare.contains("kuva-download"));
    }
}
