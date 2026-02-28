//! Scientific plotting library for bioinformatics, targeting SVG output with optional PNG and PDF backends.
//!
//! # Pipeline
//!
//! ```text
//! plot definition  →  Layout  →  Scene (primitives)  →  backend output
//! ```
//!
//! 1. Build a plot struct using its builder API (e.g. [`plot::scatter::ScatterPlot`]).
//! 2. Collect plots into a `Vec<`[`render::plots::Plot`]`>` — use `.into()` on any plot struct.
//! 3. Build a [`render::layout::Layout`] with [`render::layout::Layout::auto_from_plots`] and customise it.
//! 4. Call [`render_to_svg`] (or [`render_to_png`] / [`render_to_pdf`]) to get the output in one step.
//!
//! # Example
//!
//! ```rust
//! use kuva::prelude::*;
//!
//! let scatter = ScatterPlot::new()
//!     .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
//!     .with_color("steelblue");
//!
//! let plots: Vec<Plot> = vec![scatter.into()];
//! let svg = kuva::render_to_svg(plots, Layout::auto_from_plots(&[]));
//! assert!(svg.contains("<svg"));
//! ```
//!
//! # Feature flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `png`   | Enables [`PngBackend`] for rasterising SVG scenes via `resvg`. |
//! | `pdf`   | Enables [`PdfBackend`] for vector PDF output via `svg2pdf`. |
//! | `cli`   | Enables the `kuva` CLI binary (pulls in `clap`). |
//! | `full`  | Enables `png` + `pdf`. |

pub mod plot;
pub mod backend;
pub mod render;
pub mod prelude;

pub use backend::terminal::TerminalBackend;

#[cfg(feature = "png")]
pub use backend::png::PngBackend;

#[cfg(feature = "pdf")]
pub use backend::pdf::PdfBackend;

pub use render::theme::Theme;
pub use render::palette::Palette;
pub use render::layout::TickFormat;
pub use render::render::render_twin_y;
pub use render::render::render_sankey;
pub use render::render::render_phylo_tree;
pub use render::render::render_synteny;
pub use render::datetime::{DateTimeAxis, DateUnit, ymd, ymd_hms};

/// Render a collection of plots to an SVG string in one call.
///
/// See also [`render_to_png`] and [`render_to_pdf`] for raster and vector alternatives.
///
/// This collapses the four-step pipeline into a single expression:
///
/// ```rust
/// use kuva::prelude::*;
///
/// let scatter = ScatterPlot::new()
///     .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
///     .with_color("steelblue");
///
/// let plots: Vec<Plot> = vec![scatter.into()];
/// let svg = kuva::render_to_svg(plots, Layout::auto_from_plots(&[]));
/// assert!(svg.contains("<svg"));
/// ```
///
/// For fine-grained control (custom layout, twin axes, special-case plot types)
/// use [`render::render::render_multiple`] and [`backend::svg::SvgBackend`] directly.
pub fn render_to_svg(plots: Vec<render::plots::Plot>, layout: render::layout::Layout) -> String {
    let scene = render::render::render_multiple(plots, layout);
    backend::svg::SvgBackend.render_scene(&scene)
}

/// Render a collection of plots to a PNG byte vector in one call (requires feature `png`).
///
/// `scale` is the pixel density multiplier: `1.0` matches the SVG logical size,
/// `2.0` (the [`PngBackend`] default) gives retina/HiDPI quality.
///
/// Returns `Err(String)` if SVG parsing or rasterisation fails.
///
/// For fine-grained control use [`render::render::render_multiple`] and
/// [`backend::png::PngBackend`] directly.
#[cfg(feature = "png")]
pub fn render_to_png(
    plots: Vec<render::plots::Plot>,
    layout: render::layout::Layout,
    scale: f32,
) -> Result<Vec<u8>, String> {
    let scene = render::render::render_multiple(plots, layout);
    backend::png::PngBackend::new().with_scale(scale).render_scene(&scene)
}

/// Render a collection of plots to a PDF byte vector in one call (requires feature `pdf`).
///
/// Returns `Err(String)` if SVG parsing or PDF conversion fails.
///
/// For fine-grained control use [`render::render::render_multiple`] and
/// [`backend::pdf::PdfBackend`] directly.
#[cfg(feature = "pdf")]
pub fn render_to_pdf(
    plots: Vec<render::plots::Plot>,
    layout: render::layout::Layout,
) -> Result<Vec<u8>, String> {
    let scene = render::render::render_multiple(plots, layout);
    backend::pdf::PdfBackend.render_scene(&scene)
}