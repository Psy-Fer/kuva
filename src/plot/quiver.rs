use crate::plot::colormap::ColorMap;

/// Where `(x, y)` sits relative to the rendered arrow.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum QuiverPivot {
    /// `(x, y)` is the back of the arrow; it points *away* from that point. **(default)**
    #[default]
    Tail,
    /// `(x, y)` is the center of the shaft; arrow extends half in each direction.
    /// Reads naturally as "what the field is doing *at* this point."
    Middle,
    /// `(x, y)` is the arrow's tip; the shaft comes *into* the point.
    Tip,
}

/// A single arrow in a quiver plot.
///
/// `(x, y)` is the arrow's tail in data coordinates; `(u, v)` is the vector
/// displacement (also in data coordinates). The rendered arrow length is
/// `(u, v)` multiplied by the plot-level [`QuiverPlot::scale`] and mapped
/// through the axis transform.
#[derive(Debug, Clone)]
pub struct QuiverArrow {
    pub x: f64,
    pub y: f64,
    pub u: f64,
    pub v: f64,
    /// Per-arrow color override. `None` uses the plot-level color or colormap.
    pub color: Option<String>,
}

impl QuiverArrow {
    pub fn magnitude(&self) -> f64 {
        (self.u * self.u + self.v * self.v).sqrt()
    }
}

/// Builder for a quiver (2-D vector field) plot.
///
/// Each arrow has a tail at `(x, y)` and points in direction `(u, v)`. The
/// arrow is drawn as a line segment (the shaft) with a filled triangle
/// (the head) at the tip.
///
/// # Scaling
///
/// - [`QuiverPlot::scale`] is a multiplier applied to the `(u, v)` displacement
///   before mapping to pixel space. Default `1.0` draws each arrow at its
///   natural data-coordinate length.
/// - [`QuiverPlot::auto_scale`] computes `scale` so that the longest arrow
///   spans roughly `fraction` of the shorter axis of the data bounding box.
///   Useful when `(u, v)` have very different magnitudes from `(x, y)`.
///
/// # Coloring
///
/// Three modes, in order of priority:
/// 1. Per-arrow color via [`QuiverArrow::color`].
/// 2. Magnitude-driven colormap via [`QuiverPlot::with_color_map`]. The plot
///    reports its magnitude range so a colorbar is rendered automatically.
/// 3. Single color via [`QuiverPlot::with_color`]. Default `"steelblue"`.
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::QuiverPlot;
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let mut arrows = Vec::new();
/// for i in 0..10 {
///     for j in 0..10 {
///         let x = i as f64;
///         let y = j as f64;
///         arrows.push((x, y, (y - 5.0) * 0.2, -(x - 5.0) * 0.2));
///     }
/// }
///
/// let plot = QuiverPlot::new()
///     .with_arrows(arrows)
///     .with_color("steelblue");
///
/// let plots = vec![Plot::Quiver(plot)];
/// let layout = Layout::auto_from_plots(&plots);
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// ```
#[derive(Debug, Clone)]
pub struct QuiverPlot {
    pub arrows: Vec<QuiverArrow>,
    /// Default arrow color (used when no colormap and no per-arrow color).
    /// Default `"steelblue"`.
    pub color: String,
    /// Multiplier applied to `(u, v)` before axis mapping.
    ///
    /// - `Some(s)` — use `s` directly (set via [`QuiverPlot::with_scale`]).
    /// - `None` — auto-compute so the longest arrow is roughly one grid
    ///   cell long (`≈ span / √n`). This prevents arrows from overlapping
    ///   each other in dense fields.
    pub scale: Option<f64>,
    /// Fraction of the nearest-neighbor distance used for the longest
    /// arrow when `scale` is `None`. Default `0.9`.
    ///
    /// Values near `1.0` pack arrows tip-to-tail; smaller values leave
    /// more breathing room. Values above `1.0` allow overlap.
    pub auto_scale_fraction: f64,
    /// Shaft stroke width in pixels. Default `1.2`.
    pub shaft_width: f64,
    /// Explicit head length in pixels. `None` → proportional to shaft length
    /// (see [`QuiverPlot::head_ratio`]). Set via [`QuiverPlot::with_head`].
    pub head_length: Option<f64>,
    /// Explicit head half-width in pixels. `None` → `head_aspect * head_length`.
    pub head_width: Option<f64>,
    /// Fraction of shaft length used for the arrow head when `head_length`
    /// is `None`. Default `0.28`.
    pub head_ratio: f64,
    /// Head half-width as a fraction of head length when `head_width` is
    /// `None`. Default `0.45`.
    pub head_aspect: f64,
    /// Minimum head length in pixels (prevents tiny arrows from losing their head).
    /// Only applies when `head_length` is `None`. Default `4.0`.
    pub head_min_px: f64,
    /// Maximum head length in pixels (prevents long arrows from growing gigantic heads).
    /// Only applies when `head_length` is `None`. Default `14.0`.
    pub head_max_px: f64,
    /// Optional colormap applied to arrow magnitude. When set, overrides
    /// [`QuiverPlot::color`] for arrows without a per-arrow override.
    pub color_map: Option<ColorMap>,
    /// Optional explicit magnitude range `(min, max)` for colormap normalization.
    /// When `None`, derived from the data.
    pub color_range: Option<(f64, f64)>,
    /// Label shown next to the colorbar (when a colormap is active).
    pub color_legend_label: Option<String>,
    pub legend_label: Option<String>,
    /// When `true`, axis bounds come from arrow *tails* only — arrows may
    /// extend past the plot box. Default `false` (bounds include arrow tips,
    /// so nothing clips). Tight bounds produce a denser-looking field that
    /// better fills the plot area.
    pub tight_bounds: bool,
    /// Clip arrows to the plot rectangle when `Some(b)`. When `None`, clipping
    /// is auto-derived from `tight_bounds` (clipping implied when tight bounds
    /// are on, since overflowing arrows would hit the axis labels). Set
    /// explicitly via [`QuiverPlot::with_clip_to_plot_area`] or `with_no_clip`.
    pub clip_to_plot_area: Option<bool>,
    /// Where `(x, y)` sits relative to the rendered arrow. Default `Tail`.
    pub pivot: QuiverPivot,
}

