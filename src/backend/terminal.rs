//! Terminal backend — renders a [`Scene`] to ANSI/Unicode braille/box-drawing text.
//!
//! **Layers** (drawn front-to-back):
//! 1. `char_grid`      — filled `Rect`s (`█`) and `Text` characters
//! 2. `line_char_grid` — box-drawing characters for axis/tick `Line`s
//! 3. `braille`        — dot-matrix characters for circles and diagonal lines/paths
//!
//! **Box-drawing bitmask** (per character cell):
//! ```text
//!   TOP=1  RIGHT=2  BOTTOM=4  LEFT=8
//! ```
//! Accumulated bits map to the correct Unicode box-drawing character so that
//! H/V intersections (e.g. T-junctions, corners) are rendered correctly.
//!
//! **Coordinate mapping** (W = scene width, H = scene height, C = cols, R = rows):
//! ```text
//!   braille_x = floor(px · C·2 / W)   range [0, C·2)
//!   braille_y = floor(py · R·4 / H)   range [0, R·4)
//!   char_col  = floor(px · C   / W)   range [0, C)
//!   char_row  = floor(py · R   / H)   range [0, R)
//! ```

use crate::render::render::{Primitive, Scene, TextAnchor};

// ── Box-drawing bit constants ─────────────────────────────────────────────────

const TOP: u8    = 1;
const RIGHT: u8  = 2;
const BOTTOM: u8 = 4;
const LEFT: u8   = 8;

