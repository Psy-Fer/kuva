//! Bundled DejaVu Sans font family — Regular, Bold, Oblique, and Mono.
//!
//! Each TTF is stored on disk as a gzip stream and inflated on first use.
//! The inflated bytes are cached for the lifetime of the process via `OnceLock`,
//! so the decompression cost is paid at most once per variant.
//!
//! License: Bitstream Vera Fonts Copyright / public domain (see assets/fonts/LICENSE).
//!
//! ## Regenerating compressed assets
//!
//! ```sh
//! gzip -9 -k -f assets/fonts/DejaVuSans.ttf
//! gzip -9 -k -f assets/fonts/DejaVuSans-Bold.ttf
//! gzip -9 -k -f assets/fonts/DejaVuSans-Oblique.ttf
//! gzip -9 -k -f assets/fonts/DejaVuSansMono.ttf
//! ```

use std::io::Read;
use std::sync::OnceLock;

use flate2::read::GzDecoder;

const DEJAVU_SANS_GZ: &[u8] = include_bytes!("../assets/fonts/DejaVuSans.ttf.gz");

#[cfg(any(feature = "png", feature = "pdf", feature = "embed_font"))]
const DEJAVU_SANS_BOLD_GZ: &[u8] = include_bytes!("../assets/fonts/DejaVuSans-Bold.ttf.gz");

#[cfg(any(feature = "png", feature = "pdf", feature = "embed_font"))]
const DEJAVU_SANS_OBLIQUE_GZ: &[u8] =
    include_bytes!("../assets/fonts/DejaVuSans-Oblique.ttf.gz");

#[cfg(any(feature = "png", feature = "pdf", feature = "embed_font"))]
const DEJAVU_SANS_MONO_GZ: &[u8] = include_bytes!("../assets/fonts/DejaVuSansMono.ttf.gz");

fn inflate(gz: &[u8], capacity: usize, label: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(capacity);
    GzDecoder::new(gz)
        .read_to_end(&mut out)
        .unwrap_or_else(|_| panic!("bundled {label} gzip stream is corrupt"));
    out
}

/// Returns the inflated DejaVu Sans Regular TTF bytes.
pub(crate) fn dejavu_sans() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| inflate(DEJAVU_SANS_GZ, 800_000, "DejaVu Sans"))
}

/// Returns the inflated DejaVu Sans Bold TTF bytes.
#[cfg(any(feature = "png", feature = "pdf", feature = "embed_font"))]
pub(crate) fn dejavu_sans_bold() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| inflate(DEJAVU_SANS_BOLD_GZ, 750_000, "DejaVu Sans Bold"))
}

/// Returns the inflated DejaVu Sans Oblique TTF bytes.
#[cfg(any(feature = "png", feature = "pdf", feature = "embed_font"))]
pub(crate) fn dejavu_sans_oblique() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| inflate(DEJAVU_SANS_OBLIQUE_GZ, 680_000, "DejaVu Sans Oblique"))
}

/// Returns the inflated DejaVu Sans Mono TTF bytes.
#[cfg(any(feature = "png", feature = "pdf", feature = "embed_font"))]
pub(crate) fn dejavu_sans_mono() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| inflate(DEJAVU_SANS_MONO_GZ, 380_000, "DejaVu Sans Mono"))
}

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

/// Returns a `<style>` block with `@font-face` rules for all four DejaVu variants.
/// The result is computed once and cached for the lifetime of the process.
pub(crate) fn dejavu_sans_style_block() -> &'static str {
    static BLOCK: OnceLock<String> = OnceLock::new();
    BLOCK.get_or_init(|| {
        let reg = base64_encode(dejavu_sans());
        let bold = base64_encode(dejavu_sans_bold());
        let oblique = base64_encode(dejavu_sans_oblique());
        let mono = base64_encode(dejavu_sans_mono());
        let url = "data:font/truetype;base64,";
        format!(
            "<style>\
             @font-face{{font-family:'DejaVu Sans';src:url('{url}{reg}') format('truetype');font-weight:normal;font-style:normal;}}\
             @font-face{{font-family:'DejaVu Sans';src:url('{url}{bold}') format('truetype');font-weight:bold;font-style:normal;}}\
             @font-face{{font-family:'DejaVu Sans';src:url('{url}{oblique}') format('truetype');font-weight:normal;font-style:italic;}}\
             @font-face{{font-family:'DejaVu Sans Mono';src:url('{url}{mono}') format('truetype');font-weight:normal;font-style:normal;}}\
             </style>"
        )
    })
}
