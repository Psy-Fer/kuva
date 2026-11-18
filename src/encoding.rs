//! Small encoders shared by the optional output features.

/// Standard base64 (RFC 4648, with `=` padding). Used to embed the font in an SVG
/// (`embed_font`) and compressed figures in HTML (`svgz`), without a base64 dependency.
pub(crate) fn base64_encode(data: &[u8]) -> String {
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

/// Gzip `data` at the default compression level.
#[cfg(feature = "svgz")]
pub(crate) fn gzip(data: &[u8]) -> Vec<u8> {
    use std::io::Write;
    let mut enc = flate2::write::GzEncoder::new(
        Vec::with_capacity(data.len() / 8),
        flate2::Compression::default(),
    );
    // Writing to a Vec cannot fail.
    enc.write_all(data).expect("gzip into memory");
    enc.finish().expect("gzip into memory")
}

#[cfg(test)]
mod tests {
    use super::base64_encode;

    #[test]
    fn base64_matches_rfc4648_vectors() {
        for (input, want) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64_encode(input.as_bytes()), want);
        }
    }
}
