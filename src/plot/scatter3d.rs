use crate::plot::heatmap::ColorMap;
use crate::plot::scatter::MarkerShape;
use crate::plot::plot3d::{DataRanges3D, Box3DConfig};
use crate::render::projection::View3D;

/// A single 3D data point.
#[derive(Debug, Clone, Copy)]
pub struct Scatter3DPoint {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Builder for a 3D scatter plot.
///
/// Projects 3D data to 2D using orthographic projection with configurable
/// view angles, then renders with the existing Scene/Primitive system.
/// Renders its own wireframe box, floor grid, ticks, and axis labels —
/// standard 2D axes are skipped.
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::scatter3d::Scatter3DPlot;
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let scatter = Scatter3DPlot::new()
///     .with_data(vec![(1.0, 2.0, 3.0), (4.0, 5.0, 6.0), (7.0, 8.0, 9.0)])
///     .with_color("steelblue");
///
/// let plots = vec![Plot::Scatter3D(scatter)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("3D Scatter");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("scatter3d.svg", svg).unwrap();
/// ```
#[derive(Clone, Debug)]
pub struct Scatter3DPlot {
    pub data: Vec<Scatter3DPoint>,
    /// Uniform point color (CSS color string). Default `"steelblue"`.
    pub color: String,
    /// Marker radius in pixels. Default `3.0`.
    pub size: f64,
    /// Legend label, if any.
    pub legend_label: Option<String>,
    /// Marker shape. Default `Circle`.
    pub marker: MarkerShape,
    /// Per-point sizes (overrides `size` when set).
    pub sizes: Option<Vec<f64>>,
    /// Per-point colors (overrides `color` when set).
    pub colors: Option<Vec<String>>,
    /// Marker fill opacity (0.0–1.0).
    pub marker_opacity: Option<f64>,
    /// Marker stroke width.
    pub marker_stroke_width: Option<f64>,
    /// Fade distant points for depth cue. Default `false`.
    pub depth_shade: bool,
    /// Color points by Z value using a colormap.
    pub z_colormap: Option<ColorMap>,
    /// Shared 3D box/grid/axes configuration.
    pub box3d: Box3DConfig,
}

impl Default for Scatter3DPlot {
    fn default() -> Self { Self::new() }
}

impl Scatter3DPlot {
    /// Create a 3D scatter plot with default settings.
    pub fn new() -> Self {
        Self {
            data: vec![],
            color: "steelblue".into(),
            size: 3.0,
            legend_label: None,
            marker: MarkerShape::Circle,
            sizes: None,
            colors: None,
            marker_opacity: None,
            marker_stroke_width: None,
            depth_shade: false,
            z_colormap: None,
            box3d: Box3DConfig::default(),
        }
    }

    /// Compute axis-aligned data ranges. Returns `None` if data is empty.
    pub fn data_ranges(&self) -> Option<DataRanges3D> {
        if self.data.is_empty() { return None; }
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        let mut z_min = f64::INFINITY;
        let mut z_max = f64::NEG_INFINITY;
        for p in &self.data {
            if p.x.is_finite() { x_min = x_min.min(p.x); x_max = x_max.max(p.x); }
            if p.y.is_finite() { y_min = y_min.min(p.y); y_max = y_max.max(p.y); }
            if p.z.is_finite() { z_min = z_min.min(p.z); z_max = z_max.max(p.z); }
        }
        if !x_min.is_finite() || !y_min.is_finite() || !z_min.is_finite() { return None; }
        if (x_max - x_min).abs() < 1e-12 { x_min -= 0.5; x_max += 0.5; }
        if (y_max - y_min).abs() < 1e-12 { y_min -= 0.5; y_max += 0.5; }
        if (z_max - z_min).abs() < 1e-12 { z_min -= 0.5; z_max += 0.5; }
        Some(DataRanges3D { x: (x_min, x_max), y: (y_min, y_max), z: (z_min, z_max) })
    }

    pub fn with_data<I>(mut self, data: I) -> Self
    where I: IntoIterator<Item = (f64, f64, f64)> {
        self.data = data.into_iter().map(|(x, y, z)| Scatter3DPoint { x, y, z }).collect();
        self
    }
    pub fn with_points(mut self, points: Vec<Scatter3DPoint>) -> Self { self.data = points; self }
    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self { self.color = color.into(); self }
    pub fn with_size(mut self, size: f64) -> Self { self.size = size; self }
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self { self.legend_label = Some(label.into()); self }
    pub fn with_marker(mut self, marker: MarkerShape) -> Self { self.marker = marker; self }
    pub fn with_sizes(mut self, sizes: Vec<f64>) -> Self { self.sizes = Some(sizes); self }
    pub fn with_colors<I, S>(mut self, colors: I) -> Self
    where I: IntoIterator<Item = S>, S: Into<String> {
        self.colors = Some(colors.into_iter().map(Into::into).collect()); self
    }
    pub fn with_marker_opacity(mut self, opacity: f64) -> Self { self.marker_opacity = Some(opacity); self }
    pub fn with_marker_stroke_width(mut self, width: f64) -> Self { self.marker_stroke_width = Some(width); self }
    pub fn with_depth_shade(mut self, enable: bool) -> Self { self.depth_shade = enable; self }
    pub fn with_z_colormap(mut self, cmap: ColorMap) -> Self { self.z_colormap = Some(cmap); self }

    // Delegate 3D box/axes config through Box3DConfig methods
    pub fn with_azimuth(mut self, deg: f64) -> Self { self.box3d = self.box3d.with_azimuth(deg); self }
    pub fn with_elevation(mut self, deg: f64) -> Self { self.box3d = self.box3d.with_elevation(deg); self }
    pub fn with_view(mut self, v: View3D) -> Self { self.box3d = self.box3d.with_view(v); self }
    pub fn with_x_label<S: Into<String>>(mut self, l: S) -> Self { self.box3d = self.box3d.with_x_label(l); self }
    pub fn with_y_label<S: Into<String>>(mut self, l: S) -> Self { self.box3d = self.box3d.with_y_label(l); self }
    pub fn with_z_label<S: Into<String>>(mut self, l: S) -> Self { self.box3d = self.box3d.with_z_label(l); self }
    pub fn with_show_grid(mut self, s: bool) -> Self { self.box3d = self.box3d.with_show_grid(s); self }
    pub fn with_show_box(mut self, s: bool) -> Self { self.box3d = self.box3d.with_show_box(s); self }
    pub fn with_grid_lines(mut self, n: usize) -> Self { self.box3d = self.box3d.with_grid_lines(n); self }
    pub fn with_z_axis_right(mut self, r: bool) -> Self { self.box3d = self.box3d.with_z_axis_right(r); self }
}
