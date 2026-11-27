//! The process-wide registry of fonts loaded at runtime (feature `custom_fonts`).
//!
//! Registering a face reads its family, weight and style from the font's own
//! tables; for a font file, only those few tables are read. The whole file is
//! read, and the face's text metrics measured into a [`FontMetrics`] that layout
//! then only looks up, the first time the face is used, so registering a
//! directory of hundreds of fonts stays cheap. Faces are matched to a requested
//! style with the CSS font-matching rules, the same rules browsers and fontdb use,
//! so every backend lands on the face layout measured.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, PoisonError, RwLock};

use crate::render::text_metrics::{FontMetrics, FontStyle};

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
    /// Width class, 1 (ultra-condensed) to 9 (ultra-expanded). Normal is 5.
    pub stretch: u16,
    /// Every glyph has the same advance.
    pub monospace: bool,
    /// A variable font. Only its default instance is used: other weights in the
    /// same file are not available, so register the static files instead.
    pub variable: bool,
    /// The font's licence flags forbid embedding it in documents (OS/2 `fsType`
    /// "restricted license"). Embedding it in SVG or PDF output may break its licence.
    pub restricted_embedding: bool,
}

/// A registered font file's data. A font registered from a path is read in full
/// only when one of its faces is first measured, drawn or embedded.
pub(crate) struct FontFile {
    path: Option<PathBuf>,
    bytes: OnceLock<Option<Arc<[u8]>>>,
}

impl FontFile {
    fn in_memory(bytes: Arc<[u8]>) -> Self {
        FontFile {
            path: None,
            bytes: OnceLock::from(Some(bytes)),
        }
    }

    fn on_disk(path: PathBuf) -> Self {
        FontFile {
            path: Some(path),
            bytes: OnceLock::new(),
        }
    }

    /// The file's bytes, read on first use. `None` when the file can no longer be
    /// read (moved or deleted since it was registered); its faces then draw as
    /// the bundled DejaVu Sans.
    fn bytes(&self) -> Option<&Arc<[u8]>> {
        self.bytes
            .get_or_init(|| {
                let path = self.path.as_ref()?;
                match std::fs::read(path) {
                    Ok(bytes) => Some(bytes.into()),
                    Err(e) => {
                        eprintln!(
                            "kuva: warning: registered font {} could not be read ({e}); \
                             its text is drawn in DejaVu Sans",
                            path.display()
                        );
                        None
                    }
                }
            })
            .as_ref()
    }
}

/// A registered face: what it is, its file (for the PNG and PDF backends and SVG
/// embedding) and its measured metrics.
pub(crate) struct Face {
    pub(crate) info: FontInfo,
    file: Arc<FontFile>,
    #[allow(dead_code)]
    pub(crate) index: u32,
    /// Measured on first use, not at registration: building the advance table
    /// walks the whole cmap, which made registering a directory of hundreds of
    /// faces take a second when most of them are never drawn.
    metrics: OnceLock<Arc<FontMetrics>>,
    /// The face parsed for the PNG rasteriser, built on first use. `None` when
    /// fontdue cannot read a face ttf-parser could (it then draws as DejaVu).
    #[cfg(feature = "png")]
    raster: OnceLock<Option<fontdue::Font>>,
    /// The `@font-face` rule embedding this face in SVG output, built on first use.
    #[cfg(feature = "embed_font")]
    css: OnceLock<Option<String>>,
}

impl fmt::Debug for Face {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Face")
            .field("info", &self.info)
            .field("index", &self.index)
            .field("path", &self.file.path)
            .finish_non_exhaustive()
    }
}

impl Face {
    /// The font file's bytes, read on first use (see [`FontFile::bytes`]).
    fn bytes(&self) -> Option<&Arc<[u8]>> {
        self.file.bytes()
    }

    /// This face's text metrics, measured once on first use. A face whose file
    /// can no longer be read or parsed measures as the bundled DejaVu Sans, which
    /// is what it then draws as.
    pub(crate) fn metrics(&self) -> &Arc<FontMetrics> {
        self.metrics.get_or_init(|| {
            self.bytes()
                .and_then(|bytes| ttf_parser::Face::parse(bytes, self.index).ok())
                .map(|face| Arc::new(FontMetrics::from_face(&face)))
                .unwrap_or_else(|| Arc::clone(FontMetrics::builtin(FontStyle::Regular)))
        })
    }

