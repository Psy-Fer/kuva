pub mod alluvial_order;
pub mod annotations;
pub mod axis;
pub mod brick_pop;
pub mod bw;
pub mod color;
pub mod coverage;
pub mod datetime;
pub mod figure;
pub mod layout;
// Inline-Unicode lowering for `$...$` math in labels. Zero-dep, used by all
// backends.
pub mod math;
pub mod palette;
pub mod plots;
pub mod projection;
#[allow(clippy::module_inception)]
pub mod render;
pub mod render_utils;
pub mod text_metrics;
pub mod theme;
pub mod track_stack;
