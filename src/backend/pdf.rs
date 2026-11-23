use std::sync::Arc;

use krilla::geom::{Size, Transform};
use krilla::page::PageSettings;
use krilla::Document;
use krilla_svg::{SurfaceExt, SvgSettings};

use crate::backend::svg::SvgBackend;
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
    /// scene's background color (white if unset).
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
/// Built on `krilla`/`krilla-svg` — the maintainer-endorsed successor to
/// `svg2pdf`, which was archived upstream as unmaintained (see CHANGELOG.md).
/// Each scene's SVG is embedded as vector content, not rasterized.
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

    /// Set the page size used for multi-page output via
    /// [`render_scenes`](Self::render_scenes). Defaults to [`PageSize::Natural`].
    /// Single-scene [`render_scene`](Self::render_scene) always uses the
    /// scene's natural size.
    pub fn with_page_size(mut self, page_size: PageSize) -> Self {
        self.page_size = page_size;
        self
    }

    /// Render a single scene to a one-page PDF.
    pub fn render_scene(&self, scene: &Scene) -> Result<Vec<u8>, String> {
        self.render_scenes(std::slice::from_ref(scene))
    }

    /// Render one scene per page into a single multi-page PDF.
    ///
    /// Pages are laid out according to the backend's [`PageSize`]. Returns
    /// `Err` if `scenes` is empty, if `PageSize::Fixed` has non-finite or
    /// non-positive dimensions, or if any scene fails to convert.
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

        let fontdb = Self::fontdb();
        let mut document = Document::new();
        // Stamp the kuva version into the PDF's document metadata for later issue triage.
        let stamp = format!("kuva {}", crate::VERSION);
        document.set_metadata(
            krilla::metadata::Metadata::new()
                .creator(stamp.clone())
                .producer(stamp),
        );

        for scene in scenes {
            let svg_str = SvgBackend::new().render_scene(scene);
            let opts = usvg_options(Arc::clone(&fontdb));
            let tree = usvg::Tree::from_str(&svg_str, &opts).map_err(|e| e.to_string())?;

            let (page_w, page_h, draw_w, draw_h, tx, ty) = self.place(scene.width, scene.height);
            let page_size = Size::from_wh(page_w as f32, page_h as f32)
                .ok_or_else(|| format!("invalid PDF page size {page_w}x{page_h}"))?;
            let draw_size = Size::from_wh(draw_w as f32, draw_h as f32)
                .ok_or_else(|| format!("invalid PDF draw size {draw_w}x{draw_h}"))?;

            let mut page = document.start_page_with(PageSettings::new(page_size));
            let mut surface = page.surface();

            if let PageSize::Fixed { .. } = self.page_size {
                // Letterbox background: a full-page filled rect built as its
                // own tiny SVG and drawn first, matching the scene's own
                // background color. Reuses `draw_svg` rather than needing
                // krilla's lower-level path/fill primitives for one rect.
                let bg_svg = background_svg(page_w, page_h, scene);
                let bg_tree = usvg::Tree::from_str(&bg_svg, &usvg_options(Arc::clone(&fontdb)))
                    .map_err(|e| e.to_string())?;
                surface.draw_svg(&bg_tree, page_size, SvgSettings::default());
            }

            surface.push_transform(&Transform::from_translate(tx as f32, ty as f32));
            surface.draw_svg(&tree, draw_size, SvgSettings::default());
            surface.pop();

            surface.finish();
            page.finish();
        }

        document.finish().map_err(|e| format!("{e:?}"))
    }

    /// Compute `(page_w, page_h, draw_w, draw_h, tx, ty)` in points for a
    /// scene of the given natural pixel dimensions (1 scene pixel = 1 PDF
    /// point, matching the previous single-page behavior).
    fn place(&self, scene_w: f64, scene_h: f64) -> (f64, f64, f64, f64, f64, f64) {
        match self.page_size {
            PageSize::Natural => (scene_w, scene_h, scene_w, scene_h, 0.0, 0.0),
            PageSize::Fixed { width, height } => {
                // Uniform scale-to-fit, then center (letterbox).
                let scale = (width / scene_w).min(height / scene_h);
                let draw_w = scene_w * scale;
                let draw_h = scene_h * scale;
                let tx = (width - draw_w) / 2.0;
                let ty = (height - draw_h) / 2.0;
                (width, height, draw_w, draw_h, tx, ty)
            }
        }
    }

    /// Build the font database used to parse kuva SVGs into `usvg` trees. Loads
    /// the fonts registered with `kuva::fonts` (feature `custom_fonts`), then the
    /// bundled DejaVu variants (so text metrics match kuva's layout), then any
    /// system fonts, and maps the generic `monospace` family to the bundled
    /// DejaVu Sans Mono so it renders the same on every system.
    ///
    /// Order matters: fontdb breaks ties between equally good faces by load
    /// order, so a registered family wins over a system font of the same name,
    /// and the PDF draws the face layout measured.
    ///
    /// Building it scans the system fonts, so it is built once per process and
    /// reused until another font is registered.
    pub(crate) fn fontdb() -> Arc<usvg::fontdb::Database> {
        static CACHE: std::sync::Mutex<Option<(u64, Arc<usvg::fontdb::Database>)>> =
            std::sync::Mutex::new(None);
        #[cfg(feature = "custom_fonts")]
        let generation = crate::fonts::generation();
        #[cfg(not(feature = "custom_fonts"))]
        let generation = 0;
        let mut cache = CACHE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some((built_at, db)) = cache.as_ref() {
            if *built_at == generation {
                return Arc::clone(db);
            }
        }
        let db = Self::build_fontdb();
        *cache = Some((generation, Arc::clone(&db)));
        db
    }

    fn build_fontdb() -> Arc<usvg::fontdb::Database> {
        let mut db = usvg::fontdb::Database::new();
        #[cfg(feature = "custom_fonts")]
        load_registered_fonts(&mut db);
        db.load_font_data(crate::fonts::dejavu_sans().to_vec());
        db.load_font_data(crate::fonts::dejavu_sans_bold().to_vec());
        db.load_font_data(crate::fonts::dejavu_sans_oblique().to_vec());
        db.load_font_data(crate::fonts::dejavu_sans_mono().to_vec());
        db.load_system_fonts();
        db.set_monospace_family("DejaVu Sans Mono");
        Arc::new(db)
    }
}