fn bitmask_to_char(bits: u8) -> char {
    match bits {
        1  => '╵',
        2  => '╶',
        3  => '└',
        4  => '╷',
        5  => '│',
        6  => '┌',
        7  => '├',
        8  => '╴',
        9  => '┘',
        10 => '─',
        11 => '┴',
        12 => '┐',
        13 => '┤',
        14 => '┬',
        15 => '┼',
        _  => ' ',
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Renders a [`Scene`] to a Unicode braille/box-drawing string for terminal display.
pub struct TerminalBackend {
    /// Number of terminal character columns.
    pub cols: usize,
    /// Number of terminal character rows.
    pub rows: usize,
}

impl TerminalBackend {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self { cols, rows }
    }

    pub fn render_scene(&self, scene: &Scene) -> String {
        // Derive text colour from the scene's theme. Falls back to light grey
        // so an unstyled Scene still looks reasonable on dark terminals.
        let text_color = scene
            .text_color
            .as_deref()
            .map(css_to_rgb)
            .unwrap_or((210, 210, 210));
        let mut canvas = Canvas::new(self.cols, self.rows, scene.width, scene.height, text_color);
        for p in &scene.elements {
            canvas.draw(p);
        }
        canvas.to_ansi_string()
    }
}

// ── Internal canvas ───────────────────────────────────────────────────────────

struct Canvas {
    cols: usize,
    rows: usize,
    scene_width: f64,
    scene_height: f64,

    /// `[row][col]` braille bitmask (U+2800 encoding).
    braille: Vec<Vec<u8>>,
    braille_color: Vec<Vec<Option<(u8, u8, u8)>>>,

    /// `[row][col]` box-drawing bitmask (TOP/RIGHT/BOTTOM/LEFT bits).
    line_char_bits: Vec<Vec<u8>>,
    line_char_color: Vec<Vec<Option<(u8, u8, u8)>>>,

    /// `[row][col]` character overlay (filled blocks and text). Drawn on top.
    char_grid: Vec<Vec<Option<(char, (u8, u8, u8))>>>,

    /// Accumulated `translate(tx,ty)` offsets — innermost frame last.
    transform_stack: Vec<(f64, f64)>,

    /// Colour used for Text primitives, derived from scene.text_color.
    text_color: (u8, u8, u8),
}

impl Canvas {
    fn new(cols: usize, rows: usize, scene_width: f64, scene_height: f64, text_color: (u8, u8, u8)) -> Self {
        let cols = cols.max(1);
        let rows = rows.max(1);
        let scene_width = scene_width.max(1.0);
        let scene_height = scene_height.max(1.0);
        Self {
            cols,
            rows,
            scene_width,
            scene_height,
            braille:           vec![vec![0u8; cols]; rows],
            braille_color:     vec![vec![None; cols]; rows],
            line_char_bits:    vec![vec![0u8; cols]; rows],
            line_char_color:   vec![vec![None; cols]; rows],
            char_grid:         vec![vec![None; cols]; rows],
            transform_stack:   vec![(0.0, 0.0)],
            text_color,
        }
    }

    /// Sum all stacked translations to get the current global offset.
    fn current_offset(&self) -> (f64, f64) {
        self.transform_stack
            .iter()
            .fold((0.0, 0.0), |(ax, ay), &(tx, ty)| (ax + tx, ay + ty))
    }

    // ── Coordinate mapping ───────────────────────────────────────────────────

    fn to_bx(&self, px: f64) -> isize {
        (px * (self.cols * 2) as f64 / self.scene_width).floor() as isize
    }

    fn to_by(&self, py: f64) -> isize {
        (py * (self.rows * 4) as f64 / self.scene_height).floor() as isize
    }

    fn to_cx(&self, px: f64) -> isize {
        (px * self.cols as f64 / self.scene_width).floor() as isize
    }

    fn to_cy(&self, py: f64) -> isize {
        (py * self.rows as f64 / self.scene_height).floor() as isize
    }

    // ── Braille drawing ──────────────────────────────────────────────────────

    fn set_dot(&mut self, bx: isize, by: isize, color: (u8, u8, u8)) {
        if bx < 0 || by < 0 {
            return;
        }
        let bx = bx as usize;
        let by = by as usize;
        if bx >= self.cols * 2 || by >= self.rows * 4 {
            return;
        }
        let tr = by / 4;
        let tc = bx / 2;
        let bit: u8 = match (bx % 2, by % 4) {
            (0, 0) => 1,
            (0, 1) => 2,
            (0, 2) => 4,
            (1, 0) => 8,
            (1, 1) => 16,
            (1, 2) => 32,
            (0, 3) => 64,
            (1, 3) => 128,
            _ => return,
        };
        self.braille[tr][tc] |= bit;
        self.braille_color[tr][tc] = Some(color);
    }

    fn set_char(&mut self, cx: isize, cy: isize, ch: char, color: (u8, u8, u8)) {
        if cx < 0 || cy < 0 {
            return;
        }
        let cx = cx as usize;
        let cy = cy as usize;
        if cx >= self.cols || cy >= self.rows {
            return;
        }
        self.char_grid[cy][cx] = Some((ch, color));
    }

    // ── Box-drawing line drawing ─────────────────────────────────────────────

    /// Accumulate box-drawing bits into a character cell.
    fn set_line_bits(&mut self, cx: isize, cy: isize, bits: u8, color: (u8, u8, u8)) {
        if cx < 0 || cy < 0 {
            return;
        }
        let cx = cx as usize;
        let cy = cy as usize;
        if cx >= self.cols || cy >= self.rows {
            return;
        }
        self.line_char_bits[cy][cx] |= bits;
        self.line_char_color[cy][cx] = Some(color);
    }

    /// Draw a horizontal box-drawing line at char_row `cy` from `cx0` to `cx1`.
    fn draw_hline(&mut self, cx0: isize, cy: isize, cx1: isize, color: (u8, u8, u8)) {
        let (lo, hi) = if cx0 <= cx1 { (cx0, cx1) } else { (cx1, cx0) };
        for cx in lo..=hi {
            let bits = if lo == hi {
                LEFT | RIGHT
            } else if cx == lo {
                RIGHT
            } else if cx == hi {
                LEFT
            } else {
                LEFT | RIGHT
            };
            self.set_line_bits(cx, cy, bits, color);
        }
    }

    /// Draw a vertical box-drawing line at char_col `cx` from `cy0` to `cy1`.
    fn draw_vline(&mut self, cx: isize, cy0: isize, cy1: isize, color: (u8, u8, u8)) {
        let (lo, hi) = if cy0 <= cy1 { (cy0, cy1) } else { (cy1, cy0) };
        for cy in lo..=hi {
            let bits = if lo == hi {
                TOP | BOTTOM
            } else if cy == lo {
                BOTTOM
            } else if cy == hi {
                TOP
            } else {
                TOP | BOTTOM
            };
            self.set_line_bits(cx, cy, bits, color);
        }
    }

    // ── Bresenham braille line ───────────────────────────────────────────────

    fn bresenham(&mut self, bx0: isize, by0: isize, bx1: isize, by1: isize, color: (u8, u8, u8)) {
        let dx = (bx1 - bx0).abs();
        let dy = (by1 - by0).abs();
        let sx: isize = if bx0 < bx1 { 1 } else { -1 };
        let sy: isize = if by0 < by1 { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = bx0;
        let mut y = by0;
        loop {
            self.set_dot(x, y, color);
            if x == bx1 && y == by1 {
                break;
            }
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    // ── Scanline polygon fill ────────────────────────────────────────────────

    /// Scanline-fill a closed polygon in the braille dot grid.
    /// `pts` is a list of scene-space (x, y) coordinates forming the polygon
    /// boundary.  Uses an even-odd fill rule: for each braille row, compute
    /// x-intersections with every edge, sort them, and fill between each pair.
    fn fill_braille_polygon(&mut self, pts: &[(f64, f64)], rgb: (u8, u8, u8)) {
        if pts.len() < 3 {
            return;
        }
        let bw = (self.cols * 2) as isize;
        let bh = (self.rows * 4) as isize;

        // Convert to braille coordinates once.
        let bpts: Vec<(f64, f64)> = pts
            .iter()
            .map(|&(sx, sy)| (self.to_bx(sx) as f64, self.to_by(sy) as f64))
            .collect();

        let by_min = bpts.iter().map(|p| p.1).fold(f64::INFINITY, f64::min)
            .max(0.0) as isize;
        let by_max = bpts.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max)
            .min((bh - 1) as f64) as isize;

        let n = bpts.len();
        for by in by_min..=by_max {
            // Sample at row midpoint to avoid vertex-touching ambiguity.
            let sy = by as f64 + 0.5;
            let mut xs: Vec<f64> = Vec::new();
            for i in 0..n {
                let p0 = bpts[i];
                let p1 = bpts[(i + 1) % n];
                let (y0, y1) = (p0.1, p1.1);
                if (y0 < sy && y1 >= sy) || (y1 < sy && y0 >= sy) {
                    let t = (sy - y0) / (y1 - y0);
                    xs.push(p0.0 + t * (p1.0 - p0.0));
                }
            }
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mut i = 0;
            while i + 1 < xs.len() {
                let x0 = (xs[i] as isize).max(0);
                let x1 = (xs[i + 1] as isize).min(bw - 1);
                for bx in x0..=x1 {
                    self.set_dot(bx, by, rgb);
                }
                i += 2;
            }
        }
    }

    // ── Cubic Bézier tessellation ────────────────────────────────────────────

    fn tessellate_cubic(
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
        p3: (f64, f64),
    ) -> Vec<(f64, f64)> {
        const N: usize = 20;
        (0..=N)
            .map(|i| {
                let t = i as f64 / N as f64;
                let mt = 1.0 - t;
                let x = mt * mt * mt * p0.0
                    + 3.0 * mt * mt * t * p1.0
                    + 3.0 * mt * t * t * p2.0
                    + t * t * t * p3.0;
                let y = mt * mt * mt * p0.1
                    + 3.0 * mt * mt * t * p1.1
                    + 3.0 * mt * t * t * p2.1
                    + t * t * t * p3.1;
                (x, y)
            })
            .collect()
    }

    /// Tessellate an SVG arc into a polyline using the endpoint-to-center parameterization.
    /// All coordinates are in scene space (translation already applied by the caller).
    fn tessellate_arc(
        (x1, y1): (f64, f64),
        (rx_in, ry_in): (f64, f64),
        x_rot_deg: f64,
        large_arc: bool,
        sweep: bool,
        (x2, y2): (f64, f64),
    ) -> Vec<(f64, f64)> {
        // Degenerate cases: same point or zero radius → straight line
        if (x1 - x2).abs() < 1e-10 && (y1 - y2).abs() < 1e-10 {
            return vec![(x1, y1), (x2, y2)];
        }
        if rx_in.abs() < 1e-10 || ry_in.abs() < 1e-10 {
            return vec![(x1, y1), (x2, y2)];
        }

        let phi = x_rot_deg.to_radians();
        let cos_phi = phi.cos();
        let sin_phi = phi.sin();

        // Step 1: midpoint in rotated frame
        let dx = (x1 - x2) / 2.0;
        let dy = (y1 - y2) / 2.0;
        let x1p =  cos_phi * dx + sin_phi * dy;
        let y1p = -sin_phi * dx + cos_phi * dy;

        // Step 2: ensure radii are large enough
        let mut rx = rx_in.abs();
        let mut ry = ry_in.abs();
        let lambda = (x1p / rx).powi(2) + (y1p / ry).powi(2);
        if lambda > 1.0 {
            let s = lambda.sqrt();
            rx *= s;
            ry *= s;
        }

        // Step 3: center in rotated frame
        let num = (rx * rx * ry * ry) - (rx * rx * y1p * y1p) - (ry * ry * x1p * x1p);
        let den = (rx * rx * y1p * y1p) + (ry * ry * x1p * x1p);
        let sq = if den < 1e-10 { 0.0 } else { (num / den).max(0.0).sqrt() };
        let sq = if large_arc == sweep { -sq } else { sq };
        let cxp =  sq * rx * y1p / ry;
        let cyp = -sq * ry * x1p / rx;

        // Step 4: center in scene space
        let cx = cos_phi * cxp - sin_phi * cyp + (x1 + x2) / 2.0;
        let cy = sin_phi * cxp + cos_phi * cyp + (y1 + y2) / 2.0;

        // Step 5: start angle and angular span
        fn angle(ux: f64, uy: f64, vx: f64, vy: f64) -> f64 {
            let n = ((ux * ux + uy * uy) * (vx * vx + vy * vy)).sqrt();
            let c = ((ux * vx + uy * vy) / n).clamp(-1.0, 1.0);
            let a = c.acos();
            if ux * vy - uy * vx < 0.0 { -a } else { a }
        }

        let ux = (x1p - cxp) / rx;
        let uy = (y1p - cyp) / ry;
        let vx = (-x1p - cxp) / rx;
        let vy = (-y1p - cyp) / ry;

        let theta1 = angle(1.0, 0.0, ux, uy);
        let mut d_theta = angle(ux, uy, vx, vy);
        if !sweep && d_theta > 0.0 {
            d_theta -= std::f64::consts::TAU;
        } else if sweep && d_theta < 0.0 {
            d_theta += std::f64::consts::TAU;
        }

        // Tessellate: ~1 segment per 5° is more than enough for braille resolution
        let n_segs = ((d_theta.abs() / (5.0_f64.to_radians())).ceil() as usize).max(2);
        (0..=n_segs)
            .map(|i| {
                let t = i as f64 / n_segs as f64;
                let theta = theta1 + t * d_theta;
                let xr = cos_phi * rx * theta.cos() - sin_phi * ry * theta.sin() + cx;
                let yr = sin_phi * rx * theta.cos() + cos_phi * ry * theta.sin() + cy;
                (xr, yr)
            })
            .collect()
    }

    // ── Primitive dispatch ───────────────────────────────────────────────────

    fn draw(&mut self, p: &Primitive) {
        let (tx, ty) = self.current_offset();

        match p {
            Primitive::Circle { cx, cy, r, fill } => {
                let rgb = css_to_rgb(fill);
                let cx_s = cx + tx;
                let cy_s = cy + ty;
                let bw = (self.cols * 2) as f64;
                let bh = (self.rows * 4) as f64;
                let bx_min = self.to_bx(cx_s - r).max(0);
                let by_min = self.to_by(cy_s - r).max(0);
                let bx_max = self.to_bx(cx_s + r).min(bw as isize - 1);
                let by_max = self.to_by(cy_s + r).min(bh as isize - 1);
                for bx in bx_min..=bx_max {
                    for by in by_min..=by_max {
                        let px = bx as f64 * self.scene_width / bw;
                        let py = by as f64 * self.scene_height / bh;
                        if (px - cx_s).powi(2) + (py - cy_s).powi(2) <= r * r {
                            self.set_dot(bx, by, rgb);
                        }
                    }
                }
            }

            Primitive::Line { x1, y1, x2, y2, stroke, .. } => {
                let rgb = css_to_rgb(stroke);
                // Strictly horizontal or vertical lines (within half a scene pixel)
                // are drawn with box-drawing characters for clean axis rendering.
                // All other lines use Bresenham braille.
                let is_h = (y1 - y2).abs() < 0.5;
                let is_v = (x1 - x2).abs() < 0.5;
                if is_h {
                    let cy = self.to_cy(y1 + ty);
                    let cx0 = self.to_cx(x1 + tx);
                    let cx1 = self.to_cx(x2 + tx);
                    self.draw_hline(cx0, cy, cx1, rgb);
                } else if is_v {
                    let cx = self.to_cx(x1 + tx);
                    let cy0 = self.to_cy(y1 + ty);
                    let cy1 = self.to_cy(y2 + ty);
                    self.draw_vline(cx, cy0, cy1, rgb);
                } else {
                    let bx0 = self.to_bx(x1 + tx);
                    let by0 = self.to_by(y1 + ty);
                    let bx1 = self.to_bx(x2 + tx);
                    let by1 = self.to_by(y2 + ty);
                    self.bresenham(bx0, by0, bx1, by1, rgb);
                }
            }

            Primitive::Path { d, stroke, fill, .. } => {
                let has_stroke = stroke != "none";
                let fill_str = fill.as_deref().unwrap_or("none");
                // SVG gradient references (url(#...)) can't be resolved in the
                // terminal; treat them as a neutral grey.
                let fill_rgb = if fill_str == "none" || fill_str.is_empty() {
                    None
                } else if fill_str.starts_with("url(") {
                    Some((110u8, 110u8, 110u8))
                } else {
                    Some(css_to_rgb(fill_str))
                };
                let has_fill = fill_rgb.is_some();

                if !has_stroke && !has_fill {
                    return;
                }

                // Fill-only paths (Sankey bands, chord ribbons, pie slices, etc.)
                // are scanline-filled in the braille dot grid, giving them a solid
                // shaded interior rather than just their outline edges.
                if has_fill && !has_stroke {
                    let rgb = fill_rgb.unwrap();
                    let mut poly: Vec<(f64, f64)> = Vec::new();
                    let mut cur = (tx, ty);
                    let mut start = cur;
                    for cmd in parse_path(d) {
                        match cmd {
                            PathCmd::MoveTo(x, y) => {
                                if poly.len() >= 3 {
                                    self.fill_braille_polygon(&poly, rgb);
                                    poly.clear();
                                }
                                cur = (x + tx, y + ty);
                                start = cur;
                                poly.push(cur);
                            }
                            PathCmd::LineTo(x, y) => {
                                cur = (x + tx, y + ty);
                                poly.push(cur);
                            }
                            PathCmd::CubicTo(x1, y1, x2, y2, x, y) => {
                                let p1 = (x1 + tx, y1 + ty);
                                let p2 = (x2 + tx, y2 + ty);
                                let p3 = (x + tx, y + ty);
                                let pts = Self::tessellate_cubic(cur, p1, p2, p3);
                                poly.extend_from_slice(&pts[1..]);
                                cur = p3;
                            }
                            PathCmd::Arc { rx, ry, x_rot, large_arc, sweep, x, y } => {
                                let end = (x + tx, y + ty);
                                let pts = Self::tessellate_arc(cur, (rx, ry), x_rot, large_arc, sweep, end);
                                poly.extend_from_slice(&pts[1..]);
                                cur = end;
                            }
                            PathCmd::ClosePath => {
                                poly.push(start);
                                cur = start;
                            }
                        }
                    }
                    if poly.len() >= 3 {
                        self.fill_braille_polygon(&poly, rgb);
                    }
                    return;
                }

                // Stroked paths — draw outline with Bresenham as before.
                let rgb = if has_stroke {
                    css_to_rgb(stroke)
                } else {
                    fill_rgb.unwrap()
                };
                let mut cur = (tx, ty);
                let mut start = cur;
                for cmd in parse_path(d) {
                    match cmd {
                        PathCmd::MoveTo(x, y) => {
                            cur = (x + tx, y + ty);
                            start = cur;
                        }
                        PathCmd::LineTo(x, y) => {
                            let next = (x + tx, y + ty);
                            self.bresenham(
                                self.to_bx(cur.0),
                                self.to_by(cur.1),
                                self.to_bx(next.0),
                                self.to_by(next.1),
                                rgb,
                            );
                            cur = next;
                        }
                        PathCmd::CubicTo(x1, y1, x2, y2, x, y) => {
                            let p1 = (x1 + tx, y1 + ty);
                            let p2 = (x2 + tx, y2 + ty);
                            let p3 = (x + tx, y + ty);
                            let pts = Self::tessellate_cubic(cur, p1, p2, p3);
                            for w in pts.windows(2) {
                                self.bresenham(
                                    self.to_bx(w[0].0),
                                    self.to_by(w[0].1),
                                    self.to_bx(w[1].0),
                                    self.to_by(w[1].1),
                                    rgb,
                                );
                            }
                            cur = (x + tx, y + ty);
                        }
                        PathCmd::Arc { rx, ry, x_rot, large_arc, sweep, x, y } => {
                            let end = (x + tx, y + ty);
                            let pts = Self::tessellate_arc(cur, (rx, ry), x_rot, large_arc, sweep, end);
                            for w in pts.windows(2) {
                                self.bresenham(
                                    self.to_bx(w[0].0),
                                    self.to_by(w[0].1),
                                    self.to_bx(w[1].0),
                                    self.to_by(w[1].1),
                                    rgb,
                                );
                            }
                            cur = end;
                        }
                        PathCmd::ClosePath => {
                            self.bresenham(
                                self.to_bx(cur.0),
                                self.to_by(cur.1),
                                self.to_bx(start.0),
                                self.to_by(start.1),
                                rgb,
                            );
                            cur = start;
                        }
                    }
                }
            }

            Primitive::Rect { x, y, width, height, fill, .. } => {
                if fill == "none" {
                    return;
                }
                let rgb = css_to_rgb(fill);
                let x_s = x + tx;
                let y_s = y + ty;
                let width = *width;
                let height = *height;
                // When a rect fits within one cell in a dimension, snap to the
                // centre so that small swatches (e.g. legend colour boxes) always
                // occupy exactly 1 cell and align with their text label, rather
                // than straddling a boundary and appearing 1 or 2 cells tall/wide
                // non-deterministically.
                let cell_w = self.scene_width / self.cols as f64;
                let cell_h = self.scene_height / self.rows as f64;
                let (cx0, cx1) = if width < cell_w {
                    let c = self.to_cx(x_s + width * 0.5)
                        .max(0)
                        .min(self.cols as isize - 1);
                    (c, c)
                } else {
                    (
                        self.to_cx(x_s).max(0),
                        self.to_cx(x_s + width).min(self.cols as isize - 1),
                    )
                };
                // Also snap when height is small in absolute SVG pixels (≤16 px
                // covers legend swatches at 12 px regardless of terminal size).
                let (cy0, cy1) = if height < cell_h.max(16.0) {
                    let r = self.to_cy(y_s + height * 0.5)
                        .max(0)
                        .min(self.rows as isize - 1);
                    (r, r)
                } else {
                    (
                        self.to_cy(y_s).max(0),
                        self.to_cy(y_s + height).min(self.rows as isize - 1),
                    )
                };
                for col in cx0..=cx1 {
                    for row in cy0..=cy1 {
                        self.set_char(col, row, '█', rgb);
                    }
                }
            }

            Primitive::Text { x, y, content, anchor, rotate, .. } => {
                let rgb = self.text_color;
                let x_s = x + tx;
                let y_s = y + ty;
                let row = self.to_cy(y_s);
                let chars: Vec<char> = content.chars().collect();
                let len = chars.len() as isize;

                let angle = rotate.unwrap_or(0.0);
                let abs_angle = angle.abs() % 180.0;

                if abs_angle > 45.0 && abs_angle < 135.0 {
                    // ~90°: sideways (y-axis label). Cannot rotate in terminal —
                    // pin to the left edge so the full string is visible.
                    for (i, ch) in chars.iter().enumerate() {
                        self.set_char(i as isize, row, *ch, rgb);
                    }
                } else {
                    let col = self.to_cx(x_s);
                    if abs_angle >= 15.0 {
                        // Rotated tick labels (e.g. -45° category / chromosome names).
                        // Left-justified: first character at the tick column so it's
                        // clear which bar/column the label belongs to.  Step down one
                        // row at a time when the target row is occupied, guaranteeing
                        // labels stack cleanly with ≥1 space gap between neighbours.
                        let start_col = col;
                        let mut draw_row = row;
                        while (draw_row as usize) < self.rows {
                            // Check [start_col-1 .. start_col+len]: the cell before
                            // and after are included so adjacent labels always have
                            // at least one blank column between them.
                            let collides = (0..len + 2).any(|i| {
                                let c = start_col - 1 + i;
                                c >= 0 && (c as usize) < self.cols
                                    && self.char_grid[draw_row as usize][c as usize].is_some()
                            });
                            if !collides { break; }
                            draw_row += 1;
                        }
                        for (i, ch) in chars.iter().enumerate() {
                            self.set_char(start_col + i as isize, draw_row, *ch, rgb);
                        }
                    } else {
                        // Unrotated text (legend labels, tick values, titles, axis
                        // labels).  Place directly at the computed position — no
                        // collision stacking.  The swatch █ immediately to the left
                        // of a legend label must not be treated as a collision.
                        let start_col = match anchor {
                            TextAnchor::Start  => col,
                            TextAnchor::Middle => col - len / 2,
                            TextAnchor::End    => col - len,
                        };
                        for (i, ch) in chars.iter().enumerate() {
                            self.set_char(start_col + i as isize, row, *ch, rgb);
                        }
                    }
                }
            }

            Primitive::GroupStart { transform } => {
                let offset = transform
                    .as_ref()
                    .map(|t| parse_translate(t))
                    .unwrap_or((0.0, 0.0));
                self.transform_stack.push(offset);
            }

            Primitive::GroupEnd => {
                if self.transform_stack.len() > 1 {
                    self.transform_stack.pop();
                }
            }
        }
    }

    // ── ANSI string output ───────────────────────────────────────────────────

    fn to_ansi_string(&self) -> String {
        let mut out = String::new();
        for row in 0..self.rows {
            let mut prev_color: Option<(u8, u8, u8)> = None;
            for col in 0..self.cols {
                // Layer 1: char_grid (rects + text)
                if let Some((ch, rgb)) = self.char_grid[row][col] {
                    emit_color(&mut out, &mut prev_color, rgb);
                    out.push(ch);
                // Layer 2: box-drawing lines (axes, ticks)
                } else if self.line_char_bits[row][col] != 0 {
                    let bits = self.line_char_bits[row][col];
                    let rgb = self.line_char_color[row][col].unwrap_or((200, 200, 200));
                    emit_color(&mut out, &mut prev_color, rgb);
                    out.push(bitmask_to_char(bits));
                // Layer 3: braille dots (circles, diagonal paths)
                } else if self.braille[row][col] != 0 {
                    let mask = self.braille[row][col];
                    let rgb = self.braille_color[row][col].unwrap_or((200, 200, 200));
                    emit_color(&mut out, &mut prev_color, rgb);
                    out.push(char::from_u32(0x2800 + mask as u32).unwrap_or(' '));
                // Empty cell
                } else {
                    if prev_color.is_some() {
                        out.push_str("\x1b[0m");
                        prev_color = None;
                    }
                    out.push(' ');
                }
            }
            if prev_color.is_some() {
                out.push_str("\x1b[0m");
            }
            out.push('\n');
        }
        out.push_str("\x1b[0m");
        out
    }
}

fn emit_color(out: &mut String, prev: &mut Option<(u8, u8, u8)>, rgb: (u8, u8, u8)) {
    if *prev != Some(rgb) {
        out.push_str(&format!("\x1b[38;2;{};{};{}m", rgb.0, rgb.1, rgb.2));
        *prev = Some(rgb);
    }
}

// ── SVG path parsing ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
enum PathCmd {
    MoveTo(f64, f64),
    LineTo(f64, f64),
    CubicTo(f64, f64, f64, f64, f64, f64),
    /// SVG arc: radii (rx,ry), x-axis rotation (deg), large-arc flag, sweep flag, endpoint (x,y).
    Arc { rx: f64, ry: f64, x_rot: f64, large_arc: bool, sweep: bool, x: f64, y: f64 },
    ClosePath,
}

#[derive(Debug, Clone, Copy)]
enum Token {
    Cmd(char),
    Num(f64),
}

fn tokenize_path(d: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = d.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_whitespace() || c == ',' {
            chars.next();
        } else if c.is_ascii_alphabetic() {
            tokens.push(Token::Cmd(c));
            chars.next();
        } else if c == '-' || c == '+' || c.is_ascii_digit() || c == '.' {
            let mut s = String::new();
            if c == '-' || c == '+' {
                s.push(chars.next().unwrap());
            }
            while chars.peek().map(|&x| x.is_ascii_digit()).unwrap_or(false) {
                s.push(chars.next().unwrap());
            }
            if chars.peek() == Some(&'.') {
                s.push(chars.next().unwrap());
                while chars.peek().map(|&x| x.is_ascii_digit()).unwrap_or(false) {
                    s.push(chars.next().unwrap());
                }
            }
            if chars.peek().map(|&x| x == 'e' || x == 'E').unwrap_or(false) {
                s.push(chars.next().unwrap());
                if chars.peek().map(|&x| x == '+' || x == '-').unwrap_or(false) {
                    s.push(chars.next().unwrap());
                }
                while chars.peek().map(|&x| x.is_ascii_digit()).unwrap_or(false) {
                    s.push(chars.next().unwrap());
                }
            }
            if let Ok(n) = s.parse::<f64>() {
                tokens.push(Token::Num(n));
            }
        } else {
            chars.next();
        }
    }
    tokens
}

struct TokenStream {
    tokens: Vec<Token>,
    pos: usize,
}

impl TokenStream {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek_cmd(&self) -> Option<char> {
        match self.tokens.get(self.pos) {
            Some(Token::Cmd(c)) => Some(*c),
            _ => None,
        }
    }

    fn consume_cmd(&mut self) {
        self.pos += 1;
    }

    fn next_num(&mut self) -> Option<f64> {
        match self.tokens.get(self.pos) {
            Some(Token::Num(n)) => {
                let n = *n;
                self.pos += 1;
                Some(n)
            }
            _ => None,
        }
    }

    fn is_at_num(&self) -> bool {
        matches!(self.tokens.get(self.pos), Some(Token::Num(_)))
    }

    fn is_empty(&self) -> bool {
        self.pos >= self.tokens.len()
    }
}

fn parse_path(d: &str) -> Vec<PathCmd> {
    let mut ts = TokenStream::new(tokenize_path(d));
    let mut cmds = Vec::new();
    let mut cur_cmd = 'M';
    let mut cur_x = 0.0_f64;
    let mut cur_y = 0.0_f64;
    let mut start_x = 0.0_f64;
    let mut start_y = 0.0_f64;

    while !ts.is_empty() {
        if let Some(c) = ts.peek_cmd() {
            cur_cmd = c;
            ts.consume_cmd();
        }

        let consumed = match cur_cmd {
            'M' => match (ts.next_num(), ts.next_num()) {
                (Some(x), Some(y)) => {
                    cmds.push(PathCmd::MoveTo(x, y));
                    cur_x = x; cur_y = y; start_x = x; start_y = y;
                    cur_cmd = 'L';
                    true
                }
                _ => false,
            },
            'm' => match (ts.next_num(), ts.next_num()) {
                (Some(dx), Some(dy)) => {
                    cur_x += dx; cur_y += dy;
                    cmds.push(PathCmd::MoveTo(cur_x, cur_y));
                    start_x = cur_x; start_y = cur_y;
                    cur_cmd = 'l';
                    true
                }
                _ => false,
            },
            'L' => match (ts.next_num(), ts.next_num()) {
                (Some(x), Some(y)) => {
                    cmds.push(PathCmd::LineTo(x, y));
                    cur_x = x; cur_y = y;
                    true
                }
                _ => false,
            },
            'l' => match (ts.next_num(), ts.next_num()) {
                (Some(dx), Some(dy)) => {
                    cur_x += dx; cur_y += dy;
                    cmds.push(PathCmd::LineTo(cur_x, cur_y));
                    true
                }
                _ => false,
            },
            'H' => match ts.next_num() {
                Some(x) => { cmds.push(PathCmd::LineTo(x, cur_y)); cur_x = x; true }
                None => false,
            },
            'h' => match ts.next_num() {
                Some(dx) => { cur_x += dx; cmds.push(PathCmd::LineTo(cur_x, cur_y)); true }
                None => false,
            },
            'V' => match ts.next_num() {
                Some(y) => { cmds.push(PathCmd::LineTo(cur_x, y)); cur_y = y; true }
                None => false,
            },
            'v' => match ts.next_num() {
                Some(dy) => { cur_y += dy; cmds.push(PathCmd::LineTo(cur_x, cur_y)); true }
                None => false,
            },
            'C' => match (
                ts.next_num(), ts.next_num(), ts.next_num(),
                ts.next_num(), ts.next_num(), ts.next_num(),
            ) {
                (Some(x1), Some(y1), Some(x2), Some(y2), Some(x), Some(y)) => {
                    cmds.push(PathCmd::CubicTo(x1, y1, x2, y2, x, y));
                    cur_x = x; cur_y = y;
                    true
                }
                _ => false,
            },
            'c' => match (
                ts.next_num(), ts.next_num(), ts.next_num(),
                ts.next_num(), ts.next_num(), ts.next_num(),
            ) {
                (Some(dx1), Some(dy1), Some(dx2), Some(dy2), Some(dx), Some(dy)) => {
                    cmds.push(PathCmd::CubicTo(
                        cur_x + dx1, cur_y + dy1,
                        cur_x + dx2, cur_y + dy2,
                        cur_x + dx,  cur_y + dy,
                    ));
                    cur_x += dx; cur_y += dy;
                    true
                }
                _ => false,
            },
            'Z' | 'z' => {
                cmds.push(PathCmd::ClosePath);
                cur_x = start_x; cur_y = start_y;
                true
            }
            'A' => match (
                ts.next_num(), ts.next_num(), ts.next_num(),
                ts.next_num(), ts.next_num(), ts.next_num(), ts.next_num(),
            ) {
                (Some(rx), Some(ry), Some(x_rot), Some(la), Some(sw), Some(x), Some(y)) => {
                    cmds.push(PathCmd::Arc { rx, ry, x_rot, large_arc: la != 0.0, sweep: sw != 0.0, x, y });
                    cur_x = x; cur_y = y;
                    true
                }
                _ => false,
            },
            'a' => match (
                ts.next_num(), ts.next_num(), ts.next_num(),
                ts.next_num(), ts.next_num(), ts.next_num(), ts.next_num(),
            ) {
                (Some(rx), Some(ry), Some(x_rot), Some(la), Some(sw), Some(dx), Some(dy)) => {
                    cur_x += dx; cur_y += dy;
                    cmds.push(PathCmd::Arc { rx, ry, x_rot, large_arc: la != 0.0, sweep: sw != 0.0, x: cur_x, y: cur_y });
                    true
                }
                _ => false,
            },
            'S' | 's' | 'Q' | 'q' => {
                let mut n = 0;
                while ts.is_at_num() && n < 4 { ts.next_num(); n += 1; }
                n > 0
            }
            'T' | 't' => {
                let mut n = 0;
                while ts.is_at_num() && n < 2 { ts.next_num(); n += 1; }
                n > 0
            }
            _ => { ts.pos += 1; true }
        };

        if !consumed {
            break;
        }
    }
    cmds
}

// ── Transform parsing ─────────────────────────────────────────────────────────

fn parse_translate(t: &str) -> (f64, f64) {
    let inner = t
        .trim()
        .strip_prefix("translate(")
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or("");
    let nums: Vec<f64> = inner
        .split(|c: char| c == ',' || c.is_ascii_whitespace())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect();
    match nums.as_slice() {
        [x, y, ..] => (*x, *y),
        [x]        => (*x, 0.0),
        _          => (0.0, 0.0),
    }
}

// ── CSS colour parsing ────────────────────────────────────────────────────────

fn css_to_rgb(s: &str) -> (u8, u8, u8) {
    let s = s.trim();
    if s.is_empty() || s.eq_ignore_ascii_case("none") || s.eq_ignore_ascii_case("transparent") {
        return (128, 128, 128);
    }
    let sl = s.to_ascii_lowercase();
    // rgb(r, g, b)
    if let Some(inner) = sl.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3 {
            let r = parts[0].trim().parse::<f64>().unwrap_or(0.0).round() as u8;
            let g = parts[1].trim().parse::<f64>().unwrap_or(0.0).round() as u8;
            let b = parts[2].trim().parse::<f64>().unwrap_or(0.0).round() as u8;
            return (r, g, b);
        }
    }
    // #RRGGBB
    if s.starts_with('#') && s.len() == 7 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&s[1..3], 16),
            u8::from_str_radix(&s[3..5], 16),
            u8::from_str_radix(&s[5..7], 16),
        ) {
            return (r, g, b);
        }
    }
    // #RGB shorthand
    if s.starts_with('#') && s.len() == 4 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&s[1..2], 16),
            u8::from_str_radix(&s[2..3], 16),
            u8::from_str_radix(&s[3..4], 16),
        ) {
            return (r * 17, g * 17, b * 17);
        }
    }
    named_color(&sl)
}