impl Default for QuiverPlot {
    fn default() -> Self {
        Self::new()
    }
}

impl QuiverPlot {
    /// Sample a vector-field closure on a regular `nx × ny` grid.
    ///
    /// `x_range` and `y_range` are `(min, max, n)` tuples where `n` is the
    /// number of samples along that axis (endpoints inclusive, matching
    /// `numpy.linspace`). The closure receives `(x, y)` and returns `(u, v)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use kuva::plot::QuiverPlot;
    /// let plot = QuiverPlot::from_function(
    ///     (-2.0, 2.0, 10),
    ///     (-2.0, 2.0, 10),
    ///     |x, y| (-y, x),   // rotational field
    /// );
    /// assert_eq!(plot.arrows.len(), 100);
    /// ```
    pub fn from_function<F>(
        x_range: (f64, f64, usize),
        y_range: (f64, f64, usize),
        mut f: F,
    ) -> Self
    where
        F: FnMut(f64, f64) -> (f64, f64),
    {
        let (x_min, x_max, nx) = x_range;
        let (y_min, y_max, ny) = y_range;
        let mut plot = Self::new();
        plot.arrows.reserve(nx.saturating_mul(ny));
        let step = |lo: f64, hi: f64, n: usize, i: usize| -> f64 {
            if n <= 1 {
                (lo + hi) * 0.5
            } else {
                lo + (hi - lo) * (i as f64) / (n - 1) as f64
            }
        };
        for i in 0..nx {
            let x = step(x_min, x_max, nx, i);
            for j in 0..ny {
                let y = step(y_min, y_max, ny, j);
                let (u, v) = f(x, y);
                plot.arrows.push(QuiverArrow {
                    x,
                    y,
                    u,
                    v,
                    color: None,
                });
            }
        }
        plot
    }

