//! Scientific plotting library for bioinformatics, targeting SVG output with optional PNG and PDF backends.
//!
//! # Pipeline
//!
//! ```text
//! plot definition  →  Layout  →  Scene (primitives)  →  backend output
//! ```
//!
//! 1. Build a plot struct using its builder API (e.g. [`plot::scatter::ScatterPlot`]).
//! 2. Wrap it in the [`render::plots::Plot`] enum.
//! 3. Compute axis ranges with [`render::layout::Layout::auto_from_plots`].
//! 4. Pass to a `render_*` function (e.g. [`render::render::render_multiple`]) to get a [`render::render::Scene`].
//! 5. Convert the scene to output with [`backend::svg::SvgBackend`].
//!
//! # Example
//!
//! ```rust
//! use visus::plot::scatter::ScatterPlot;
//! use visus::render::plots::Plot;
//! use visus::render::layout::Layout;
//! use visus::render::render::render_multiple;
//! use visus::backend::svg::SvgBackend;
//!
//! let scatter = ScatterPlot::new()
//!     .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
//!     .with_color("steelblue");
//!
//! let plots = vec![Plot::Scatter(scatter)];
//! let layout = Layout::auto_from_plots(&plots);
//! let scene = render_multiple(plots, layout);
//! let svg = SvgBackend.render_scene(&scene);
//! assert!(svg.contains("<svg"));
//! ```
//!
//! # Feature flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `png`   | Enables [`PngBackend`] for rasterising SVG scenes via `resvg`. |
//! | `pdf`   | Enables [`PdfBackend`] for vector PDF output via `svg2pdf`. |
//! | `cli`   | Enables the `visus` CLI binary (pulls in `clap`). |
//! | `full`  | Enables `png` + `pdf`. |

pub mod plot;
pub mod backend;
pub mod render;

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