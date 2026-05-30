pub mod alluvial_order;
pub mod annotations;
pub mod axis;
pub mod color;
pub mod datetime;
pub mod figure;
pub mod layout;
// Always compiled: the lookup tier (`to_unicode`) is zero-dep and used by all
// backends. The high-fidelity typst tier inside is `#[cfg(feature = "math")]`.
pub mod math;
pub mod palette;
pub mod plots;
pub mod projection;
#[allow(clippy::module_inception)]
pub mod render;
pub mod render_utils;
pub mod theme;
