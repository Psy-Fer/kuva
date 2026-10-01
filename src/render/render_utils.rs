/// XML/attribute-escape a data-derived string before it is interpolated into
/// a raw SVG attribute string (e.g. `data-group="{value}"`).
///
/// Values reaching these call sites (group names, legend labels, category
/// labels) originate from user data files, not from code, so they must never
/// be trusted to be free of `"`, `<`, `>`, or `&`.
pub(crate) fn escape_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Build an SVG path string for a filled arrow-head triangle.
///
/// Tip sits at `(tip_x, tip_y)` and the head points along the unit vector
/// `(ux, uy)`. `length` is the head's extent along the shaft; `half_width`
/// is the perpendicular distance from the shaft axis to each base corner.
///
/// The caller is responsible for wrapping the returned string in a
/// `Primitive::Path` with appropriate fill/stroke.
pub fn arrow_head_path(
    tip_x: f64,
    tip_y: f64,
    ux: f64,
    uy: f64,
    length: f64,
    half_width: f64,
) -> String {
    let base_x = tip_x - ux * length;
    let base_y = tip_y - uy * length;
    // Perpendicular to the shaft direction.
    let px = -uy;
    let py = ux;
    format!(
        "M {:.2} {:.2} L {:.2} {:.2} L {:.2} {:.2} Z",
        tip_x,
        tip_y,
        base_x + px * half_width,
        base_y + py * half_width,
        base_x - px * half_width,
        base_y - py * half_width,
    )
}

/// compute ticks so things look nice
/// compute_tick_step(min, max, target_ticks)
pub fn compute_tick_step(min: f64, max: f64, target_ticks: usize) -> f64 {
    let raw_step = (max - min) / target_ticks as f64;
    // A nonzero subnormal range can underflow when divided by the tick count.
    let raw_step = if raw_step == 0.0 && max > min {
        f64::from_bits(1)
    } else {
        raw_step
    };
    let magnitude = 10f64.powf(raw_step.abs().log10().floor());
    let magnitude = if magnitude == 0.0 && raw_step > 0.0 {
        f64::from_bits(1)
    } else {
        magnitude
    };
    let residual = raw_step / magnitude;

    // handle between 1 and 10
    let nice_residual = if residual < 1.5 {
        1.0
    } else if residual < 2.25 {
        2.0
    } else if residual < 3.5 {
        2.5
    } else if residual < 7.5 {
        5.0
    } else {
        10.0
    };
    // now multiply the nice value by the mag to get the nice tick
    let step = nice_residual * magnitude;
    if step.is_infinite() && raw_step.is_finite() {
        raw_step
    } else {
        step
    }
}

/// Generate nice ticks for an axis.
/// Returns no ticks for non-finite/reversed bounds or an unrepresentable grid count.
/// Dense grids retain distinct floats even when the virtual index count overflows.
pub fn generate_ticks(min: f64, max: f64, target_ticks: usize) -> Vec<f64> {
    let step = compute_tick_step(min, max, target_ticks);
    linear_ticks(min, max, step, |value| {
        let factor = if value == 0.0 {
            1.0
        } else {
            let d = value.abs().log10().floor() as i32;
            10f64.powi(6 - d)
        };
        round_tick(value, step, factor, min, max)
    })
}

// Suppress float noise without erasing the spacing or leaving the axis range.
// Decimal scaling can overflow, underflow, or lose low bits at large offsets.
fn round_tick(value: f64, step: f64, factor: f64, min: f64, max: f64) -> f64 {
    let scaled = value * factor;
    // If rounding changes nothing, dividing back can only introduce noise.
    if !scaled.is_finite() || scaled.fract() == 0.0 {
        return value;
    }
    let rounded = scaled.round() / factor;
    if rounded.is_finite()
        && rounded >= min
        && rounded <= max
        && (rounded - value).abs() <= step.abs() * 1e-6
    {
        rounded
    } else {
        value
    }
}

// Retain the grid phase when adding a small displacement to a large minimum.
// Each sum's error and the product's FMA residual correct double rounding.
fn shifted_tick(min: f64, index: f64, step: f64, offset: f64) -> f64 {
    let product = index * step;
    let product_error = index.mul_add(step, -product);
    let distance = product + offset;
    let part = distance - product;
    let distance_error = (product - (distance - part)) + (offset - part);
    let value = min + distance;
    if !value.is_finite() {
        return value;
    }
    let part = value - min;
    let value_error = (min - (value - part)) + (distance - part);
    value + (value_error + distance_error + product_error)
}

// Find the first index whose monotone position exceeds value, or end if none does.
fn next_distinct_index(
    mut first: usize,
    mut end: usize,
    value: f64,
    position: impl Fn(usize) -> f64,
) -> usize {
    while first < end {
        let middle = first + (end - first) / 2;
        if position(middle) > value {
            end = middle;
        } else {
            first = middle + 1;
        }
    }
    first
}

fn linear_ticks(min: f64, max: f64, step: f64, round: impl Fn(f64) -> f64) -> Vec<f64> {
    if !min.is_finite() || !max.is_finite() || max < min || !step.is_finite() || step <= 0.0 {
        return Vec::new();
    }
    let low = min / step;
    let high = max / step;
    // At these indices every integer and its neighbors are representable.
    // Check the rounded grid positions, so e.g. 5 * 0.2 still reaches tick 1.
    let (first, offset, count) =
        if low.abs() <= (1u64 << 52) as f64 && high.abs() <= (1u64 << 52) as f64 {
            let first = low.floor();
            let first = if first * step < min {
                first + 1.0
            } else {
                first
            };
            let last = high.ceil();
            let last = if last * step > max { last - 1.0 } else { last };
            (Some(first), 0.0, last - first)
        } else {
            // Remainders retain the grid phase when division overflows or cannot
            // distinguish consecutive indices. Keep the offset separate from min.
            let offset = -(min % step);
            // Start one grid point before the truncating remainder's anchor.
            // Check positions against max instead of trusting a rounded last index.
            let count = ((max - min) / step).ceil().next_up() + 3.0;
            (None, offset, count)
        };
    if !count.is_finite() || count >= usize::MAX as f64 {
        let closest = if min >= 0.0 {
            min
        } else if max <= 0.0 {
            -max
        } else {
            0.0
        };
        let gap = (closest.next_up() - closest).min(closest - closest.next_down());
        // Each float's rounding interval is wider than this step, so every
        // float in the same-sign range has at least one grid point rounding to it.
        if step <= gap * 0.5 {
            let mut ticks = Vec::new();
            let mut value = min;
            loop {
                ticks.push(value);
                let next = value.next_up();
                if next > max {
                    break;
                }
                value = next;
            }
            return ticks;
        }
        return Vec::new();
    }
    if count < 0.0 {
        return Vec::new();
    }
    let position = |index: usize| match first {
        Some(first) => (first + index as f64) * step,
        None => shifted_tick(min, index as f64 - 1.0, step, offset),
    };
    let last = count as usize;
    let mut ticks = Vec::new();
    let mut index = 0usize;
    while index <= last {
        let value = position(index);
        if value < min {
            let Some(next_index) = index.checked_add(1) else {
                break;
            };
            index = next_index;
            continue;
        }
        if !value.is_finite() || value > max {
            break;
        }
        let rounded = round(value);
        if rounded >= min
            && rounded <= max
            && ticks.last().is_none_or(|&previous| rounded > previous)
        {
            ticks.push(rounded);
        }
        let Some(next_index) = index.checked_add(1) else {
            break;
        };
        // A tiny step at a large offset can map many indices to the same float.
        // Jump to the next representable position instead of scanning those indices.
        if first.is_none() && position(next_index) <= value {
            // Aim at the next rounding midpoint, not its full float value:
            // a larger jump can skip a distinct position when step is near one ULP.
            let half_gap = (value.next_up() - value) * 0.5;
            let next = ((((value - min) - offset) + half_gap) / step)
                .next_down()
                .next_down()
                .floor()
                + 1.0;
            if !next.is_finite() || next >= usize::MAX as f64 {
                break;
            }
            index = (next as usize).max(next_index);
        } else {
            index = next_index;
        }
    }
    ticks
}