/// Font data shared with the registry, viewed as bytes for fontdb (which needs a
/// sized `AsRef<[u8]>` behind its `Arc`).
#[cfg(feature = "custom_fonts")]
struct SharedFontData(Arc<[u8]>);

#[cfg(feature = "custom_fonts")]
impl AsRef<[u8]> for SharedFontData {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Load every registered face. fontdb loads all faces of a collection, so faces
/// the registry no longer holds (replaced by a later registration) are removed.
#[cfg(feature = "custom_fonts")]
fn load_registered_fonts(db: &mut usvg::fontdb::Database) {
    use crate::fonts::FontSource;
    for (source, indices) in crate::fonts::registered_sources() {
        let source = match source {
            FontSource::File(path) => usvg::fontdb::Source::File(path),
            FontSource::Bytes(bytes) => {
                usvg::fontdb::Source::Binary(Arc::new(SharedFontData(bytes)))
            }
        };
        for id in db.load_font_source(source) {
            let keep = db
                .face(id)
                .is_some_and(|face| indices.contains(&face.index));
            if !keep {
                db.remove_face(id);
            }
        }
    }
}

/// Parse options for kuva's own SVG. `<image>` hrefs that name a file or
/// URL are never loaded: kuva only emits inline `data:` images, so the
/// default resolver (which reads local files) would only ever serve
/// injected markup.
fn usvg_options(fontdb: Arc<usvg::fontdb::Database>) -> usvg::Options<'static> {
    let mut opts = usvg::Options {
        fontdb,
        ..Default::default()
    };
    opts.image_href_resolver.resolve_string = Box::new(|_, _| None);
    opts
}

/// A minimal one-rect SVG covering `w`×`h`, filled with `scene`'s background
/// color (or white if unset), the letterbox fill for `PageSize::Fixed`.
/// The colour is caller-supplied, so it is attribute-escaped.
fn background_svg(w: f64, h: f64, scene: &Scene) -> String {
    let color = crate::render::render_utils::escape_attr(
        scene.background_color.as_deref().unwrap_or("white"),
    );
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}"><rect width="100%" height="100%" fill="{color}"/></svg>"#
    )
}

