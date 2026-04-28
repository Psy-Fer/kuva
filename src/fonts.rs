/// Bundled DejaVu Sans Regular — always available as a fallback regardless of
/// what fonts are installed on the host system.
///
/// License: Bitstream Vera Fonts Copyright / public domain (see assets/fonts/LICENSE).
pub(crate) const DEJAVU_SANS: &[u8] =
    include_bytes!("../assets/fonts/DejaVuSans.ttf");

fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[(n >> 18) & 0x3f] as char);
        out.push(TABLE[(n >> 12) & 0x3f] as char);
        out.push(if chunk.len() > 1 { TABLE[(n >> 6) & 0x3f] as char } else { '=' });
        out.push(if chunk.len() > 2 { TABLE[n & 0x3f] as char } else { '=' });
    }
    out
}

/// Returns a `<style>` block containing a base64-encoded `@font-face` for DejaVu Sans.
/// The result is computed once and cached for the lifetime of the process.
pub(crate) fn dejavu_sans_style_block() -> &'static str {
    use std::sync::OnceLock;
    static BLOCK: OnceLock<String> = OnceLock::new();
    BLOCK.get_or_init(|| {
        let b64 = base64_encode(DEJAVU_SANS);
        format!(
            "<style>@font-face{{font-family:'DejaVu Sans';\
             src:url('data:font/truetype;base64,{b64}') format('truetype');\
             font-weight:normal;font-style:normal;}}</style>"
        )
    })
}
