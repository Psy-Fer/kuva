//! Direct raster backend: Scene primitives → RGBA pixel buffer → PNG bytes.
//!
//! No SVG serialization round-trip. Geometry is rasterized directly:
//!   - Thin lines via Xiaolin Wu's AA algorithm
//!   - Thick lines via perpendicular quad tessellation + round caps
//!   - Circles via signed-distance-field coverage (1px AA ramp)
//!   - CircleBatch via precomputed per-radius mask (hot path for scatter)
//!   - Paths via SVG parser → adaptive bezier tessellation → scanline fill
//!   - SVG arcs converted to cubic beziers (SVG 1.1 spec, Appendix F)
//!   - Text via fontdue with glyph caching; rotated text via offscreen blit
//!   - Clip rects and translate transforms fully respected

use std::collections::HashMap;
use std::sync::OnceLock;

use fontdue::{Font, FontSettings, Metrics};

use crate::render::color::Color as KColor;
use crate::render::render::{Primitive, Scene, TextAnchor};

// ── RGBA ──────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
struct Rgba {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Rgba {
    #[inline]
    fn opaque(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    #[inline]
    fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }
}

// ── Clip rect (half-open [x0,x1) × [y0,y1) in integer pixel space) ───────────

#[derive(Clone, Copy, Debug)]
struct Clip {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
}

impl Clip {
    fn full(w: u32, h: u32) -> Self {
        Self {
            x0: 0,
            y0: 0,
            x1: w as i32,
            y1: h as i32,
        }
    }

    fn intersect(self, other: Clip) -> Self {
        Self {
            x0: self.x0.max(other.x0),
            y0: self.y0.max(other.y0),
            x1: self.x1.min(other.x1),
            y1: self.y1.min(other.y1),
        }
    }
}

// ── Canvas ────────────────────────────────────────────────────────────────────

struct Canvas {
    width: u32,
    height: u32,
    pixels: Vec<u8>, // RGBA row-major
}

impl Canvas {
    fn new(w: u32, h: u32) -> Self {
        Self {
            width: w,
            height: h,
            pixels: vec![0u8; w as usize * h as usize * 4],
        }
    }

    fn fill_background(&mut self, c: Rgba) {
        for p in self.pixels.chunks_exact_mut(4) {
            p[0] = c.r;
            p[1] = c.g;
            p[2] = c.b;
            p[3] = c.a;
        }
    }

    /// Composite `color` at `cov` coverage over the pixel at (x, y).
    /// The background is always opaque after `fill_background`, so dst_a == 255
    /// always, collapsing the Porter-Duff "src over" formula to:
    ///   out_rgb = src_rgb * eff_a + dst_rgb * (1 - eff_a)
    #[inline(always)]
    fn blend(&mut self, x: i32, y: i32, c: Rgba, cov: f32) {
        if (x as u32) >= self.width || (y as u32) >= self.height {
            return;
        }
        let eff = (cov * c.a as f32 + 0.5) as u32;
        if eff == 0 {
            return;
        }
        let eff = eff.min(255);
        let idx = (y as usize * self.width as usize + x as usize) * 4;
        let p = &mut self.pixels[idx..idx + 4];
        if eff == 255 {
            p[0] = c.r;
            p[1] = c.g;
            p[2] = c.b;
            p[3] = 255;
        } else {
            let inv = 255 - eff;
            p[0] = ((c.r as u32 * eff + p[0] as u32 * inv) / 255) as u8;
            p[1] = ((c.g as u32 * eff + p[1] as u32 * inv) / 255) as u8;
            p[2] = ((c.b as u32 * eff + p[2] as u32 * inv) / 255) as u8;
            p[3] = 255;
        }
    }

    #[inline(always)]
    fn blend_c(&mut self, x: i32, y: i32, c: Rgba, cov: f32, clip: Clip) {
        if x >= clip.x0 && x < clip.x1 && y >= clip.y0 && y < clip.y1 {
            self.blend(x, y, c, cov);
        }
    }

    // ── Rectangle fill ────────────────────────────────────────────────────────

    fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: Rgba, clip: Clip) {
        let x0 = (x.floor() as i32).max(clip.x0);
        let y0 = (y.floor() as i32).max(clip.y0);
        let x1 = ((x + w).ceil() as i32).min(clip.x1);
        let y1 = ((y + h).ceil() as i32).min(clip.y1);
        for py in y0..y1 {
            let yc = partial_cov(y, y + h, py);
            if yc <= 0.0 {
                continue;
            }
            for px in x0..x1 {
                let xc = partial_cov(x, x + w, px);
                if xc > 0.0 {
                    self.blend(px, py, c, xc * yc);
                }
            }
        }
    }

    // ── Xiaolin Wu anti-aliased thin line ─────────────────────────────────────
    //
    // For width ≤ 1.5px. Produces two blended pixels per step along the major
    // axis; coverage is the fractional distance from each pixel's integer boundary.

    fn draw_line_wu(
        &mut self,
        mut x0: f32,
        mut y0: f32,
        mut x1: f32,
        mut y1: f32,
        c: Rgba,
        clip: Clip,
    ) {
        let steep = (y1 - y0).abs() > (x1 - x0).abs();
        if steep {
            std::mem::swap(&mut x0, &mut y0);
            std::mem::swap(&mut x1, &mut y1);
        }
        if x0 > x1 {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
        }

        let dx = x1 - x0;
        let dy = y1 - y0;
        let grad = if dx.abs() < 1e-6 { 1.0f32 } else { dy / dx };

        // First endpoint
        let xend = x0.round();
        let yend = y0 + grad * (xend - x0);
        let xgap = 1.0 - (x0 + 0.5).fract();
        let ix0 = xend as i32;
        let iy0 = yend.floor() as i32;
        let frac = yend.fract();
        if steep {
            self.blend_c(iy0, ix0, c, (1.0 - frac) * xgap, clip);
            self.blend_c(iy0 + 1, ix0, c, frac * xgap, clip);
        } else {
            self.blend_c(ix0, iy0, c, (1.0 - frac) * xgap, clip);
            self.blend_c(ix0, iy0 + 1, c, frac * xgap, clip);
        }
        let mut intery = yend + grad;

        // Second endpoint
        let xend2 = x1.round();
        let yend2 = y1 + grad * (xend2 - x1);
        let xgap2 = (x1 + 0.5).fract();
        let ix1 = xend2 as i32;
        let iy1 = yend2.floor() as i32;
        let frac2 = yend2.fract();
        if steep {
            self.blend_c(iy1, ix1, c, (1.0 - frac2) * xgap2, clip);
            self.blend_c(iy1 + 1, ix1, c, frac2 * xgap2, clip);
        } else {
            self.blend_c(ix1, iy1, c, (1.0 - frac2) * xgap2, clip);
            self.blend_c(ix1, iy1 + 1, c, frac2 * xgap2, clip);
        }

        // Interior
        for ix in (ix0 + 1)..ix1 {
            let iy = intery.floor() as i32;
            let f = intery.fract();
            if steep {
                self.blend_c(iy, ix, c, 1.0 - f, clip);
                self.blend_c(iy + 1, ix, c, f, clip);
            } else {
                self.blend_c(ix, iy, c, 1.0 - f, clip);
                self.blend_c(ix, iy + 1, c, f, clip);
            }
            intery += grad;
        }
    }

    // ── Thick line via parallelogram + round caps ─────────────────────────────

    #[allow(clippy::too_many_arguments)]
    fn draw_line_thick(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        c: Rgba,
        width: f32,
        clip: Clip,
    ) {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.01 {
            self.fill_circle_aa(x0, y0, width * 0.5, c, clip);
            return;
        }
        let hw = width * 0.5;
        let px = -dy / len * hw;
        let py = dx / len * hw;
        self.fill_polygon(
            &[
                (x0 + px, y0 + py),
                (x0 - px, y0 - py),
                (x1 - px, y1 - py),
                (x1 + px, y1 + py),
            ],
            c,
            clip,
        );
        self.fill_circle_aa(x0, y0, hw, c, clip);
        self.fill_circle_aa(x1, y1, hw, c, clip);
    }

