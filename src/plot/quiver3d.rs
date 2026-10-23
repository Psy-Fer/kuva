//! 3D quiver (vector field) plot.

use crate::plot::colormap::ColorMap;
use crate::plot::plot3d::{Box3DConfig, DataRanges3D, View3D};
use crate::plot::quiver::QuiverPivot;

/// A single arrow in a 3D quiver plot.
///
/// `(x, y, z)` is the arrow's anchor in data coordinates — where on the arrow
/// that anchor sits is set by [`Quiver3DPlot::pivot`] (the tail by default).
/// `(u, v, w)` is the vector displacement, also in data coordinates; the
/// rendered length is `(u, v, w)` multiplied by the plot-level scale.
#[derive(Debug, Clone)]
pub struct Quiver3DArrow {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub u: f64,
    pub v: f64,
    pub w: f64,
    /// Per-arrow color override. `None` uses the plot-level color or colormap.
    pub color: Option<String>,
}

impl Quiver3DArrow {
    /// Euclidean length of the `(u, v, w)` vector.
    pub fn magnitude(&self) -> f64 {
        (self.u * self.u + self.v * self.v + self.w * self.w).sqrt()
    }

    fn all_finite(&self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.z.is_finite()
            && self.u.is_finite()
            && self.v.is_finite()
            && self.w.is_finite()
    }
}

/// Builder for a 3D quiver (vector field) plot.
///
/// Each arrow is anchored at `(x, y, z)` and points along `(u, v, w)`. Arrows
/// are projected with the same orthographic camera as
/// [`Scatter3DPlot`](crate::plot::scatter3d::Scatter3DPlot) and drawn inside the same
/// open-box wireframe, painted back to front. Arrow heads are flat triangles
/// built in screen space along the projected shaft, so every arrow reads as
/// an arrow from any view angle.
///
/// # Scaling
///
/// By default the scale multiplier is auto-computed so the longest arrow is
/// roughly one grid cell long, measured in the box's own normalised
/// coordinates so every axis stays legible even on very different data
/// ranges. Pin it with [`Quiver3DPlot::with_scale`] or change the target
/// fraction with [`Quiver3DPlot::with_auto_scale`].
///
/// # Bounds
///
/// The coordinate box encloses every arrow, tail to tip, so nothing is drawn
/// outside it. [`Quiver3DPlot::with_tight_bounds`] sizes the box from the
/// anchors alone instead, letting the outer layer of arrows extend past the
/// faces. Arrows are never clipped in either mode.
///
/// # Coloring
///
/// Three modes, in order of priority:
/// 1. Per-arrow color via [`Quiver3DArrow::color`].
/// 2. Magnitude-driven colormap via [`Quiver3DPlot::with_color_map`]; a
///    colorbar is rendered automatically.
/// 3. Single color via [`Quiver3DPlot::with_color`]. Default `"steelblue"`.
///
/// # Painter's algorithm
///
/// Arrows are sorted by the depth of their projected midpoint and drawn back
/// to front. Two arrows that cross each other in 3D can therefore overlap in
/// the wrong order; there is no per-pixel depth test in a vector backend.
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::Quiver3DPlot;
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// // Vortex around the z axis with a gentle updraft.
/// let plot = Quiver3DPlot::from_function(
///     (-2.0, 2.0, 5),
///     (-2.0, 2.0, 5),
///     (0.0, 4.0, 5),
///     |x, y, _z| (-y, x, 0.5),
/// )
/// .with_color("steelblue");
///
/// let plots = vec![Plot::Quiver3D(plot)];
/// let layout = Layout::auto_from_plots(&plots).with_title("3D Vector Field");
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("quiver3d.svg", svg).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Quiver3DPlot {
    pub arrows: Vec<Quiver3DArrow>,
    /// Default arrow color (used when no colormap and no per-arrow color).
    /// Default `"steelblue"`.
    pub color: String,
    /// Multiplier applied to `(u, v, w)` before projection.
    ///
    /// - `Some(s)` — use `s` directly (set via [`Quiver3DPlot::with_scale`]).
    /// - `None` — auto-compute so the longest arrow is roughly one grid cell
    ///   long (`≈ span / ∛n`).
    pub scale: Option<f64>,
    /// Fraction of the grid-cell size used for the longest arrow when
    /// `scale` is `None`. Default `0.9`.
    pub auto_scale_fraction: f64,
    /// Shaft width in pixels. Default `1.2`.
    pub shaft_width: f64,
    /// Explicit head length in pixels. `None` → proportional to the projected
    /// shaft length (see [`Quiver3DPlot::head_ratio`]).
    pub head_length: Option<f64>,
    /// Explicit head half-width in pixels. `None` → `head_aspect * head_length`.
    pub head_width: Option<f64>,
    /// Head length as a fraction of the projected shaft length when
    /// `head_length` is `None`. Default `0.28`.
    pub head_ratio: f64,
    /// Head half-width as a fraction of head length when `head_width` is
    /// `None`. Default `0.45`.
    pub head_aspect: f64,
    /// Minimum proportional head length in pixels. Default `4.0`.
    pub head_min_px: f64,
    /// Maximum proportional head length in pixels. Default `14.0`.
    pub head_max_px: f64,
    /// Optional colormap applied to arrow magnitude. When set, overrides
    /// [`Quiver3DPlot::color`] for arrows without a per-arrow override.
    pub color_map: Option<ColorMap>,
    /// Optional explicit magnitude range `(min, max)` for colormap
    /// normalization. `None` derives it from the data.
    pub color_range: Option<(f64, f64)>,
    /// Label shown next to the colorbar (when a colormap is active).
    pub color_legend_label: Option<String>,
    pub legend_label: Option<String>,
    /// Where `(x, y, z)` sits on the rendered arrow. Default `Tail`.
    pub pivot: QuiverPivot,
    /// Fade distant arrows for a depth cue. Default `false`.
    pub depth_shade: bool,
    /// When `true`, the 3D box is sized from the arrow anchors only and the
    /// outer arrows extend past its faces. Default `false` (the box encloses
    /// every tail and tip).
    pub tight_bounds: bool,
    /// Shared 3D box/grid/axes configuration.
    pub box3d: Box3DConfig,
}

