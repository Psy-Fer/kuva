//! Fonts.
//!
//! kuva bundles DejaVu Sans, which layout measures and the PNG/PDF backends draw.
//! With the `custom_fonts` feature you can also load your own fonts at runtime and
//! select them by family name:
//!
//! ```no_run
//! # fn main() -> Result<(), kuva::fonts::FontError> {
//! use kuva::fonts;
//!
//! fonts::register_font_file("fonts/Inter-Regular.ttf")?;
//! fonts::register_font_file("fonts/Inter-Bold.ttf")?;
//!
//! // Warn about styles the family lacks: kuva never fakes a bold or italic.
//! if let Some(styles) = fonts::family_styles("Inter") {
//!     for style in styles.missing() {
//!         eprintln!("Inter has no {style} face");
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Registered fonts are process-wide and are never removed. Registering a face with
//! the same family, weight and style as an earlier one replaces it for new lookups.

#[cfg(any(
    feature = "embed_font",
    feature = "png",
    feature = "pdf",
    feature = "typst-math"
))]
mod bundled;
#[cfg(any(
    feature = "embed_font",
    feature = "png",
    feature = "pdf",
    feature = "typst-math"
))]
pub(crate) use bundled::*;

#[cfg(feature = "custom_fonts")]
mod registry;
#[cfg(all(feature = "custom_fonts", feature = "pdf"))]
pub(crate) use registry::registered_sources;
#[cfg(feature = "custom_fonts")]
pub use registry::{
    family_styles, register_font_bytes, register_font_file, register_fonts_dir, registered_fonts,
    DirReport, FamilyStyles, FontError, FontInfo,
};
#[cfg(feature = "custom_fonts")]
pub(crate) use registry::{match_face, Face};
