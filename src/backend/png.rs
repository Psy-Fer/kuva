use crate::backend::raster::RasterBackend;
use crate::render::render::Scene;

/// SVG-path-through-resvg backend, now delegating to the direct raster backend.
/// Kept for API compatibility; prefer [`RasterBackend`] directly.
pub struct PngBackend {
    pub scale: f32,
}

impl Default for PngBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl PngBackend {
    pub fn new() -> Self {
        Self { scale: 2.0 }
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn render_scene(&self, scene: &Scene) -> Result<Vec<u8>, String> {
        RasterBackend { scale: self.scale }.render_scene(scene)
    }
}