    /// The `@font-face` rule that embeds this face in an SVG (base64 data URL),
    /// built once. `None` for a face inside a collection (TTC/OTC): a web font
    /// cannot pick one face out of a collection file.
    #[cfg(feature = "embed_font")]
    pub(crate) fn font_face_css(&self) -> Option<&str> {
        self.css
            .get_or_init(|| {
                let bytes = self.bytes()?;
                if ttf_parser::fonts_in_collection(bytes).is_some() {
                    return None;
                }
                let (mime, format) = if bytes.starts_with(b"OTTO") {
                    ("font/otf", "opentype")
                } else {
                    ("font/ttf", "truetype")
                };
                let stretch = match self.info.stretch {
                    1 => "50%",
                    2 => "62.5%",
                    3 => "75%",
                    4 => "87.5%",
                    6 => "112.5%",
                    7 => "125%",
                    8 => "150%",
                    9 => "200%",
                    _ => "100%",
                };
                Some(format!(
                    "@font-face{{font-family:'{family}';src:url('data:{mime};base64,{data}') \
                     format('{format}');font-weight:{weight};font-style:{style};\
                     font-stretch:{stretch};}}",
                    family = css_string(&self.info.family),
                    data = crate::encoding::base64_encode(bytes),
                    weight = self.info.weight,
                    style = if self.info.italic { "italic" } else { "normal" },
                ))
            })
            .as_deref()
    }

