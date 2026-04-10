use crate::plot::heatmap::ColorMap;
use crate::plot::plot3d::{DataRanges3D, Box3DConfig, View3D};

/// A 3D surface plot rendered as a depth-sorted quadrilateral mesh.
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::surface3d::Surface3DPlot;
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let z_data: Vec<Vec<f64>> = (0..20).map(|i| {
///     (0..20).map(|j| {
///         let x = (i as f64 - 10.0) / 5.0;
///         let y = (j as f64 - 10.0) / 5.0;
///         (x * x + y * y).sqrt().sin()
///     }).collect()
/// }).collect();
///
/// let surface = Surface3DPlot::new(z_data)
///     .with_z_colormap(kuva::plot::heatmap::ColorMap::Viridis);
///
/// let plots = vec![Plot::Surface3D(surface)];
/// let layout = Layout::auto_from_plots(&plots).with_title("Surface");
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("surface3d.svg", svg).unwrap();
/// ```
#[derive(Clone, Debug)]
pub struct Surface3DPlot {
    /// Grid Z values: z_data[row][col]. All rows must have the same length.
    pub z_data: Vec<Vec<f64>>,
    /// Explicit X coordinates for each column. If None, 0..ncols.
    pub x_coords: Option<Vec<f64>>,
    /// Explicit Y coordinates for each row. If None, 0..nrows.
    pub y_coords: Option<Vec<f64>>,
    /// Uniform surface color. Default `"steelblue"`.
    pub color: String,
    /// Color faces by average Z value.
    pub z_colormap: Option<ColorMap>,
    /// Show wireframe edges on mesh faces. Default `true`.
    pub show_wireframe: bool,
    /// Wireframe edge color. Default `"#333333"`.
    pub wireframe_color: String,
    /// Wireframe stroke width. Default `0.5`.
    pub wireframe_width: f64,
    /// Surface opacity (0.0–1.0). Default `1.0`.
    pub alpha: f64,
    /// Legend label, if any.
    pub legend_label: Option<String>,
    /// Shared 3D box/grid/axes configuration.
    pub box3d: Box3DConfig,
}

impl Default for Surface3DPlot {
    fn default() -> Self { Self::new(vec![]) }
}

impl Surface3DPlot {
    pub fn new(z_data: Vec<Vec<f64>>) -> Self {
        Self {
            z_data,
            x_coords: None,
            y_coords: None,
            color: "steelblue".into(),
            z_colormap: None,
            show_wireframe: true,
            wireframe_color: "#333333".into(),
            wireframe_width: 0.5,
            alpha: 1.0,
            legend_label: None,
            box3d: Box3DConfig::default(),
        }
    }

    pub fn nrows(&self) -> usize { self.z_data.len() }
    pub fn ncols(&self) -> usize { self.z_data.first().map_or(0, |r| r.len()) }

    pub fn data_ranges(&self) -> Option<DataRanges3D> {
        let nrows = self.nrows();
        let ncols = self.ncols();
        if nrows < 2 || ncols < 2 { return None; }

        let (x_min, x_max) = if let Some(ref xc) = self.x_coords {
            (xc.iter().cloned().fold(f64::INFINITY, f64::min),
             xc.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
        } else {
            (0.0, (ncols - 1) as f64)
        };
        let (y_min, y_max) = if let Some(ref yc) = self.y_coords {
            (yc.iter().cloned().fold(f64::INFINITY, f64::min),
             yc.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
        } else {
            (0.0, (nrows - 1) as f64)
        };

        let mut z_min = f64::INFINITY;
        let mut z_max = f64::NEG_INFINITY;
        for row in &self.z_data {
            for &z in row {
                if z.is_finite() { z_min = z_min.min(z); z_max = z_max.max(z); }
            }
        }
        if !z_min.is_finite() || !z_max.is_finite() { return None; }

        let mut xr = (x_min, x_max);
        let mut yr = (y_min, y_max);
        let mut zr = (z_min, z_max);
        if (xr.1 - xr.0).abs() < 1e-12 { xr.0 -= 0.5; xr.1 += 0.5; }
        if (yr.1 - yr.0).abs() < 1e-12 { yr.0 -= 0.5; yr.1 += 0.5; }
        if (zr.1 - zr.0).abs() < 1e-12 { zr.0 -= 0.5; zr.1 += 0.5; }

        Some(DataRanges3D { x: xr, y: yr, z: zr })
    }

    pub fn x_at(&self, j: usize) -> f64 {
        self.x_coords.as_ref().map_or(j as f64, |xc| {
            xc.get(j).copied().unwrap_or(j as f64)
        })
    }
    pub fn y_at(&self, i: usize) -> f64 {
        self.y_coords.as_ref().map_or(i as f64, |yc| {
            yc.get(i).copied().unwrap_or(i as f64)
        })
    }

    /// Generate a surface from a function `f(x, y) -> z` over the given ranges.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::surface3d::Surface3DPlot;
    /// let surface = Surface3DPlot::new(vec![])
    ///     .with_data_fn(|x, y| x * x + y * y, -2.0..=2.0, -2.0..=2.0, 40, 40);
    /// ```
    pub fn with_data_fn<F>(
        mut self, f: F,
        x_range: std::ops::RangeInclusive<f64>, y_range: std::ops::RangeInclusive<f64>,
        x_res: usize, y_res: usize,
    ) -> Self where F: Fn(f64, f64) -> f64 {
        let x_res = x_res.max(2);
        let y_res = y_res.max(2);
        let (x_start, x_end) = (*x_range.start(), *x_range.end());
        let (y_start, y_end) = (*y_range.start(), *y_range.end());
        let xs: Vec<f64> = (0..x_res).map(|i| {
            x_start + (x_end - x_start) * i as f64 / (x_res - 1) as f64
        }).collect();
        let ys: Vec<f64> = (0..y_res).map(|i| {
            y_start + (y_end - y_start) * i as f64 / (y_res - 1) as f64
        }).collect();
        self.z_data = ys.iter().map(|&y| xs.iter().map(|&x| f(x, y)).collect()).collect();
        self.x_coords = Some(xs);
        self.y_coords = Some(ys);
        self
    }

    pub fn with_z_data(mut self, z_data: Vec<Vec<f64>>) -> Self { self.z_data = z_data; self }
    pub fn with_x_coords(mut self, xc: Vec<f64>) -> Self { self.x_coords = Some(xc); self }
    pub fn with_y_coords(mut self, yc: Vec<f64>) -> Self { self.y_coords = Some(yc); self }
    pub fn with_color<S: Into<String>>(mut self, c: S) -> Self { self.color = c.into(); self }
    pub fn with_z_colormap(mut self, cm: ColorMap) -> Self { self.z_colormap = Some(cm); self }
    pub fn with_no_wireframe(mut self) -> Self { self.show_wireframe = false; self }
    pub fn with_wireframe_color<S: Into<String>>(mut self, c: S) -> Self { self.wireframe_color = c.into(); self }
    pub fn with_wireframe_width(mut self, w: f64) -> Self { self.wireframe_width = w; self }
    pub fn with_alpha(mut self, a: f64) -> Self { self.alpha = a; self }
    pub fn with_legend<S: Into<String>>(mut self, l: S) -> Self { self.legend_label = Some(l.into()); self }

    // Delegate 3D box/axes config through Box3DConfig methods
    pub fn with_azimuth(mut self, deg: f64) -> Self { self.box3d = self.box3d.with_azimuth(deg); self }
    pub fn with_elevation(mut self, deg: f64) -> Self { self.box3d = self.box3d.with_elevation(deg); self }
    pub fn with_view(mut self, v: View3D) -> Self { self.box3d = self.box3d.with_view(v); self }
    pub fn with_x_label<S: Into<String>>(mut self, l: S) -> Self { self.box3d = self.box3d.with_x_label(l); self }
    pub fn with_y_label<S: Into<String>>(mut self, l: S) -> Self { self.box3d = self.box3d.with_y_label(l); self }
    pub fn with_z_label<S: Into<String>>(mut self, l: S) -> Self { self.box3d = self.box3d.with_z_label(l); self }
    pub fn with_no_grid(mut self) -> Self { self.box3d = self.box3d.with_no_grid(); self }
    pub fn with_no_box(mut self) -> Self { self.box3d = self.box3d.with_no_box(); self }
    pub fn with_grid_lines(mut self, n: usize) -> Self { self.box3d = self.box3d.with_grid_lines(n); self }
    pub fn with_z_axis_right(mut self, r: bool) -> Self { self.box3d = self.box3d.with_z_axis_right(r); self }
    pub fn with_z_axis_auto(mut self) -> Self { self.box3d = self.box3d.with_z_axis_auto(); self }
}