/// Generate x-axis ticks for a histogram so every tick falls exactly on a bin
/// boundary.
///
/// Finds the smallest integer multiplier `n` such that `n` divides `total_bins`
/// evenly and the resulting tick count stays within `target_ticks`. The tick
/// step is then `n * bin_width`, guaranteeing alignment with bar edges.
/// Returns no ticks for non-finite bounds/width or an unrepresentable bin count/step.
pub fn generate_ticks_bin_aligned(
    x_min: f64,
    x_max: f64,
    bin_width: f64,
    target_ticks: usize,
) -> Vec<f64> {
    if !x_min.is_finite() || !x_max.is_finite() || !bin_width.is_finite() {
        return Vec::new();
    }
    if bin_width <= 0.0 || x_max <= x_min {
        return generate_ticks(x_min, x_max, target_ticks);
    }
    let count = ((x_max - x_min) / bin_width).round();
    if !count.is_finite() || count >= usize::MAX as f64 {
        return Vec::new();
    }
    let total_bins = count as usize;
    if total_bins == 0 {
        return vec![x_min, x_max];
    }
    // Maximum number of tick intervals that keeps labels readable.
    let target_intervals = target_ticks.saturating_sub(1).max(2).min(total_bins);
    // The largest permitted interval count gives the smallest bin multiplier.
    // Check requested counts alongside factor pairs through sqrt(total_bins).
    // Small requests finish quickly; large requests avoid a long descending scan.
    let mut intervals = target_intervals;
    let mut factor = 1usize;
    let mut best = 1usize;
    let num_steps = loop {
        if total_bins.is_multiple_of(intervals) {
            break intervals;
        }
        let complement = total_bins / factor;
        if factor > complement {
            break best;
        }
        if total_bins.is_multiple_of(factor) {
            // Increasing factors give decreasing complements. The first
            // permitted complement is the largest possible interval count.
            if complement <= target_intervals {
                break complement;
            }
            if factor <= target_intervals {
                best = factor;
            }
        }
        intervals -= 1;
        factor += 1;
    };
    let n = total_bins / num_steps;
    let step = n as f64 * bin_width;
    if !step.is_finite() {
        return Vec::new();
    }
    let position = |k: usize| {
        // Form the bin index first; k * n <= total_bins for every checked index.
        let bin = k * n;
        // Match the renderer's separate multiplication and addition at bar edges.
        x_min + bin as f64 * bin_width
    };
    // The guarded bin count leaves room for this exclusive end index.
    let end = num_steps + 1;
    let mut ticks = Vec::new();
    let mut k = 0usize;
    let mut value = position(k);
    while k < end {
        let rounded = round_tick(value, step, 1e9, x_min, x_max);
        if rounded.is_finite()
            && rounded >= x_min
            && rounded <= x_max
            && ticks.last().is_none_or(|&previous| rounded > previous)
        {
            ticks.push(rounded);
        }
        let next = k + 1;
        if next == end {
            break;
        }
        let next_value = position(next);
        if next_value <= value {
            k = next_distinct_index(next, end, value, position);
            if k == end {
                break;
            }
            value = position(k);
        } else {
            k = next;
            value = next_value;
        }
    }
    ticks
}

/// Generate ticks at multiples of `step` within [min, max].
/// Duplicate rounded positions are omitted. Finite nonpositive steps use automatic ticks;
/// non-finite bounds/steps or an unrepresentable grid count return no ticks.
/// Dense grids retain distinct floats even when the virtual index count overflows.
pub fn generate_ticks_with_step(min: f64, max: f64, step: f64) -> Vec<f64> {
    if !step.is_finite() {
        return Vec::new();
    }
    if step <= 0.0 {
        return generate_ticks(min, max, 5);
    }
    linear_ticks(min, max, step, |value| {
        round_tick(value, step, 1e9, min, max)
    })
}

/// Generate minor tick positions between each pair of consecutive major ticks.
/// `subdivisions` is the total number of sub-intervals (e.g. 5 -> 4 minor marks per gap).
/// Non-finite/reversed gaps and duplicate or endpoint-rounded positions are omitted.
pub fn generate_minor_ticks(major_ticks: &[f64], subdivisions: u32) -> Vec<f64> {
    if major_ticks.len() < 2 || subdivisions < 2 {
        return Vec::new();
    }
    let mut minor = Vec::new();
    for pair in major_ticks.windows(2) {
        let lo = pair[0];
        let hi = pair[1];
        if !lo.is_finite() || !hi.is_finite() || lo.next_up() >= hi {
            continue;
        }
        let n = subdivisions as f64;
        // Opposite signs need endpoint weights to preserve exact cancellation.
        let opposite = lo < 0.0 && hi > 0.0;
        let delta = hi - lo;
        // Powers of two retain the endpoint bits while bounding u32-weighted products.
        let scale = if opposite && lo.abs().max(hi.abs()) > f64::MAX / n * 0.5 {
            (1u64 << 32) as f64
        } else {
            1.0
        };
        let a = lo / scale;
        let b = hi / scale;
        let step = if opposite {
            (b - a) / n * scale
        } else {
            delta / n
        };
        let position = |k: u32| {
            if !opposite {
                return if step.is_normal() {
                    (k as f64).mul_add(step, lo)
                } else {
                    (k as f64 / n).mul_add(delta, lo)
                };
            }
            // Evaluate ((n-k)*lo + k*hi)/n with product, sum, and division residuals.
            let left = (subdivisions - k) as f64 * a;
            let right = k as f64 * b;
            let errors =
                ((subdivisions - k) as f64).mul_add(a, -left) + (k as f64).mul_add(b, -right);
            let sum = left + right;
            let part = sum - left;
            let error = (left - (sum - part)) + (right - part) + errors;
            let value = sum / n;
            let remainder = (-value).mul_add(n, sum) + error;
            (value + remainder / n) * scale
        };
        let mut k = 1u32;
        let mut value = position(k);
        while k < subdivisions {
            if value >= hi {
                break;
            }
            let rounded = round_tick(value, step, 1e9, lo, hi);
            if rounded.is_finite()
                && rounded > lo
                && rounded < hi
                && minor.last().is_none_or(|&previous| rounded > previous)
            {
                minor.push(rounded);
            }
            let next = k + 1;
            if next >= subdivisions {
                break;
            }
            let next_value = position(next);
            // Find the first different float instead of scanning repeated positions.
            if next_value <= value {
                k = next_distinct_index(next as usize, subdivisions as usize, value, |index| {
                    position(index as u32)
                }) as u32;
                value = position(k);
            } else {
                k = next;
                value = next_value;
            }
        }
    }
    minor
}

/// Estimate a good number of ticks based on axis pixel size
pub fn auto_tick_count(axis_pixels: f64) -> usize {
    let spacing = 40.0; // pixels between ticks
    let count = (axis_pixels / spacing).round() as usize;
    count.clamp(2, 10) // lock into appropriate size
}

/// Compute a nice range that fully includes the data,
pub fn auto_nice_range(data_min: f64, data_max: f64, target_ticks: usize) -> (f64, f64) {
    if data_min == data_max {
        // gotta have some range on the data
        let delta = if data_min.abs() > 1.0 { 1.0 } else { 0.1 };
        return (data_min - delta, data_max + delta);
    }

    let step = compute_tick_step(data_min, data_max, target_ticks);
    let nice_min = (data_min / step).floor() * step;
    let nice_max = (data_max / step).ceil() * step;
    (nice_min, nice_max)
}