fn named_color(s: &str) -> (u8, u8, u8) {
    match s {
        "black"                             => (0, 0, 0),
        "white"                             => (255, 255, 255),
        "red"                               => (255, 0, 0),
        "green"                             => (0, 128, 0),
        "lime"                              => (0, 255, 0),
        "blue"                              => (0, 0, 255),
        "gray"  | "grey"                    => (128, 128, 128),
        "lightgray" | "lightgrey"           => (211, 211, 211),
        "darkgray"  | "darkgrey"            => (169, 169, 169),
        "silver"                            => (192, 192, 192),
        "steelblue"                         => (70, 130, 180),
        "orange"                            => (255, 165, 0),
        "darkorange"                        => (255, 140, 0),
        "purple"                            => (128, 0, 128),
        "yellow"                            => (255, 255, 0),
        "cyan"  | "aqua"                    => (0, 255, 255),
        "magenta" | "fuchsia"               => (255, 0, 255),
        "darkred"                           => (139, 0, 0),
        "darkgreen"                         => (0, 100, 0),
        "darkblue"                          => (0, 0, 139),
        "salmon"                            => (250, 128, 114),
        "teal"                              => (0, 128, 128),
        "coral"                             => (255, 127, 80),
        "indigo"                            => (75, 0, 130),
        "pink"                              => (255, 192, 203),
        "hotpink"                           => (255, 105, 180),
        "gold"                              => (255, 215, 0),
        "olive"                             => (128, 128, 0),
        "navy"                              => (0, 0, 128),
        "maroon"                            => (128, 0, 0),
        "crimson"                           => (220, 20, 60),
        "tomato"                            => (255, 99, 71),
        "chocolate"                         => (210, 105, 30),
        "sienna"                            => (160, 82, 45),
        "tan"                               => (210, 180, 140),
        "khaki"                             => (240, 230, 140),
        "limegreen"                         => (50, 205, 50),
        "forestgreen"                       => (34, 139, 34),
        "seagreen"                          => (46, 139, 87),
        "darkturquoise"                     => (0, 206, 209),
        "royalblue"                         => (65, 105, 225),
        "slateblue"                         => (106, 90, 205),
        "mediumpurple"                      => (147, 112, 219),
        "orchid"                            => (218, 112, 214),
        "plum"                              => (221, 160, 221),
        "violet"                            => (238, 130, 238),
        "deeppink"                          => (255, 20, 147),
        "orangered"                         => (255, 69, 0),
        "firebrick"                         => (178, 34, 34),
        "brown"                             => (165, 42, 42),
        "saddlebrown"                       => (139, 69, 19),
        "slategray"  | "slategrey"          => (112, 128, 144),
        "darkslategray" | "darkslategrey"   => (47, 79, 79),
        _                                   => (150, 150, 150),
    }
}