    // ── Line dispatcher ───────────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    fn draw_line(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        c: Rgba,
        width: f32,
        dasharray: Option<&str>,
        clip: Clip,
    ) {
        if let Some(dash) = dasharray.filter(|s| !s.is_empty()) {
            self.draw_dashed_line(x0, y0, x1, y1, c, width, dash, clip);
        } else if width <= 1.5 {
            self.draw_line_wu(x0, y0, x1, y1, c, clip);
        } else {
            self.draw_line_thick(x0, y0, x1, y1, c, width, clip);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_dashed_line(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        c: Rgba,
        width: f32,
        dasharray: &str,
        clip: Clip,
    ) {
        let dashes = parse_dasharray(dasharray);
        if dashes.is_empty() {
            self.draw_line(x0, y0, x1, y1, c, width, None, clip);
            return;
        }
        let dx = x1 - x0;
        let dy = y1 - y0;
        let total = (dx * dx + dy * dy).sqrt();
        if total < 0.01 {
            return;
        }
        let ux = dx / total;
        let uy = dy / total;
        let mut pos = 0.0f32;
        let mut di = 0usize;
        let mut drawing = true;
        while pos < total {
            let seg = dashes[di % dashes.len()];
            let end = (pos + seg).min(total);
            if drawing {
                let sx = x0 + ux * pos;
                let sy = y0 + uy * pos;
                let ex = x0 + ux * end;
                let ey = y0 + uy * end;
                if width <= 1.5 {
                    self.draw_line_wu(sx, sy, ex, ey, c, clip);
                } else {
                    self.draw_line_thick(sx, sy, ex, ey, c, width, clip);
                }
            }
            pos = end;
            di += 1;
            drawing = !drawing;
        }
    }

    // ── Stadium (capsule) fill ────────────────────────────────────────────────
    //
    // Fills the set of all pixels within `hw` of the line segment p0→p1.
    // This is the exact shape of a thick stroke segment: a rectangle body
    // with semicircular caps at each end.
    //
    // For each scanline, the x span is computed analytically as the union of:
    //   1. The semicircle cap at p0: x ∈ [x0 ± sqrt(hw² - (fy-y0)²)]
    //   2. The semicircle cap at p1: x ∈ [x1 ± sqrt(hw² - (fy-y1)²)]
    //   3. The rectangle body: where the perpendicular distance to the segment
    //      line is ≤ hw AND the foot-of-perpendicular falls within [p0, p1].
    //
    // The stadium is always convex, so there are no self-intersection issues.
    // When successive stadiums are rendered for a polyline, their caps overlap
    // at each vertex and naturally form correct round joins without any join
    // geometry construction.
    fn fill_stadium(&mut self, p0: (f32, f32), p1: (f32, f32), hw: f32, c: Rgba, clip: Clip) {
        let (x0, y0) = p0;
        let (x1, y1) = p1;
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len_sq = dx * dx + dy * dy;

        if len_sq < 0.01 {
            self.fill_circle_aa(x0, y0, hw, c, clip);
            return;
        }

        let len = len_sq.sqrt();
        let hw_sq = hw * hw;

        let y_lo = y0.min(y1) - hw;
        let y_hi = y0.max(y1) + hw;
        let py_start = (y_lo.floor() as i32).max(clip.y0);
        let py_end = (y_hi.ceil() as i32).min(clip.y1 - 1);

        for py in py_start..=py_end {
            let fy = py as f32 + 0.5;

            let mut x_lo = f32::INFINITY;
            let mut x_hi = f32::NEG_INFINITY;

            // Cap at p0
            let dy0 = fy - y0;
            let rem0 = hw_sq - dy0 * dy0;
            if rem0 >= 0.0 {
                let r0 = rem0.sqrt();
                x_lo = x_lo.min(x0 - r0);
                x_hi = x_hi.max(x0 + r0);
            }

            // Cap at p1
            let dy1 = fy - y1;
            let rem1 = hw_sq - dy1 * dy1;
            if rem1 >= 0.0 {
                let r1 = rem1.sqrt();
                x_lo = x_lo.min(x1 - r1);
                x_hi = x_hi.max(x1 + r1);
            }

            // Body rectangle: perpendicular distance to segment line ≤ hw,
            // intersected with the projection band t ∈ [0, 1].
            if dy.abs() > 1e-6 {
                // Perpendicular condition: |P(x)| ≤ hw*len where P(x) = -dy*(x-x0) + dx*(fy-y0)
                let base = dx * (fy - y0);
                let a = x0 + (base - hw * len) / dy;
                let b = x0 + (base + hw * len) / dy;
                let (perp_lo, perp_hi) = if a <= b { (a, b) } else { (b, a) };

                // Projection condition: t = (dx*(x-x0) + dy*(fy-y0)) / len_sq ∈ [0, 1]
                let proj_off = -dy * (fy - y0); // = dx*(x-x0) at t=0
                let (proj_lo, proj_hi) = if dx > 1e-6 {
                    (x0 + proj_off / dx, x0 + (proj_off + len_sq) / dx)
                } else if dx < -1e-6 {
                    (x0 + (proj_off + len_sq) / dx, x0 + proj_off / dx)
                } else {
                    // Vertical: t depends only on y; x is unconstrained within segment
                    let t = (fy - y0) / dy;
                    if (0.0..=1.0).contains(&t) {
                        (f32::NEG_INFINITY, f32::INFINITY)
                    } else {
                        (f32::INFINITY, f32::NEG_INFINITY)
                    }
                };

                let body_lo = perp_lo.max(proj_lo);
                let body_hi = perp_hi.min(proj_hi);
                if body_lo < body_hi {
                    x_lo = x_lo.min(body_lo);
                    x_hi = x_hi.max(body_hi);
                }
            } else {
                // Near-horizontal: perpendicular distance ≈ |fy - y0|, x unconstrained
                if (fy - y0).abs() <= hw {
                    x_lo = x_lo.min(x0.min(x1));
                    x_hi = x_hi.max(x0.max(x1));
                }
            }

            if x_lo < x_hi {
                self.fill_span_aa(x_lo, x_hi, py, c, clip);
            }
        }
    }

    // ── AA circle fill — SDF coverage ─────────────────────────────────────────
    //
    // For each pixel, coverage = clamp(r_outer - dist_to_center, 0, 1) where
    // r_outer = r + 0.5. This gives a 1-pixel-wide AA fringe. Pixels fully
    // inside r - 0.5 receive coverage = 1 without a sqrt call.

    fn fill_circle_aa(&mut self, cx: f32, cy: f32, r: f32, c: Rgba, clip: Clip) {
        let r_outer = r + 0.5;
        let r_inner = (r - 0.5).max(0.0);
        let r_outer2 = r_outer * r_outer;
        let r_inner2 = r_inner * r_inner;

        let x0 = ((cx - r_outer).floor() as i32).max(clip.x0);
        let x1 = ((cx + r_outer).ceil() as i32).min(clip.x1 - 1);
        let y0 = ((cy - r_outer).floor() as i32).max(clip.y0);
        let y1 = ((cy + r_outer).ceil() as i32).min(clip.y1 - 1);

        for py in y0..=y1 {
            let dy = py as f32 + 0.5 - cy;
            let dy2 = dy * dy;
            if dy2 >= r_outer2 {
                continue;
            }
            for px in x0..=x1 {
                let dx = px as f32 + 0.5 - cx;
                let d2 = dx * dx + dy2;
                let cov = if d2 <= r_inner2 {
                    1.0
                } else if d2 >= r_outer2 {
                    continue;
                } else {
                    r_outer - d2.sqrt()
                };
                self.blend(px, py, c, cov);
            }
        }
    }

    /// Annular stroke for a circle. Each pixel's coverage is the minimum of
    /// its inward and outward distances to the band edges, giving smooth AA on
    /// both the inner and outer boundaries.
    fn stroke_circle_aa(&mut self, cx: f32, cy: f32, r: f32, c: Rgba, sw: f32, clip: Clip) {
        let hw = sw * 0.5;
        let r_outer = r + hw + 0.5;
        let r_inner = (r - hw - 0.5).max(0.0);
        let outer2 = r_outer * r_outer;
        let inner2 = r_inner * r_inner;
        let band_outer = r + hw; // nominal outer edge
        let band_inner = (r - hw).max(0.0); // nominal inner edge

        let x0 = ((cx - r_outer).floor() as i32).max(clip.x0);
        let x1 = ((cx + r_outer).ceil() as i32).min(clip.x1 - 1);
        let y0 = ((cy - r_outer).floor() as i32).max(clip.y0);
        let y1 = ((cy + r_outer).ceil() as i32).min(clip.y1 - 1);

        for py in y0..=y1 {
            let dy = py as f32 + 0.5 - cy;
            let dy2 = dy * dy;
            if dy2 >= outer2 {
                continue;
            }
            for px in x0..=x1 {
                let dx = px as f32 + 0.5 - cx;
                let d2 = dx * dx + dy2;
                if d2 < inner2 || d2 > outer2 {
                    continue;
                }
                let d = d2.sqrt();
                let out_cov = ((band_outer + 0.5) - d).clamp(0.0, 1.0);
                let in_cov = (d - (band_inner - 0.5)).clamp(0.0, 1.0);
                let cov = out_cov.min(in_cov);
                if cov > 0.0 {
                    self.blend(px, py, c, cov);
                }
            }
        }
    }

    // ── Precomputed circle mask ────────────────────────────────────────────────
    //
    // Coverage values stored as u8 (0-255). The mask is (2*half+1)² pixels,
    // centered at (half + 0.5, half + 0.5). Built once per unique radius and
    // stamped per point in CircleBatch — avoids sqrt per point per pixel.

    fn make_circle_mask(r: f32) -> (i32, Vec<u8>) {
        let half = (r + 1.5).ceil() as i32;
        let size = half * 2 + 1;
        let center = half as f32 + 0.5;
        let r_outer = r + 0.5;
        let r_inner = (r - 0.5).max(0.0);
        let r_outer2 = r_outer * r_outer;
        let r_inner2 = r_inner * r_inner;

        let mut mask = vec![0u8; (size * size) as usize];
        for my in 0..size {
            let dy = my as f32 + 0.5 - center;
            let dy2 = dy * dy;
            for mx in 0..size {
                let dx = mx as f32 + 0.5 - center;
                let d2 = dx * dx + dy2;
                let cov: f32 = if d2 <= r_inner2 {
                    1.0
                } else if d2 >= r_outer2 {
                    0.0
                } else {
                    r_outer - d2.sqrt()
                };
                mask[(my * size + mx) as usize] = (cov * 255.0 + 0.5) as u8;
            }
        }
        (half, mask)
    }

    fn blit_circle_mask(&mut self, ix: i32, iy: i32, half: i32, mask: &[u8], c: Rgba, clip: Clip) {
        let size = half * 2 + 1;
        let ox = ix - half;
        let oy = iy - half;
        for my in 0..size {
            let py = oy + my;
            if py < clip.y0 || py >= clip.y1 {
                continue;
            }
            let row = (my * size) as usize;
            for mx in 0..size {
                let px = ox + mx;
                if px < clip.x0 || px >= clip.x1 {
                    continue;
                }
                let byte = mask[row + mx as usize];
                if byte > 0 {
                    self.blend(px, py, c, byte as f32 * (1.0 / 255.0));
                }
            }
        }
    }

    // ── Scanline polygon fill — AET + nonzero winding rule ────────────────────
    //
    // Edges are bucketed by first active scanline (AET) so each scanline only
    // processes currently-crossing edges (~4 for a stroke polygon, not 2n).
    //
    // Nonzero winding fill (not even-odd): for a consistently-wound stroke
    // polygon, self-intersecting regions caused by miter overreach on real-world
    // data wind in the same direction as the rest of the interior (winding stays
    // nonzero) and are therefore filled rather than left as transparent holes.
    // For simple non-self-intersecting polygons the result is identical to
    // even-odd fill.

    fn fill_polygon(&mut self, pts: &[(f32, f32)], c: Rgba, clip: Clip) {
        if pts.len() < 3 {
            return;
        }

        let y_lo = pts.iter().map(|p| p.1).fold(f32::INFINITY, f32::min);
        let y_hi = pts.iter().map(|p| p.1).fold(f32::NEG_INFINITY, f32::max);
        let scan_y0 = (y_lo.floor() as i32).max(clip.y0);
        let scan_y1 = (y_hi.ceil() as i32).min(clip.y1 - 1);
        if scan_y0 > scan_y1 {
            return;
        }

        struct AetEdge {
            y_max: i32,
            x: f32,
            inv_slope: f32,
            // +1 for edges going down in screen y (ay < by), -1 for upward (ay > by).
            winding: i32,
        }

        let height = (scan_y1 - scan_y0 + 1) as usize;
        let mut et: Vec<Vec<AetEdge>> = (0..height).map(|_| Vec::new()).collect();
        let n = pts.len();

        for i in 0..n {
            let (ax, ay) = pts[i];
            let (bx, by) = pts[(i + 1) % n];
            if (ay - by).abs() < 1e-6 {
                continue;
            }
            // Canonical orientation: ay_c < by_c (going down in screen y).
            // Downward edges contribute +1 winding; upward (flipped) contribute -1.
            let (ax_c, ay_c, bx_c, by_c, winding) = if ay < by {
                (ax, ay, bx, by, 1i32)
            } else {
                (bx, by, ax, ay, -1i32)
            };

            // Active for scanlines where ay_c <= py+0.5 < by_c.
            let py_start = (ay_c - 0.5).ceil() as i32;
            let py_end = (by_c - 0.5).ceil() as i32;
            if py_start >= py_end {
                continue;
            }

            let inv_slope = (bx_c - ax_c) / (by_c - ay_c);
            let eff_start = py_start.max(scan_y0);
            let eff_end = py_end.min(scan_y1 + 1);
            if eff_start >= eff_end {
                continue;
            }

            let x_eff = ax_c + (eff_start as f32 + 0.5 - ay_c) * inv_slope;
            et[(eff_start - scan_y0) as usize].push(AetEdge {
                y_max: eff_end,
                x: x_eff,
                inv_slope,
                winding,
            });
        }

        let mut aet: Vec<AetEdge> = Vec::with_capacity(8);
        let mut xs: Vec<(f32, i32)> = Vec::with_capacity(8);

        for py in scan_y0..=scan_y1 {
            let bucket = (py - scan_y0) as usize;
            aet.append(&mut et[bucket]);
            aet.retain(|e| e.y_max > py);

            if !aet.is_empty() {
                xs.clear();
                xs.extend(aet.iter().map(|e| (e.x, e.winding)));
                xs.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

                // Nonzero winding: fill while cumulative winding != 0.
                let mut winding = 0i32;
                let mut span_start = 0.0f32;
                for &(x, w) in &xs {
                    let was_inside = winding != 0;
                    winding += w;
                    let is_inside = winding != 0;
                    if !was_inside && is_inside {
                        span_start = x;
                    } else if was_inside && !is_inside {
                        self.fill_span_aa(span_start, x, py, c, clip);
                    }
                }
            }

            for e in aet.iter_mut() {
                e.x += e.inv_slope;
            }
        }
    }

    /// Fill the horizontal span [x_lo, x_hi] at scanline row `py`.
    /// Left and right edge pixels receive exact fractional coverage.
    #[inline]
    fn fill_span_aa(&mut self, x_lo: f32, x_hi: f32, py: i32, c: Rgba, clip: Clip) {
        if x_hi <= x_lo {
            return;
        }

        let ix_lo = x_lo.floor() as i32;
        let ix_hi = x_hi.floor() as i32;

        if ix_lo == ix_hi {
            let cov = (x_hi - x_lo).min(1.0);
            self.blend_c(ix_lo, py, c, cov, clip);
            return;
        }

        // Left partial pixel: how much of [ix_lo, ix_lo+1) is right of x_lo
        let left_cov = (ix_lo + 1) as f32 - x_lo;
        self.blend_c(ix_lo, py, c, left_cov.min(1.0), clip);

        // Interior: fully covered
        let int_lo = (ix_lo + 1).max(clip.x0);
        let int_hi = ix_hi.min(clip.x1);
        for px in int_lo..int_hi {
            self.blend(px, py, c, 1.0);
        }

        // Right partial pixel: how much of [ix_hi, ix_hi+1) is left of x_hi
        let right_cov = x_hi - ix_hi as f32;
        if right_cov > 0.0 {
            self.blend_c(ix_hi, py, c, right_cov.min(1.0), clip);
        }
    }

    // ── Text rendering ────────────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    fn draw_text(
        &mut self,
        x: f32,
        y: f32,
        text: &str,
        size: f32,
        c: Rgba,
        anchor: TextAnchor,
        rotate: Option<f32>,
        font: &Font,
        cache: &mut GlyphCache,
        clip: Clip,
    ) {
        if text.is_empty() {
            return;
        }
        if let Some(angle) = rotate {
            self.draw_text_rotated(x, y, text, size, c, anchor, angle, font, cache, clip);
        } else {
            let pen_x = anchor_pen_x(x, text, size, anchor, font, cache);
            self.rasterize_glyphs(pen_x, y, text, size, c, font, cache, clip);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn rasterize_glyphs(
        &mut self,
        pen_x: f32,
        baseline_y: f32,
        text: &str,
        size: f32,
        c: Rgba,
        font: &Font,
        cache: &mut GlyphCache,
        clip: Clip,
    ) {
        let mut px = pen_x;
        for ch in text.chars() {
            let key = glyph_key(ch, size);
            cache
                .0
                .entry(key)
                .or_insert_with(|| font.rasterize(ch, size));
            let (metrics, bitmap) = &cache.0[&key];
            let gx = (px + metrics.xmin as f32).round() as i32;
            // fontdue ymin: pixels above baseline the bottom of the bitmap sits.
            // Positive = above baseline (ascenders). Negative = below (descenders).
            // In screen coords (y down): bitmap_top = baseline - ymin - height
            let gy = (baseline_y as i32) - metrics.ymin - metrics.height as i32;
            let mw = metrics.width;
            let adv = metrics.advance_width;
            for row in 0..metrics.height {
                let sy = gy + row as i32;
                if sy < clip.y0 || sy >= clip.y1 {
                    continue;
                }
                let row_off = row * mw;
                for col in 0..mw {
                    let sx = gx + col as i32;
                    if sx < clip.x0 || sx >= clip.x1 {
                        continue;
                    }
                    let byte = bitmap[row_off + col];
                    if byte > 0 {
                        self.blend(sx, sy, c, byte as f32 * (1.0 / 255.0));
                    }
                }
            }
            px += adv;
        }
    }

    /// Render text unrotated into a small offscreen canvas, then rotate-blit
    /// it into the main canvas using bilinear sampling. Rotation is around
    /// the anchor point (x, y) — matching SVG's `transform="rotate(a,x,y)"`.
    #[allow(clippy::too_many_arguments)]
    fn draw_text_rotated(
        &mut self,
        anchor_x: f32,
        anchor_y: f32,
        text: &str,
        size: f32,
        c: Rgba,
        anchor: TextAnchor,
        angle_deg: f32,
        font: &Font,
        cache: &mut GlyphCache,
        clip: Clip,
    ) {
        let text_w = measure_text(text, size, font, cache);
        let text_h = size * 1.5;
        let pad = 2.0f32;

        let buf_w = (text_w + pad * 2.0).ceil() as u32 + 4;
        let buf_h = (text_h + pad * 2.0).ceil() as u32 + 4;

        // In the offscreen buffer the anchor point is at (off_ax, off_ay).
        // off_ay is the baseline; off_ax depends on text anchor.
        let off_ax = match anchor {
            TextAnchor::Start => pad,
            TextAnchor::Middle => pad + text_w * 0.5,
            TextAnchor::End => pad + text_w,
        };
        let off_ay = buf_h as f32 - pad - size * 0.3; // baseline near bottom

        let mut offscreen = Canvas::new(buf_w, buf_h);
        // Transparent bg — pixels start at 0,0,0,0
        let off_pen_x = match anchor {
            TextAnchor::Start => pad,
            TextAnchor::Middle => pad,
            TextAnchor::End => pad,
        };
        offscreen.rasterize_glyphs(
            off_pen_x,
            off_ay,
            text,
            size,
            c,
            font,
            cache,
            Clip::full(buf_w, buf_h),
        );

        // Inverse-rotate each destination pixel to find its source in offscreen.
        // SVG rotate(θ) is CW in screen coords (y-down), matching positive θ here.
        let rad = angle_deg * std::f32::consts::PI / 180.0;
        let cos_a = rad.cos();
        let sin_a = rad.sin();

        // Bounding box of the rotated offscreen rectangle in canvas space
        let corners = [
            (0.0f32, 0.0f32),
            (buf_w as f32, 0.0f32),
            (buf_w as f32, buf_h as f32),
            (0.0f32, buf_h as f32),
        ];
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for (cx, cy) in &corners {
            let dx = cx - off_ax;
            let dy = cy - off_ay;
            let wx = anchor_x + cos_a * dx - sin_a * dy;
            let wy = anchor_y + sin_a * dx + cos_a * dy;
            min_x = min_x.min(wx);
            max_x = max_x.max(wx);
            min_y = min_y.min(wy);
            max_y = max_y.max(wy);
        }

        let dx0 = (min_x.floor() as i32).max(clip.x0);
        let dx1 = (max_x.ceil() as i32).min(clip.x1 - 1);
        let dy0 = (min_y.floor() as i32).max(clip.y0);
        let dy1 = (max_y.ceil() as i32).min(clip.y1 - 1);

        for dy in dy0..=dy1 {
            for dx in dx0..=dx1 {
                // Inverse rotation: canvas (dx, dy) → offscreen (src_x, src_y)
                let dxw = dx as f32 + 0.5 - anchor_x;
                let dyw = dy as f32 + 0.5 - anchor_y;
                let src_x = off_ax + cos_a * dxw + sin_a * dyw;
                let src_y = off_ay - sin_a * dxw + cos_a * dyw;
                let sample =
                    bilinear_rgba(&offscreen.pixels, buf_w, buf_h, src_x - 0.5, src_y - 0.5);
                if sample.a > 0 {
                    self.blend(dx, dy, sample, sample.a as f32 * (1.0 / 255.0));
                }
            }
        }
    }

    // ── PNG encode ────────────────────────────────────────────────────────────

    fn encode_png(self) -> Result<Vec<u8>, String> {
        let mut buf = Vec::with_capacity(self.pixels.len() / 4);
        let mut enc = png::Encoder::new(std::io::Cursor::new(&mut buf), self.width, self.height);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header()
            .and_then(|mut w| w.write_image_data(&self.pixels))
            .map_err(|e| e.to_string())?;
        Ok(buf)
    }
}

// ── Glyph cache ───────────────────────────────────────────────────────────────

struct GlyphCache(HashMap<(char, u32), (Metrics, Vec<u8>)>);

impl GlyphCache {
    fn new() -> Self {
        Self(HashMap::new())
    }
}

/// Cache key: char + size in thousandths of a pixel (avoids f32 as HashMap key).
#[inline]
fn glyph_key(ch: char, size: f32) -> (char, u32) {
    (ch, (size * 1000.0).round() as u32)
}

fn measure_text(text: &str, size: f32, font: &Font, cache: &mut GlyphCache) -> f32 {
    let mut total = 0.0f32;
    for ch in text.chars() {
        let key = glyph_key(ch, size);
        cache
            .0
            .entry(key)
            .or_insert_with(|| font.rasterize(ch, size));
        total += cache.0[&key].0.advance_width;
    }
    total
}

fn anchor_pen_x(
    x: f32,
    text: &str,
    size: f32,
    anchor: TextAnchor,
    font: &Font,
    cache: &mut GlyphCache,
) -> f32 {
    match anchor {
        TextAnchor::Start => x,
        TextAnchor::Middle => x - measure_text(text, size, font, cache) * 0.5,
        TextAnchor::End => x - measure_text(text, size, font, cache),
    }
}

// ── Shared font ───────────────────────────────────────────────────────────────

fn shared_font() -> &'static Font {
    static FONT: OnceLock<Font> = OnceLock::new();
    FONT.get_or_init(|| {
        Font::from_bytes(crate::fonts::dejavu_sans(), FontSettings::default())
            .expect("bundled DejaVu Sans TTF is valid")
    })
}

// ── SVG path parsing ──────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
enum PathCmd {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    /// Cubic bezier: cp1x cp1y cp2x cp2y ex ey
    CubicTo(f32, f32, f32, f32, f32, f32),
    ClosePath,
}

fn parse_svg_path(d: &str) -> Vec<PathCmd> {
    let mut out = Vec::new();
    let b = d.as_bytes();
    let mut i = 0;
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut sub_start = (0.0f32, 0.0f32);

    fn skip(b: &[u8], i: &mut usize) {
        while *i < b.len() && matches!(b[*i], b' ' | b',' | b'\n' | b'\r' | b'\t') {
            *i += 1;
        }
    }

    fn num(b: &[u8], i: &mut usize) -> Option<f32> {
        skip(b, i);
        let s = *i;
        if *i < b.len() && matches!(b[*i], b'-' | b'+') {
            *i += 1;
        }
        let mut dot = false;
        while *i < b.len() && (b[*i].is_ascii_digit() || (b[*i] == b'.' && !dot)) {
            if b[*i] == b'.' {
                dot = true;
            }
            *i += 1;
        }
        if *i < b.len() && matches!(b[*i], b'e' | b'E') {
            *i += 1;
            if *i < b.len() && matches!(b[*i], b'-' | b'+') {
                *i += 1;
            }
            while *i < b.len() && b[*i].is_ascii_digit() {
                *i += 1;
            }
        }
        if s == *i {
            return None;
        }
        std::str::from_utf8(&b[s..*i]).ok()?.parse().ok()
    }

    fn flag(b: &[u8], i: &mut usize) -> Option<u8> {
        skip(b, i);
        if *i < b.len() && matches!(b[*i], b'0' | b'1') {
            let v = b[*i] - b'0';
            *i += 1;
            Some(v)
        } else {
            None
        }
    }

    while i < b.len() {
        skip(b, &mut i);
        if i >= b.len() {
            break;
        }
        let cmd = b[i];
        if cmd.is_ascii_alphabetic() {
            i += 1;
        }
        let rel = cmd.is_ascii_lowercase() && !matches!(cmd, b'z' | b'Z');

        match cmd | 0x20 {
            // to lowercase
            b'm' => {
                while let Some(x) = num(b, &mut i) {
                    let Some(y) = num(b, &mut i) else { break };
                    let (ax, ay) = if rel { (cx + x, cy + y) } else { (x, y) };
                    cx = ax;
                    cy = ay;
                    sub_start = (ax, ay);
                    out.push(PathCmd::MoveTo(ax, ay));
                    skip(b, &mut i);
                    if i >= b.len() || b[i].is_ascii_alphabetic() {
                        break;
                    }
                    // Implicit L after M
                    let Some(x2) = num(b, &mut i) else { break };
                    let Some(y2) = num(b, &mut i) else { break };
                    let (bx2, by2) = if rel { (cx + x2, cy + y2) } else { (x2, y2) };
                    cx = bx2;
                    cy = by2;
                    out.push(PathCmd::LineTo(bx2, by2));
                    skip(b, &mut i);
                    if i >= b.len() || b[i].is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            b'l' => {
                while let Some(x) = num(b, &mut i) {
                    let Some(y) = num(b, &mut i) else { break };
                    let (ax, ay) = if rel { (cx + x, cy + y) } else { (x, y) };
                    cx = ax;
                    cy = ay;
                    out.push(PathCmd::LineTo(ax, ay));
                    skip(b, &mut i);
                    if i >= b.len() || b[i].is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            b'h' => {
                while let Some(x) = num(b, &mut i) {
                    cx = if rel { cx + x } else { x };
                    out.push(PathCmd::LineTo(cx, cy));
                    skip(b, &mut i);
                    if i >= b.len() || b[i].is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            b'v' => {
                while let Some(y) = num(b, &mut i) {
                    cy = if rel { cy + y } else { y };
                    out.push(PathCmd::LineTo(cx, cy));
                    skip(b, &mut i);
                    if i >= b.len() || b[i].is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            b'c' => {
                while let Some(x1) = num(b, &mut i) {
                    let Some(y1) = num(b, &mut i) else { break };
                    let Some(x2) = num(b, &mut i) else { break };
                    let Some(y2) = num(b, &mut i) else { break };
                    let Some(x) = num(b, &mut i) else { break };
                    let Some(y) = num(b, &mut i) else { break };
                    let base = if rel { (cx, cy) } else { (0.0, 0.0) };
                    let (ax1, ay1) = (base.0 + x1, base.1 + y1);
                    let (ax2, ay2) = (base.0 + x2, base.1 + y2);
                    let (ax, ay) = (base.0 + x, base.1 + y);
                    cx = ax;
                    cy = ay;
                    out.push(PathCmd::CubicTo(ax1, ay1, ax2, ay2, ax, ay));
                    skip(b, &mut i);
                    if i >= b.len() || b[i].is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            b'a' => {
                while let Some(rx) = num(b, &mut i) {
                    let Some(ry) = num(b, &mut i) else { break };
                    let Some(xrot) = num(b, &mut i) else { break };
                    let Some(la) = flag(b, &mut i) else { break };
                    let Some(sw) = flag(b, &mut i) else { break };
                    let Some(x) = num(b, &mut i) else { break };
                    let Some(y) = num(b, &mut i) else { break };
                    let (ex, ey) = if rel { (cx + x, cy + y) } else { (x, y) };
                    svg_arc_to_cubics(cx, cy, rx, ry, xrot, la != 0, sw != 0, ex, ey, &mut out);
                    cx = ex;
                    cy = ey;
                    skip(b, &mut i);
                    if i >= b.len() || b[i].is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            b'z' => {
                out.push(PathCmd::ClosePath);
                cx = sub_start.0;
                cy = sub_start.1;
            }
            _ => {
                i += 1;
            }
        }
    }
    out
}

// ── SVG arc → cubic beziers ───────────────────────────────────────────────────
//
// Implements SVG 1.1 spec Appendix F endpoint-to-center parameterization,
// then splits into ≤90° segments and approximates each with a cubic bezier
// using the standard Riškus/Maisonobe α formula.

#[allow(clippy::too_many_arguments)]
fn svg_arc_to_cubics(
    x1: f32,
    y1: f32,
    mut rx: f32,
    mut ry: f32,
    x_rot_deg: f32,
    large_arc: bool,
    sweep: bool,
    x2: f32,
    y2: f32,
    out: &mut Vec<PathCmd>,
) {
    if (x1 - x2).abs() < 1e-5 && (y1 - y2).abs() < 1e-5 {
        return;
    }
    rx = rx.abs();
    ry = ry.abs();
    if rx < 1e-5 || ry < 1e-5 {
        out.push(PathCmd::LineTo(x2, y2));
        return;
    }

    let phi = x_rot_deg.to_radians();
    let (sin_phi, cos_phi) = phi.sin_cos();

    // Midpoint in rotated frame (SVG spec F.6.5.1)
    let dx2 = (x1 - x2) * 0.5;
    let dy2 = (y1 - y2) * 0.5;
    let x1p = cos_phi * dx2 + sin_phi * dy2;
    let y1p = -sin_phi * dx2 + cos_phi * dy2;

    // Ensure radii are large enough (spec F.6.6.3)
    let lambda = (x1p / rx) * (x1p / rx) + (y1p / ry) * (y1p / ry);
    if lambda > 1.0 {
        let s = lambda.sqrt();
        rx *= s;
        ry *= s;
    }

    let rx2 = rx * rx;
    let ry2 = ry * ry;
    let x1p2 = x1p * x1p;
    let y1p2 = y1p * y1p;

    // Center in rotated frame (spec F.6.5.2)
    let num = (rx2 * ry2 - rx2 * y1p2 - ry2 * x1p2).max(0.0);
    let den = rx2 * y1p2 + ry2 * x1p2;
    let sq = if den.abs() < 1e-10 {
        0.0
    } else {
        (num / den).sqrt()
    };
    let sign = if large_arc == sweep { -1.0f32 } else { 1.0f32 };
    let cxp = sign * sq * (rx * y1p / ry);
    let cyp = -sign * sq * (ry * x1p / rx);

    // Center in original frame (spec F.6.5.3)
    let cx = cos_phi * cxp - sin_phi * cyp + (x1 + x2) * 0.5;
    let cy = sin_phi * cxp + cos_phi * cyp + (y1 + y2) * 0.5;

    // Angles (spec F.6.5.5–6)
    let ux = (x1p - cxp) / rx;
    let uy = (y1p - cyp) / ry;
    let vx = (-x1p - cxp) / rx;
    let vy = (-y1p - cyp) / ry;

    let theta1 = vec2_angle(1.0, 0.0, ux, uy);
    let mut d_theta = vec2_angle(ux, uy, vx, vy);
    if !sweep && d_theta > 0.0 {
        d_theta -= std::f32::consts::TAU;
    }
    if sweep && d_theta < 0.0 {
        d_theta += std::f32::consts::TAU;
    }

    // Split into ≤90° segments
    let n = (d_theta.abs() / (std::f32::consts::PI * 0.5)).ceil() as i32;
    let n = n.max(1);
    let seg = d_theta / n as f32;
    let mut theta = theta1;
    for _ in 0..n {
        arc_seg_cubic(cx, cy, rx, ry, cos_phi, sin_phi, theta, seg, out);
        theta += seg;
    }
}

fn vec2_angle(ux: f32, uy: f32, vx: f32, vy: f32) -> f32 {
    let dot = (ux * vx + uy * vy).clamp(-1.0, 1.0)
        / ((ux * ux + uy * uy) * (vx * vx + vy * vy))
            .sqrt()
            .max(1e-10);
    let a = dot.acos();
    if ux * vy - uy * vx < 0.0 {
        -a
    } else {
        a
    }
}

/// Convert a single arc segment (angle `theta` to `theta + d_theta`, guaranteed
/// ≤90°) to a cubic bezier via the standard α = (4/3)·tan(dθ/4) formula.
#[allow(clippy::too_many_arguments)]
fn arc_seg_cubic(
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    cos_phi: f32,
    sin_phi: f32,
    theta: f32,
    d_theta: f32,
    out: &mut Vec<PathCmd>,
) {
    let alpha = (4.0 / 3.0) * (d_theta * 0.25).tan();
    let (sin_t, cos_t) = theta.sin_cos();
    let (sin_t2, cos_t2) = (theta + d_theta).sin_cos();

    // Points and tangent derivatives on the (centered, axis-aligned) ellipse
    let (ex1, ey1) = (rx * cos_t, ry * sin_t);
    let (ex2, ey2) = (rx * cos_t2, ry * sin_t2);
    let (dx1, dy1) = (-rx * sin_t, ry * cos_t); // tangent at start
    let (dx2, dy2) = (-rx * sin_t2, ry * cos_t2); // tangent at end

    // Control points in the rotated+translated frame
    let cp1 = ellipse_to_canvas(
        ex1 + alpha * dx1,
        ey1 + alpha * dy1,
        cx,
        cy,
        cos_phi,
        sin_phi,
    );
    let cp2 = ellipse_to_canvas(
        ex2 - alpha * dx2,
        ey2 - alpha * dy2,
        cx,
        cy,
        cos_phi,
        sin_phi,
    );
    let p2 = ellipse_to_canvas(ex2, ey2, cx, cy, cos_phi, sin_phi);

    out.push(PathCmd::CubicTo(cp1.0, cp1.1, cp2.0, cp2.1, p2.0, p2.1));
}

#[inline]
fn ellipse_to_canvas(ex: f32, ey: f32, cx: f32, cy: f32, cos_phi: f32, sin_phi: f32) -> (f32, f32) {
    (
        cos_phi * ex - sin_phi * ey + cx,
        sin_phi * ex + cos_phi * ey + cy,
    )
}

// ── Adaptive bezier tessellation (de Casteljau) ───────────────────────────────
//
// Recursion stops when the midpoint of the bezier deviates from the chord by
// less than `tol` pixels. This concentrates segments where the curve bends.

fn tessellate_cubic(
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
    tol: f32,
    out: &mut Vec<(f32, f32)>,
) {
    // Midpoint of the bezier vs midpoint of the chord
    let mid = (
        0.125 * (p0.0 + 3.0 * p1.0 + 3.0 * p2.0 + p3.0),
        0.125 * (p0.1 + 3.0 * p1.1 + 3.0 * p2.1 + p3.1),
    );
    let lin = ((p0.0 + p3.0) * 0.5, (p0.1 + p3.1) * 0.5);
    let dx = mid.0 - lin.0;
    let dy = mid.1 - lin.1;
    if dx * dx + dy * dy < tol * tol {
        out.push(p3);
        return;
    }
    // De Casteljau split at t = 0.5
    let q0 = mid2(p0, p1);
    let q1 = mid2(p1, p2);
    let q2 = mid2(p2, p3);
    let r0 = mid2(q0, q1);
    let r1 = mid2(q1, q2);
    let s = mid2(r0, r1);
    tessellate_cubic(p0, q0, r0, s, tol, out);
    tessellate_cubic(s, r1, q2, p3, tol, out);
}

#[inline]
fn mid2(a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
    ((a.0 + b.0) * 0.5, (a.1 + b.1) * 0.5)
}

/// Tessellate a list of path commands into sub-paths of (f32,f32) vertices.
fn tessellate_path(cmds: &[PathCmd], tol: f32) -> Vec<Vec<(f32, f32)>> {
    let mut paths: Vec<Vec<(f32, f32)>> = Vec::new();
    let mut cur: Vec<(f32, f32)> = Vec::new();
    let mut pen = (0.0f32, 0.0f32);
    let mut sub_start = (0.0f32, 0.0f32);

    for &cmd in cmds {
        match cmd {
            PathCmd::MoveTo(x, y) => {
                if cur.len() > 1 {
                    paths.push(std::mem::take(&mut cur));
                } else {
                    cur.clear();
                }
                cur.push((x, y));
                pen = (x, y);
                sub_start = (x, y);
            }
            PathCmd::LineTo(x, y) => {
                cur.push((x, y));
                pen = (x, y);
            }
            PathCmd::CubicTo(x1, y1, x2, y2, x, y) => {
                tessellate_cubic(pen, (x1, y1), (x2, y2), (x, y), tol, &mut cur);
                pen = (x, y);
            }
            PathCmd::ClosePath => {
                if (pen.0 - sub_start.0).abs() > 0.01 || (pen.1 - sub_start.1).abs() > 0.01 {
                    cur.push(sub_start);
                }
                if cur.len() > 1 {
                    paths.push(std::mem::take(&mut cur));
                } else {
                    cur.clear();
                }
                pen = sub_start;
            }
        }
    }
    if cur.len() > 1 {
        paths.push(cur);
    }
    paths
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Coverage of pixel column/row `px` under a 1D interval [lo, hi].
/// Computes the overlap of [lo, hi] with the pixel interval [px, px+1).
#[inline]
fn partial_cov(lo: f32, hi: f32, px: i32) -> f32 {
    let p0 = px as f32;
    (hi.min(p0 + 1.0) - lo.max(p0)).clamp(0.0, 1.0)
}

fn parse_dasharray(s: &str) -> Vec<f32> {
    s.split([' ', ','])
        .filter_map(|t| t.trim().parse::<f32>().ok())
        .filter(|&v| v > 0.0)
        .collect()
}

/// Bilinear RGBA sample from a pixel buffer. Returns (0,0,0,0) for out-of-bounds.
fn bilinear_rgba(pixels: &[u8], w: u32, h: u32, x: f32, y: f32) -> Rgba {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let fx = x - x.floor();
    let fy = y - y.floor();

    let get = |xi: i32, yi: i32| -> [f32; 4] {
        if xi < 0 || yi < 0 || xi >= w as i32 || yi >= h as i32 {
            return [0.0; 4];
        }
        let idx = (yi as usize * w as usize + xi as usize) * 4;
        [
            pixels[idx] as f32,
            pixels[idx + 1] as f32,
            pixels[idx + 2] as f32,
            pixels[idx + 3] as f32,
        ]
    };

    let [r00, g00, b00, a00] = get(x0, y0);
    let [r10, g10, b10, a10] = get(x0 + 1, y0);
    let [r01, g01, b01, a01] = get(x0, y0 + 1);
    let [r11, g11, b11, a11] = get(x0 + 1, y0 + 1);

    let lerp = |a: f32, b: f32, c: f32, d: f32| -> u8 {
        ((a + (b - a) * fx) + ((c + (d - c) * fx) - (a + (b - a) * fx)) * fy) as u8
    };

    Rgba {
        r: lerp(r00, r10, r01, r11),
        g: lerp(g00, g10, g01, g11),
        b: lerp(b00, b10, b01, b11),
        a: lerp(a00, a10, a01, a11),
    }
}

// ── Color conversion ──────────────────────────────────────────────────────────

fn kcolor_to_rgba(c: &KColor) -> Option<Rgba> {
    match c {
        KColor::Rgb(r, g, b) => Some(Rgba::opaque(*r, *g, *b)),
        KColor::None => None,
        // Reuse the existing parser in color.rs which covers all CSS names used
        // in kuva. Any color it returns as Css(_) is genuinely unknown; skip it.
        KColor::Css(s) => match KColor::from(s.as_ref()) {
            KColor::Rgb(r, g, b) => Some(Rgba::opaque(r, g, b)),
            _ => None,
        },
    }
}

fn css_to_rgba(s: &str) -> Option<Rgba> {
    kcolor_to_rgba(&KColor::from(s))
}

// ── Transform stack helper ────────────────────────────────────────────────────

fn parse_translate(t: &str) -> (f32, f32) {
    let t = t.trim();
    if let Some(inner) = t
        .strip_prefix("translate(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let mut parts = inner.split(',');
        let x = parts
            .next()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0.0);
        let y = parts
            .next()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0.0);
        return (x, y);
    }
    (0.0, 0.0)
}

// ── Public backend ────────────────────────────────────────────────────────────

pub struct RasterBackend {
    pub scale: f32,
}

impl Default for RasterBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RasterBackend {
    pub fn new() -> Self {
        Self { scale: 2.0 }
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn render_scene(&self, scene: &Scene) -> Result<Vec<u8>, String> {
        let w = (scene.width as f32 * self.scale).ceil() as u32;
        let h = (scene.height as f32 * self.scale).ceil() as u32;
        if w == 0 || h == 0 {
            return Err("scene has zero dimensions".into());
        }

        let mut canvas = Canvas::new(w, h);
        let bg = scene
            .background_color
            .as_deref()
            .and_then(css_to_rgba)
            .unwrap_or(Rgba::opaque(255, 255, 255));
        canvas.fill_background(bg);

        let font = shared_font();
        let mut glyph_cache = GlyphCache::new();
        let s = self.scale;

        let default_text = scene
            .text_color
            .as_deref()
            .and_then(css_to_rgba)
            .unwrap_or(Rgba::opaque(0, 0, 0));

        let mut tx_stack: Vec<(f32, f32)> = vec![(0.0, 0.0)];
        let mut clip_stack: Vec<Clip> = vec![Clip::full(w, h)];

        // Precomputed circle masks: key = radius in 16ths of a scaled pixel
        let mut mask_cache: HashMap<u32, (i32, Vec<u8>)> = HashMap::new();

        for elem in &scene.elements {
            let (tx, ty) = *tx_stack.last().unwrap();
            let clip = *clip_stack.last().unwrap();

            // Helper: apply scale + translate to a scene coordinate
            macro_rules! sx {
                ($v:expr) => {
                    ($v as f32 + tx) * s
                };
            }
            macro_rules! sy {
                ($v:expr) => {
                    ($v as f32 + ty) * s
                };
            }

            match elem {
                Primitive::GroupStart { transform, .. } => {
                    let (dx, dy) = transform
                        .as_deref()
                        .map(parse_translate)
                        .unwrap_or((0.0, 0.0));
                    tx_stack.push((tx + dx, ty + dy));
                    clip_stack.push(clip);
                }
                Primitive::GroupEnd => {
                    if tx_stack.len() > 1 {
                        tx_stack.pop();
                    }
                    if clip_stack.len() > 1 {
                        clip_stack.pop();
                    }
                }
                Primitive::ClipStart {
                    x,
                    y,
                    width,
                    height,
                    ..
                } => {
                    let new_clip = Clip {
                        x0: sx!(*x).floor() as i32,
                        y0: sy!(*y).floor() as i32,
                        x1: sx!(*x + *width).ceil() as i32,
                        y1: sy!(*y + *height).ceil() as i32,
                    };
                    clip_stack.push(clip.intersect(new_clip));
                }
                Primitive::ClipEnd => {
                    if clip_stack.len() > 1 {
                        clip_stack.pop();
                    }
                }

                Primitive::Rect {
                    x,
                    y,
                    width,
                    height,
                    fill,
                    stroke,
                    stroke_width,
                    opacity,
                } => {
                    let (fx, fy, fw, fh) =
                        (sx!(*x), sy!(*y), *width as f32 * s, *height as f32 * s);
                    if let Some(rgba) = kcolor_to_rgba(fill) {
                        let rgba = if let Some(op) = opacity {
                            rgba.with_alpha((op.clamp(0.0, 1.0) * rgba.a as f64) as u8)
                        } else {
                            rgba
                        };
                        canvas.fill_rect(fx, fy, fw, fh, rgba, clip);
                    }
                    if let Some(sc) = stroke {
                        if let Some(sc_rgba) = kcolor_to_rgba(sc) {
                            let sw = stroke_width.unwrap_or(1.0) as f32 * s;
                            let pts = [
                                (fx, fy),
                                (fx + fw, fy),
                                (fx + fw, fy + fh),
                                (fx, fy + fh),
                                (fx, fy),
                            ];
                            for i in 0..4 {
                                canvas.draw_line(
                                    pts[i].0,
                                    pts[i].1,
                                    pts[i + 1].0,
                                    pts[i + 1].1,
                                    sc_rgba,
                                    sw,
                                    None,
                                    clip,
                                );
                            }
                        }
                    }
                }

                Primitive::RectBatch {
                    x,
                    y,
                    w: ws,
                    h: hs,
                    fills,
                } => {
                    for i in 0..x.len() {
                        if let Some(rgba) = kcolor_to_rgba(&fills[i]) {
                            canvas.fill_rect(
                                sx!(x[i]),
                                sy!(y[i]),
                                ws[i] as f32 * s,
                                hs[i] as f32 * s,
                                rgba,
                                clip,
                            );
                        }
                    }
                }

                Primitive::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    stroke,
                    stroke_width,
                    stroke_dasharray,
                } => {
                    if let Some(rgba) = kcolor_to_rgba(stroke) {
                        canvas.draw_line(
                            sx!(*x1),
                            sy!(*y1),
                            sx!(*x2),
                            sy!(*y2),
                            rgba,
                            *stroke_width as f32 * s,
                            stroke_dasharray.as_deref(),
                            clip,
                        );
                    }
                }

                Primitive::Circle {
                    cx,
                    cy,
                    r,
                    fill,
                    fill_opacity,
                    stroke,
                    stroke_width,
                } => {
                    let (fcx, fcy, fr) = (sx!(*cx), sy!(*cy), *r as f32 * s);
                    if let Some(rgba) = kcolor_to_rgba(fill) {
                        let rgba = fill_opacity
                            .map(|op| rgba.with_alpha((op.clamp(0.0, 1.0) * 255.0) as u8))
                            .unwrap_or(rgba);
                        canvas.fill_circle_aa(fcx, fcy, fr, rgba, clip);
                    }
                    if let Some(sc) = stroke {
                        if let Some(sc_rgba) = kcolor_to_rgba(sc) {
                            canvas.stroke_circle_aa(
                                fcx,
                                fcy,
                                fr,
                                sc_rgba,
                                stroke_width.unwrap_or(1.0) as f32 * s,
                                clip,
                            );
                        }
                    }
                }

                Primitive::CircleBatch {
                    cx,
                    cy,
                    r,
                    fill,
                    fill_opacity,
                    stroke,
                    stroke_width,
                } => {
                    if let Some(rgba) = kcolor_to_rgba(fill) {
                        let rgba = fill_opacity
                            .map(|op| rgba.with_alpha((op.clamp(0.0, 1.0) * 255.0) as u8))
                            .unwrap_or(rgba);
                        let fr = *r as f32 * s;
                        let key = (fr * 16.0).round() as u32;
                        let entry = mask_cache
                            .entry(key)
                            .or_insert_with(|| Canvas::make_circle_mask(fr));
                        let half = entry.0;
                        let mask = &entry.1 as *const Vec<u8>; // raw ptr to avoid borrow split

                        for i in 0..cx.len() {
                            let ix = sx!(cx[i]).round() as i32;
                            let iy = sy!(cy[i]).round() as i32;
                            // SAFETY: mask_cache is not mutated in this loop
                            canvas.blit_circle_mask(ix, iy, half, unsafe { &*mask }, rgba, clip);
                        }

                        if let Some(sc) = stroke {
                            if let Some(sc_rgba) = kcolor_to_rgba(sc) {
                                let sw = stroke_width.unwrap_or(1.0) as f32 * s;
                                for i in 0..cx.len() {
                                    canvas.stroke_circle_aa(
                                        sx!(cx[i]),
                                        sy!(cy[i]),
                                        fr,
                                        sc_rgba,
                                        sw,
                                        clip,
                                    );
                                }
                            }
                        }
                    }
                }

                Primitive::Path(pd) => {
                    let cmds = parse_svg_path(&pd.d);
                    let subs = tessellate_path(&cmds, 0.25);

                    // Apply scale + translate to tessellated points
                    let transform_pts = |sub: &Vec<(f32, f32)>| -> Vec<(f32, f32)> {
                        sub.iter()
                            .map(|&(x, y)| ((x + tx) * s, (y + ty) * s))
                            .collect()
                    };

                    if let Some(ref fc) = pd.fill {
                        if let Some(mut rgba) = kcolor_to_rgba(fc) {
                            if let Some(op) = pd.opacity {
                                rgba = rgba.with_alpha((op.clamp(0.0, 1.0) * 255.0) as u8);
                            }
                            for sub in &subs {
                                canvas.fill_polygon(&transform_pts(sub), rgba, clip);
                            }
                        }
                    }

                    if !matches!(pd.stroke, KColor::None) {
                        if let Some(rgba) = kcolor_to_rgba(&pd.stroke) {
                            let sw = pd.stroke_width as f32 * s;
                            let dash = pd.stroke_dasharray.as_deref();
                            for sub in &subs {
                                let pts = transform_pts(sub);
                                for j in 0..pts.len().saturating_sub(1) {
                                    canvas.draw_line(
                                        pts[j].0,
                                        pts[j].1,
                                        pts[j + 1].0,
                                        pts[j + 1].1,
                                        rgba,
                                        sw,
                                        dash,
                                        clip,
                                    );
                                }
                            }
                        }
                    }
                }

                Primitive::PolyLine {
                    points,
                    stroke,
                    stroke_width,
                    stroke_dasharray,
                } => {
                    if let Some(rgba) = kcolor_to_rgba(stroke) {
                        let sw = *stroke_width as f32 * s;
                        let n = points.len();
                        if n < 2 {
                            // nothing
                        } else if let Some(dash_str) = stroke_dasharray.as_deref() {
                            // Dashed: per-segment (correctness over speed)
                            for w in points.windows(2) {
                                canvas.draw_line(
                                    sx!(w[0].0),
                                    sy!(w[0].1),
                                    sx!(w[1].0),
                                    sy!(w[1].1),
                                    rgba,
                                    sw,
                                    Some(dash_str),
                                    clip,
                                );
                            }
                        } else if sw <= 1.5 {
                            // Thin: single Wu row per segment
                            for w in points.windows(2) {
                                canvas.draw_line_wu(
                                    sx!(w[0].0),
                                    sy!(w[0].1),
                                    sx!(w[1].0),
                                    sy!(w[1].1),
                                    rgba,
                                    clip,
                                );
                            }
                        } else {
                            // Thick: per-segment stadium (capsule) fill.
                            // Each segment is filled as the set of all pixels within hw of
                            // that segment. Adjacent stadiums overlap at joins, naturally
                            // producing round joins without any join geometry construction.
                            // This is convex-per-segment so there are no self-intersections,
                            // no below-baseline arc artifacts, and no polygon construction cost.
                            let hw = sw * 0.5;
                            let pts_s: Vec<(f32, f32)> =
                                points.iter().map(|&(x, y)| (sx!(x), sy!(y))).collect();
                            for w in pts_s.windows(2) {
                                canvas.fill_stadium(w[0], w[1], hw, rgba, clip);
                            }
                        }
                    }
                }

                Primitive::Text {
                    x,
                    y,
                    content,
                    size,
                    anchor,
                    rotate,
                    bold: _,
                    color,
                } => {
                    let rgba = color
                        .as_ref()
                        .and_then(kcolor_to_rgba)
                        .unwrap_or(default_text);
                    canvas.draw_text(
                        sx!(*x),
                        sy!(*y),
                        content,
                        *size as f32 * s,
                        rgba,
                        *anchor,
                        rotate.map(|r| r as f32),
                        font,
                        &mut glyph_cache,
                        clip,
                    );
                }

                Primitive::RichText {
                    x,
                    y,
                    spans,
                    size,
                    anchor,
                    color,
                } => {
                    let rgba = color
                        .as_ref()
                        .and_then(kcolor_to_rgba)
                        .unwrap_or(default_text);
                    // Flatten spans — bold/italic require additional font faces;
                    // treat as plain text for now (a separate bold TTF can be added later).
                    let content: String = spans.iter().map(|sp| sp.text.as_str()).collect();
                    canvas.draw_text(
                        sx!(*x),
                        sy!(*y),
                        &content,
                        *size as f32 * s,
                        rgba,
                        *anchor,
                        None,
                        font,
                        &mut glyph_cache,
                        clip,
                    );
                }
            }
        }

        canvas.encode_png()
    }
}