/// Like [`auto_nice_range`], but avoids rounding a whole extra major tick
/// onto the axis just because a small breathing-room pad (added by the
/// caller so a data point at the exact boundary doesn't render flush
/// against the plot edge) tipped `ceil`/`floor` over a step boundary the
/// *raw* data didn't actually need.
///
/// `padded_min`/`padded_max` are the caller's already-padded range (used,
/// same as `auto_nice_range`, to pick the tick step and as the normal
/// rounding input); `raw_min`/`raw_max` are the unpadded data extent. When
/// rounding the padded value lands on a *different* (larger) multiple than
/// rounding the raw value would have, that extra step exists purely because
/// of the padding — for an axis with few, large-value ticks this can
/// otherwise inflate the range by 15-25%+ for data that already fits
/// snugly. In that case the margin is capped at `min(step / 2, 5% of the
/// raw span)` instead, and the resulting boundary may not itself land on a
/// tick — ticks are generated separately and simply stop at the boundary.
pub fn auto_nice_range_capped(
    padded_min: f64,
    padded_max: f64,
    raw_min: f64,
    raw_max: f64,
    target_ticks: usize,
) -> (f64, f64) {
    if padded_min == padded_max {
        let delta = if padded_min.abs() > 1.0 { 1.0 } else { 0.1 };
        return (padded_min - delta, padded_max + delta);
    }

    let step = compute_tick_step(padded_min, padded_max, target_ticks);
    let raw_span = (raw_max - raw_min).max(f64::EPSILON);
    let tol = step * 1e-9;

    let nice_max_padded = (padded_max / step).ceil() * step;
    let nice_max_raw = (raw_max / step).ceil() * step;
    let nice_max = if nice_max_padded > nice_max_raw + tol {
        (raw_max + (step * 0.5).min(raw_span * 0.05)).max(nice_max_raw)
    } else {
        nice_max_padded
    };

    let nice_min_padded = (padded_min / step).floor() * step;
    let nice_min_raw = (raw_min / step).floor() * step;
    let nice_min = if nice_min_padded < nice_min_raw - tol {
        (raw_min - (step * 0.5).min(raw_span * 0.05)).min(nice_min_raw)
    } else {
        nice_min_padded
    };

    (nice_min, nice_max)
}

/// Compute a nice log-scale range that fully includes the data.
/// Rounds to powers of 10 so boundaries always align with generated ticks.
pub fn auto_nice_range_log(data_min: f64, data_max: f64) -> (f64, f64) {
    let clamped_max = if data_max <= 0.0 {
        eprintln!(
            "warning: log scale data_max ({}) <= 0, clamping to 1.0",
            data_max
        );
        1.0
    } else {
        data_max
    };
    let clamped_min = if data_min <= 0.0 {
        // Use a reasonable lower bound relative to max (~7 decades spread)
        // This handles the common case where pad_min() zeroed out a small positive value
        clamped_max * 1e-7
    } else {
        data_min
    };

    let nice_min = 10f64.powf(clamped_min.log10().floor());
    let nice_max = 10f64.powf(clamped_max.log10().ceil());

    // Ensure at least one decade of range
    if (nice_max / nice_min - 1.0).abs() < 1e-8 {
        (nice_min / 10.0, nice_max * 10.0)
    } else {
        (nice_min, nice_max)
    }
}

/// Selects the log-tick multiplier set for a given axis span: `[1, 2, 5]` per
/// decade for narrow ranges, or pure powers of ten for wide ones. Shared by
/// [`generate_ticks_log`] and [`log_tick_after`]/[`log_tick_before`] so both
/// agree on which pattern a given axis range actually uses.
pub(crate) fn log_multipliers(min: f64, max: f64) -> &'static [f64] {
    let log_min = min.max(1e-10).log10().floor() as i32;
    let log_max = max.log10().ceil() as i32;
    let decades = (log_max - log_min).max(0) as usize;
    if decades <= 3 {
        &[1.0, 2.0, 5.0]
    } else {
        &[1.0]
    }
}

/// Generate tick marks for a log-scale axis.
/// For narrow ranges (≤ 3 decades), include 2x and 5x sub-ticks.
/// For wider ranges, only powers of 10.
pub fn generate_ticks_log(min: f64, max: f64) -> Vec<f64> {
    let log_min = min.max(1e-10).log10().floor() as i32;
    let log_max = max.log10().ceil() as i32;
    let multipliers = log_multipliers(min, max);

    let mut ticks = Vec::new();
    for exp in log_min..=log_max {
        let base = 10f64.powi(exp);
        for &mult in multipliers {
            let tick = base * mult;
            if tick >= min * (1.0 - 1e-8) && tick <= max * (1.0 + 1e-8) {
                ticks.push(tick);
            }
        }
    }
    ticks
}

/// The next tick strictly greater than `v` in the `[1, 2, 5] × 10^n` (or pure
/// `10^n`) pattern described by `multipliers` (see [`log_multipliers`]).
/// Used to find the real tick that would exist just past the last major on a
/// log axis, instead of guessing it from the ratio of the outermost pair —
/// which is wrong whenever that pair straddles a `2x`/`5x` sub-tick rather
/// than a power-of-ten boundary.
pub(crate) fn log_tick_after(v: f64, multipliers: &[f64]) -> f64 {
    let exp = v.log10().floor() as i32;
    for e in exp..=exp + 2 {
        let base = 10f64.powi(e);
        for &mult in multipliers {
            let candidate = base * mult;
            if candidate > v * (1.0 + 1e-9) {
                return candidate;
            }
        }
    }
    v * 10.0 // unreachable given a finite multiplier set; keeps the axis usable
}

/// The previous tick strictly less than `v` — see [`log_tick_after`].
pub(crate) fn log_tick_before(v: f64, multipliers: &[f64]) -> f64 {
    let exp = v.log10().ceil() as i32;
    for e in (exp - 2..=exp).rev() {
        let base = 10f64.powi(e);
        for &mult in multipliers.iter().rev() {
            let candidate = base * mult;
            if candidate < v * (1.0 - 1e-9) {
                return candidate;
            }
        }
    }
    v / 10.0 // unreachable given a finite multiplier set; keeps the axis usable
}

/// Format a tick value for display on a log-scale axis
pub fn format_log_tick(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    let log_val = value.abs().log10();
    // Check if it's an exact power of 10
    if (log_val - log_val.round()).abs() < 1e-8 {
        let exp = log_val.round() as i32;
        if (0..=6).contains(&exp) {
            format!("{}", 10f64.powi(exp) as u64)
        } else {
            format!("1e{}", exp)
        }
    } else if value >= 1.0 {
        format!("{:.0}", value)
    } else {
        // For small values, use enough precision
        let digits = (-log_val.floor() as i32 + 1).max(1) as usize;
        format!("{:.*}", digits, value)
    }
}

// TODO: move helper
pub fn percentile(sorted: &[f64], p: f64) -> f64 {
    let rank = p / 100.0 * (sorted.len() - 1) as f64;
    let low = rank.floor() as usize;
    let high = rank.ceil() as usize;
    let weight = rank - low as f64;
    sorted[low] * (1.0 - weight) + sorted[high] * weight
}

/// Silverman's rule of thumb for automatic KDE bandwidth selection.
/// h = 0.9 * A * n^(-1/5), where A = min(σ, IQR/1.34)
pub fn silverman_bandwidth(values: &[f64]) -> f64 {
    let n = values.len();
    if n < 2 {
        return 1.0;
    }

    let mean = values.iter().sum::<f64>() / n as f64;
    let std_dev = (values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64).sqrt();

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let iqr = percentile(&sorted, 75.0) - percentile(&sorted, 25.0);

    let a = if iqr > 0.0 {
        std_dev.min(iqr / 1.34)
    } else {
        std_dev
    };
    if a == 0.0 {
        return 1.0;
    } // degenerate: all identical values

    0.9 * a * (n as f64).powf(-0.2)
}