    /// The face as a fontdue font for the PNG backend, parsed once.
    #[cfg(feature = "png")]
    pub(crate) fn raster_font(&self) -> Option<&fontdue::Font> {
        self.raster
            .get_or_init(|| {
                fontdue::Font::from_bytes(
                    &self.bytes()?[..],
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

/// Bumped on every registration, so a backend can cache what it built from the
/// registry (the PDF font database) until the registry changes.
static GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The registry's current generation (see [`GENERATION`]).
#[cfg_attr(not(feature = "pdf"), allow(dead_code))]
pub(crate) fn generation() -> u64 {
    GENERATION.load(std::sync::atomic::Ordering::Acquire)
}

fn registry() -> &'static RwLock<Vec<Arc<Face>>> {
    static REGISTRY: OnceLock<RwLock<Vec<Arc<Face>>>> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(Vec::new()))
}

/// Register every face in a font file (TTF, OTF, or a TTC/OTC collection).
/// Returns what was registered.
///
/// A collection may hold at most [`MAX_COLLECTION_FACES`] faces, each listed
/// once; otherwise nothing is registered and the error says why. A family
/// name record longer than [`MAX_FAMILY_NAME_CHARS`] is not used.
pub fn register_font_file(path: impl AsRef<Path>) -> Result<Vec<FontInfo>, FontError> {
    let path = path.as_ref();
    // Read only the tables registration needs; the rest of the file is read the
    // first time a face is used.
    if let Some(infos) = read_face_infos(path) {
        return Ok(insert(
            Arc::new(FontFile::on_disk(path.to_path_buf())),
            infos,
        ));
    }
    // Anything unusual: read and parse the whole file, which also gives the
    // proper error.
    let bytes = std::fs::read(path).map_err(|source| FontError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    register(bytes.into(), &path.display().to_string())
}

/// Register every face in font data already in memory (for example a font
/// fetched by a web page). The same limits as [`register_font_file`] apply.
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

/// Where a registered font file's data comes from, for backends that load
/// fonts into their own database (PDF).
#[cfg_attr(not(feature = "pdf"), allow(dead_code))]
pub(crate) enum FontSource {
    /// Registered from a path: the backend reads (or maps) the file itself, so
    /// kuva doesn't have to read every registered file to make a PDF.
    File(PathBuf),
    /// Registered from bytes in memory.
    Bytes(Arc<[u8]>),
}

/// Every registered font file, once each, with the face indices still
/// registered from it (a face replaced by a later registration is left out).
/// For backends that load fonts into their own database (PDF).
#[cfg_attr(not(feature = "pdf"), allow(dead_code))]
pub(crate) fn registered_sources() -> Vec<(FontSource, Vec<u32>)> {
    let mut files: Vec<(Arc<FontFile>, Vec<u32>)> = Vec::new();
    for face in read_registry().iter() {
        match files
            .iter_mut()
            .find(|(file, _)| Arc::ptr_eq(file, &face.file))
        {
            Some((_, indices)) => indices.push(face.index),
            None => files.push((Arc::clone(&face.file), vec![face.index])),
        }
    }
    files
        .into_iter()
        .filter_map(|(file, indices)| {
            let source = match &file.path {
                Some(path) => FontSource::File(path.clone()),
                None => FontSource::Bytes(Arc::clone(file.bytes()?)),
            };
            Some((source, indices))
        })
        .collect()
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
/// The CSS order, the same one fontdb (the PDF backend) applies: width first (kuva
/// always asks for normal width), then style (faces with the requested italic
/// flag, or the others when there are none; italic and oblique count as one), then
/// weight by CSS's nearest-weight rule.
pub(crate) fn match_face(family: &str, weight: u16, italic: bool) -> Option<Arc<Face>> {
    let reg = read_registry();
    let in_family: Vec<&Arc<Face>> = reg
        .iter()
        .filter(|f| same_family(&f.info.family, family))
        .collect();
    let width = closest_stretch(in_family.iter().map(|f| f.info.stretch))?;
    let in_family: Vec<&Arc<Face>> = in_family
        .into_iter()
        .filter(|f| f.info.stretch == width)
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

/// The width CSS font matching picks for a normal-width request: normal (5) when
/// present, else the nearest narrower, else the nearest wider.
fn closest_stretch(widths: impl Iterator<Item = u16>) -> Option<u16> {
    const NORMAL: u16 = 5;
    let widths: Vec<u16> = widths.collect();
    if widths.contains(&NORMAL) {
        return Some(NORMAL);
    }
    widths
        .iter()
        .copied()
        .filter(|&w| w < NORMAL)
        .max()
        .or_else(|| widths.iter().copied().filter(|&w| w > NORMAL).min())
}

/// A font family name made safe inside a single-quoted CSS string in an SVG
/// `<style>` block. The name comes from the font file, so anything beyond
/// letters, digits, space, `-`, `_` and `.` is written as a CSS hex escape: no
/// quote, backslash, `<` or `&` reaches the output.
#[cfg(feature = "embed_font")]
fn css_string(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.') {
            out.push(c);
        } else {
            out.push_str(&format!("\\{:x} ", c as u32));
        }
    }
    out
}

fn same_family(a: &str, b: &str) -> bool {
    // Registration runs this against every face already registered, so ASCII
    // names (nearly all of them) compare without allocating.
    let lower = |s: &str| {
        s.trim()
            .chars()
            .flat_map(char::to_lowercase)
            .collect::<Vec<_>>()
    };
    if a.is_ascii() && b.is_ascii() {
        a.trim().eq_ignore_ascii_case(b.trim())
    } else {
        lower(a) == lower(b)
    }
}

fn read_registry() -> std::sync::RwLockReadGuard<'static, Vec<Arc<Face>>> {
    registry().read().unwrap_or_else(PoisonError::into_inner)
}

fn register(data: Arc<[u8]>, origin: &str) -> Result<Vec<FontInfo>, FontError> {
    // Parse every face before touching the registry, so a collection with one bad
    // face registers nothing.
    let infos = parse_faces(&data, origin)?;
    Ok(insert(Arc::new(FontFile::in_memory(data)), infos))
}

/// Add the faces of one file (face `i` of the file is `infos[i]`), replacing any
/// registered face with the same family, weight, style and stretch.
fn insert(file: Arc<FontFile>, infos: Vec<FontInfo>) -> Vec<FontInfo> {
    let mut reg = registry().write().unwrap_or_else(PoisonError::into_inner);
    for (index, info) in (0u32..).zip(&infos) {
        reg.retain(|old| {
            !(same_family(&old.info.family, &info.family)
                && old.info.weight == info.weight
                && old.info.italic == info.italic
                && old.info.stretch == info.stretch)
        });
        reg.push(Arc::new(Face {
            info: info.clone(),
            file: Arc::clone(&file),
            index,
            metrics: OnceLock::new(),
            #[cfg(feature = "png")]
            raster: OnceLock::new(),
            #[cfg(feature = "embed_font")]
            css: OnceLock::new(),
        }));
    }
    // Still under the write lock, so a reader that sees the new generation
    // also sees the new faces.
    GENERATION.fetch_add(1, std::sync::atomic::Ordering::Release);
    infos
}

fn parse_faces(data: &[u8], origin: &str) -> Result<Vec<FontInfo>, FontError> {
    let parse_err = |reason: String| FontError::Parse {
        origin: origin.to_string(),
        reason,
    };
    let count = ttf_parser::fonts_in_collection(data).unwrap_or(1);
    if count == 0 {
        return Err(parse_err("the font collection is empty".into()));
    }
    // The count comes from the file, and each face needs a 4-byte offset after
    // the 12-byte header: a tiny file claiming billions of faces would otherwise
    // reserve memory for all of them (and trips a debug assertion in ttf-parser).
    if u64::from(count) > (data.len().saturating_sub(12) / 4) as u64 {
        return Err(parse_err(
            "the collection header claims more faces than the file holds".into(),
        ));
    }
    if data.starts_with(b"ttcf") {
        let offsets: Vec<u32> = (0..count as usize)
            .map(|i| {
                let at = 12 + 4 * i;
                u32::from_be_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]])
            })
            .collect();
        check_collection(&offsets).map_err(parse_err)?;
    }
    let mut infos = Vec::with_capacity(count as usize);
    for index in 0..count {
        let face = ttf_parser::Face::parse(data, index).map_err(|e| parse_err(e.to_string()))?;
        infos.push(face_info(&face, origin)?);
    }
    Ok(infos)
}

