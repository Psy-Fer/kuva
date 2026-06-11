//! Bundled DejaVu Sans Regular — always available as a fallback regardless of
//! what fonts are installed on the host system.
//!
//! The TTF is stored on disk as a gzip stream (~350 KB) and inflated on first
//! use. The inflated bytes (~740 KB) are cached for the lifetime of the process
//! via `OnceLock`, so the decompression cost is paid at most once.
//!
//! Using gzip framing (rather than raw DEFLATE or zlib) means the on-disk
//! asset is round-trippable with standard tools — `gunzip DejaVuSans.ttf.gz`
//! works as expected.
//!
//! License: Bitstream Vera Fonts Copyright / public domain (see assets/fonts/LICENSE).
//!
//! ## Regenerating the compressed asset
//!
//! If `DejaVuSans.ttf` is ever replaced, regenerate the gzip asset with:
//!
//! ```sh
//! libdeflate-gzip -12 -k -f assets/fonts/DejaVuSans.ttf
//! ```
//!
//! Any standards-compliant gzip encoder works (`gzip -9`, `pigz -11`, etc.);
//! `libdeflate-gzip -12` just produces the smallest output.

use std::io::Read;
use std::sync::OnceLock;

use flate2::read::GzDecoder;

/// Gzip-compressed bytes of DejaVu Sans Regular, embedded at compile time.
const DEJAVU_SANS_GZ: &[u8] = include_bytes!("../assets/fonts/DejaVuSans.ttf.gz");

/// Inflate a gzipped embedded asset into a `Vec<u8>`. `what` names the asset
/// for the panic message if the stream is corrupt.
fn inflate_gz(gz: &[u8], capacity: usize, what: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(capacity);
    GzDecoder::new(gz)
        .read_to_end(&mut out)
        .unwrap_or_else(|_| panic!("bundled {what} gzip stream is corrupt"));
    out
}

/// Returns the inflated DejaVu Sans TTF bytes. Inflated once and cached.
pub(crate) fn dejavu_sans() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| inflate_gz(DEJAVU_SANS_GZ, 800_000, "DejaVu Sans"))
}

/// Gzip-compressed New Computer Modern Math (OFL/GUST), embedded at compile
/// time. Used only by the `math` feature, as the math font fed to the typst
/// compiler. ~1.1 MB inflated; ~0.75 MB on disk.
#[cfg(feature = "math")]
const NEWCM_MATH_GZ: &[u8] = include_bytes!("../assets/fonts/NewCMMath-Regular.otf.gz");

/// Returns the inflated New Computer Modern Math OTF bytes. Inflated once and
/// cached. Bundled (rather than pulled from `typst-assets`) so the `math`
/// feature ships ~1 MB of font rather than ~15 MB.
#[cfg(feature = "math")]
pub(crate) fn newcm_math() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| inflate_gz(NEWCM_MATH_GZ, 1_200_000, "NewCM Math"))
}

/// Bundled DejaVu Sans parsed as a `fontdue::Font`, for the raster backend's
/// glyph rasterisation.
#[cfg(feature = "png")]
pub(crate) fn shared_font() -> &'static fontdue::Font {
    static FONT: OnceLock<fontdue::Font> = OnceLock::new();
    FONT.get_or_init(|| {
        fontdue::Font::from_bytes(dejavu_sans(), fontdue::FontSettings::default())
            .expect("bundled DejaVu Sans TTF is valid")
    })
}

// Used only by `dejavu_sans_style_block` (the `embed_font` SVG path); the
// `fonts` module is also compiled for `png`/`pdf`/`math`, where this is dead.
#[cfg(feature = "embed_font")]
fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 {
            chunk[1] as usize
        } else {
            0
        };
        let b2 = if chunk.len() > 2 {
            chunk[2] as usize
        } else {
            0
        };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[(n >> 18) & 0x3f] as char);
        out.push(TABLE[(n >> 12) & 0x3f] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(n >> 6) & 0x3f] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[n & 0x3f] as char
        } else {
            '='
        });
    }
    out
}

/// Returns a `<style>` block containing a base64-encoded `@font-face` for DejaVu Sans.
/// The result is computed once and cached for the lifetime of the process.
#[cfg(feature = "embed_font")]
pub(crate) fn dejavu_sans_style_block() -> &'static str {
    static BLOCK: OnceLock<String> = OnceLock::new();
    BLOCK.get_or_init(|| {
        let b64 = base64_encode(dejavu_sans());
        format!(
            "<style>@font-face{{font-family:'DejaVu Sans';\
             src:url('data:font/truetype;base64,{b64}') format('truetype');\
             font-weight:normal;font-style:normal;}}</style>"
        )
    })
}
