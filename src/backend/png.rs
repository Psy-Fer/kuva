use crate::render::render::Scene;
use crate::backend::svg::SvgBackend;

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

        let mut fontdb = resvg::usvg::fontdb::Database::new();
        fontdb.load_system_fonts();
        let options = resvg::usvg::Options {
            fontdb: std::sync::Arc::new(fontdb),
            ..Default::default()
        };

        let tree = resvg::usvg::Tree::from_str(&svg_str, &options)
            .map_err(|e| e.to_string())?;

        let size = tree.size().to_int_size().scale_by(self.scale)
            .expect("canvas too large for the requested scale factor");
        let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
            .ok_or_else(|| "failed to allocate pixmap".to_string())?;

        let transform = resvg::tiny_skia::Transform::from_scale(self.scale, self.scale);
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        pixmap.encode_png().map_err(|e| e.to_string())
    }
}