    /// Create an empty quiver plot with default styling.
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
            tight_bounds: false,
            clip_to_plot_area: None,
            pivot: QuiverPivot::Tail,
        }
    }

    /// Add one arrow at `(x, y)` with vector `(u, v)`. Non-finite values
    /// (NaN / infinity) are silently dropped.
    pub fn with_arrow(
        mut self,
        x: impl Into<f64>,
        y: impl Into<f64>,
        u: impl Into<f64>,
        v: impl Into<f64>,
    ) -> Self {
        let (x, y, u, v) = (x.into(), y.into(), u.into(), v.into());
        if x.is_finite() && y.is_finite() && u.is_finite() && v.is_finite() {
            self.arrows.push(QuiverArrow {
                x,
                y,
                u,
                v,
                color: None,
            });
        }
        self
    }

    /// Add many arrows from an iterator of `(x, y, u, v)` tuples. Non-finite
    /// rows are silently dropped.
    ///
    /// All four tuple positions must use the *same* numeric type within the
    /// iterator (e.g. all `f64` or all `i32`). For mixed types, map at the
    /// call site: `data.iter().map(|(x, y, u, v)| (*x as f64, *y as f64, ...))`.
    pub fn with_arrows<X, Y, U, V, I>(mut self, arrows: I) -> Self
    where
        X: Into<f64>,
        Y: Into<f64>,
        U: Into<f64>,
        V: Into<f64>,
        I: IntoIterator<Item = (X, Y, U, V)>,
    {
        for (x, y, u, v) in arrows {
            let (x, y, u, v) = (x.into(), y.into(), u.into(), v.into());
            if x.is_finite() && y.is_finite() && u.is_finite() && v.is_finite() {
                self.arrows.push(QuiverArrow {
                    x,
                    y,
                    u,
                    v,
                    color: None,
                });
            }
        }
        self
    }

    /// Add an arrow with a per-arrow color override. Non-finite values are
    /// silently dropped.
    pub fn with_colored_arrow(
        mut self,
        x: impl Into<f64>,
        y: impl Into<f64>,
        u: impl Into<f64>,
        v: impl Into<f64>,
        color: impl Into<String>,
    ) -> Self {
        let (x, y, u, v) = (x.into(), y.into(), u.into(), v.into());
        if x.is_finite() && y.is_finite() && u.is_finite() && v.is_finite() {
            self.arrows.push(QuiverArrow {
                x,
                y,
                u,
                v,
                color: Some(color.into()),
            });
        }
        self
    }

    /// Set the default arrow color.
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = color.into();
        self
    }

    /// Pin the scale multiplier to an explicit value. Overrides the default
    /// auto-scaling behavior.
    ///
    /// # Panics
    /// Panics in debug builds if `s` is not finite. Pass a positive, finite
    /// value; zero is allowed but produces zero-length arrows.
    pub fn with_scale(mut self, s: impl Into<f64>) -> Self {
        let s = s.into();
        debug_assert!(
            s.is_finite(),
            "QuiverPlot::with_scale: scale must be finite, got {s}"
        );
        self.scale = Some(s);
        self
    }

    /// Override the fraction of the nearest-neighbor distance used for the
    /// longest arrow when auto-scaling (i.e. when [`QuiverPlot::with_scale`]
    /// hasn't been called). Default `0.9`.
    ///
    /// Values near `1.0` pack arrows tip-to-tail; smaller values leave more
    /// breathing room. Values above `1.0` allow arrows to overlap each other.
    ///
    /// # Panics
    /// In debug builds, panics if `fraction` is not finite.
    pub fn with_auto_scale(mut self, fraction: impl Into<f64>) -> Self {
        let fraction = fraction.into();
        debug_assert!(
            fraction.is_finite(),
            "QuiverPlot::with_auto_scale: fraction must be finite, got {fraction}"
        );
        self.scale = None;
        self.auto_scale_fraction = fraction;
        self
    }

    /// Resolve the scale multiplier, auto-computing when `scale` is `None`.
    ///
    /// The auto-scale target is: longest arrow ≈ `fraction × nearest-neighbor
    /// distance`, where the nearest-neighbor distance for `n` arrows on an
    /// `R`-wide span is approximated as `R / √n`. This prevents arrows from
    /// overlapping each other in dense fields.
    pub(crate) fn effective_scale(&self) -> f64 {
        if let Some(s) = self.scale {
            return s;
        }
        let n = self.arrows.len();
        if n < 2 {
            return 1.0;
        }
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        let mut max_mag = 0.0_f64;
        for a in &self.arrows {
            x_min = x_min.min(a.x);
            x_max = x_max.max(a.x);
            y_min = y_min.min(a.y);
            y_max = y_max.max(a.y);
            max_mag = max_mag.max(a.magnitude());
        }
        let x_span = x_max - x_min;
        let y_span = y_max - y_min;
        // Pick the smaller non-zero span (or the non-zero one when arrows are
        // collinear on one axis).
        let span = match (x_span > 0.0, y_span > 0.0) {
            (true, true) => x_span.min(y_span),
            (true, false) => x_span,
            (false, true) => y_span,
            (false, false) => 0.0,
        };
        if max_mag > 0.0 && span.is_finite() && span > 0.0 {
            // Grid-cell heuristic: for an n-arrow field on a span R, the
            // typical nearest-neighbor distance is ~R/√n. Target the longest
            // arrow to be `fraction` of that.
            let cell = span / (n as f64).sqrt();
            self.auto_scale_fraction * cell / max_mag
        } else {
            1.0
        }
    }

    /// Compute effective scale **and** the data-coordinate origin extent in one
    /// pass. Returns `(scale, x_min, x_max, y_min, y_max)`. Used by `bounds()`
    /// so the arrow collection is only iterated once instead of twice when
    /// auto-scaling is active.
    pub(crate) fn effective_scale_and_data_extent(&self) -> (f64, f64, f64, f64, f64) {
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        if self.arrows.is_empty() {
            return (1.0, x_min, x_max, y_min, y_max);
        }
        let mut max_mag = 0.0_f64;
        for a in &self.arrows {
            x_min = x_min.min(a.x);
            x_max = x_max.max(a.x);
            y_min = y_min.min(a.y);
            y_max = y_max.max(a.y);
            max_mag = max_mag.max(a.magnitude());
        }
        let scale = if let Some(s) = self.scale {
            s
        } else {
            let n = self.arrows.len();
            if n < 2 {
                1.0
            } else {
                let x_span = x_max - x_min;
                let y_span = y_max - y_min;
                let span = match (x_span > 0.0, y_span > 0.0) {
                    (true, true) => x_span.min(y_span),
                    (true, false) => x_span,
                    (false, true) => y_span,
                    (false, false) => 0.0,
                };
                if max_mag > 0.0 && span.is_finite() && span > 0.0 {
                    let cell = span / (n as f64).sqrt();
                    self.auto_scale_fraction * cell / max_mag
                } else {
                    1.0
                }
            }
        };
        (scale, x_min, x_max, y_min, y_max)
    }

    /// Set the shaft stroke width in pixels. Default `1.2`.
    pub fn with_shaft_width(mut self, w: impl Into<f64>) -> Self {
        let w = w.into();
        debug_assert!(
            w.is_finite() && w >= 0.0,
            "QuiverPlot::with_shaft_width: width must be finite and non-negative, got {w}"
        );
        self.shaft_width = w;
        self
    }

    /// Pin arrow head dimensions to explicit pixel values. `length` is along
    /// the shaft, `half_width` is perpendicular to it. Overrides the default
    /// proportional sizing. Heads are still capped at the shaft length.
    pub fn with_head(mut self, length: impl Into<f64>, half_width: impl Into<f64>) -> Self {
        let (length, half_width) = (length.into(), half_width.into());
        debug_assert!(
            length.is_finite() && half_width.is_finite(),
            "QuiverPlot::with_head: length and half_width must be finite"
        );
        self.head_length = Some(length);
        self.head_width = Some(half_width);
        self
    }

    /// Pin just the head length in pixels. `with_head_width` remains
    /// independently overridable; unset dimensions fall back to the
    /// proportional default.
    pub fn with_head_length(mut self, length: impl Into<f64>) -> Self {
        let length = length.into();
        debug_assert!(
            length.is_finite(),
            "QuiverPlot::with_head_length: must be finite"
        );
        self.head_length = Some(length);
        self
    }

    /// Pin just the head half-width in pixels.
    pub fn with_head_width(mut self, half_width: impl Into<f64>) -> Self {
        let half_width = half_width.into();
        debug_assert!(
            half_width.is_finite(),
            "QuiverPlot::with_head_width: must be finite"
        );
        self.head_width = Some(half_width);
        self
    }

    /// Set the head length as a fraction of the shaft length (used when no
    /// explicit `with_head` is set). Default `0.28`.
    pub fn with_head_ratio(mut self, ratio: impl Into<f64>) -> Self {
        let ratio = ratio.into();
        debug_assert!(
            ratio.is_finite(),
            "QuiverPlot::with_head_ratio: must be finite"
        );
        self.head_ratio = ratio;
        self
    }

    /// Minimum head length in pixels when proportional sizing is in effect.
    /// Prevents tiny arrows from losing their head entirely. Default `4.0`.
    pub fn with_head_min_px(mut self, px: impl Into<f64>) -> Self {
        let px = px.into();
        debug_assert!(
            px.is_finite(),
            "QuiverPlot::with_head_min_px: must be finite"
        );
        self.head_min_px = px;
        self
    }

    /// Maximum head length in pixels when proportional sizing is in effect.
    /// Prevents long arrows from growing gigantic heads. Default `14.0`.
    pub fn with_head_max_px(mut self, px: impl Into<f64>) -> Self {
        let px = px.into();
        debug_assert!(
            px.is_finite(),
            "QuiverPlot::with_head_max_px: must be finite"
        );
        self.head_max_px = px;
        self
    }

    /// Resolve `(head_length, half_width)` in pixels for a shaft of length
    /// `shaft_px`. Honors explicit pixel overrides, else falls back to
    /// proportional sizing clamped by `head_min_px` / `head_max_px`.
    pub(crate) fn resolve_head(&self, shaft_px: f64) -> (f64, f64) {
        let length = match self.head_length {
            Some(px) => px.min(shaft_px),
            None => {
                let target = shaft_px * self.head_ratio;
                let lo = self.head_min_px.min(shaft_px);
                let hi = self.head_max_px.min(shaft_px);
                target.clamp(lo, hi)
            }
        };
        let half_w = match self.head_width {
            Some(px) => px,
            None => length * self.head_aspect,
        };
        (length, half_w)
    }

    /// Color arrows by magnitude using the given colormap. Overrides
    /// [`QuiverPlot::color`] for arrows without a per-arrow override.
    pub fn with_color_map(mut self, cmap: ColorMap) -> Self {
        self.color_map = Some(cmap);
        self
    }

    /// Override the magnitude range used for colormap normalization.
    /// Default: derived from the data.
    pub fn with_color_range(mut self, lo: impl Into<f64>, hi: impl Into<f64>) -> Self {
        let (lo, hi) = (lo.into(), hi.into());
        debug_assert!(
            lo.is_finite() && hi.is_finite(),
            "QuiverPlot::with_color_range: lo and hi must be finite"
        );
        self.color_range = Some((lo, hi));
        self
    }

    /// Label rendered next to the colorbar (when a colormap is active).
    pub fn with_color_legend_label(mut self, label: impl Into<String>) -> Self {
        self.color_legend_label = Some(label.into());
        self
    }

    /// Shorthand for the common "color by magnitude with a labeled colorbar"
    /// pattern. Equivalent to
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

    /// Derive axis bounds from arrow tails only. Arrows may extend past the
    /// plot box. Useful for dense grids where tip-inclusive bounds produce
    /// too much whitespace around the field. When clipping hasn't been
    /// explicitly pinned, this also turns clipping on.
    pub fn with_tight_bounds(mut self) -> Self {
        self.tight_bounds = true;
        self
    }

    /// Force arrows to be clipped to the plot rectangle regardless of bounds
    /// mode. Useful when you want tip-inclusive bounds but still don't want
    /// the occasional outlier arrow to overflow into an axis label.
    pub fn with_clip_to_plot_area(mut self) -> Self {
        self.clip_to_plot_area = Some(true);
        self
    }

    /// Disable arrow clipping even when `tight_bounds` is on. Arrows will be
    /// drawn whole and may extend past the axis box into the margins.
    pub fn with_no_clip(mut self) -> Self {
        self.clip_to_plot_area = Some(false);
        self
    }

    /// Resolve whether arrows should be clipped to the plot rectangle,
    /// falling back to `tight_bounds` when no explicit override is set.
    pub(crate) fn should_clip(&self) -> bool {
        self.clip_to_plot_area.unwrap_or(self.tight_bounds)
    }

    /// Where `(x, y)` sits relative to the rendered arrow. Default `Tail`.
    pub fn with_pivot(mut self, pivot: QuiverPivot) -> Self {
        self.pivot = pivot;
        self
    }

    /// Resolve an arrow's tail and tip endpoints in data coordinates,
    /// given a precomputed scale multiplier.
    pub(crate) fn endpoints_with_scale(
        &self,
        arrow: &QuiverArrow,
        scale: f64,
    ) -> ((f64, f64), (f64, f64)) {
        let du = arrow.u * scale;
        let dv = arrow.v * scale;
        let (sx, sy) = match self.pivot {
            QuiverPivot::Tail => (0.0, 0.0),
            QuiverPivot::Middle => (-du * 0.5, -dv * 0.5),
            QuiverPivot::Tip => (-du, -dv),
        };
        let tail = (arrow.x + sx, arrow.y + sy);
        let tip = (tail.0 + du, tail.1 + dv);
        (tail, tip)
    }

    /// Min and max arrow magnitudes in the current data.
    pub(crate) fn magnitude_extent(&self) -> (f64, f64) {
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for a in &self.arrows {
            let m = a.magnitude();
            lo = lo.min(m);
            hi = hi.max(m);
        }
        if !lo.is_finite() {
            return (0.0, 0.0);
        }
        (lo, hi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pivot_middle_vs_tail_shifts_endpoints() {
        let tail = QuiverPlot::new()
            .with_arrow(0.0, 0.0, 2.0, 0.0)
            .with_scale(1.0)
            .with_pivot(QuiverPivot::Tail);
        let mid = QuiverPlot::new()
            .with_arrow(0.0, 0.0, 2.0, 0.0)
            .with_scale(1.0)
            .with_pivot(QuiverPivot::Middle);
        let (tail_t, tip_t) = tail.endpoints_with_scale(&tail.arrows[0], 1.0);
        let (tail_m, tip_m) = mid.endpoints_with_scale(&mid.arrows[0], 1.0);
        assert_eq!(tail_t, (0.0, 0.0));
        assert_eq!(tip_t, (2.0, 0.0));
        assert_eq!(tail_m, (-1.0, 0.0));
        assert_eq!(tip_m, (1.0, 0.0));
    }

    #[test]
    fn pivot_tip_places_tip_at_data_point() {
        let q = QuiverPlot::new()
            .with_arrow(5.0, 5.0, 1.0, 1.0)
            .with_scale(1.0)
            .with_pivot(QuiverPivot::Tip);
        let (_tail, tip) = q.endpoints_with_scale(&q.arrows[0], 1.0);
        assert_eq!(tip, (5.0, 5.0));
    }

    #[test]
    fn auto_scale_shrinks_huge_vectors() {
        let q = QuiverPlot::new()
            .with_arrow(0.0, 0.0, 10.0, 0.0)
            .with_arrow(1.0, 0.0, 10.0, 0.0);
        let s = q.effective_scale();
        assert!(
            s > 0.0 && s < 0.1,
            "auto-scale should shrink huge vectors; got {s}"
        );
    }

    #[test]
    fn auto_scale_matches_grid_cell_formula() {
        // Pin the exact formula: fraction * span / (√n * max_mag).
        // 2 arrows, span 1.0 (x: 0..1, y=0 everywhere — falls back to x_span),
        // max_mag 10, fraction 0.9 → 0.9 * 1.0 / (√2 * 10) ≈ 0.06364.
        let q = QuiverPlot::new()
            .with_arrow(0.0, 0.0, 10.0, 0.0)
            .with_arrow(1.0, 0.0, 10.0, 0.0);
        let expected = 0.9 * 1.0 / ((2.0_f64).sqrt() * 10.0);
        let actual = q.effective_scale();
        assert!(
            (actual - expected).abs() < 1e-9,
            "auto-scale formula drift: expected {expected}, got {actual}"
        );
    }

    #[test]
    fn auto_scale_honors_custom_fraction() {
        // Changing auto_scale_fraction should linearly scale the result.
        let a = QuiverPlot::new()
            .with_arrow(0.0, 0.0, 10.0, 0.0)
            .with_arrow(1.0, 0.0, 10.0, 0.0);
        let b = a.clone().with_auto_scale(0.45);
        // 0.45 is half of the 0.9 default → scale should be half.
        assert!(
            (b.effective_scale() - a.effective_scale() * 0.5).abs() < 1e-9,
            "with_auto_scale(0.45) should halve the 0.9-default scale"
        );
    }

    #[test]
    fn with_scale_pins_exact_value() {
        let q = QuiverPlot::new()
            .with_arrow(0.0, 0.0, 10.0, 0.0)
            .with_arrow(1.0, 0.0, 10.0, 0.0)
            .with_scale(0.5);
        assert_eq!(q.effective_scale(), 0.5);
    }

    #[test]
    fn effective_scale_fallback_when_empty() {
        assert_eq!(QuiverPlot::new().effective_scale(), 1.0);
    }

    #[test]
    fn effective_scale_fallback_when_all_magnitudes_zero() {
        let q = QuiverPlot::new()
            .with_arrow(0.0, 0.0, 0.0, 0.0)
            .with_arrow(1.0, 1.0, 0.0, 0.0);
        assert_eq!(q.effective_scale(), 1.0);
    }
}
