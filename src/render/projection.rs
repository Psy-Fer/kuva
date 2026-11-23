//! 3D→2D projection for orthographic rendering of 3D plot types.
//!
//! The projection pipeline:
//! 1. Normalize data coordinates to [-0.5, 0.5]³ using data ranges
//! 2. Rotate by combined matrix Rx(elevation) * Rz(azimuth)
//! 3. Orthographic projection: `screen_x = rotated[0], screen_y = -rotated[2]`
//!    (a proper rotation: screen-right, screen-up and toward-viewer form a
//!    right-handed frame, so a counter-clockwise field reads counter-clockwise)
//! 4. Scale uniformly to fit the plot area
//! 5. Translate to center in the pixel bounding box

pub use crate::plot::plot3d::View3D;

/// A projected point: `(screen_x, screen_y, depth)`.
pub(crate) type ScreenPoint = (f64, f64, f64);

/// Pre-computed 3D→2D orthographic projection.
///
/// Projects data-space (x, y, z) coordinates to pixel-space (screen_x, screen_y, depth).
/// Depth is used for painter's algorithm sorting (back-to-front rendering).
pub struct Projection3D {
    /// Combined rotation matrix (3×3 stored row-major as [row][col])
    rot: [[f64; 3]; 3],
    /// Data range normalization: (min, 1/half_span) for each axis
    norm_x: (f64, f64),
    norm_y: (f64, f64),
    norm_z: (f64, f64),
    /// Uniform scale from normalized projected space to pixel space
    scale: f64,
    /// Translation offset to center in pixel bounding box
    offset_x: f64,
    offset_y: f64,
}

impl Projection3D {
    /// Create a new projection from viewing parameters and data/pixel geometry.
    ///
    /// - `view`: azimuth and elevation angles
    /// - `x_range`, `y_range`, `z_range`: data-space extents (min, max)
    /// - `plot_cx`, `plot_cy`: center of the plot area in pixel space
    /// - `plot_size`: size of the square plot area (min of width, height)
    pub fn new(
        view: View3D,
        x_range: (f64, f64),
        y_range: (f64, f64),
        z_range: (f64, f64),
        plot_cx: f64,
        plot_cy: f64,
        plot_size: f64,
    ) -> Self {
        let az = view.azimuth.to_radians();
        let el = view.elevation.to_radians();

        let cos_az = az.cos();
        let sin_az = az.sin();
        let cos_el = el.cos();
        let sin_el = el.sin();

        // Combined rotation: Rx(elevation) * Rz(azimuth)
        // Rz = [[cos_az, -sin_az, 0], [sin_az, cos_az, 0], [0, 0, 1]]
        // Rx = [[1, 0, 0], [0, cos_el, -sin_el], [0, sin_el, cos_el]]
        let rot = [
            [cos_az, -sin_az, 0.0],
            [sin_az * cos_el, cos_az * cos_el, -sin_el],
            [sin_az * sin_el, cos_az * sin_el, cos_el],
        ];

        let norm_x = Self::norm_params(x_range);
        let norm_y = Self::norm_params(y_range);
        let norm_z = Self::norm_params(z_range);

        // Project all 8 corners of the unit cube to find bounding box.
        // screen_x = +rx keeps the screen frame right-handed; negating it
        // (as this once did) mirrored every 3D figure.
        let mut sx_min = f64::INFINITY;
        let mut sx_max = f64::NEG_INFINITY;
        let mut sy_min = f64::INFINITY;
        let mut sy_max = f64::NEG_INFINITY;

        for &nx in &[-0.5_f64, 0.5] {
            for &ny in &[-0.5_f64, 0.5] {
                for &nz in &[-0.5_f64, 0.5] {
                    let rx = rot[0][0] * nx + rot[0][1] * ny + rot[0][2] * nz;
                    let rz = rot[2][0] * nx + rot[2][1] * ny + rot[2][2] * nz;
                    let sx = rx;
                    let sy = -rz; // negate for SVG y-down
                    sx_min = sx_min.min(sx);
                    sx_max = sx_max.max(sx);
                    sy_min = sy_min.min(sy);
                    sy_max = sy_max.max(sy);
                }
            }
        }

        let proj_width = sx_max - sx_min;
        let proj_height = sy_max - sy_min;
        let scale = if proj_width > 0.0 && proj_height > 0.0 {
            plot_size * 0.85 / proj_width.max(proj_height)
        } else {
            plot_size * 0.85
        };

        let proj_cx = (sx_min + sx_max) / 2.0;
        let proj_cy = (sy_min + sy_max) / 2.0;
        let offset_x = plot_cx - proj_cx * scale;
        let offset_y = plot_cy - proj_cy * scale;

        Self {
            rot,
            norm_x,
            norm_y,
            norm_z,
            scale,
            offset_x,
            offset_y,
        }
    }