impl Default for Quiver3DPlot {
    fn default() -> Self {
        Self::new()
    }
}

impl Quiver3DPlot {
    /// Create an empty 3D quiver plot with default styling.
    pub fn new() -> Self {
        Self {
            arrows: vec![],
            color: "steelblue".into(),
            scale: None,
            auto_scale_fraction: 0.9,
            shaft_width: 1.2,
            head_length: None,
            head_width: None,
            head_ratio: 0.28,
            head_aspect: 0.45,
            head_min_px: 4.0,
            head_max_px: 14.0,
            color_map: None,
            color_range: None,
            color_legend_label: None,
            legend_label: None,
            pivot: QuiverPivot::Tail,
            depth_shade: false,
            tight_bounds: false,
            box3d: Box3DConfig::default(),
        }
    }

    /// Sample a vector-field closure on a regular `nx × ny × nz` grid.
    ///
    /// Each range is a `(min, max, n)` tuple where `n` is the number of
    /// samples along that axis, endpoints inclusive; `n == 1` samples the
    /// midpoint of the range. The closure receives `(x, y, z)` and returns
    /// `(u, v, w)`.
    ///
    /// ```rust
    /// use kuva::plot::Quiver3DPlot;
    /// let plot = Quiver3DPlot::from_function(
    ///     (-1.0, 1.0, 4),
    ///     (-1.0, 1.0, 4),
    ///     (0.0, 2.0, 3),
    ///     |x, y, _z| (-y, x, 0.2),
    /// );
    /// assert_eq!(plot.arrows.len(), 48);
    /// ```
    pub fn from_function<F>(
        x_range: (f64, f64, usize),
        y_range: (f64, f64, usize),
        z_range: (f64, f64, usize),
        mut f: F,
    ) -> Self
    where
        F: FnMut(f64, f64, f64) -> (f64, f64, f64),
    {
        let (x_min, x_max, nx) = x_range;
        let (y_min, y_max, ny) = y_range;
        let (z_min, z_max, nz) = z_range;
        let step = |lo: f64, hi: f64, n: usize, i: usize| -> f64 {
            if n <= 1 {
                (lo + hi) * 0.5
            } else {
                lo + (hi - lo) * (i as f64) / (n - 1) as f64
            }
        };
        let mut plot = Self::new();
        plot.arrows
            .reserve(nx.saturating_mul(ny).saturating_mul(nz));
        for i in 0..nx {
            let x = step(x_min, x_max, nx, i);
            for j in 0..ny {
                let y = step(y_min, y_max, ny, j);
                for k in 0..nz {
                    let z = step(z_min, z_max, nz, k);
                    let (u, v, w) = f(x, y, z);
                    plot = plot.with_arrow(x, y, z, u, v, w);
                }
            }
        }
        plot
    }

