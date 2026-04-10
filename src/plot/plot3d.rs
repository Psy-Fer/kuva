//! Shared types for 3D plot types (Scatter3D, Surface3D).

use crate::render::projection::View3D;

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
    pub z_axis_right: bool,
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
            z_axis_right: true,
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
    pub fn with_show_grid(mut self, s: bool) -> Self { self.show_grid = s; self }
    pub fn with_show_box(mut self, s: bool) -> Self { self.show_box = s; self }
    pub fn with_grid_lines(mut self, n: usize) -> Self { self.grid_lines = n; self }
    pub fn with_z_axis_right(mut self, r: bool) -> Self { self.z_axis_right = r; self }
}
