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
const DEJAVU_SANS_OBLIQUE_GZ: &[u8] = include_bytes!("../assets/fonts/DejaVuSans-Oblique.ttf.gz");

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

/// Gzip-compressed New Computer Modern Math (OFL/GUST), embedded at compile
/// time. Used only by the `typst-math` feature, as the math font fed to the typst
/// compiler. ~1.1 MB inflated; ~0.75 MB on disk.
#[cfg(feature = "typst-math")]
const NEWCM_MATH_GZ: &[u8] = include_bytes!("../assets/fonts/NewCMMath-Regular.otf.gz");

/// Returns the inflated New Computer Modern Math OTF bytes. Inflated once and
/// cached. Bundled (rather than pulled from `typst-assets`) so the typst tier
/// ships ~1 MB of font rather than ~15 MB.
#[cfg(feature = "typst-math")]
pub(crate) fn newcm_math() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| inflate(NEWCM_MATH_GZ, 1_200_000, "NewCM Math"))
}

/// Returns a `<style>` block with `@font-face` rules for all four DejaVu variants.
/// The result is computed once and cached for the lifetime of the process.
#[cfg(feature = "embed_font")]
pub(crate) fn dejavu_sans_style_block() -> &'static str {
    static BLOCK: OnceLock<String> = OnceLock::new();
    BLOCK.get_or_init(|| {
        use crate::encoding::base64_encode;
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