/// Gaussian kernel density estimate.
/// Extends the evaluation range by 3*bandwidth on each side so Gaussian tails
/// taper smoothly rather than terminating sharply at the data extremes.
///
/// Uses a truncated kernel: for each evaluation point only the sorted values
/// within 4*bandwidth contribute (Gaussian contribution beyond that is < 0.003%).
/// This gives O(window × samples) instead of O(n × samples).
pub fn simple_kde(values: &[f64], bandwidth: f64, samples: usize) -> Vec<(f64, f64)> {
    use std::cmp::Ordering;
    if values.is_empty() || samples == 0 {
        return Vec::new();
    }

    let mut sorted = values.to_vec();
    sorted.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    let lo = sorted[0] - 3.0 * bandwidth;
    let hi = sorted[sorted.len() - 1] + 3.0 * bandwidth;
    let step = (hi - lo) / (samples - 1).max(1) as f64;
    let cutoff = 4.0 * bandwidth;

    (0..samples)
        .map(|i| {
            let x = lo + i as f64 * step;
            let lo_idx = sorted.partition_point(|v| *v < x - cutoff);
            let hi_idx = sorted.partition_point(|v| *v <= x + cutoff);
            let y: f64 = sorted[lo_idx..hi_idx]
                .iter()
                .map(|v| {
                    let u = (x - v) / bandwidth;
                    (-0.5 * u * u).exp()
                })
                .sum();
            (x, y)
        })
        .collect()
}

/// Gaussian KDE with boundary reflection for bounded domains.
///
/// Uses the reflection method (same approach as ggplot2 `geom_density(bounds=)`)
/// to correct the boundary bias that arises when a standard Gaussian kernel
/// places probability mass outside the valid domain.
///
/// For each data point within 3×bandwidth of an active boundary, a ghost point
/// is mirrored across that boundary. The KDE is then evaluated only within
/// `[lo, hi]` using the augmented dataset. Normalising by the original `n`
/// (not the reflected count) preserves the density integral over the bounded
/// domain — so the curve integrates to 1 over `[lo, hi]` and terminates
/// smoothly rather than terminating abruptly mid-peak.
///
/// `reflect_lo` / `reflect_hi` control whether reflection is applied at each
/// boundary; setting both to `false` with custom `lo`/`hi` gives a simple
/// truncated evaluation range without reflection.
pub fn simple_kde_reflect(
    values: &[f64],
    bandwidth: f64,
    samples: usize,
    lo: f64,
    hi: f64,
    reflect_lo: bool,
    reflect_hi: bool,
) -> Vec<(f64, f64)> {
    use std::cmp::Ordering;
    if values.is_empty() || samples == 0 || lo >= hi {
        return Vec::new();
    }

    let reflect_threshold = 3.0 * bandwidth;
    let mut aug: Vec<f64> = Vec::with_capacity(values.len() * 3);
    aug.extend_from_slice(values);
    for &v in values {
        if reflect_lo && (v - lo) < reflect_threshold {
            aug.push(2.0 * lo - v);
        }
        if reflect_hi && (hi - v) < reflect_threshold {
            aug.push(2.0 * hi - v);
        }
    }
    aug.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    let step = (hi - lo) / (samples - 1).max(1) as f64;
    let cutoff = 4.0 * bandwidth;

    (0..samples)
        .map(|i| {
            let x = lo + i as f64 * step;
            let lo_idx = aug.partition_point(|v| *v < x - cutoff);
            let hi_idx = aug.partition_point(|v| *v <= x + cutoff);
            let y: f64 = aug[lo_idx..hi_idx]
                .iter()
                .map(|v| {
                    let u = (x - v) / bandwidth;
                    (-0.5 * u * u).exp()
                })
                .sum();
            (x, y)
        })
        .collect()
}

/// linear regression of a scatter plot so we can make the equation and get correlation
pub fn linear_regression<I>(points: I) -> Option<(f64, f64, f64)>
where
    I: IntoIterator,
    I::Item: Into<(f64, f64)>,
{
    let mut vals = Vec::new();

    for (x, y) in points.into_iter().map(Into::into) {
        vals.push((x, y));
    }

    if vals.len() < 2 {
        return None;
    }

    let n = vals.len() as f64;
    let (sum_x, sum_y, sum_xy, sum_x2) = vals.iter().fold((0.0, 0.0, 0.0, 0.0), |acc, (x, y)| {
        (acc.0 + x, acc.1 + y, acc.2 + x * y, acc.3 + x * x)
    });

    let denom = n * sum_x2 - sum_x * sum_x;
    if denom.abs() < 1e-8 {
        return None;
    }

    let slope = (n * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / n;

    // Pearson correlation coefficient
    let r = pearson_corr(&vals)?;

    // y = mx+b and r
    Some((slope, intercept, r))
}

/// LOESS / LOWESS: locally-weighted linear regression smoother.
///
/// For each of `n_out` query points evenly spaced across the data's x-range, fits
/// a degree-1 weighted least-squares line to the `span` fraction of nearest points
/// (weighted by the tricube kernel of scaled distance) and evaluates it there.
/// Returns `(x, y_smoothed)` pairs sorted by x. `span` is clamped to `[0.05, 1.0]`;
/// returns an empty vec if there are fewer than 3 points or the x-range is degenerate.
pub fn loess<I>(points: I, span: f64, n_out: usize) -> Vec<(f64, f64)>
where
    I: IntoIterator,
    I::Item: Into<(f64, f64)>,
{
    let mut pts: Vec<(f64, f64)> = points.into_iter().map(Into::into).collect();
    if pts.len() < 3 || n_out == 0 {
        return Vec::new();
    }
    pts.sort_by(|a, b| a.0.total_cmp(&b.0));
    let n = pts.len();
    let x_min = pts[0].0;
    let x_max = pts[n - 1].0;
    if !(x_max - x_min).is_finite() || x_max <= x_min {
        return Vec::new();
    }
    let span = span.clamp(0.05, 1.0);
    let k = ((span * n as f64).ceil() as usize).clamp(2, n);

    let tricube = |u: f64| {
        let a = (1.0 - u.abs().powi(3)).max(0.0);
        a * a * a
    };

    let mut out = Vec::with_capacity(n_out);
    for i in 0..n_out {
        let t = i as f64 / (n_out - 1) as f64;
        let x0 = x_min + t * (x_max - x_min);

        // The k nearest neighbours by |x - x0| set the local bandwidth.
        let mut dist: Vec<f64> = pts.iter().map(|(x, _)| (x - x0).abs()).collect();
        let mut idx: Vec<usize> = (0..n).collect();
        idx.sort_by(|&a, &b| dist[a].total_cmp(&dist[b]));
        let d_max = dist[idx[k - 1]].max(1e-12);

        // Weighted degree-1 fit over the k neighbours.
        let (mut sw, mut swx, mut swy, mut swxx, mut swxy) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for &j in idx.iter().take(k) {
            let (x, y) = pts[j];
            let w = tricube(dist[j] / d_max);
            sw += w;
            swx += w * x;
            swy += w * y;
            swxx += w * x * x;
            swxy += w * x * y;
        }
        dist.clear();

        let denom = sw * swxx - swx * swx;
        let y0 = if sw <= 0.0 {
            continue;
        } else if denom.abs() < 1e-12 {
            swy / sw // degenerate (all neighbours share an x) -> weighted mean
        } else {
            let slope = (sw * swxy - swx * swy) / denom;
            let intercept = (swy - slope * swx) / sw;
            slope * x0 + intercept
        };
        out.push((x0, y0));
    }
    out
}

/// One label for the force-directed [`repel_labels`] layout. `pos` is the label
/// box centre (mutated in place); `anchor` is the fixed data point it belongs to;
/// `half_w`/`half_h` are half the label's bounding-box width/height in pixels.
#[derive(Debug, Clone, Copy)]
pub struct RepelItem {
    pub anchor: (f64, f64),
    pub half_w: f64,
    pub half_h: f64,
    pub pos: (f64, f64),
}

/// Force-directed label placement (ggrepel / adjustText style).
///
/// Iteratively pushes label boxes off each other and off every anchor point,
/// with a weak spring pulling each label back toward its own anchor, then clamps
/// each into `bounds` = `(x_min, y_min, x_max, y_max)`. Mutates `items[*].pos`.
/// O(iterations · n²); intended for the small top-N label set, not every point.
pub fn repel_labels(items: &mut [RepelItem], bounds: (f64, f64, f64, f64), iterations: usize) {
    let n = items.len();
    if n == 0 {
        return;
    }
    let (xmin, ymin, xmax, ymax) = bounds;
    let pad = 2.0;
    let spring = 0.02;
    let step = 0.6;

    for _ in 0..iterations {
        let mut force = vec![(0.0_f64, 0.0_f64); n];

        // Label vs label: resolve AABB overlap along the axis of least penetration.
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = items[i].pos.0 - items[j].pos.0;
                let dy = items[i].pos.1 - items[j].pos.1;
                let ox = (items[i].half_w + items[j].half_w + pad) - dx.abs();
                let oy = (items[i].half_h + items[j].half_h + pad) - dy.abs();
                if ox > 0.0 && oy > 0.0 {
                    if ox <= oy {
                        let s = if dx == 0.0 { 1.0 } else { dx.signum() };
                        force[i].0 += ox * 0.5 * s;
                        force[j].0 -= ox * 0.5 * s;
                    } else {
                        let s = if dy == 0.0 { 1.0 } else { dy.signum() };
                        force[i].1 += oy * 0.5 * s;
                        force[j].1 -= oy * 0.5 * s;
                    }
                }
            }
        }

        // Label vs every anchor point: push the label box off overlapping points.
        for i in 0..n {
            for k in 0..n {
                let (ax, ay) = items[k].anchor;
                let dx = items[i].pos.0 - ax;
                let dy = items[i].pos.1 - ay;
                let ox = (items[i].half_w + pad) - dx.abs();
                let oy = (items[i].half_h + pad) - dy.abs();
                if ox > 0.0 && oy > 0.0 {
                    if ox <= oy {
                        let s = if dx == 0.0 { 1.0 } else { dx.signum() };
                        force[i].0 += ox * s;
                    } else {
                        // Prefer pushing labels upward when directly over their point.
                        let s = if dy == 0.0 { -1.0 } else { dy.signum() };
                        force[i].1 += oy * s;
                    }
                }
            }
        }

        // Weak spring back to the anchor, then integrate and clamp into bounds.
        for i in 0..n {
            force[i].0 += spring * (items[i].anchor.0 - items[i].pos.0);
            force[i].1 += spring * (items[i].anchor.1 - items[i].pos.1);
            items[i].pos.0 += force[i].0 * step;
            items[i].pos.1 += force[i].1 * step;
            items[i].pos.0 = items[i]
                .pos
                .0
                .clamp(xmin + items[i].half_w, xmax - items[i].half_w);
            items[i].pos.1 = items[i]
                .pos
                .1
                .clamp(ymin + items[i].half_h, ymax - items[i].half_h);
        }
    }
}