    /// `(min, 1 / half_span)` for [`Self::normalize`]. Works on halves so a
    /// finite range wider than `f64::MAX` (say -1e308 to 1e308) doesn't
    /// overflow to an infinite span, which made the scale 0 and some
    /// projected points `inf * 0 = NaN`. Halving is exact, so ranges that
    /// never overflowed normalise to exactly the same values as before.
    fn norm_params(range: (f64, f64)) -> (f64, f64) {
        let half_span = range.1 * 0.5 - range.0 * 0.5;
        if half_span.abs() < 0.5e-15 {
            (range.0, 2.0) // degenerate: all points map to -0.5 (pre-padded by data_ranges)
        } else {
            (range.0, 1.0 / half_span)
        }
    }

    /// Map `v` into `[-0.5, 0.5]` with [`Self::norm_params`]' output. Finite for
    /// any finite `v` inside the range.
    #[inline]
    fn normalize(v: f64, (min, inv_half_span): (f64, f64)) -> f64 {
        (v * 0.5 - min * 0.5) * inv_half_span - 0.5
    }

    /// Project a data-space point to pixel-space.
    /// Returns `(screen_x, screen_y, depth)` where depth is used for sorting.
    #[inline]
    pub fn project(&self, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        let nx = Self::normalize(x, self.norm_x);
        let ny = Self::normalize(y, self.norm_y);
        let nz = Self::normalize(z, self.norm_z);
        self.project_normalized(nx, ny, nz)
    }

    /// Project the data-space segment `a`-`b`, clipped to a cube of `±limit`
    /// in normalised box units (the box itself is `±0.5`). Returns `None` when
    /// the segment lies wholly outside the cube.
    ///
    /// A point far outside the box (an arrow tip with `with_tight_bounds()`
    /// and a huge vector) would otherwise project to infinite pixel
    /// coordinates, and the arrow-head geometry built from them to NaN.
    /// Clipping in normalised space keeps the segment's direction. Segments
    /// already inside the cube project exactly as [`Self::project`] would.
    pub(crate) fn project_segment_clipped(
        &self,
        a: [f64; 3],
        b: [f64; 3],
        limit: f64,
    ) -> Option<(ScreenPoint, ScreenPoint)> {
        // Saturate, so a coordinate that overflowed during normalisation stays
        // finite and the differences below can't overflow either.
        const SAT: f64 = 1e300;
        let norm = |p: [f64; 3]| {
            [
                Self::normalize(p[0], self.norm_x).clamp(-SAT, SAT),
                Self::normalize(p[1], self.norm_y).clamp(-SAT, SAT),
                Self::normalize(p[2], self.norm_z).clamp(-SAT, SAT),
            ]
        };
        let (mut na, mut nb) = (norm(a), norm(b));
        let inside = |p: &[f64; 3]| p.iter().all(|c| c.abs() <= limit);
        if !(inside(&na) && inside(&nb)) {
            // Liang-Barsky against the cube.
            let d = [nb[0] - na[0], nb[1] - na[1], nb[2] - na[2]];
            let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
            for i in 0..3 {
                for (p, q) in [(-d[i], na[i] + limit), (d[i], limit - na[i])] {
                    if p == 0.0 {
                        if q < 0.0 {
                            return None;
                        }
                    } else {
                        let r = q / p;
                        if p < 0.0 {
                            if r > t1 {
                                return None;
                            }
                            t0 = t0.max(r);
                        } else {
                            if r < t0 {
                                return None;
                            }
                            t1 = t1.min(r);
                        }
                    }
                }
            }
            let at = |t: f64| [na[0] + t * d[0], na[1] + t * d[1], na[2] + t * d[2]];
            (na, nb) = (at(t0), at(t1));
        }
        Some((
            self.project_normalized(na[0], na[1], na[2]),
            self.project_normalized(nb[0], nb[1], nb[2]),
        ))
    }

    /// Project a point already in normalized [-0.5, 0.5]³ space.
    /// Used for axes/grid (unit cube coords).
    #[inline]
    pub fn project_normalized(&self, nx: f64, ny: f64, nz: f64) -> (f64, f64, f64) {
        let rx = self.rot[0][0] * nx + self.rot[0][1] * ny + self.rot[0][2] * nz;
        let ry = self.rot[1][0] * nx + self.rot[1][1] * ny + self.rot[1][2] * nz;
        let rz = self.rot[2][0] * nx + self.rot[2][1] * ny + self.rot[2][2] * nz;

        let sx = rx * self.scale + self.offset_x;
        let sy = -rz * self.scale + self.offset_y; // negate for SVG y-down
        let depth = ry; // used for sorting (larger = further from viewer)

        (sx, sy, depth)
    }

