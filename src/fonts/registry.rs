//! The process-wide registry of fonts loaded at runtime (feature `custom_fonts`).
//!
//! Each face is parsed once, when it is registered: its family, weight and style
//! are read from the font's own tables, and its text metrics are measured into a
//! [`FontMetrics`] that layout then only looks up. Faces are matched to a requested
//! style with the CSS font-matching rules, the same rules browsers and fontdb use,
//! so every backend lands on the face layout measured.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, PoisonError, RwLock};

use crate::render::text_metrics::FontMetrics;

/// What kuva read from a registered font face.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct FontInfo {
    /// Family name, the name to pass to `with_font_family`. Read from the
    /// typographic family name when the font has one, so `Inter-Bold.ttf` is
    /// family `Inter`, not `Inter Bold`.
    pub family: String,
    /// Weight class, 100 (thin) to 900 (black). Regular is 400, bold 700.
    pub weight: u16,
    /// Italic or oblique.
    pub italic: bool,
    /// Every glyph has the same advance.
    pub monospace: bool,
    /// A variable font. Only its default instance is used: other weights in the
    /// same file are not available, so register the static files instead.
    pub variable: bool,
    /// The font's licence flags forbid embedding it in documents (OS/2 `fsType`
    /// "restricted license"). Embedding it in SVG or PDF output may break its licence.
    pub restricted_embedding: bool,
}

/// A registered face: what it is, its bytes (for the PNG and PDF backends) and
/// its measured metrics.
pub(crate) struct Face {
    pub(crate) info: FontInfo,
    // Read by the PDF backend and SVG embedding (#134 steps 5-6).
    #[allow(dead_code)]
    pub(crate) bytes: Arc<[u8]>,
    #[allow(dead_code)]
    pub(crate) index: u32,
    pub(crate) metrics: Arc<FontMetrics>,
    /// The face parsed for the PNG rasteriser, built on first use. `None` when
    /// fontdue cannot read a face ttf-parser could (it then draws as DejaVu).
    #[cfg(feature = "png")]
    raster: OnceLock<Option<fontdue::Font>>,
}

impl fmt::Debug for Face {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Face")
            .field("info", &self.info)
            .field("index", &self.index)
            .field("bytes", &self.bytes.len())
            .finish_non_exhaustive()
    }
}

impl Face {
    /// The face as a fontdue font for the PNG backend, parsed once.
    #[cfg(feature = "png")]
    pub(crate) fn raster_font(&self) -> Option<&fontdue::Font> {
        self.raster
            .get_or_init(|| {
                fontdue::Font::from_bytes(
                    &self.bytes[..],
                    fontdue::FontSettings {
                        collection_index: self.index,
                        ..fontdue::FontSettings::default()
                    },
                )
                .ok()
            })
            .as_ref()
    }
}

/// Why a font could not be registered.
#[derive(Debug)]
#[non_exhaustive]
pub enum FontError {
    /// The file or directory could not be read.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// The data is not a font kuva can read (TTF, OTF, TTC and OTC are supported).
    Parse { origin: String, reason: String },
    /// The font has no family name, so it could never be selected.
    NoFamilyName { origin: String },
    /// A directory held no font that could be registered. `skipped` lists the font
    /// files that failed and why.
    NoFonts {
        path: PathBuf,
        skipped: Vec<(PathBuf, FontError)>,
    },
}

impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FontError::Io { path, source } => {
                write!(f, "cannot read font {}: {source}", path.display())
            }
            FontError::Parse { origin, reason } => {
                write!(f, "{origin} is not a font kuva can read: {reason}")
            }
            FontError::NoFamilyName { origin } => {
                write!(f, "{origin} has no family name, so it cannot be selected")
            }
            FontError::NoFonts { path, skipped } => {
                write!(f, "no usable fonts in {}", path.display())?;
                if !skipped.is_empty() {
                    write!(f, " ({} font file(s) could not be read)", skipped.len())?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for FontError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FontError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// The result of [`register_fonts_dir`]: the faces registered, and the font files
/// that were skipped because they could not be read.
#[derive(Debug)]
#[non_exhaustive]
pub struct DirReport {
    pub registered: Vec<FontInfo>,
    pub skipped: Vec<(PathBuf, FontError)>,
}

/// The face each style of a family resolves to. See [`family_styles`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct FamilyStyles {
    pub regular: FontInfo,
    pub bold: FontInfo,
    pub italic: FontInfo,
    pub bold_italic: FontInfo,
}

impl FamilyStyles {
    /// The styles this family has no real face for, by name (`"regular"`, `"bold"`,
    /// `"italic"`, `"bold italic"`). kuva never synthesises a missing style: text in
    /// it uses the nearest face instead (usually the regular one), in every backend.
    pub fn missing(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if self.regular.italic || self.regular.weight >= BOLD_THRESHOLD {
            out.push("regular");
        }
        if self.bold.weight < BOLD_THRESHOLD {
            out.push("bold");
        }
        if !self.italic.italic {
            out.push("italic");
        }
        if self.bold_italic.weight < BOLD_THRESHOLD || !self.bold_italic.italic {
            out.push("bold italic");
        }
        out
    }
}

/// Weight at and above which a face counts as bold.
const BOLD_THRESHOLD: u16 = 600;
const REGULAR_WEIGHT: u16 = 400;
const BOLD_WEIGHT: u16 = 700;

fn registry() -> &'static RwLock<Vec<Arc<Face>>> {
    static REGISTRY: OnceLock<RwLock<Vec<Arc<Face>>>> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(Vec::new()))
}

