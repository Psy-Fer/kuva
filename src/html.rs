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
//!   data), to drop anywhere in your own page.
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
  function inflate(fig) {
    if (fig.getAttribute("data-kuva-state")) return;
    fig.setAttribute("data-kuva-state", "loading");
    var data = fig.querySelector('script[type="application/x-kuva-svgz"]');
    var img = fig.querySelector("img");
    if (!data || !img) return;
    if (typeof DecompressionStream === "undefined") {
      fig.setAttribute("data-kuva-state", "unsupported");
      img.alt = "This browser cannot show compressed figures (needs Chrome 80+, Firefox 113+ or Safari 16.4+).";
      return;
    }
    var bin = atob(data.textContent.trim());
    var bytes = new Uint8Array(bin.length);
    for (var i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
    var svg = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("gzip"));
    new Response(svg).blob().then(function (blob) {
      var url = URL.createObjectURL(new Blob([blob], { type: "image/svg+xml" }));
      img.src = url;
      var link = fig.querySelector("a.kuva-download");
      if (link) link.href = url;
      fig.setAttribute("data-kuva-state", "ready");
    });
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
    }, { rootMargin: "400px" });
    for (var j = 0; j < figs.length; j++) io.observe(figs[j]);
  }
  window.kuvaInflate = inflate;
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", start);
  else start();
})();
</script>
"#;

/// One compressed figure as an HTML snippet: a `<figure class="kuva-figure">` holding an
/// `<img>`, a "Download SVG" link (`a.kuva-download`) and the figure's gzip data in base64.
///
/// `svgz` is gzipped SVG, e.g. from
/// [`SvgBackend::render_scene_svgz`](crate::backend::svg::SvgBackend::render_scene_svgz).
/// `name` is the image's alt text and the download's file name (`.svg` is added unless it is
/// already there). The figure stays blank until [`loader_script`] is on the page.
pub fn figure(svgz: &[u8], name: &str) -> String {
    let file = if name.to_ascii_lowercase().ends_with(".svg") {
        name.to_string()
    } else {
        format!("{name}.svg")
    };
    let data = base64_encode(svgz);
    let mut out = String::with_capacity(data.len() + 400);
    out.push_str("<figure class=\"kuva-figure\">\n<img alt=\"");
    out.push_str(&escape_attr(name));
    out.push_str("\">\n<figcaption><a class=\"kuva-download\" download=\"");
    out.push_str(&escape_attr(&file));
    out.push_str("\">Download SVG</a></figcaption>\n<script type=\"");
    out.push_str(DATA_TYPE);
    out.push_str("\">");
    // Base64 has no `<`, so it can never close the script element early.
    out.push_str(&data);
    out.push_str("</script>\n</figure>\n");
    out
}

/// The `<script>` element that decompresses and shows every [`figure`] on the page. Include it
/// once per page, after the figures (e.g. just before `</body>`).
///
/// Figures are decompressed as they come within 400 px of the viewport, so a long page only
/// pays for what is looked at. To decompress one yourself (for example when a hidden panel is
/// shown), call `kuvaInflate(figureElement)` from your own script.
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
         .kuva-figure img { display: block; max-width: 100%; height: auto; }\n\
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
}
