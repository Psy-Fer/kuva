//! Glyph-outline extraction for the sequence-logo table. Not compiled into the
//! library: included (via `#[path]`) by `examples/gen_logo_glyphs.rs`, which
//! writes `data.rs`, and by the drift test in `logo_glyphs/mod.rs`, so both use
//! exactly this code. Needs `ttf-parser`, a dev-dependency only.

use ttf_parser::{Face, OutlineBuilder};

/// Characters given an outline: every letter, digit, and the gap/wildcard
/// symbols, so any common alphabet (custom ones included) is covered. Anything
/// else falls back to a filled box with a centred letter at render time.
pub fn candidates() -> Vec<char> {
    ('A'..='Z')
        .chain('a'..='z')
        .chain('0'..='9')
        .chain(['-', '.', '*', '?'])
        .collect()
}

/// One extracted glyph: character, path `d` in font units (y-up), and the
/// outline's bounding box `[x_min, y_min, x_max, y_max]`.
pub struct Extracted {
    pub ch: char,
    pub path: String,
    pub bbox: [f64; 4],
}

/// Extract every candidate glyph present in `font`, sorted by character.
pub fn extract(font: &[u8]) -> Vec<Extracted> {
    let face = Face::parse(font, 0).expect("logo font failed to parse");
    let mut out: Vec<Extracted> = candidates()
        .into_iter()
        .filter_map(|ch| {
            let id = face.glyph_index(ch)?;
            let mut b = PathBuilder::default();
            let bb = face.outline_glyph(id, &mut b)?;
            Some(Extracted {
                ch,
                path: b.path,
                bbox: [
                    bb.x_min as f64,
                    bb.y_min as f64,
                    bb.x_max as f64,
                    bb.y_max as f64,
                ],
            })
        })
        .collect();
    out.sort_by_key(|g| g.ch);
    out
}

/// Builds an SVG path `d` string emitting only `M` / `L` / `C` / `Z`.
///
/// TrueType outlines are quadratic, but kuva's rasteriser understands only
/// `m l h v c a z`, so every quadratic is degree-elevated to its exact cubic
/// equivalent (the curve is identical, not an approximation) and all backends
/// see the same commands.
#[derive(Default)]
struct PathBuilder {
    path: String,
    /// Current pen position: the quadratic's implicit start point.
    cx: f32,
    cy: f32,
    /// Start of the current subpath, restored by `close`.
    start: (f32, f32),
}

impl OutlineBuilder for PathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        use std::fmt::Write;
        let _ = write!(self.path, "M{:.2} {:.2}", x, y);
        self.cx = x;
        self.cy = y;
        self.start = (x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        use std::fmt::Write;
        let _ = write!(self.path, "L{:.2} {:.2}", x, y);
        self.cx = x;
        self.cy = y;
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        use std::fmt::Write;
        // C1 = P0 + 2/3 (P1 - P0), C2 = P2 + 2/3 (P1 - P2)
        const T: f32 = 2.0 / 3.0;
        let c1x = self.cx + T * (x1 - self.cx);
        let c1y = self.cy + T * (y1 - self.cy);
        let c2x = x + T * (x1 - x);
        let c2y = y + T * (y1 - y);
        let _ = write!(
            self.path,
            "C{:.2} {:.2} {:.2} {:.2} {:.2} {:.2}",
            c1x, c1y, c2x, c2y, x, y
        );
        self.cx = x;
        self.cy = y;
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        use std::fmt::Write;
        let _ = write!(
            self.path,
            "C{:.2} {:.2} {:.2} {:.2} {:.2} {:.2}",
            x1, y1, x2, y2, x, y
        );
        self.cx = x;
        self.cy = y;
    }
    fn close(&mut self) {
        self.path.push('Z');
        self.cx = self.start.0;
        self.cy = self.start.1;
    }
}