/// Register every face in a font file (TTF, OTF, or a TTC/OTC collection).
/// Returns what was registered.
pub fn register_font_file(path: impl AsRef<Path>) -> Result<Vec<FontInfo>, FontError> {
    let path = path.as_ref();
    let bytes = std::fs::read(path).map_err(|source| FontError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    register(bytes.into(), &path.display().to_string())
}

/// Register every face in font data already in memory (for example a font
/// fetched by a web page).
pub fn register_font_bytes(bytes: Vec<u8>) -> Result<Vec<FontInfo>, FontError> {
    register(bytes.into(), "font data")
}

/// Register every font file (`.ttf`, `.otf`, `.ttc`, `.otc`) in a directory and its
/// subdirectories, in path order. Symlinked directories are not followed.
///
/// A file that fails to parse is skipped and listed in [`DirReport::skipped`]. It is
/// an error only when the directory cannot be read or nothing in it registered.
pub fn register_fonts_dir(path: impl AsRef<Path>) -> Result<DirReport, FontError> {
    let path = path.as_ref();
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    collect_font_files(path, &mut files, &mut skipped).map_err(|source| FontError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    files.sort();

    let mut registered = Vec::new();
    for file in files {
        match register_font_file(&file) {
            Ok(infos) => registered.extend(infos),
            Err(e) => skipped.push((file, e)),
        }
    }
    if registered.is_empty() {
        return Err(FontError::NoFonts {
            path: path.to_path_buf(),
            skipped,
        });
    }
    Ok(DirReport {
        registered,
        skipped,
    })
}

/// Everything registered so far, in registration order (replaced faces excluded).
pub fn registered_fonts() -> Vec<FontInfo> {
    read_registry().iter().map(|f| f.info.clone()).collect()
}

/// The face each of kuva's four text styles resolves to in a registered family
/// (matched case-insensitively), or `None` when no face of that family is
/// registered. Use [`FamilyStyles::missing`] to warn about styles the family lacks.
pub fn family_styles(family: &str) -> Option<FamilyStyles> {
    let pick = |weight, italic| match_face(family, weight, italic).map(|f| f.info.clone());
    Some(FamilyStyles {
        regular: pick(REGULAR_WEIGHT, false)?,
        bold: pick(BOLD_WEIGHT, false)?,
        italic: pick(REGULAR_WEIGHT, true)?,
        bold_italic: pick(BOLD_WEIGHT, true)?,
    })
}

/// The registered face of `family` that CSS font matching picks for `weight` and
/// `italic`, or `None` when the family is not registered.
///
/// Style first: faces with the requested italic flag, or the others when there are
/// none (italic and oblique count as one). Then weight, by CSS's nearest-weight rule.
pub(crate) fn match_face(family: &str, weight: u16, italic: bool) -> Option<Arc<Face>> {
    let reg = read_registry();
    let in_family: Vec<&Arc<Face>> = reg
        .iter()
        .filter(|f| same_family(&f.info.family, family))
        .collect();
    let styled: Vec<&Arc<Face>> = in_family
        .iter()
        .copied()
        .filter(|f| f.info.italic == italic)
        .collect();
    let pool = if styled.is_empty() { in_family } else { styled };
    let best = closest_weight(pool.iter().map(|f| f.info.weight), weight)?;
    Some(Arc::clone(pool[best]))
}

/// Index of the weight CSS font matching picks for `desired`
/// (CSS Fonts 4, "font-weight" step of the matching algorithm):
/// - desired 400 to 500: weights from desired up to 500, then lighter (nearest
///   first), then heavier than 500 (nearest first);
/// - desired below 400: lighter or equal (nearest first), then heavier;
/// - desired above 500: heavier or equal (nearest first), then lighter.
fn closest_weight(weights: impl Iterator<Item = u16>, desired: u16) -> Option<usize> {
    let rank = |w: u16| -> (u8, u16) {
        if (400..=500).contains(&desired) {
            if (desired..=500).contains(&w) {
                (0, w - desired)
            } else if w < desired {
                (1, desired - w)
            } else {
                (2, w - desired)
            }
        } else if desired < 400 {
            if w <= desired {
                (0, desired - w)
            } else {
                (1, w - desired)
            }
        } else if w >= desired {
            (0, w - desired)
        } else {
            (1, desired - w)
        }
    };
    weights
        .enumerate()
        .min_by_key(|&(i, w)| (rank(w), i))
        .map(|(i, _)| i)
}

fn same_family(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}

fn read_registry() -> std::sync::RwLockReadGuard<'static, Vec<Arc<Face>>> {
    registry().read().unwrap_or_else(PoisonError::into_inner)
}

fn register(data: Arc<[u8]>, origin: &str) -> Result<Vec<FontInfo>, FontError> {
    // Parse every face before touching the registry, so a collection with one bad
    // face registers nothing.
    let faces = parse_faces(&data, origin)?;
    let infos = faces.iter().map(|f| f.info.clone()).collect();
    let mut reg = registry().write().unwrap_or_else(PoisonError::into_inner);
    for face in faces {
        reg.retain(|old| {
            !(same_family(&old.info.family, &face.info.family)
                && old.info.weight == face.info.weight
                && old.info.italic == face.info.italic)
        });
        reg.push(Arc::new(face));
    }
    Ok(infos)
}

fn parse_faces(data: &Arc<[u8]>, origin: &str) -> Result<Vec<Face>, FontError> {
    let parse_err = |reason: String| FontError::Parse {
        origin: origin.to_string(),
        reason,
    };
    let count = ttf_parser::fonts_in_collection(data).unwrap_or(1);
    if count == 0 {
        return Err(parse_err("the font collection is empty".into()));
    }
    let mut faces = Vec::with_capacity(count as usize);
    for index in 0..count {
        let face = ttf_parser::Face::parse(data, index).map_err(|e| parse_err(e.to_string()))?;
        let family = family_name(&face).ok_or_else(|| FontError::NoFamilyName {
            origin: origin.to_string(),
        })?;
        let info = FontInfo {
            family,
            weight: face.weight().to_number(),
            italic: face.is_italic() || face.is_oblique() || face.italic_angle() != 0.0,
            monospace: face.is_monospaced(),
            variable: face.is_variable(),
            restricted_embedding: matches!(
                face.permissions(),
                Some(ttf_parser::Permissions::Restricted)
            ),
        };
        faces.push(Face {
            info,
            bytes: Arc::clone(data),
            index,
            metrics: Arc::new(FontMetrics::from_face(&face)),
            #[cfg(feature = "png")]
            raster: OnceLock::new(),
        });
    }
    Ok(faces)
}

/// The typographic family name (name ID 16), else the family name (ID 1),
/// preferring the US English record.
fn family_name(face: &ttf_parser::Face) -> Option<String> {
    use ttf_parser::name_id;
    for id in [name_id::TYPOGRAPHIC_FAMILY, name_id::FAMILY] {
        let mut first = None;
        for name in face.names() {
            if name.name_id != id {
                continue;
            }
            let Some(text) = name.to_string() else {
                continue;
            };
            let text = text.trim().to_string();
            if text.is_empty() {
                continue;
            }
            if name.language() == ttf_parser::Language::English_UnitedStates {
                return Some(text);
            }
            first.get_or_insert(text);
        }
        if first.is_some() {
            return first;
        }
    }
    None
}

fn is_font_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            matches!(
                e.to_ascii_lowercase().as_str(),
                "ttf" | "otf" | "ttc" | "otc"
            )
        })
        .unwrap_or(false)
}