/// Greedy beeswarm layout: returns x pixel offsets from group center for each
/// point such that no two points overlap (Euclidean distance ≥ 2*point_r).
/// Placement tries x=0, then ±step, ±2×step, … (step = point_r).
pub fn beeswarm_positions(y_screen: &[f64], point_r: f64) -> Vec<f64> {
    let n = y_screen.len();
    if n == 0 {
        return vec![];
    }

    let mut result = vec![0.0f64; n];
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        y_screen[a]
            .partial_cmp(&y_screen[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut placed: Vec<(f64, f64)> = Vec::with_capacity(n);
    let min_dist_sq = (2.0 * point_r) * (2.0 * point_r);
    let step = point_r;

    for &idx in &order {
        let y = y_screen[idx];
        let mut chosen_x = 0.0;

        // k=0 → x=0; k=1 → +step; k=2 → -step; k=3 → +2*step; k=4 → -2*step; …
        for k in 0usize..=2000 {
            let x_try = if k == 0 {
                0.0
            } else {
                let magnitude = k.div_ceil(2) as f64 * step;
                if k % 2 == 1 {
                    magnitude
                } else {
                    -magnitude
                }
            };
            let ok = placed.iter().all(|&(px, py)| {
                let dx = x_try - px;
                let dy = y - py;
                dx * dx + dy * dy >= min_dist_sq
            });
            if ok {
                chosen_x = x_try;
                break;
            }
        }

        placed.push((chosen_x, y));
        result[idx] = chosen_x;
    }

    result
}

// Pearson correlation coefficient (r)
pub fn pearson_corr(data: &[(f64, f64)]) -> Option<f64> {
    let n = data.len();
    if n < 2 {
        return None;
    }

    let (mut sum_x, mut sum_y) = (0.0, 0.0);
    for &(x, y) in data {
        sum_x += x;
        sum_y += y;
    }

    let mean_x = sum_x / n as f64;
    let mean_y = sum_y / n as f64;

    let (mut cov, mut var_x, mut var_y) = (0.0, 0.0, 0.0);
    for &(x, y) in data {
        let dx = x - mean_x;
        let dy = y - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    if var_x == 0.0 || var_y == 0.0 {
        return None;
    }

    Some(cov / (var_x.sqrt() * var_y.sqrt()))
}

// ── Phylogenetic tree helpers ─────────────────────────────────────────────────

/// UPGMA hierarchical clustering. Returns `(nodes, root_id)`.
///
/// `labels` must have the same length as `dist` (square symmetric matrix).
pub fn upgma(labels: &[&str], dist: &[Vec<f64>]) -> (Vec<crate::plot::phylo::PhyloNode>, usize) {
    use crate::plot::phylo::PhyloNode;

    let n = labels.len();
    assert!(n >= 1, "UPGMA requires at least one label");

    // Create leaf nodes
    let mut nodes: Vec<PhyloNode> = (0..n)
        .map(|i| PhyloNode {
            id: i,
            label: Some(labels[i].to_string()),
            parent: None,
            children: Vec::new(),
            branch_length: 0.0,
            support: None,
        })
        .collect();

    if n == 1 {
        return (nodes, 0);
    }

    // Working distance matrix (extended to hold internal nodes)
    let total = 2 * n - 1;
    let mut dm = vec![vec![0.0f64; total]; total];
    for i in 0..n {
        for j in 0..n {
            dm[i][j] = dist[i][j];
        }
    }

    let mut active: Vec<usize> = (0..n).collect();
    let mut size: Vec<usize> = vec![1; total];
    let mut height: Vec<f64> = vec![0.0; total];
    let mut next_id = n;

    while active.len() > 1 {
        // Find the pair with minimum distance
        let mut min_d = f64::INFINITY;
        let mut best = (0usize, 1usize); // indices into `active`
        for ai in 0..active.len() {
            for aj in (ai + 1)..active.len() {
                let d = dm[active[ai]][active[aj]];
                if d < min_d {
                    min_d = d;
                    best = (ai, aj);
                }
            }
        }
        let (ai, aj) = best;
        let ci = active[ai];
        let cj = active[aj];

        let h_new = min_d / 2.0;
        let bl_i = (h_new - height[ci]).max(0.0);
        let bl_j = (h_new - height[cj]).max(0.0);

        nodes[ci].branch_length = bl_i;
        nodes[cj].branch_length = bl_j;

        let new_id = next_id;
        let new_size = size[ci] + size[cj];
        next_id += 1;

        nodes.push(PhyloNode {
            id: new_id,
            label: None,
            parent: None,
            children: vec![ci, cj],
            branch_length: 0.0,
            support: None,
        });
        nodes[ci].parent = Some(new_id);
        nodes[cj].parent = Some(new_id);

        size[new_id] = new_size;
        height[new_id] = h_new;

        // Update distances for the new cluster
        for &ck in &active {
            if ck == ci || ck == cj {
                continue;
            }
            let d_new =
                (dm[ck][ci] * size[ci] as f64 + dm[ck][cj] * size[cj] as f64) / new_size as f64;
            dm[ck][new_id] = d_new;
            dm[new_id][ck] = d_new;
        }

        // Remove ci and cj (remove larger index first to keep smaller valid)
        if ai < aj {
            active.remove(aj);
            active.remove(ai);
        } else {
            active.remove(ai);
            active.remove(aj);
        }
        active.push(new_id);
    }

    let root = active[0];
    // Ensure all node ids are consistent
    for (i, node) in nodes.iter_mut().enumerate() {
        node.id = i;
    }
    (nodes, root)
}

/// Convert a scipy / R linkage matrix into a `PhyloNode` tree.
///
/// Each row is `[left_idx, right_idx, distance, n_leaves]`.
/// Original leaf indices are `0..n`; internal nodes get indices `n..`.
pub fn linkage_to_nodes(
    labels: &[&str],
    linkage: &[[f64; 4]],
) -> (Vec<crate::plot::phylo::PhyloNode>, usize) {
    use crate::plot::phylo::PhyloNode;

    let n = labels.len();

    let mut nodes: Vec<PhyloNode> = (0..n)
        .map(|i| PhyloNode {
            id: i,
            label: Some(labels[i].to_string()),
            parent: None,
            children: Vec::new(),
            branch_length: 0.0,
            support: None,
        })
        .collect();

    for (row_idx, row) in linkage.iter().enumerate() {
        let left = row[0] as usize;
        let right = row[1] as usize;
        let dist = row[2];
        let new_id = n + row_idx;

        // Height of a cluster = half its merge distance
        let height_left = if left < n {
            0.0
        } else {
            linkage[left - n][2] / 2.0
        };
        let height_right = if right < n {
            0.0
        } else {
            linkage[right - n][2] / 2.0
        };
        let h_new = dist / 2.0;

        let bl_left = (h_new - height_left).max(0.0);
        let bl_right = (h_new - height_right).max(0.0);

        // Apply branch lengths to the children that already exist in nodes
        if left < nodes.len() {
            nodes[left].branch_length = bl_left;
            nodes[left].parent = Some(new_id);
        }
        if right < nodes.len() {
            nodes[right].branch_length = bl_right;
            nodes[right].parent = Some(new_id);
        }

        nodes.push(PhyloNode {
            id: new_id,
            label: None,
            parent: None,
            children: vec![left, right],
            branch_length: 0.0,
            support: None,
        });
    }

    let root = nodes.len() - 1;
    (nodes, root)
}

// ── Text utilities ───────────────────────────────────────────────────────────

/// Wrap `text` if `max_chars` is `Some`, otherwise return the text as a single-element vec.
///
/// Convenience wrapper around [`wrap_text`] for call sites that hold an
/// `Option<usize>` wrap setting.
pub fn wrap_or_single(text: &str, max_chars: Option<usize>) -> Vec<String> {
    match max_chars {
        Some(mc) => wrap_text(text, mc),
        None => vec![text.to_string()],
    }
}

/// Word-wrap `text` so that no line exceeds `max_chars` characters.
///
/// - Splits on existing `\n` first (preserving intentional line breaks).
/// - Within each segment, breaks at whitespace boundaries.
/// - A single word longer than `max_chars` is hard-broken at the limit.
/// - Returns `vec![text.to_string()]` when wrapping is unnecessary or disabled
///   (`max_chars == 0`).
pub fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    if max_chars == 0 || (text.chars().count() <= max_chars && !text.contains('\n')) {
        return vec![text.to_string()];
    }

    let mut result = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            result.push(String::new());
            continue;
        }
        let words: Vec<&str> = paragraph.split_whitespace().collect();
        if words.is_empty() {
            result.push(String::new());
            continue;
        }
        let mut line = String::new();
        let mut line_chars: usize = 0;
        for word in words {
            let word_chars = word.chars().count();
            if word_chars > max_chars {
                // Flush current line if non-empty.
                if !line.is_empty() {
                    result.push(std::mem::take(&mut line));
                }
                // Hard-break the long word.
                let mut chunk = String::new();
                let mut chunk_len = 0;
                for c in word.chars() {
                    chunk.push(c);
                    chunk_len += 1;
                    if chunk_len == max_chars {
                        result.push(chunk);
                        chunk = String::new();
                        chunk_len = 0;
                    }
                }
                // Leftover becomes the start of the next line.
                line = chunk;
                line_chars = chunk_len;
            } else if line.is_empty() {
                line = word.to_string();
                line_chars = word_chars;
            } else if line_chars + 1 + word_chars <= max_chars {
                line.push(' ');
                line.push_str(word);
                line_chars += 1 + word_chars;
            } else {
                result.push(line);
                line = word.to_string();
                line_chars = word_chars;
            }
        }
        if !line.is_empty() {
            result.push(line);
        }
    }
    if result.is_empty() {
        result.push(String::new());
    }
    result
}

