/// 3D→2D projection for orthographic rendering of 3D plot types.
///
/// The projection pipeline:
/// 1. Normalize data coordinates to [-0.5, 0.5]³ using data ranges
/// 2. Rotate by combined matrix Rx(elevation) * Rz(azimuth)
/// 3. Orthographic projection: screen_x = -rotated[0], screen_y = -rotated[2]
/// 4. Scale uniformly to fit the plot area
/// 5. Translate to center in the pixel bounding box

/// Viewing angles for 3D projection.
#[derive(Debug, Clone, Copy)]
pub struct View3D {
    /// Azimuth angle in degrees (rotation around Z-axis). Default: -60.
    pub azimuth: f64,
    /// Elevation angle in degrees (rotation from XY-plane). Default: 30.
    pub elevation: f64,
}

impl Default for View3D {
    fn default() -> Self {
        Self { azimuth: -60.0, elevation: 30.0 }
    }
}

impl View3D {
    /// Compute the rotation matrix row 1 (depth axis) for these view angles.
    /// Used to determine depth without building a full Projection3D.
    fn depth_row(&self) -> [f64; 3] {
        let az = self.azimuth.to_radians();
        let el = self.elevation.to_radians();
        [az.sin() * el.cos(), az.cos() * el.cos(), -el.sin()]
    }

    /// Find the bottom-face corner closest to the viewer (smallest depth).
    /// Returns the normalized (x, y) signs of that corner, e.g. (0.5, -0.5).
    /// This is the "open front corner" where axes originate.
    pub fn front_bottom_corner(&self) -> (f64, f64) {
        let row1 = self.depth_row();
        let mut best_x = -0.5_f64;
        let mut best_y = -0.5_f64;
        let mut best_d = f64::INFINITY;
        for &nx in &[-0.5_f64, 0.5] {
            for &ny in &[-0.5_f64, 0.5] {
                let d = row1[0] * nx + row1[1] * ny + row1[2] * (-0.5);
                if d < best_d {
                    best_d = d;
                    best_x = nx;
                    best_y = ny;
                }
            }
        }
        (best_x, best_y)
    }
}

/// Pre-computed 3D→2D orthographic projection.
///
/// Projects data-space (x, y, z) coordinates to pixel-space (screen_x, screen_y, depth).
/// Depth is used for painter's algorithm sorting (back-to-front rendering).
pub struct Projection3D {
    /// Combined rotation matrix (3×3 stored row-major as [row][col])
    rot: [[f64; 3]; 3],
    /// Data range normalization: (min, 1/span) for each axis
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
            [cos_az,           -sin_az,            0.0    ],
            [sin_az * cos_el,   cos_az * cos_el,  -sin_el ],
            [sin_az * sin_el,   cos_az * sin_el,   cos_el ],
        ];

        let norm_x = Self::norm_params(x_range);
        let norm_y = Self::norm_params(y_range);
        let norm_z = Self::norm_params(z_range);

        // Project all 8 corners of the unit cube to find bounding box.
        // screen_x = -rx to match the standard right-handed convention
        // where +X data → right on screen at azimuth=-60°.
        let mut sx_min = f64::INFINITY;
        let mut sx_max = f64::NEG_INFINITY;
        let mut sy_min = f64::INFINITY;
        let mut sy_max = f64::NEG_INFINITY;

        for &nx in &[-0.5_f64, 0.5] {
            for &ny in &[-0.5_f64, 0.5] {
                for &nz in &[-0.5_f64, 0.5] {
                    let rx = rot[0][0] * nx + rot[0][1] * ny + rot[0][2] * nz;
                    let rz = rot[2][0] * nx + rot[2][1] * ny + rot[2][2] * nz;
                    let sx = -rx; // negate for right-handed screen convention
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

    fn norm_params(range: (f64, f64)) -> (f64, f64) {
        let span = range.1 - range.0;
        if span.abs() < 1e-15 {
            (range.0, 1.0) // degenerate range → center at 0
        } else {
            (range.0, 1.0 / span)
        }
    }

    /// Project a data-space point to pixel-space.
    /// Returns `(screen_x, screen_y, depth)` where depth is used for sorting.
    #[inline]
    pub fn project(&self, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        let nx = (x - self.norm_x.0) * self.norm_x.1 - 0.5;
        let ny = (y - self.norm_y.0) * self.norm_y.1 - 0.5;
        let nz = (z - self.norm_z.0) * self.norm_z.1 - 0.5;
        self.project_normalized(nx, ny, nz)
    }

    /// Project a point already in normalized [-0.5, 0.5]³ space.
    /// Used for axes/grid (unit cube coords).
    #[inline]
    pub fn project_normalized(&self, nx: f64, ny: f64, nz: f64) -> (f64, f64, f64) {
        let rx = self.rot[0][0] * nx + self.rot[0][1] * ny + self.rot[0][2] * nz;
        let ry = self.rot[1][0] * nx + self.rot[1][1] * ny + self.rot[1][2] * nz;
        let rz = self.rot[2][0] * nx + self.rot[2][1] * ny + self.rot[2][2] * nz;

        let sx = -rx * self.scale + self.offset_x; // negate for right-handed screen convention
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
