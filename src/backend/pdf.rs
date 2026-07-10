use std::collections::HashMap;
use std::sync::Arc;

use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, TextStr};
use svg2pdf::usvg;

use crate::backend::svg::SvgBackend;
use crate::render::color::Color;
use crate::render::render::Scene;

/// Controls the page dimensions used when rendering a multi-page PDF with
/// [`PdfBackend::render_scenes`].
#[derive(Clone, Copy, Debug, Default)]
pub enum PageSize {
    /// Each page takes the natural pixel dimensions of its own scene, so pages
    /// may differ in size. This is the default.
    #[default]
    Natural,
    /// Every page is exactly `width` × `height` PDF points (1 pt = 1/72 inch).
    /// Each scene is scaled uniformly to fit inside the page — preserving its
    /// aspect ratio — and centered. Any leftover margin is filled with the
    /// scene's background color when it resolves to a solid RGB value (a hex
    /// code or a recognized named color); otherwise the margin is white.
    Fixed { width: f64, height: f64 },
}

impl PageSize {
    /// A fixed page size given in PDF points (1 pt = 1/72 inch).
    ///
    /// `width` and `height` must be finite and positive. Degenerate values are
    /// rejected at render time by [`PdfBackend::render_scenes`]; in debug builds
    /// they also trip an assertion here to surface the mistake at its source.
    pub fn points(width: f64, height: f64) -> Self {
        debug_assert!(
            width.is_finite() && width > 0.0 && height.is_finite() && height > 0.0,
            "PageSize dimensions must be finite and positive, got {width}×{height}"
        );
        Self::Fixed { width, height }
    }

    /// A fixed page size given in inches, e.g. `PageSize::inches(11.0, 8.5)` for
    /// US Letter in landscape orientation.
    pub fn inches(width: f64, height: f64) -> Self {
        Self::points(width * 72.0, height * 72.0)
    }
}

/// Vector PDF backend (requires feature `pdf`).
///
/// Single-scene output goes through [`svg2pdf::to_pdf`]; multi-page output
/// embeds each scene's SVG as a Form XObject via [`svg2pdf::to_chunk`] and
/// assembles the pages with `pdf-writer`.
pub struct PdfBackend {
    page_size: PageSize,
}

impl Default for PdfBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl PdfBackend {
    pub const fn new() -> Self {
        Self {
            page_size: PageSize::Natural,
        }
    }

    /// Set the page size used by [`render_scenes`](Self::render_scenes) for
    /// multi-page output. Defaults to [`PageSize::Natural`]. Single-scene
    /// [`render_scene`](Self::render_scene) always uses the scene's natural size.
    pub fn with_page_size(mut self, page_size: PageSize) -> Self {
        self.page_size = page_size;
        self
    }

    /// Render a single scene to a one-page PDF.
    pub fn render_scene(&self, scene: &Scene) -> Result<Vec<u8>, String> {
        let svg_str = SvgBackend::new().render_scene(scene);
        let options = usvg::Options {
            fontdb: Self::fontdb(),
            ..Default::default()
        };
        let tree = usvg::Tree::from_str(&svg_str, &options).map_err(|e| e.to_string())?;

        svg2pdf::to_pdf(
            &tree,
            svg2pdf::ConversionOptions::default(),
            svg2pdf::PageOptions::default(),
        )
        .map_err(|e| e.to_string())
    }