fn face_info(face: &ttf_parser::Face, origin: &str) -> Result<FontInfo, FontError> {
    let family = family_name(face).ok_or_else(|| FontError::NoFamilyName {
        origin: origin.to_string(),
    })?;
    Ok(FontInfo {
        family,
        weight: face.weight().to_number(),
        italic: face.is_italic() || face.is_oblique() || face.italic_angle() != 0.0,
        stretch: face.width().to_number(),
        monospace: face.is_monospaced(),
        variable: face.is_variable(),
        restricted_embedding: matches!(
            face.permissions(),
            Some(ttf_parser::Permissions::Restricted)
        ),
    })
}

/// Largest single table [`read_face_infos`] reads; past it, the whole-file path
/// takes over.
const MAX_HEADER_TABLE_BYTES: u64 = 16 << 20;

/// Most table bytes [`read_face_infos`] reads for one file, across all its
/// faces; past it, the whole-file path takes over.
const MAX_HEADER_FILE_BYTES: u64 = 64 << 20;

/// Most faces a font collection (TTC/OTC) may hold. Large collections have
/// tens to a few hundred.
pub const MAX_COLLECTION_FACES: u32 = 1024;

/// Longest family name, in characters, that registration accepts. A longer
/// name record is skipped.
pub const MAX_FAMILY_NAME_CHARS: usize = 256;

/// Check a collection's face count and offsets before any per-face work: at
/// most [`MAX_COLLECTION_FACES`] faces, each at its own offset. Real
/// collections share tables between faces but never repeat a face, while a
/// crafted file can repeat one offset to claim thousands of faces.
fn check_collection(offsets: &[u32]) -> Result<(), String> {
    if offsets.len() > MAX_COLLECTION_FACES as usize {
        return Err(format!(
            "the collection has {} faces, more than the {MAX_COLLECTION_FACES} kuva reads",
            offsets.len()
        ));
    }
    let mut sorted = offsets.to_vec();
    sorted.sort_unstable();
    if sorted.windows(2).any(|w| w[0] == w[1]) {
        return Err("the collection lists the same face more than once".into());
    }
    Ok(())
}