    /// Get the view direction vector (points from scene toward viewer).
    /// Used to classify front/back faces of the wireframe box.
    pub fn view_direction(&self) -> [f64; 3] {
        // The view direction in rotated space is [0, -1, 0] (viewer looks along -Y after rotation).
        // Transform back to data space: view_dir = R^T * [0, -1, 0]
        [-self.rot[1][0], -self.rot[1][1], -self.rot[1][2]]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Screen-space image of a data-space direction: `(right, up, toward)`.
    /// `up` negates SVG's y-down and `toward` negates depth (larger = further).
    fn screen_vec(p: &Projection3D, d: [f64; 3]) -> [f64; 3] {
        let (ox, oy, od) = p.project(0.0, 0.0, 0.0);
        let (x, y, dep) = p.project(d[0], d[1], d[2]);
        [x - ox, -(y - oy), -(dep - od)]
    }

    fn triple(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
        a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
            + a[2] * (b[0] * c[1] - b[1] * c[0])
    }

    /// The projection must be a proper rotation at every view angle: the
    /// images of +x, +y, +z keep a positive triple product, so a right-handed
    /// helix stays right-handed and a counter-clockwise field about +z reads
    /// counter-clockwise from above. (It used to negate screen x, which
    /// mirrored the scene in half of all views, the default one included.)
    #[test]
    fn segment_clipping_keeps_inside_segments_and_bounds_far_ones() {
        let p = Projection3D::new(
            View3D {
                azimuth: -60.0,
                elevation: 30.0,
            },
            (0.0, 19.0),
            (0.0, 1.0),
            (0.0, 1.0),
            200.0,
            200.0,
            300.0,
        );
        // Inside the cube: exactly what `project` gives.
        let (a, b) = ([1.0, 0.2, 0.3], [5.0, 0.8, 0.9]);
        let (pa, pb) = p.project_segment_clipped(a, b, 1e3).unwrap();
        assert_eq!(pa, p.project(a[0], a[1], a[2]));
        assert_eq!(pb, p.project(b[0], b[1], b[2]));
        // A tip at f64::MAX projects to inf unclipped; clipped it stays finite
        // and the tail is untouched.
        let (a, b) = ([1.0, 0.5, 0.5], [f64::MAX, 0.5, 0.5]);
        assert!(!p.project(b[0], b[1], b[2]).0.is_finite());
        let (pa, pb) = p.project_segment_clipped(a, b, 1e3).unwrap();
        assert_eq!(pa, p.project(a[0], a[1], a[2]));
        assert!(pb.0.is_finite() && pb.1.is_finite() && pb.2.is_finite());
        // Wholly outside the cube: nothing to draw.
        let (a, b) = ([1e300, 0.5, 0.5], [f64::MAX, 0.5, 0.5]);
        assert!(p.project_segment_clipped(a, b, 1e3).is_none());
    }

    #[test]
    fn projection_is_right_handed_at_every_view() {
        for az in [-150.0, -120.0, -60.0, -30.0, 0.0, 45.0, 60.0, 120.0, 180.0] {
            for el in [-30.0, 10.0, 30.0, 60.0, 89.0] {
                let p = Projection3D::new(
                    View3D {
                        azimuth: az,
                        elevation: el,
                    },
                    (-1.0, 1.0),
                    (-1.0, 1.0),
                    (-1.0, 1.0),
                    100.0,
                    100.0,
                    100.0,
                );
                let ex = screen_vec(&p, [1.0, 0.0, 0.0]);
                let ey = screen_vec(&p, [0.0, 1.0, 0.0]);
                let ez = screen_vec(&p, [0.0, 0.0, 1.0]);
                assert!(
                    triple(ex, ey, ez) > 0.0,
                    "left-handed screen frame at azimuth {az}, elevation {el}"
                );
            }
        }
    }

    /// Viewed from straight above, +x points right and +y points up.
    #[test]
    fn top_down_view_has_x_right_and_y_up() {
        let p = Projection3D::new(
            View3D {
                azimuth: 0.0,
                elevation: 90.0,
            },
            (-1.0, 1.0),
            (-1.0, 1.0),
            (-1.0, 1.0),
            100.0,
            100.0,
            100.0,
        );
        let ex = screen_vec(&p, [1.0, 0.0, 0.0]);
        let ey = screen_vec(&p, [0.0, 1.0, 0.0]);
        assert!(
            ex[0] > 0.0 && ex[1].abs() < 1e-9,
            "+x should point right: {ex:?}"
        );
        assert!(
            ey[1] > 0.0 && ey[0].abs() < 1e-9,
            "+y should point up: {ey:?}"
        );
    }
}
