//! Shared types for 3D plot types (Scatter3D, Surface3D).

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
    pub(crate) fn depth_row(&self) -> [f64; 3] {
        let az = self.azimuth.to_radians();
        let el = self.elevation.to_radians();
        [az.sin() * el.cos(), az.cos() * el.cos(), -el.sin()]
    }

    /// Find the floor-face corner closest to the viewer (smallest depth).
    /// Returns the normalized (x, y) signs of that corner, e.g. (0.5, -0.5).
    /// This is the "open front corner" where axes originate.
    ///
    /// For positive elevation the floor is z=-0.5; for negative elevation
    /// (viewing from below) the floor is z=+0.5.
    pub fn front_bottom_corner(&self) -> (f64, f64) {
        let row1 = self.depth_row();
        let floor_z = if self.elevation >= 0.0 { -0.5 } else { 0.5 };
        let mut best_x = -0.5_f64;
        let mut best_y = -0.5_f64;
        let mut best_d = f64::INFINITY;
        for &nx in &[-0.5_f64, 0.5] {
            for &ny in &[-0.5_f64, 0.5] {
                let d = row1[0] * nx + row1[1] * ny + row1[2] * floor_z;
                if d < best_d {
                    best_d = d;
                    best_x = nx;
                    best_y = ny;
                }
            }
        }
        (best_x, best_y)
    }

    /// Derive which screen side the Z axis should appear on for this view.
    ///
    /// When the front corner is at `fc_x >= 0` (e.g. default azimuth -60°),
    /// the rightmost back edge is the natural Z axis position.  When the view
    /// is mirrored (`fc_x < 0`, e.g. azimuth +60°), the leftmost edge reads
    /// more naturally.  This matches matplotlib's default behaviour across all
    /// azimuths without the user needing to know about it.
    pub fn auto_z_axis_right(&self) -> bool {
        self.front_bottom_corner().0 >= 0.0
    }
}

/// Axis-aligned bounding box for 3D data.
#[derive(Debug, Clone, Copy)]
pub struct DataRanges3D {
    pub x: (f64, f64),
    pub y: (f64, f64),
    pub z: (f64, f64),
}

/// Shared configuration for the 3D open-box wireframe, grid, and axes.
/// Embedded by both `Scatter3DPlot` and `Surface3DPlot`.
#[derive(Clone, Debug)]
pub struct Box3DConfig {
    pub view: View3D,
    pub x_label: Option<String>,
    pub y_label: Option<String>,
    pub z_label: Option<String>,
    pub show_grid: bool,
    pub show_box: bool,
    pub grid_lines: usize,
    /// Override Z-axis side: `Some(true)` = right, `Some(false)` = left, `None` = auto.
    /// Auto derives the side from the view angles — right when `fc_x >= 0`, left otherwise.
    pub z_axis_right: Option<bool>,
}

impl Default for Box3DConfig {
    fn default() -> Self {
        Self {
            view: View3D::default(),
            x_label: None,
            y_label: None,
            z_label: None,
            show_grid: true,
            show_box: true,
            grid_lines: 5,
            z_axis_right: None,
        }
    }
}

impl Box3DConfig {
    pub fn with_azimuth(mut self, deg: f64) -> Self { self.view.azimuth = deg; self }
    pub fn with_elevation(mut self, deg: f64) -> Self { self.view.elevation = deg; self }
    pub fn with_view(mut self, v: View3D) -> Self { self.view = v; self }
    pub fn with_x_label<S: Into<String>>(mut self, l: S) -> Self { self.x_label = Some(l.into()); self }
    pub fn with_y_label<S: Into<String>>(mut self, l: S) -> Self { self.y_label = Some(l.into()); self }
    pub fn with_z_label<S: Into<String>>(mut self, l: S) -> Self { self.z_label = Some(l.into()); self }
    pub fn with_no_grid(mut self) -> Self { self.show_grid = false; self }
    pub fn with_no_box(mut self) -> Self { self.show_box = false; self }
    pub fn with_grid_lines(mut self, n: usize) -> Self { self.grid_lines = n; self }
    /// Force the Z axis to a specific side. Pass `true` for right, `false` for left.
    /// Call `.with_z_axis_auto()` to restore automatic placement.
    pub fn with_z_axis_right(mut self, r: bool) -> Self { self.z_axis_right = Some(r); self }
    /// Use automatic Z-axis placement based on the view angles (default).
    pub fn with_z_axis_auto(mut self) -> Self { self.z_axis_right = None; self }
}