#[cfg(all(test, feature = "custom_fonts"))]
mod font_tests {
    use super::*;

    #[test]
    fn font_database_is_reused_until_a_font_is_registered() {
        // Other tests register fonts concurrently: only compare two builds when
        // the registry didn't change between them.
        let before = crate::fonts::generation();
        let a = PdfBackend::fontdb();
        let b = PdfBackend::fontdb();
        if crate::fonts::generation() == before {
            assert!(
                Arc::ptr_eq(&a, &b),
                "unchanged registry reuses the database"
            );
        }
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/fonts/DejaVuSerif-Bold.ttf"
        );
        crate::fonts::register_font_file(path).unwrap();
        let c = PdfBackend::fontdb();
        assert!(!Arc::ptr_eq(&a, &c), "registering rebuilds the database");
        assert!(c
            .faces()
            .any(|f| f.families.iter().any(|(name, _)| name == "DejaVu Serif")));
    }

    #[test]
    fn registered_fonts_come_first_in_the_font_database() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/fonts/DejaVuSerif.ttf"
        );
        crate::fonts::register_font_file(path).unwrap();
        let db = PdfBackend::fontdb();
        // The first DejaVu Serif face fontdb sees is the registered one, ahead of
        // any system-installed copy, so ties resolve to it. A font registered from
        // a path reaches fontdb as that file; one registered from bytes (another
        // test in this process may do that) as in-memory data.
        let first = db
            .faces()
            .find(|f| f.families.iter().any(|(name, _)| name == "DejaVu Serif"))
            .expect("registered family is in the database");
        let registered = match &first.source {
            usvg::fontdb::Source::Binary(_) => true,
            usvg::fontdb::Source::File(p) | usvg::fontdb::Source::SharedFile(p, _) => {
                p == std::path::Path::new(path)
            }
        };
        assert!(
            registered,
            "first DejaVu Serif face should be the registered font"
        );
    }
}

#[cfg(test)]
mod injection_tests {
    use super::*;

    fn count(group: &usvg::Group, images: &mut usize, paths: &mut usize) {
        for node in group.children() {
            match node {
                usvg::Node::Group(g) => count(g, images, paths),
                usvg::Node::Image(_) => *images += 1,
                usvg::Node::Path(_) => *paths += 1,
                usvg::Node::Text(_) => {}
            }
        }
    }

    fn png_path() -> String {
        format!(
            "{}/docs/src/assets/overview/all_plots_complex.png",
            env!("CARGO_MANIFEST_DIR")
        )
    }

    #[test]
    fn hostile_background_colour_stays_inside_the_fill_attribute() {
        let mut scene = Scene::new(100.0, 100.0);
        scene.background_color = Some(format!(
            r#"red"/><image href="{}" width="50" height="50"/><rect fill="blue"#,
            png_path()
        ));
        let svg = background_svg(100.0, 100.0, &scene);
        let tree = usvg::Tree::from_str(&svg, &usvg_options(PdfBackend::fontdb())).unwrap();
        let (mut images, mut paths) = (0, 0);
        count(tree.root(), &mut images, &mut paths);
        assert_eq!(images, 0);
        assert!(
            paths <= 1,
            "expected at most the one background rect, got {paths}"
        );
    }

    #[test]
    fn image_hrefs_to_local_files_are_not_loaded() {
        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><image href="{}" width="50" height="50"/></svg>"#,
            png_path()
        );
        // The default resolver does read the file, which is what this guards against.
        let default_tree = usvg::Tree::from_str(&svg, &usvg::Options::default()).unwrap();
        let (mut images, mut paths) = (0, 0);
        count(default_tree.root(), &mut images, &mut paths);
        assert_eq!(images, 1, "default usvg options should load the file");

        let tree = usvg::Tree::from_str(&svg, &usvg_options(PdfBackend::fontdb())).unwrap();
        let (mut images, mut paths) = (0, 0);
        count(tree.root(), &mut images, &mut paths);
        assert_eq!(images, 0);
    }
}