    /// Add one arrow anchored at `(x, y, z)` with vector `(u, v, w)`.
    /// Non-finite values (NaN / infinity) are silently dropped.
    pub fn with_arrow(
        self,
        x: impl Into<f64>,
        y: impl Into<f64>,
        z: impl Into<f64>,
        u: impl Into<f64>,
        v: impl Into<f64>,
        w: impl Into<f64>,
    ) -> Self {
        self.push(Quiver3DArrow {
            x: x.into(),
            y: y.into(),
            z: z.into(),
            u: u.into(),
            v: v.into(),
            w: w.into(),
            color: None,
        })
    }

    /// Add many arrows from an iterator of `(x, y, z, u, v, w)` tuples.
    /// Non-finite rows are silently dropped.
    pub fn with_arrows<I>(mut self, arrows: I) -> Self
    where
        I: IntoIterator<Item = (f64, f64, f64, f64, f64, f64)>,
    {
        for (x, y, z, u, v, w) in arrows {
            self = self.with_arrow(x, y, z, u, v, w);
        }
        self
    }

    /// Add an arrow with a per-arrow color override. Non-finite values are
    /// silently dropped.
    #[allow(clippy::too_many_arguments)]
    pub fn with_colored_arrow(
        self,
        x: impl Into<f64>,
        y: impl Into<f64>,
        z: impl Into<f64>,
        u: impl Into<f64>,
        v: impl Into<f64>,
        w: impl Into<f64>,
        color: impl Into<String>,
    ) -> Self {
        self.push(Quiver3DArrow {
            x: x.into(),
            y: y.into(),
            z: z.into(),
            u: u.into(),
            v: v.into(),
            w: w.into(),
            color: Some(color.into()),
        })
    }

    fn push(mut self, arrow: Quiver3DArrow) -> Self {
        if arrow.all_finite() {
            self.arrows.push(arrow);
        }
        self
    }