    /// Render one scene per page into a single multi-page PDF.
    ///
    /// Pages are laid out according to the backend's [`PageSize`]. Returns
    /// `Err` if `scenes` is empty or if any scene fails to convert.
    ///
    /// Note: fonts are subset and embedded per page, a limitation of the
    /// underlying `svg2pdf` conversion — documents with many pages will repeat
    /// the (compressed) font subset once per page.
    pub fn render_scenes(&self, scenes: &[Scene]) -> Result<Vec<u8>, String> {
        if scenes.is_empty() {
            return Err("at least one scene is required to render a PDF".to_string());
        }
        if let PageSize::Fixed { width, height } = self.page_size {
            if !(width.is_finite() && width > 0.0 && height.is_finite() && height > 0.0) {
                return Err(format!(
                    "PageSize::Fixed requires finite, positive dimensions, got {width}×{height}"
                ));
            }
        }

        // Preserve the exact, well-tested single-page output for the common
        // natural-sized single-scene case.
        if scenes.len() == 1 {
            if let PageSize::Natural = self.page_size {
                return self.render_scene(&scenes[0]);
            }
        }

        let options = usvg::Options {
            fontdb: Self::fontdb(),
            ..Default::default()
        };
        let conversion = svg2pdf::ConversionOptions::default();

        let mut pdf = Pdf::new();
        let mut alloc = Ref::new(1);
        let catalog_id = alloc.bump();
        let page_tree_id = alloc.bump();
        let mut page_ids: Vec<Ref> = Vec::with_capacity(scenes.len());

        for scene in scenes {
            let svg_str = SvgBackend::new().render_scene(scene);
            let tree = usvg::Tree::from_str(&svg_str, &options).map_err(|e| e.to_string())?;
            let (chunk, svg_ref) =
                svg2pdf::to_chunk(&tree, conversion).map_err(|e| e.to_string())?;

            // Renumber the SVG's objects into this document's ref space. The
            // chunk allocates from 1; map each to a fresh id past ours.
            let mut remap = HashMap::new();
            let chunk = chunk.renumber(|old| *remap.entry(old).or_insert_with(|| alloc.bump()));
            let svg_ref = remap[&svg_ref];
            pdf.extend(&chunk);

            let page_id = alloc.bump();
            let content_id = alloc.bump();
            page_ids.push(page_id);

            let (page_w, page_h, placement) = self.place(scene.width, scene.height);

            // Content stream: optional background fill (only when letterboxing a
            // fixed page), then place the SVG XObject.
            let svg_name = Name(b"S1");
            let mut content = Content::new();
            if let PageSize::Fixed { .. } = self.page_size {
                let [r, g, b] = page_background_rgb(scene);
                content
                    .set_fill_rgb(r, g, b)
                    .rect(0.0, 0.0, page_w as f32, page_h as f32)
                    .fill_nonzero();
            }
            content.transform(placement).x_object(svg_name);
            let content_data = content.finish();
            pdf.stream(content_id, &content_data);

            let mut page = pdf.page(page_id);
            page.media_box(Rect::new(0.0, 0.0, page_w as f32, page_h as f32));
            page.parent(page_tree_id);
            page.resources().x_objects().pair(svg_name, svg_ref);
            page.contents(content_id);
            page.finish();
        }

        pdf.catalog(catalog_id).pages(page_tree_id);
        pdf.pages(page_tree_id)
            .kids(page_ids.iter().copied())
            .count(page_ids.len() as i32);

        let info_id = alloc.bump();
        pdf.document_info(info_id).producer(TextStr("kuva"));

        Ok(pdf.finish())
    }

    /// Compute the page size (in points) and the content-stream placement
    /// matrix for a scene of the given natural dimensions.
    ///
    /// The XObject produced by [`svg2pdf::to_chunk`] is a 1×1-point unit, so the
    /// matrix both scales it to the drawn size and translates it into place.
    fn place(&self, scene_w: f64, scene_h: f64) -> (f64, f64, [f32; 6]) {
        match self.page_size {
            PageSize::Natural => (
                scene_w,
                scene_h,
                [scene_w as f32, 0.0, 0.0, scene_h as f32, 0.0, 0.0],
            ),
            PageSize::Fixed { width, height } => {
                // Uniform scale-to-fit, then center (letterbox).
                let scale = (width / scene_w).min(height / scene_h);
                let draw_w = scene_w * scale;
                let draw_h = scene_h * scale;
                let tx = (width - draw_w) / 2.0;
                let ty = (height - draw_h) / 2.0;
                (
                    width,
                    height,
                    [draw_w as f32, 0.0, 0.0, draw_h as f32, tx as f32, ty as f32],
                )
            }
        }
    }

    /// Build the font database used to parse kuva SVGs into `usvg` trees. Loads
    /// the bundled DejaVu variants (so text metrics match kuva's layout) plus
    /// any system fonts.
    fn fontdb() -> Arc<usvg::fontdb::Database> {
        let mut db = usvg::fontdb::Database::new();
        db.load_font_data(crate::fonts::dejavu_sans().to_vec());
        db.load_font_data(crate::fonts::dejavu_sans_bold().to_vec());
        db.load_font_data(crate::fonts::dejavu_sans_oblique().to_vec());
        db.load_font_data(crate::fonts::dejavu_sans_mono().to_vec());
        db.load_system_fonts();
        Arc::new(db)
    }
}

/// Resolve a scene's background color to RGB in `0.0..=1.0` for the page fill,
/// defaulting to white when the background is unset or not a solid color.
fn page_background_rgb(scene: &Scene) -> [f32; 3] {
    match scene.background_color.as_deref().map(Color::from) {
        Some(Color::Rgb(r, g, b)) => [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0],
        _ => [1.0, 1.0, 1.0],
    }
}

// Backward-compat shim in the value namespace (mirrors `SvgBackend`).
// `PdfBackend.render_scene(...)` resolves `PdfBackend` to this const;
// `PdfBackend::new()` / `PdfBackend { .. }` resolve it to the type.
// TODO: To phase out later: add #[deprecated(note = "Use PdfBackend::new()")] here.
#[allow(non_upper_case_globals)]
pub const PdfBackend: PdfBackend = PdfBackend::new();