/// Inverse normal CDF (probit function) — Acklam's rational approximation.
/// Accurate to ~9 significant digits for p ∈ (0, 1).
pub fn probit(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }

    #[allow(clippy::excessive_precision)]
    const A: [f64; 6] = [
        -3.969683028665376e+01,
        2.209460984245205e+02,
        -2.759285104469687e+02,
        1.38357751867269e+02,
        -3.066479806614716e+01,
        2.506628277459239e+00,
    ];
    #[allow(clippy::excessive_precision)]
    const B: [f64; 5] = [
        -5.447609879822406e+01,
        1.615858368580409e+02,
        -1.556989798598866e+02,
        6.680131188771972e+01,
        -1.328068155288572e+01,
    ];
    #[allow(clippy::excessive_precision)]
    const C: [f64; 6] = [
        -7.784894002430293e-03,
        -3.223964580411365e-01,
        -2.400758277161838e+00,
        -2.549732539343734e+00,
        4.374664141464968e+00,
        2.938163982698783e+00,
    ];
    #[allow(clippy::excessive_precision)]
    const D: [f64; 4] = [
        7.784695709041462e-03,
        3.224671290700398e-01,
        2.445134137142996e+00,
        3.754408661907416e+00,
    ];

    const P_LOW: f64 = 0.02425;
    const P_HIGH: f64 = 1.0 - P_LOW;

    if p < P_LOW {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if p <= P_HIGH {
        let q = p - 0.5;
        let r = q * q;
        q * (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5])
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_ticks_preserve_tiny_and_subnormal_steps() {
        for step in [1e-12, 1e-310] {
            let ticks = generate_ticks_with_step(0.0, 4.0 * step, step);
            assert_eq!(ticks, (0..=4).map(|k| k as f64 * step).collect::<Vec<_>>());
        }
        let smallest = f64::from_bits(1);
        assert_eq!(generate_ticks(0.0, smallest, 5), [0.0, smallest]);
        let ticks = generate_ticks(0.0, 1e-310, 5);
        assert_eq!(ticks.len(), 6);
        assert!(ticks
            .iter()
            .all(|v| v.is_finite() && (0.0..=1e-310).contains(v)));
        assert!(ticks.windows(2).all(|p| p[0] < p[1]));
    }

    #[test]
    fn linear_ticks_skip_unrepresentable_positions() {
        for lo in [-1e20_f64, 1e20] {
            let hi = lo.next_up();
            for step in [2500.0, 1e-9, 1e-20, 1e-320] {
                assert_eq!(generate_ticks_with_step(lo, hi, step), [lo, hi]);
            }
            assert_eq!(generate_ticks(lo, hi, 6), [lo, hi]);
        }
        for (lo, hi) in [
            (-f64::MAX, (-f64::MAX).next_up()),
            (f64::MAX.next_down(), f64::MAX),
        ] {
            assert_eq!(generate_ticks_with_step(lo, hi, 2e292), [lo, hi]);
            assert_eq!(generate_ticks(lo, hi, 1), [lo, hi]);
        }
    }

    #[test]
    fn linear_ticks_keep_spacing_at_large_offsets() {
        let min = 43481.24576689991_f64;
        let max = 43481.24576689998;
        let expected = std::iter::successors(Some(min), |&value| Some(value.next_up()))
            .take(10)
            .collect::<Vec<_>>();
        assert_eq!(generate_ticks(min, max, 9), expected);
        assert_eq!(generate_ticks_with_step(min, max, 5e-12), expected);
        let ticks = generate_ticks(1e9, 1e9 + 1.0, 5);
        assert!(ticks.len() >= 5);
        assert!(ticks.windows(2).all(|p| p[0] < p[1]));
        assert!(ticks.iter().all(|v| (1e9..=1e9 + 1.0).contains(v)));
        // Decimal-looking endpoints need not be exact multiples of a binary step.
        assert_eq!(
            generate_ticks_with_step(-0.3, 0.3, 0.1),
            [-0.2, -0.1, 0.0, 0.1, 0.2]
        );
    }

    #[test]
    fn bin_ticks_preserve_representable_boundaries() {
        for (lo, width) in [(0.0, 1e-12), (0.0, 1e-310), (1e20, 16384.0)] {
            for bins in [4, 30] {
                let hi = lo + bins as f64 * width;
                let multiplier = if bins == 4 { 1 } else { 6 };
                assert_eq!(
                    generate_ticks_bin_aligned(lo, hi, width, 6),
                    (0..=bins / multiplier)
                        .map(|k| lo + (k * multiplier) as f64 * width)
                        .collect::<Vec<_>>()
                );
            }
        }
        let lo = -3.1611038473044104;
        let hi = 89.87440451078888;
        let width = (hi - lo) / 403.0;
        let ticks = generate_ticks_bin_aligned(lo, hi, width, 9);
        assert_eq!(ticks.len(), 2);
        assert_eq!(ticks[1], hi);
        for lo in [-1e20_f64, 1e20] {
            let hi = lo.next_up();
            let width = (hi - lo) / 1_000_000_000.0;
            assert_eq!(
                generate_ticks_bin_aligned(lo, hi, width, 1_000_000_001),
                [lo, hi]
            );
        }
    }

    #[test]
    fn bin_ticks_handle_a_large_prime_count() {
        for target in [6, 1_000_000_006] {
            assert_eq!(
                generate_ticks_bin_aligned(0.0, 1_000_000_007.0, 1.0, target),
                [0.0, 1_000_000_007.0]
            );
        }
    }

    #[test]
    fn minor_ticks_preserve_representable_subdivisions() {
        let ticks = generate_minor_ticks(&[0.0, 1e-12, 2e-12], 5);
        assert_eq!(ticks.len(), 8);
        assert!(ticks.windows(2).all(|p| p[0] < p[1]));
        for (actual, expected) in ticks.iter().zip([
            2e-13, 4e-13, 6e-13, 8e-13, 1.2e-12, 1.4e-12, 1.6e-12, 1.8e-12,
        ]) {
            assert!((actual - expected).abs() <= 4.0 * f64::EPSILON * expected);
        }
        let smallest = f64::from_bits(1);
        assert_eq!(generate_minor_ticks(&[0.0, 2.0 * smallest], 5), [smallest]);
        assert_eq!(
            generate_minor_ticks(&[0.0, 5.0 * smallest], 3),
            [2.0 * smallest, 3.0 * smallest]
        );
        assert!(generate_minor_ticks(&[1e20, 1e20_f64.next_up()], 5).is_empty());
        assert!(generate_minor_ticks(&[1e20, 1e20_f64.next_up()], u32::MAX).is_empty());
        let middle = 1e20_f64.next_up();
        assert_eq!(
            generate_minor_ticks(&[1e20, middle.next_up()], u32::MAX),
            [middle]
        );
        for endpoint in [1e8, 1e20, 1e308] {
            let ticks = generate_minor_ticks(&[-endpoint, endpoint], 6);
            assert_eq!(ticks.len(), 5);
            assert_eq!(ticks[2], 0.0);
            assert_eq!(ticks[0], -ticks[4]);
            assert_eq!(ticks[1], -ticks[3]);
        }
        assert_eq!(
            generate_minor_ticks(&[-5.406240039962616e-308, -5.406240039962612e-308], 6).len(),
            3
        );
        assert_eq!(generate_minor_ticks(&[-1e20, 2e20], 3), [0.0, 1e20]);
        let ticks = generate_minor_ticks(&[-1e308, 1e308], 5);
        for (actual, expected) in ticks.iter().zip([-6e307, -2e307, 2e307, 6e307]) {
            assert!(actual.is_finite());
            assert!((actual - expected).abs() <= 4.0 * f64::EPSILON * expected.abs());
        }
        assert_eq!(ticks.len(), 4);
    }

    #[test]
    fn tick_generators_omit_non_finite_grids() {
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(generate_ticks(invalid, 1.0, 5).is_empty());
            assert!(generate_ticks_with_step(0.0, invalid, 0.25).is_empty());
            assert!(generate_ticks_with_step(0.0, 1.0, invalid).is_empty());
            assert!(generate_ticks_bin_aligned(0.0, 1.0, invalid, 5).is_empty());
            assert!(generate_minor_ticks(&[0.0, invalid], 5).is_empty());
        }
    }

    // ── repel_labels ─────────────────────────────────────────────────────

    #[test]
    fn repel_separates_overlapping_labels() {
        // Three identical boxes stacked on the same spot must end up non-overlapping.
        let mk = |x: f64, y: f64| RepelItem {
            anchor: (x, y),
            half_w: 20.0,
            half_h: 6.0,
            pos: (x, y),
        };
        let mut items = vec![mk(100.0, 100.0), mk(100.0, 100.0), mk(100.0, 100.0)];
        repel_labels(&mut items, (0.0, 0.0, 400.0, 400.0), 300);
        // Force-directed layout minimizes but does not guarantee zero overlap; allow a
        // small tolerance (roughly the internal padding). The key property is that the
        // stacked boxes are pushed clearly apart rather than staying coincident.
        let tol = 3.0;
        for i in 0..items.len() {
            for j in (i + 1)..items.len() {
                let dx = (items[i].pos.0 - items[j].pos.0).abs();
                let dy = (items[i].pos.1 - items[j].pos.1).abs();
                let overlap = dx < items[i].half_w + items[j].half_w - tol
                    && dy < items[i].half_h + items[j].half_h - tol;
                assert!(
                    !overlap,
                    "labels {i} and {j} still overlap: dx={dx}, dy={dy}"
                );
            }
        }
    }

    #[test]
    fn repel_keeps_labels_within_bounds() {
        let mut items: Vec<RepelItem> = (0..8)
            .map(|_| RepelItem {
                anchor: (200.0, 200.0),
                half_w: 15.0,
                half_h: 6.0,
                pos: (200.0, 200.0),
            })
            .collect();
        let bounds = (10.0, 10.0, 390.0, 390.0);
        repel_labels(&mut items, bounds, 200);
        for it in &items {
            assert!(
                it.pos.0 >= bounds.0 + it.half_w - 1e-6 && it.pos.0 <= bounds.2 - it.half_w + 1e-6
            );
            assert!(
                it.pos.1 >= bounds.1 + it.half_h - 1e-6 && it.pos.1 <= bounds.3 - it.half_h + 1e-6
            );
        }
    }

    // ── loess ────────────────────────────────────────────────────────────

    #[test]
    fn loess_recovers_a_linear_relationship() {
        // Exactly-linear data: the smoother should reproduce y = 2x + 1 closely.
        let data: Vec<(f64, f64)> = (0..40).map(|i| (i as f64, 2.0 * i as f64 + 1.0)).collect();
        let curve = loess(data.iter().copied(), 0.5, 50);
        assert_eq!(curve.len(), 50);
        for &(x, y) in &curve {
            assert!(
                (y - (2.0 * x + 1.0)).abs() < 1e-6,
                "loess off at x={x}: y={y}"
            );
        }
        // Endpoints span the data range.
        assert!((curve.first().unwrap().0 - 0.0).abs() < 1e-9);
        assert!((curve.last().unwrap().0 - 39.0).abs() < 1e-9);
    }

    #[test]
    fn loess_smooths_noise_within_data_range() {
        // Noisy sine: the smoothed curve must stay within the data's y-range (no blow-up)
        // and be smoother than the raw data (small point-to-point deltas).
        let data: Vec<(f64, f64)> = (0..120)
            .map(|i| {
                let x = i as f64 / 120.0 * 10.0;
                let noise = ((i as f64 * 12.9898).sin() * 43758.5453).fract() - 0.5;
                (x, x.sin() + 0.6 * noise)
            })
            .collect();
        let (dmin, dmax) = data
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |a, &(_, y)| {
                (a.0.min(y), a.1.max(y))
            });
        let curve = loess(data.iter().copied(), 0.4, 60);
        assert!(!curve.is_empty());
        for &(_, y) in &curve {
            assert!(
                y >= dmin - 1e-9 && y <= dmax + 1e-9,
                "loess left data range: {y}"
            );
        }
        let max_step = curve
            .windows(2)
            .map(|w| (w[1].1 - w[0].1).abs())
            .fold(0.0_f64, f64::max);
        assert!(
            max_step < 0.5,
            "smoothed curve should have small steps, got {max_step}"
        );
    }

    #[test]
    fn loess_degenerate_inputs_return_empty() {
        assert!(loess([(0.0, 1.0), (1.0, 2.0)], 0.5, 10).is_empty()); // < 3 points
                                                                      // All same x -> degenerate range.
        assert!(loess([(1.0, 1.0), (1.0, 2.0), (1.0, 3.0)], 0.5, 10).is_empty());
    }

    // ── wrap_text ────────────────────────────────────────────────────────

    #[test]
    fn wrap_no_op_when_short() {
        assert_eq!(wrap_text("short", 20), vec!["short"]);
    }

    #[test]
    fn wrap_disabled_when_zero() {
        assert_eq!(wrap_text("hello world", 0), vec!["hello world"]);
    }

    #[test]
    fn wrap_empty_string() {
        assert_eq!(wrap_text("", 10), vec![""]);
    }

    #[test]
    fn wrap_basic_word_boundary() {
        assert_eq!(wrap_text("hello world foo", 11), vec!["hello world", "foo"]);
    }

    #[test]
    fn wrap_multiple_lines() {
        assert_eq!(
            wrap_text("one two three four five", 10),
            vec!["one two", "three four", "five"]
        );
    }

    #[test]
    fn wrap_exact_fit() {
        // "hello" is exactly 5 chars with max_chars=5 → no wrap
        assert_eq!(wrap_text("hello", 5), vec!["hello"]);
    }

    #[test]
    fn wrap_one_char_over() {
        assert_eq!(wrap_text("hello world", 10), vec!["hello", "world"]);
    }

    #[test]
    fn wrap_long_word_hard_break() {
        assert_eq!(wrap_text("abcdefghij", 4), vec!["abcd", "efgh", "ij"]);
    }

    #[test]
    fn wrap_long_word_mixed() {
        assert_eq!(
            wrap_text("hi abcdefghij bye", 5),
            vec!["hi", "abcde", "fghij", "bye"]
        );
    }

    #[test]
    fn wrap_preserves_newlines() {
        assert_eq!(
            wrap_text("line one\nline two", 20),
            vec!["line one", "line two"]
        );
    }

    #[test]
    fn wrap_newline_plus_wrapping() {
        assert_eq!(
            wrap_text("hello world\nfoo bar baz", 8),
            vec!["hello", "world", "foo bar", "baz"]
        );
    }

    #[test]
    fn wrap_max_chars_one() {
        assert_eq!(wrap_text("ab cd", 1), vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn wrap_consecutive_newlines() {
        assert_eq!(wrap_text("a\n\nb", 10), vec!["a", "", "b"]);
    }

    // ── log_tick_after / log_tick_before ────────────────────────────────

    // Regression (PR #101 follow-up): the phantom-tick extrapolation used to
    // guess the next/previous major from the ratio of the outermost real
    // tick pair, which is wrong for the `[1,2,5]` per-decade pattern (ratios
    // alternate 2x/2.5x, not constant). `log_tick_after`/`log_tick_before`
    // must instead walk the actual pattern.

    #[test]
    fn log_tick_after_crosses_a_5x_to_next_decade_1x() {
        let m = log_multipliers(1.0, 35.0); // decades<=3 → [1,2,5]
        assert_eq!(log_tick_after(20.0, m), 50.0, "next after 20 is 50, not 40");
    }

    #[test]
    fn log_tick_after_within_same_decade() {
        let m = log_multipliers(1.0, 35.0);
        assert_eq!(log_tick_after(2.0, m), 5.0);
        assert_eq!(log_tick_after(5.0, m), 10.0);
    }

    #[test]
    fn log_tick_before_crosses_a_1x_to_previous_decade_5x() {
        let m = log_multipliers(1.0, 35.0);
        assert_eq!(
            log_tick_before(20.0, m),
            10.0,
            "previous before 20 is 10, matching the pattern"
        );
        assert_eq!(
            log_tick_before(10.0, m),
            5.0,
            "previous before 10 is 5, not some fraction of a constant ratio"
        );
    }

    #[test]
    fn log_tick_pure_power_of_ten_pattern_unaffected() {
        // >3 decades → multiplier set collapses to [1.0]; ratio is always 10.
        let m = log_multipliers(1.0, 30_000.0);
        assert_eq!(log_tick_after(1000.0, m), 10_000.0);
        assert_eq!(log_tick_before(1000.0, m), 100.0);
    }

    // ── auto_nice_range_capped (issue #98) ──────────────────────────────

    #[test]
    fn capped_range_caps_expansion_when_raw_max_lands_exactly_on_a_tick() {
        // Raw data 0..20 lands exactly on a tick once rounded; the caller's
        // 1% breathing-room pad (0..20.2) alone would push `ceil` up a full
        // extra step to 25 — a 25% expansion for data that already fits
        // snugly. Capped: extend by at most 5% of the raw span instead.
        let (lo, hi) = auto_nice_range_capped(0.0, 20.2, 0.0, 20.0, 5);
        assert_eq!(lo, 0.0);
        assert_eq!(
            hi, 21.0,
            "expected data_max + 5% of span (20 + 1.0), not a full tick step to 25"
        );
    }

    #[test]
    fn capped_range_leaves_natural_overshoot_untouched() {
        // Raw max 17 does NOT land on the step-2.5 grid, so ordinary
        // nice-rounding already gives natural headroom (17.5) — the pad
        // doesn't cross an extra boundary the raw data didn't already need,
        // so the result must be identical to plain auto_nice_range.
        let padded = auto_nice_range(0.0, 17.17, 5);
        let capped = auto_nice_range_capped(0.0, 17.17, 0.0, 17.0, 5);
        assert_eq!(capped, padded);
    }

    #[test]
    fn capped_range_handles_symmetric_negative_case() {
        // Raw range exactly (-20, 20), both ends on the tick grid; old
        // behavior would round out to (-30, 30) (a 50% larger span).
        let (lo, hi) = auto_nice_range_capped(-20.2, 20.2, -20.0, 20.0, 5);
        assert_eq!(lo, -22.0);
        assert_eq!(hi, 22.0);
    }

    #[test]
    fn capped_range_never_drops_below_the_raw_nice_rounding() {
        // Degenerate case: raw span is tiny relative to the step, so 5% of
        // it is smaller than f64::EPSILON's effect — the result must still
        // be at least as large as rounding the raw value alone would give.
        let (_, hi) = auto_nice_range_capped(1.0, 1.0001, 1.0, 1.0, 5);
        assert!(hi >= 1.0);
    }
}