    /// Set the default arrow color.
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = color.into();
        self
    }

    /// Pin the scale multiplier to an explicit value, disabling auto-scaling.
    ///
    /// # Panics
    /// Panics in debug builds if `s` is not finite.
    pub fn with_scale(mut self, s: impl Into<f64>) -> Self {
        let s = s.into();
        debug_assert!(
            s.is_finite(),
            "Quiver3DPlot::with_scale: scale must be finite, got {s}"
        );
        self.scale = Some(s);
        self
    }

    /// Keep auto-scaling on but change the fraction of a grid cell the
    /// longest arrow should span. Default `0.9`; values above `1.0` let
    /// neighbouring arrows overlap.
    ///
    /// # Panics
    /// Panics in debug builds if `fraction` is not finite.
    pub fn with_auto_scale(mut self, fraction: impl Into<f64>) -> Self {
        let fraction = fraction.into();
        debug_assert!(
            fraction.is_finite(),
            "Quiver3DPlot::with_auto_scale: fraction must be finite, got {fraction}"
        );
        self.scale = None;
        self.auto_scale_fraction = fraction;
        self
    }

    /// Set the shaft width in pixels. Default `1.2`.
    pub fn with_shaft_width(mut self, w: impl Into<f64>) -> Self {
        let w = w.into();
        debug_assert!(
            w.is_finite() && w >= 0.0,
            "Quiver3DPlot::with_shaft_width: width must be finite and non-negative, got {w}"
        );
        self.shaft_width = w;
        self
    }

    /// Pin arrow head dimensions to explicit pixel values. `length` is along
    /// the shaft, `half_width` perpendicular to it. Heads are still capped at
    /// the projected shaft length.
    pub fn with_head(mut self, length: impl Into<f64>, half_width: impl Into<f64>) -> Self {
        let (length, half_width) = (length.into(), half_width.into());
        debug_assert!(
            length.is_finite() && half_width.is_finite(),
            "Quiver3DPlot::with_head: length and half_width must be finite"
        );
        self.head_length = Some(length);
        self.head_width = Some(half_width);
        self
    }

    /// Pin just the head length in pixels.
    pub fn with_head_length(mut self, length: impl Into<f64>) -> Self {
        let length = length.into();
        debug_assert!(
            length.is_finite(),
            "Quiver3DPlot::with_head_length: must be finite"
        );
        self.head_length = Some(length);
        self
    }

    /// Pin just the head half-width in pixels.
    pub fn with_head_width(mut self, half_width: impl Into<f64>) -> Self {
        let half_width = half_width.into();
        debug_assert!(
            half_width.is_finite(),
            "Quiver3DPlot::with_head_width: must be finite"
        );
        self.head_width = Some(half_width);
        self
    }

    /// Head length as a fraction of the projected shaft length (used when no
    /// explicit head length is set). Default `0.28`.
    pub fn with_head_ratio(mut self, ratio: impl Into<f64>) -> Self {
        let ratio = ratio.into();
        debug_assert!(
            ratio.is_finite(),
            "Quiver3DPlot::with_head_ratio: must be finite"
        );
        self.head_ratio = ratio;
        self
    }

    /// Head half-width as a fraction of head length (used when no explicit
    /// head width is set). Default `0.45`.
    pub fn with_head_aspect(mut self, aspect: impl Into<f64>) -> Self {
        let aspect = aspect.into();
        debug_assert!(
            aspect.is_finite(),
            "Quiver3DPlot::with_head_aspect: must be finite"
        );
        self.head_aspect = aspect;
        self
    }

    /// Minimum proportional head length in pixels, so short arrows keep a
    /// visible head. Default `4.0`.
    pub fn with_head_min_px(mut self, px: impl Into<f64>) -> Self {
        let px = px.into();
        debug_assert!(
            px.is_finite(),
            "Quiver3DPlot::with_head_min_px: must be finite"
        );
        self.head_min_px = px;
        self
    }

    /// Maximum proportional head length in pixels, so long arrows do not grow
    /// gigantic heads. Default `14.0`.
    pub fn with_head_max_px(mut self, px: impl Into<f64>) -> Self {
        let px = px.into();
        debug_assert!(
            px.is_finite(),
            "Quiver3DPlot::with_head_max_px: must be finite"
        );
        self.head_max_px = px;
        self
    }

    /// Color arrows by magnitude using the given colormap.
    pub fn with_color_map(mut self, cmap: ColorMap) -> Self {
        self.color_map = Some(cmap);
        self
    }

    /// Override the magnitude range used for colormap normalization.
    pub fn with_color_range(mut self, lo: impl Into<f64>, hi: impl Into<f64>) -> Self {
        let (lo, hi) = (lo.into(), hi.into());
        debug_assert!(
            lo.is_finite() && hi.is_finite(),
            "Quiver3DPlot::with_color_range: lo and hi must be finite"
        );
        self.color_range = Some((lo, hi));
        self
    }

    /// Label rendered next to the colorbar (when a colormap is active).
    pub fn with_color_legend_label(mut self, label: impl Into<String>) -> Self {
        self.color_legend_label = Some(label.into());
        self
    }

    /// Shorthand for "color by magnitude with a labeled colorbar":
    /// `self.with_color_map(cmap).with_color_legend_label(label)`.
    pub fn with_magnitude_colormap(mut self, cmap: ColorMap, label: impl Into<String>) -> Self {
        self.color_map = Some(cmap);
        self.color_legend_label = Some(label.into());
        self
    }

    /// Attach a legend entry for this series.
    pub fn with_legend(mut self, label: impl Into<String>) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    /// Where `(x, y, z)` sits on the rendered arrow. Default `Tail`.
    pub fn with_pivot(mut self, pivot: QuiverPivot) -> Self {
        self.pivot = pivot;
        self
    }

    /// Fade distant arrows for a depth cue.
    pub fn with_depth_shade(mut self) -> Self {
        self.depth_shade = true;
        self
    }

    /// Size the 3D box from the arrow anchors only. The sampled region then
    /// fills the box and the outer layer of arrows extends past its faces,
    /// which reads naturally for dense fields but can run into the axis
    /// tick labels on coarse ones. Arrows are never clipped.
    pub fn with_tight_bounds(mut self) -> Self {
        self.tight_bounds = true;
        self
    }

    // Delegate 3D box/axes config through Box3DConfig methods
    pub fn with_azimuth(mut self, deg: f64) -> Self {
        self.box3d = self.box3d.with_azimuth(deg);
        self
    }
    pub fn with_elevation(mut self, deg: f64) -> Self {
        self.box3d = self.box3d.with_elevation(deg);
        self
    }
    pub fn with_view(mut self, v: View3D) -> Self {
        self.box3d = self.box3d.with_view(v);
        self
    }
    pub fn with_x_label<S: Into<String>>(mut self, l: S) -> Self {
        self.box3d = self.box3d.with_x_label(l);
        self
    }
    pub fn with_y_label<S: Into<String>>(mut self, l: S) -> Self {
        self.box3d = self.box3d.with_y_label(l);
        self
    }
    pub fn with_z_label<S: Into<String>>(mut self, l: S) -> Self {
        self.box3d = self.box3d.with_z_label(l);
        self
    }
    pub fn with_no_grid(mut self) -> Self {
        self.box3d = self.box3d.with_no_grid();
        self
    }
    pub fn with_no_box(mut self) -> Self {
        self.box3d = self.box3d.with_no_box();
        self
    }
    pub fn with_grid_lines(mut self, n: usize) -> Self {
        self.box3d = self.box3d.with_grid_lines(n);
        self
    }
    pub fn with_z_axis_right(mut self, r: bool) -> Self {
        self.box3d = self.box3d.with_z_axis_right(r);
        self
    }
    pub fn with_z_axis_auto(mut self) -> Self {
        self.box3d = self.box3d.with_z_axis_auto();
        self
    }

    /// Raw `(min, max)` extent of the arrow anchors on each axis, or `None`
    /// when there are no arrows.
    fn anchor_extent(&self) -> Option<DataRanges3D> {
        let first = self.arrows.first()?;
        let mut r = DataRanges3D {
            x: (first.x, first.x),
            y: (first.y, first.y),
            z: (first.z, first.z),
        };
        for a in &self.arrows {
            r.x = (r.x.0.min(a.x), r.x.1.max(a.x));
            r.y = (r.y.0.min(a.y), r.y.1.max(a.y));
            r.z = (r.z.0.min(a.z), r.z.1.max(a.z));
        }
        Some(r)
    }

    /// Pad a zero-span axis to `±0.5` so the box stays a box.
    fn pad(r: DataRanges3D) -> DataRanges3D {
        let pad = |(lo, hi): (f64, f64)| {
            if (hi - lo).abs() < 1e-12 {
                (lo - 0.5, hi + 0.5)
            } else {
                (lo, hi)
            }
        };
        DataRanges3D {
            x: pad(r.x),
            y: pad(r.y),
            z: pad(r.z),
        }
    }

    /// Axis-aligned data ranges used to size the 3D box. By default the box
    /// encloses every arrow's tail and tip; with
    /// [`Quiver3DPlot::with_tight_bounds`] it encloses the anchors only. A
    /// zero-span axis is padded to `±0.5`. Returns `None` when there are no
    /// arrows.
    pub fn data_ranges(&self) -> Option<DataRanges3D> {
        let anchors = self.anchor_extent()?;
        if self.tight_bounds {
            return Some(Self::pad(anchors));
        }
        let scale = self.effective_scale();
        let mut r = anchors;
        for a in &self.arrows {
            let (tail, tip) = self.endpoints_with_scale(a, scale);
            for p in [tail, tip] {
                r.x = (r.x.0.min(p[0]), r.x.1.max(p[0]));
                r.y = (r.y.0.min(p[1]), r.y.1.max(p[1]));
                r.z = (r.z.0.min(p[2]), r.z.1.max(p[2]));
            }
        }
        Some(Self::pad(r))
    }

    /// Resolve the scale multiplier, auto-computing when `scale` is `None`.
    ///
    /// The target is measured in the box's normalised coordinates, where each
    /// axis spans one unit whatever its data range, so a field on anisotropic
    /// ranges gets arrows of the intended on-screen length along every axis.
    /// For `n` arrows spread over `k` non-degenerate anchor axes the grid cell
    /// is `n^(-1/k)` box units (`√n` for a planar field, `∛n` for a volume),
    /// and the longest arrow is scaled to `fraction` of that cell. When every
    /// anchor coincides (a single arrow, or several vectors from one origin)
    /// there is no spacing to measure, so each axis is the padded unit box
    /// and the cell is taken as `n^(-1/3)`.
    ///
    /// The cell is measured against the anchor extent rather than the final
    /// box, which by default also encloses the arrow tips; arrows therefore
    /// come out slightly shorter on screen than `fraction` of a cell.
    pub(crate) fn effective_scale(&self) -> f64 {
        if let Some(s) = self.scale {
            return s;
        }
        let Some(anchors) = self.anchor_extent() else {
            return 1.0;
        };
        let raw = [anchors.x, anchors.y, anchors.z].map(|(lo, hi)| hi - lo);
        let k = match raw.iter().filter(|s| **s > 0.0).count() {
            0 => 3,
            k => k,
        };
        let padded = Self::pad(anchors);
        let span = [padded.x, padded.y, padded.z].map(|(lo, hi)| hi - lo);
        let max_norm_mag = self
            .arrows
            .iter()
            .map(|a| {
                let (u, v, w) = (a.u / span[0], a.v / span[1], a.w / span[2]);
                (u * u + v * v + w * w).sqrt()
            })
            .fold(0.0_f64, f64::max);
        if max_norm_mag <= 0.0 {
            return 1.0;
        }
        let cell = (self.arrows.len() as f64).powf(-1.0 / k as f64);
        self.auto_scale_fraction * cell / max_norm_mag
    }

    /// Tail and tip of an arrow in data coordinates for a given scale.
    pub(crate) fn endpoints_with_scale(
        &self,
        arrow: &Quiver3DArrow,
        scale: f64,
    ) -> ([f64; 3], [f64; 3]) {
        let d = [arrow.u * scale, arrow.v * scale, arrow.w * scale];
        let back = match self.pivot {
            QuiverPivot::Tail => 0.0,
            QuiverPivot::Middle => 0.5,
            QuiverPivot::Tip => 1.0,
        };
        let tail = [
            arrow.x - d[0] * back,
            arrow.y - d[1] * back,
            arrow.z - d[2] * back,
        ];
        let tip = [tail[0] + d[0], tail[1] + d[1], tail[2] + d[2]];
        (tail, tip)
    }

    /// Min and max arrow magnitudes, or `(0, 0)` when there are no arrows.
    pub(crate) fn magnitude_extent(&self) -> (f64, f64) {
        self.arrows
            .iter()
            .map(Quiver3DArrow::magnitude)
            .fold(None, |acc: Option<(f64, f64)>, m| match acc {
                None => Some((m, m)),
                Some((lo, hi)) => Some((lo.min(m), hi.max(m))),
            })
            .unwrap_or((0.0, 0.0))
    }

    /// Resolve `(head_length, half_width)` in pixels for a projected shaft of
    /// `shaft_px`. Honors explicit pixel overrides, else proportional sizing
    /// bounded by `head_min_px` / `head_max_px`. The head never exceeds the
    /// shaft, so a short arrow is never a floating triangle.
    pub(crate) fn resolve_head(&self, shaft_px: f64) -> (f64, f64) {
        let length = match self.head_length {
            Some(px) => px.min(shaft_px),
            None => {
                let target = shaft_px * self.head_ratio;
                let lo = self.head_min_px.min(shaft_px);
                let hi = self.head_max_px.min(shaft_px);
                // max-then-min: an inverted (min > max) pair resolves to the
                // upper bound instead of panicking like `f64::clamp` would.
                target.max(lo).min(hi)
            }
        };
        let half_w = match self.head_width {
            Some(px) => px,
            None => length * self.head_aspect,
        };
        (length, half_w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_function_samples_full_grid() {
        let q =
            Quiver3DPlot::from_function((0.0, 1.0, 3), (0.0, 1.0, 4), (0.0, 1.0, 5), |x, y, z| {
                (x, y, z)
            });
        assert_eq!(q.arrows.len(), 60);
        let last = q.arrows.last().unwrap();
        assert_eq!((last.x, last.y, last.z), (1.0, 1.0, 1.0));
    }

    #[test]
    fn non_finite_rows_are_dropped() {
        let q = Quiver3DPlot::new()
            .with_arrow(0.0, 0.0, 0.0, 1.0, 0.0, 0.0)
            .with_arrow(f64::NAN, 0.0, 0.0, 1.0, 0.0, 0.0)
            .with_arrow(0.0, 0.0, 0.0, 1.0, f64::INFINITY, 0.0)
            .with_arrows([(1.0, 1.0, 1.0, 0.0, 0.0, f64::NEG_INFINITY)]);
        assert_eq!(q.arrows.len(), 1);
    }

    #[test]
    fn pivot_shifts_endpoints() {
        let a = Quiver3DArrow {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            u: 2.0,
            v: 0.0,
            w: 0.0,
            color: None,
        };
        let tail = Quiver3DPlot::new().with_pivot(QuiverPivot::Tail);
        let mid = Quiver3DPlot::new().with_pivot(QuiverPivot::Middle);
        let tip = Quiver3DPlot::new().with_pivot(QuiverPivot::Tip);
        assert_eq!(
            tail.endpoints_with_scale(&a, 1.0),
            ([1.0, 2.0, 3.0], [3.0, 2.0, 3.0])
        );
        assert_eq!(
            mid.endpoints_with_scale(&a, 1.0),
            ([0.0, 2.0, 3.0], [2.0, 2.0, 3.0])
        );
        assert_eq!(
            tip.endpoints_with_scale(&a, 1.0),
            ([-1.0, 2.0, 3.0], [1.0, 2.0, 3.0])
        );
    }

    #[test]
    fn auto_scale_uses_cube_root_cell() {
        // 8 arrows on a unit cube, max magnitude 10, fraction 0.9:
        // cell = 1 / ∛8 = 0.5 → scale = 0.9 × 0.5 / 10.
        let q =
            Quiver3DPlot::from_function((0.0, 1.0, 2), (0.0, 1.0, 2), (0.0, 1.0, 2), |_, _, _| {
                (10.0, 0.0, 0.0)
            });
        let expected = 0.9 * 0.5 / 10.0;
        assert!((q.effective_scale() - expected).abs() < 1e-12);
        assert_eq!(q.clone().with_scale(0.25).effective_scale(), 0.25);
        assert!((q.with_auto_scale(0.45).effective_scale() - expected * 0.5).abs() < 1e-12);
    }

    #[test]
    fn auto_scale_uses_square_root_cell_for_planar_fields() {
        // All z equal: 4 arrows on a unit square → cell = 1/√4 = 0.5, u=1
        // spans the full 1.0 x-range → scale = 0.9 × 0.5.
        let q =
            Quiver3DPlot::from_function((0.0, 1.0, 2), (0.0, 1.0, 2), (0.0, 0.0, 1), |_, _, _| {
                (1.0, 0.0, 0.0)
            });
        assert!(
            (q.effective_scale() - 0.45).abs() < 1e-12,
            "got {}",
            q.effective_scale()
        );
    }

    #[test]
    fn auto_scale_is_measured_in_normalised_box_units() {
        // 5×5×5 grid on x,y ∈ [0,100], z ∈ [0,1]: cell = 1/5 of the box.
        // u=1 is 1/100 of the x span → scale 0.9 × 0.2 × 100 = 18, i.e. an
        // 18-unit arrow that is 0.9 of a cell on screen, not a 0.18-unit sliver.
        let wide = Quiver3DPlot::from_function(
            (0.0, 100.0, 5),
            (0.0, 100.0, 5),
            (0.0, 1.0, 5),
            |_, _, _| (1.0, 0.0, 0.0),
        );
        assert!(
            (wide.effective_scale() - 18.0).abs() < 1e-9,
            "got {}",
            wide.effective_scale()
        );
        // w=1 spans the whole 1.0 z-range → scale 0.9 × 0.2 = 0.18.
        let tall = Quiver3DPlot::from_function(
            (0.0, 100.0, 5),
            (0.0, 100.0, 5),
            (0.0, 1.0, 5),
            |_, _, _| (0.0, 0.0, 1.0),
        );
        assert!(
            (tall.effective_scale() - 0.18).abs() < 1e-9,
            "got {}",
            tall.effective_scale()
        );
    }

    #[test]
    fn auto_scale_fallbacks() {
        assert_eq!(Quiver3DPlot::new().effective_scale(), 1.0);
        let zero_mag = Quiver3DPlot::new()
            .with_arrow(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
            .with_arrow(1.0, 1.0, 1.0, 0.0, 0.0, 0.0);
        assert_eq!(zero_mag.effective_scale(), 1.0);
    }

    #[test]
    fn coincident_anchors_scale_to_the_padded_box() {
        // One arrow: the box is padded to 1.0 wide, so a raw 500-unit vector
        // must be scaled down to `fraction` of that box, not drawn off-canvas.
        let one = Quiver3DPlot::new().with_arrow(0.0, 0.0, 0.0, 500.0, 0.0, 0.0);
        assert!((one.effective_scale() * 500.0 - 0.9).abs() < 1e-12);
        // Three basis vectors from one origin: cell = 1 / ∛3.
        let basis = Quiver3DPlot::new()
            .with_arrow(0.0, 0.0, 0.0, 2.0, 0.0, 0.0)
            .with_arrow(0.0, 0.0, 0.0, 0.0, 2.0, 0.0)
            .with_arrow(0.0, 0.0, 0.0, 0.0, 0.0, 2.0);
        let expected = 0.9 / 3.0_f64.cbrt() / 2.0;
        assert!((basis.effective_scale() - expected).abs() < 1e-12);
    }

    #[test]
    fn data_ranges_enclose_arrows_by_default_and_anchors_when_tight() {
        let q = Quiver3DPlot::new()
            .with_arrow(0.0, 0.0, 2.0, 1.0, 0.0, 0.0)
            .with_arrow(4.0, 1.0, 2.0, 1.0, 0.0, 0.0)
            .with_scale(1.0);
        let r = q.data_ranges().unwrap();
        assert_eq!(r.x, (0.0, 5.0), "tip at x=5 is inside the box");
        assert_eq!(r.y, (0.0, 1.0));
        assert_eq!(r.z, (1.5, 2.5), "flat z is padded to a 1.0-wide box");
        let t = q.with_tight_bounds().data_ranges().unwrap();
        assert_eq!(t.x, (0.0, 4.0), "tight bounds stop at the anchors");
        assert!(Quiver3DPlot::new().data_ranges().is_none());
    }

    #[test]
    fn head_never_exceeds_shaft() {
        let q = Quiver3DPlot::new();
        let (len, _) = q.resolve_head(2.0);
        assert!(len <= 2.0, "proportional head {len} longer than 2px shaft");
        let (len, _) = q.clone().with_head(20.0, 6.0).resolve_head(5.0);
        assert_eq!(len, 5.0);
        let (len, _) = q.resolve_head(100.0);
        assert_eq!(len, 14.0, "proportional head should cap at head_max_px");
    }

    #[test]
    fn inverted_head_bounds_do_not_panic() {
        let q = Quiver3DPlot::new()
            .with_head_min_px(20.0)
            .with_head_max_px(4.0);
        let (len, _) = q.resolve_head(50.0);
        assert!(len.is_finite());
    }

    #[test]
    fn head_setters_drive_proportional_sizing() {
        let q = Quiver3DPlot::new()
            .with_head_aspect(1.0)
            .with_head_min_px(6.0)
            .with_head_max_px(30.0);
        let (len, half_w) = q.resolve_head(100.0);
        assert!(
            (len - 28.0).abs() < 1e-9,
            "28% of a 100px shaft, under the 30px cap: {len}"
        );
        assert!(
            (half_w - len).abs() < 1e-9,
            "aspect 1.0 → half-width equals length"
        );
        let (len, _) = q.resolve_head(10.0);
        assert_eq!(len, 6.0, "min head length applies to short shafts");
    }

    #[test]
    fn magnitude_extent_and_colormap_defaults() {
        let q = Quiver3DPlot::new()
            .with_arrow(0.0, 0.0, 0.0, 3.0, 4.0, 0.0)
            .with_arrow(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        assert_eq!(q.magnitude_extent(), (1.0, 5.0));
        assert_eq!(Quiver3DPlot::new().magnitude_extent(), (0.0, 0.0));
    }
}
