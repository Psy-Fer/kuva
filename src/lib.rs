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
//!
//! # Fonts
//!
//! DejaVu Sans is bundled inside the crate. The PNG and PDF backends always load
//! it before scanning system fonts, so text renders correctly even in minimal
//! environments (containers, CI pipelines) with no installed fonts.
//!
//! SVG output references fonts by name and relies on the viewer to resolve them.
//! For self-contained SVGs that work anywhere, use
//! [`backend::svg::SvgBackend::with_embedded_font`] or the `--embed-font` CLI flag.
//! This bakes DejaVu Sans as a base64 `@font-face` block into the SVG at the cost
//! of roughly 1 MB of added file size.

pub mod backend;
pub mod plot;
pub mod prelude;
pub mod render;

pub(crate) mod fonts;

pub use backend::terminal::TerminalBackend;

#[cfg(feature = "png")]
pub use backend::png::PngBackend;

#[cfg(feature = "png")]
pub use backend::raster::RasterBackend;

#[cfg(feature = "pdf")]
pub use backend::pdf::PdfBackend;

pub use render::datetime::{ymd, ymd_hms, DateTimeAxis, DateUnit};
/// KDE bandwidth via Silverman's rule of thumb: `h = 1.06 σ n^{-1/5}`.
///
/// Use this together with [`simple_kde`] or [`simple_kde_reflect`] to
/// pre-compute a density curve before passing it to
/// [`plot::DensityPlot::from_curve`].  Pre-computing lets you inspect the y
/// range and set custom axis bounds before rendering.
pub use render::render_utils::silverman_bandwidth;

/// Gaussian kernel density estimate evaluated at `samples` equally-spaced
/// points spanning `[data_min − 3h, data_max + 3h]`.
///
/// Returns `(x, unnormalised_kernel_sum)` pairs; divide y by `n · h · √(2π)`
/// to get probability density.  For data bounded at a known limit use
/// [`simple_kde_reflect`] instead.
pub use render::render_utils::simple_kde;

pub use render::layout::{AxisLine, TickAlign, TickFormat, TickPos};
pub use render::palette::Palette;
pub use render::render::render_calendar;
pub use render::render::render_phylo_tree;
pub use render::render::render_sankey;
pub use render::render::render_synteny;
pub use render::render::render_twin_y;
/// Like [`simple_kde`] but applies boundary reflection at `x_lo` and/or
/// `x_hi` so the curve does not bleed into physically impossible values.
///
/// Set `reflect_lo = true` when data cannot go below `x_lo` (e.g. identity
/// scores ≥ 0); set `reflect_hi = true` when data cannot exceed `x_hi`.
pub use render::render_utils::simple_kde_reflect;
pub use render::theme::Theme;

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
/// For fine-grained control — custom layout, twin axes, or embedded-font SVG —
/// use [`render::render::render_multiple`] and [`struct@backend::svg::SvgBackend`] directly.
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
    backend::png::PngBackend::new()
        .with_scale(scale)
        .render_scene(&scene)
}

/// Render a collection of plots directly to a PNG byte vector via `tiny_skia`,
/// bypassing SVG serialization and re-parsing (requires feature `png`).
///
/// This is significantly faster than [`render_to_png`] for data-heavy plots
/// (scatter, manhattan, heatmap) because it skips the SVG round-trip.
/// Text elements (axis labels, titles) are still rendered via resvg for
/// correct font shaping.
///
/// `scale` is the pixel density multiplier.
#[cfg(feature = "png")]
pub fn render_to_raster(
    plots: Vec<render::plots::Plot>,
    layout: render::layout::Layout,
    scale: f32,
) -> Result<Vec<u8>, String> {
    let scene = render::render::render_multiple(plots, layout);
    backend::raster::RasterBackend::new()
        .with_scale(scale)
        .render_scene(&scene)
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