/// The [`FontInfo`] of every face in a font file, read from the table directory
/// and the few tables it needs (`head`, `hhea`, `maxp`, `name`, `OS/2`, `post`,
/// `fvar`) without reading glyph data. `None` when anything is unusual (a short
/// or malformed directory, an oversized table, no family name); the caller then
/// reads and parses the whole file, which also gives the proper error.
fn read_face_infos(path: &Path) -> Option<Vec<FontInfo>> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(path).ok()?;
    let len = file.metadata().ok()?.len();
    let mut budget = MAX_HEADER_FILE_BYTES;
    let mut read_at = |offset: u64, n: u64| -> Option<Vec<u8>> {
        if n > MAX_HEADER_TABLE_BYTES || n > budget || offset.checked_add(n)? > len {
            return None;
        }
        budget -= n;
        file.seek(SeekFrom::Start(offset)).ok()?;
        let mut buf = vec![0; n as usize];
        file.read_exact(&mut buf).ok()?;
        Some(buf)
    };
    let be_u32 = |b: &[u8], at: usize| u32::from_be_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]]);

    let header = read_at(0, 12)?;
    let face_offsets: Vec<u64> = if &header[..4] == b"ttcf" {
        // Each face needs a 4-byte offset, so the file bounds the count.
        let count = u64::from(be_u32(&header, 8));
        if count == 0 || count > (len - 12) / 4 {
            return None;
        }
        let raw = read_at(12, count * 4)?;
        let offsets: Vec<u32> = raw.chunks_exact(4).map(|o| be_u32(o, 0)).collect();
        // The whole-file path reports the problem.
        check_collection(&offsets).ok()?;
        offsets.into_iter().map(u64::from).collect()
    } else {
        vec![0]
    };

    let origin = path.display().to_string();
    let mut infos = Vec::with_capacity(face_offsets.len());
    for offset in face_offsets {
        let dir = read_at(offset, 12)?;
        let num_tables = u64::from(u16::from_be_bytes([dir[4], dir[5]]));
        let records = read_at(offset + 12, num_tables * 16)?;
        // Outer `None`: unreadable, give up. Inner `None`: the table is absent.
        let mut table = |tag: &[u8; 4]| -> Option<Option<Vec<u8>>> {
            match records.chunks_exact(16).find(|r| &r[..4] == tag) {
                Some(r) => read_at(u64::from(be_u32(r, 8)), u64::from(be_u32(r, 12))).map(Some),
                None => Some(None),
            }
        };
        let head = table(b"head")??;
        let hhea = table(b"hhea")??;
        let maxp = table(b"maxp")??;
        let name = table(b"name")?;
        let os2 = table(b"OS/2")?;
        let post = table(b"post")?;
        let fvar = table(b"fvar")?;
        let face = ttf_parser::Face::from_raw_tables(ttf_parser::RawFaceTables {
            head: &head,
            hhea: &hhea,
            maxp: &maxp,
            name: name.as_deref(),
            os2: os2.as_deref(),
            post: post.as_deref(),
            fvar: fvar.as_deref(),
            ..Default::default()
        })
        .ok()?;
        infos.push(face_info(&face, &origin).ok()?);
    }
    Some(infos)
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
            // UTF-16 or Mac Roman: at most 4 bytes per character. Checked
            // before decoding, so a huge record is not copied.
            if name.name.len() > 4 * MAX_FAMILY_NAME_CHARS {
                continue;
            }
            let Some(text) = name.to_string() else {
                continue;
            };
            let text = text.trim().to_string();
            if text.is_empty() || text.chars().count() > MAX_FAMILY_NAME_CHARS {
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

    /// Reading the header tables of a collection stops at
    /// `MAX_HEADER_FILE_BYTES` for the whole file, even when every face shares
    /// one large table; registration then reads the whole file instead.
    #[test]
    fn header_reads_stop_at_the_file_budget() {
        use crate::render::text_metrics::test_font::{
            collection_sharing_tables, font_with_name_padding,
        };
        let face = font_with_name_padding("Kuva Big Name", 9 << 20);
        let path = std::env::temp_dir().join(format!("kuva-budget-{}.ttc", std::process::id()));
        // 8 faces x 9 MiB of name table: over the 64 MiB budget.
        std::fs::write(&path, collection_sharing_tables(&face, 8)).unwrap();
        assert!(read_face_infos(&path).is_none());
        let infos = register_font_file(&path).unwrap();
        assert_eq!(infos.len(), 8);
        assert!(infos.iter().all(|i| i.family == "Kuva Big Name"));

        // 6 faces (54 MiB) stay within it.
        std::fs::write(&path, collection_sharing_tables(&face, 6)).unwrap();
        assert_eq!(read_face_infos(&path).map(|v| v.len()), Some(6));
        std::fs::remove_file(&path).ok();
    }

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
    fn normal_width_wins_then_narrower_then_wider() {
        assert_eq!(closest_stretch([3, 5, 7].into_iter()), Some(5));
        assert_eq!(closest_stretch([2, 3, 7].into_iter()), Some(3));
        assert_eq!(closest_stretch([6, 8].into_iter()), Some(6));
        assert_eq!(closest_stretch(std::iter::empty()), None);
    }

    #[cfg(feature = "embed_font")]
    #[test]
    fn css_string_escapes_everything_that_could_break_out() {
        assert_eq!(css_string("DejaVu Serif"), "DejaVu Serif");
        let nasty = css_string("x'</style><script>&\\");
        for bad in ['\'', '<', '>', '&'] {
            assert!(!nasty.contains(bad), "{nasty}");
        }
        assert!(nasty.starts_with("x\\27 "), "{nasty}");
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
            stretch: 5,
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
