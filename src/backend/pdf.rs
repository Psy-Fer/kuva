use crate::render::render::Scene;
use crate::backend::svg::SvgBackend;

pub struct PdfBackend;

impl Default for PdfBackend {
    fn default() -> Self { Self::new() }
}

impl PdfBackend {
    pub fn new() -> Self {
        Self
    }

    pub fn render_scene(&self, scene: &Scene) -> Result<Vec<u8>, String> {
        let svg_str = SvgBackend.render_scene(scene);

        let mut fontdb = svg2pdf::usvg::fontdb::Database::new();
        fontdb.load_system_fonts();
        let options = svg2pdf::usvg::Options {
            fontdb: std::sync::Arc::new(fontdb),
            ..Default::default()
        };

        let tree = svg2pdf::usvg::Tree::from_str(&svg_str, &options)
            .map_err(|e| e.to_string())?;

        svg2pdf::to_pdf(
            &tree,
            svg2pdf::ConversionOptions::default(),
            svg2pdf::PageOptions::default(),
        )
        .map_err(|e| e.to_string())
    }
}