/// Collect font files under `dir`. An unreadable subdirectory is recorded in
/// `skipped`; only an unreadable `dir` itself is an error.
fn collect_font_files(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    skipped: &mut Vec<(PathBuf, FontError)>,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if let Err(source) = collect_font_files(&path, files, skipped) {
                skipped.push((
                    path.clone(),
                    FontError::Io {
                        path: path.clone(),
                        source,
                    },
                ));
            }
        } else if (file_type.is_file() || (file_type.is_symlink() && path.is_file()))
            && is_font_file(&path)
        {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pick(weights: &[u16], desired: u16) -> u16 {
        weights[closest_weight(weights.iter().copied(), desired).unwrap()]
    }

    #[test]
    fn regular_prefers_400_to_500_then_lighter_then_heavier() {
        assert_eq!(pick(&[300, 400, 700], 400), 400);
        assert_eq!(pick(&[300, 500, 700], 400), 500);
        assert_eq!(pick(&[200, 300, 700], 400), 300);
        assert_eq!(pick(&[600, 900], 400), 600);
    }

    #[test]
    fn bold_prefers_heavier_then_lighter() {
        assert_eq!(pick(&[400, 700, 900], 700), 700);
        assert_eq!(pick(&[400, 800, 900], 700), 800);
        assert_eq!(pick(&[300, 400, 600], 700), 600);
        assert_eq!(pick(&[400], 700), 400);
    }

    #[test]
    fn light_and_black_only_family() {
        // Regular text gets Light, bold gets Black: the case from the design notes.
        assert_eq!(pick(&[300, 900], 400), 300);
        assert_eq!(pick(&[300, 900], 700), 900);
    }

    #[test]
    fn light_request_prefers_lighter() {
        assert_eq!(pick(&[100, 400], 300), 100);
        assert_eq!(pick(&[400, 500], 300), 400);
    }

    #[test]
    fn empty_pool_has_no_match() {
        assert_eq!(closest_weight(std::iter::empty(), 400), None);
    }

    #[test]
    fn family_names_compare_case_insensitively() {
        assert!(same_family("DejaVu Serif", " dejavu serif "));
        assert!(!same_family("DejaVu Serif", "DejaVu Sans"));
    }

    #[test]
    fn font_file_extensions() {
        assert!(is_font_file(Path::new("a/Inter-Bold.TTF")));
        assert!(is_font_file(Path::new("x.otc")));
        assert!(!is_font_file(Path::new("LICENSE")));
        assert!(!is_font_file(Path::new("font.woff2")));
    }

    #[test]
    fn missing_lists_styles_without_a_real_face() {
        let face = |weight, italic| FontInfo {
            family: "X".into(),
            weight,
            italic,
            monospace: false,
            variable: false,
            restricted_embedding: false,
        };
        let only_regular = FamilyStyles {
            regular: face(400, false),
            bold: face(400, false),
            italic: face(400, false),
            bold_italic: face(400, false),
        };
        assert_eq!(only_regular.missing(), ["bold", "italic", "bold italic"]);
        let full = FamilyStyles {
            regular: face(400, false),
            bold: face(700, false),
            italic: face(400, true),
            bold_italic: face(700, true),
        };
        assert!(full.missing().is_empty());
    }
}
