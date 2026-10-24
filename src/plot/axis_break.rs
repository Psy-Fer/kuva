//! Axis break: a gap in a numeric axis that skips a stretch of values, so a
//! few far-off points (an outlier bar, a late time window) no longer squash
//! the rest of the data into a sliver. Pure configuration, shared by the plot
//! and render layers; the piecewise mapping lives in
//! [`crate::render::layout::ComputedLayout`].

/// A gap in a linear axis between two data values.
///
/// Values up to `lo` and from `hi` on are drawn to scale on either side of a
/// short gap; the stretch in between is skipped. The axis line is split with
/// slanted break marks at both edges and anything crossing the gap is hidden.
/// Set it on a layout with [`Layout::with_y_break`](crate::render::layout::Layout::with_y_break)
/// or [`Layout::with_x_break`](crate::render::layout::Layout::with_x_break).
///
/// A break is ignored, not an error, when it does not fit: `lo >= hi`, a log
/// or categorical axis, `equal_aspect`, or a break that is not strictly inside
/// the resolved axis range.
///
/// ```
/// use kuva::plot::AxisBreak;
/// let b = AxisBreak::new(12.0, 940.0).with_gap(0.06).with_share(0.5);
/// assert!(b.is_valid());
/// assert!(b.contains(500.0) && !b.contains(5.0));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AxisBreak {
    /// Last value drawn before the gap.
    pub lo: f64,
    /// First value drawn after the gap.
    pub hi: f64,
    /// Width of the gap as a fraction of the plot's axis length. Default 0.04.
    pub gap: f64,
    /// Draw slanted break marks on the axis line at both edges. Default on.
    pub marks: bool,
    /// Fraction of the axis (gap excluded) given to the segment before the
    /// break. `None` follows the data spans, clamped to `0.3..=0.7` so a
    /// short segment next to a long one still gets a readable share.
    pub share: Option<f64>,
}

impl AxisBreak {
    /// Skip the values between `lo` and `hi`.
    pub fn new(lo: f64, hi: f64) -> Self {
        Self {
            lo,
            hi,
            gap: 0.04,
            marks: true,
            share: None,
        }
    }

    /// Give the segment before the break this fraction of the axis length
    /// (gap excluded), e.g. `0.5` for an even split. Must be in `0.05..=0.95`.
    pub fn with_share(mut self, frac: f64) -> Self {
        self.share = Some(frac);
        self
    }

    /// Pixel share of the segment before the break: explicit, or the data
    /// spans' ratio clamped to `0.3..=0.7`.
    pub fn share_before(&self, range: (f64, f64)) -> f64 {
        self.share.unwrap_or_else(|| {
            let before = (self.lo - range.0).max(0.0);
            let after = (range.1 - self.hi).max(0.0);
            let total = before + after;
            if total > 0.0 {
                (before / total).clamp(0.3, 0.7)
            } else {
                0.5
            }
        })
    }

    /// Gap width as a fraction of the axis length (default 0.04).
    pub fn with_gap(mut self, frac: f64) -> Self {
        self.gap = frac;
        self
    }

    /// Leave the axis line plain: no slanted marks at the gap edges.
    pub fn with_no_marks(mut self) -> Self {
        self.marks = false;
        self
    }

    /// Finite, ordered, and a gap that leaves room for the data.
    pub fn is_valid(&self) -> bool {
        self.lo.is_finite()
            && self.hi.is_finite()
            && self.lo < self.hi
            && self.gap.is_finite()
            && (0.0..0.5).contains(&self.gap)
            && self
                .share
                .is_none_or(|f| f.is_finite() && (0.05..=0.95).contains(&f))
    }

    /// Whether `v` falls inside the skipped stretch (exclusive).
    pub fn contains(&self, v: f64) -> bool {
        v > self.lo && v < self.hi
    }
}
