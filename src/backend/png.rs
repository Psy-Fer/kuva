use std::sync::{Arc, OnceLock};

use crate::render::render::Scene;
use crate::backend::svg::SvgBackend;

/// Cached font database shared across all PngBackend render calls.
/// Loading system fonts is expensive (100ms+); do it only once.
fn shared_fontdb() -> Arc<resvg::usvg::fontdb::Database> {
    static FONTDB: OnceLock<Arc<resvg::usvg::fontdb::Database>> = OnceLock::new();
    FONTDB.get_or_init(|| {
        let mut db = resvg::usvg::fontdb::Database::new();
        db.load_system_fonts();
        Arc::new(db)
    }).clone()
}

pub struct PngBackend {
    /// Pixel density multiplier.
    /// 1.0 = same logical pixel dimensions as the SVG.
    /// 2.0 = 2× / retina quality (default).
    pub scale: f32,
}

impl Default for PngBackend {
    fn default() -> Self { Self::new() }
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
        let svg_str = SvgBackend.render_scene(scene);

        let options = resvg::usvg::Options {
            fontdb: shared_fontdb(),
            ..Default::default()
        };

        let tree = resvg::usvg::Tree::from_str(&svg_str, &options)
            .map_err(|e| e.to_string())?;

        let size = tree.size().to_int_size().scale_by(self.scale)
            .ok_or_else(|| "canvas too large for the requested scale factor".to_string())?;
        let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
            .ok_or_else(|| "failed to allocate pixmap".to_string())?;

        let transform = resvg::tiny_skia::Transform::from_scale(self.scale, self.scale);
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        pixmap.encode_png().map_err(|e| e.to_string())
    }
}
