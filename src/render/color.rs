/// Compact color representation that avoids heap-allocating a `String` per
/// primitive.  The common case in data-heavy plots — hex colors from
/// colormaps, `"none"`, and well-known named colors — is stored inline
/// with zero heap allocation.
#[derive(Debug, Clone)]
pub enum Color {
    /// Packed RGB — no heap allocation.  Covers hex colors (#rrggbb),
    /// `rgb(r,g,b)` from colormaps, and named colors resolved at
    /// construction time.
    Rgb(u8, u8, u8),
    /// Transparent / no-paint.  Equivalent to SVG `fill="none"`.
    None,
    /// Arbitrary CSS color string for anything not covered above.
    Css(Box<str>),
}

impl Color {
    /// Write the SVG representation into `buf`.
    #[inline]
    pub fn write_svg(&self, buf: &mut String) {
        match self {
            Color::Rgb(r, g, b) => {
                const HEX: &[u8; 16] = b"0123456789abcdef";
                let bytes = [
                    b'#',
                    HEX[(*r >> 4) as usize],
                    HEX[(*r & 0xf) as usize],
                    HEX[(*g >> 4) as usize],
                    HEX[(*g & 0xf) as usize],
                    HEX[(*b >> 4) as usize],
                    HEX[(*b & 0xf) as usize],
                ];
                // SAFETY: all bytes are ASCII
                buf.push_str(unsafe { std::str::from_utf8_unchecked(&bytes) });
            }
            Color::None => buf.push_str("none"),
            Color::Css(s) => buf.push_str(s),
        }
    }

    /// Return the SVG string representation.  Prefer [`write_svg`] in
    /// loops to avoid per-call allocation.
    pub fn to_svg_string(&self) -> String {
        let mut s = String::with_capacity(7);
        self.write_svg(&mut s);
        s
    }
}

// ── From impls ──────────────────────────────────────────────────────────────
// These let all existing `.into()` call sites work transparently.

impl From<&str> for Color {
    #[inline]
    fn from(s: &str) -> Self {
        parse_color_str(s)
    }
}

impl From<String> for Color {
    #[inline]
    fn from(s: String) -> Self {
        parse_color_str(&s)
    }
}

impl From<&String> for Color {
    #[inline]
    fn from(s: &String) -> Self {
        parse_color_str(s.as_str())
    }
}

fn parse_color_str(s: &str) -> Color {
    if s.is_empty() || s.eq_ignore_ascii_case("none") || s.eq_ignore_ascii_case("transparent") {
        return Color::None;
    }

    // #RRGGBB
    if s.len() == 7 && s.as_bytes()[0] == b'#' {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&s[1..3], 16),
            u8::from_str_radix(&s[3..5], 16),
            u8::from_str_radix(&s[5..7], 16),
        ) {
            return Color::Rgb(r, g, b);
        }
    }

    // #RGB shorthand
    if s.len() == 4 && s.as_bytes()[0] == b'#' {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&s[1..2], 16),
            u8::from_str_radix(&s[2..3], 16),
            u8::from_str_radix(&s[3..4], 16),
        ) {
            return Color::Rgb(r * 17, g * 17, b * 17);
        }
    }

    // rgb(r, g, b)
    if let Some(inner) = s.strip_prefix("rgb(").and_then(|t| t.strip_suffix(')')) {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                parts[0].trim().parse::<f64>(),
                parts[1].trim().parse::<f64>(),
                parts[2].trim().parse::<f64>(),
            ) {
                return Color::Rgb(r.round() as u8, g.round() as u8, b.round() as u8);
            }
        }
    }

    // Common named CSS colors → inline Rgb
    match s.to_ascii_lowercase().as_str() {
        "black" => Color::Rgb(0, 0, 0),
        "white" => Color::Rgb(255, 255, 255),
        "red" => Color::Rgb(255, 0, 0),
        "green" => Color::Rgb(0, 128, 0),
        "blue" => Color::Rgb(0, 0, 255),
        "steelblue" => Color::Rgb(70, 130, 180),
        "gray" | "grey" => Color::Rgb(128, 128, 128),
        "lightgray" | "lightgrey" => Color::Rgb(211, 211, 211),
        "darkgray" | "darkgrey" => Color::Rgb(169, 169, 169),
        "orange" => Color::Rgb(255, 165, 0),
        "yellow" => Color::Rgb(255, 255, 0),
        "purple" => Color::Rgb(128, 0, 128),
        "pink" => Color::Rgb(255, 192, 203),
        "brown" => Color::Rgb(165, 42, 42),
        "cyan" => Color::Rgb(0, 255, 255),
        "magenta" => Color::Rgb(255, 0, 255),
        "coral" => Color::Rgb(255, 127, 80),
        "salmon" => Color::Rgb(250, 128, 114),
        "navy" => Color::Rgb(0, 0, 128),
        "teal" => Color::Rgb(0, 128, 128),
        "olive" => Color::Rgb(128, 128, 0),
        "maroon" => Color::Rgb(128, 0, 0),
        "gold" => Color::Rgb(255, 215, 0),
        "tomato" => Color::Rgb(255, 99, 71),
        "crimson" => Color::Rgb(220, 20, 60),
        "dodgerblue" => Color::Rgb(30, 144, 255),
        "limegreen" => Color::Rgb(50, 205, 50),
        "orangered" => Color::Rgb(255, 69, 0),
        "darkred" => Color::Rgb(139, 0, 0),
        "darkblue" => Color::Rgb(0, 0, 139),
        "darkgreen" => Color::Rgb(0, 100, 0),
        "firebrick" => Color::Rgb(178, 34, 34),
        "royalblue" => Color::Rgb(65, 105, 225),
        "indianred" => Color::Rgb(205, 92, 92),
        "forestgreen" => Color::Rgb(34, 139, 34),
        "sienna" => Color::Rgb(160, 82, 45),
        "chocolate" => Color::Rgb(210, 105, 30),
        "peru" => Color::Rgb(205, 133, 63),
        "violet" => Color::Rgb(238, 130, 238),
        "turquoise" => Color::Rgb(64, 224, 208),
        "cornflowerblue" => Color::Rgb(100, 149, 237),
        "darkorange" => Color::Rgb(255, 140, 0),
        "deeppink" => Color::Rgb(255, 20, 147),
        "hotpink" => Color::Rgb(255, 105, 180),
        "silver" => Color::Rgb(192, 192, 192),
        _ => Color::Css(s.into()),
    }
}

/// Equality is by visual equivalence: two Colors that produce the same SVG
/// attribute value are equal.  This is only used for deduplication, not for
/// the hot path.
impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => r1 == r2 && g1 == g2 && b1 == b2,
            (Color::None, Color::None) => true,
            (Color::Css(a), Color::Css(b)) => a == b,
            _ => false,
        }
    }
}
