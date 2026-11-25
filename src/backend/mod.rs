pub mod interactive_js;
pub mod svg;
#[cfg(feature = "typst-math")]
pub mod svg_math;
pub mod terminal;

#[cfg(feature = "png")]
pub mod png;

#[cfg(feature = "png")]
pub mod raster;

#[cfg(feature = "pdf")]
pub mod pdf;

#[cfg(feature = "typst")]
pub mod typst;

/// The part of the segment `p0 -> p1` inside the box `lo..=hi`, as a range of
/// the segment parameter `t` (`0` at `p0`, `1` at `p1`). `None` when the
/// segment misses the box or a coordinate is not finite.
///
/// A segment that lies inside the box gets exactly `(0.0, 1.0)`, so callers can
/// tell it apart and draw it from the original endpoints, unchanged. The
/// raster and terminal backends clip lines to the canvas plus a wide margin
/// with this, so a line to a far-off or huge coordinate doesn't step through
/// every pixel on the way.
pub(crate) fn clip_segment(
    p0: (f64, f64),
    p1: (f64, f64),
    lo: (f64, f64),
    hi: (f64, f64),
) -> Option<(f64, f64)> {
    if ![p0.0, p0.1, p1.0, p1.1].iter().all(|v| v.is_finite()) {
        return None;
    }
    let (dx, dy) = (p1.0 - p0.0, p1.1 - p0.1);
    let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
    // Liang-Barsky: each edge as p * t <= q.
    for (p, q) in [
        (-dx, p0.0 - lo.0),
        (dx, hi.0 - p0.0),
        (-dy, p0.1 - lo.1),
        (dy, hi.1 - p0.1),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let r = q / p;
            if p < 0.0 {
                t0 = t0.max(r);
            } else {
                t1 = t1.min(r);
            }
        }
    }
    (t0 <= t1).then_some((t0, t1))
}

#[cfg(test)]
mod tests {
    use super::clip_segment;

    #[test]
    fn clip_segment_keeps_inside_segments_exact_and_cuts_outside_ones() {
        let (lo, hi) = ((0.0, 0.0), (10.0, 10.0));
        assert_eq!(
            clip_segment((1.0, 1.0), (9.0, 3.0), lo, hi),
            Some((0.0, 1.0))
        );
        assert_eq!(
            clip_segment((-10.0, 5.0), (20.0, 5.0), lo, hi),
            Some((1.0 / 3.0, 2.0 / 3.0))
        );
        assert_eq!(clip_segment((-5.0, -5.0), (-1.0, 20.0), lo, hi), None);
        assert_eq!(clip_segment((5.0, 5.0), (f64::INFINITY, 5.0), lo, hi), None);
        assert_eq!(clip_segment((f64::NAN, 5.0), (5.0, 5.0), lo, hi), None);
        let (t0, t1) = clip_segment((5.0, 5.0), (1e300, 5.0), lo, hi).unwrap();
        assert_eq!(t0, 0.0);
        assert!((t1 * 1e300 - 5.0).abs() < 1e-6);
    }
}
