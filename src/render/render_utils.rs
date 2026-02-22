

/// compute ticks so things look nice
/// compute_tick_step(min, max, target_ticks)
pub fn compute_tick_step(min: f64, max: f64, target_ticks: usize) -> f64 {
    let raw_step = (max - min) / target_ticks as f64;
    let magnitude = 10f64.powf(raw_step.abs().log10().floor());
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
    nice_residual * magnitude
}


/// Generate nice ticks for an axis
pub fn generate_ticks(min: f64, max: f64, target_ticks: usize) -> Vec<f64> {
    // get a clean step size
    let step = compute_tick_step(min, max, target_ticks);
    // ceil and floor so tick is bound by axis line
    let start = (min / step).ceil() * step;
    let end = (max / step).floor() * step;

    let mut ticks = Vec::new();
    let mut tick = start;
    while tick <= end + 1e-8 {
        ticks.push((tick * 1e6).round() / 1e6); // round to avoid float spam
        tick += step;
    }

    ticks
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

/// Compute a nice log-scale range that fully includes the data.
/// Rounds to powers of 10 so boundaries always align with generated ticks.
pub fn auto_nice_range_log(data_min: f64, data_max: f64) -> (f64, f64) {
    let clamped_max = if data_max <= 0.0 {
        eprintln!("warning: log scale data_max ({}) <= 0, clamping to 1.0", data_max);
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

/// Generate tick marks for a log-scale axis.
/// For narrow ranges (≤ 3 decades), include 2x and 5x sub-ticks.
/// For wider ranges, only powers of 10.
pub fn generate_ticks_log(min: f64, max: f64) -> Vec<f64> {
    let log_min = min.max(1e-10).log10().floor() as i32;
    let log_max = max.log10().ceil() as i32;
    let decades = (log_max - log_min) as usize;

    let multipliers: &[f64] = if decades <= 3 {
        &[1.0, 2.0, 5.0]
    } else {
        &[1.0]
    };

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

/// Format a tick value for display on a log-scale axis
pub fn format_log_tick(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    let log_val = value.abs().log10();
    // Check if it's an exact power of 10
    if (log_val - log_val.round()).abs() < 1e-8 {
        let exp = log_val.round() as i32;
        if exp >= 0 && exp <= 6 {
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
    if n < 2 { return 1.0; }

    let mean = values.iter().sum::<f64>() / n as f64;
    let std_dev = (values.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
        / (n - 1) as f64).sqrt();

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let iqr = percentile(&sorted, 75.0) - percentile(&sorted, 25.0);

    let a = if iqr > 0.0 { std_dev.min(iqr / 1.34) } else { std_dev };
    if a == 0.0 { return 1.0; } // degenerate: all identical values

    0.9 * a * (n as f64).powf(-0.2)
}

/// Gaussian kernel density estimate.
/// Extends the evaluation range by 3*bandwidth on each side so Gaussian tails
/// taper smoothly rather than terminating sharply at the data extremes.
pub fn simple_kde(values: &[f64], bandwidth: f64, samples: usize) -> Vec<(f64, f64)> {
    if values.is_empty() || samples == 0 { return Vec::new(); }

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let lo = min - 3.0 * bandwidth;
    let hi = max + 3.0 * bandwidth;
    let step = (hi - lo) / (samples - 1).max(1) as f64;

    (0..samples).map(|i| {
        let x = lo + i as f64 * step;
        let y: f64 = values.iter().map(|v| {
            let u = (x - v) / bandwidth;
            (-0.5 * u * u).exp()
        }).sum();
        (x, y)
    }).collect()
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

    if vals.len() < 2 { return None; }

    let n = vals.len() as f64;
    let (sum_x, sum_y, sum_xy, sum_x2) = vals.iter().fold((0.0, 0.0, 0.0, 0.0), |acc, (x, y)| {
        (acc.0 + x, acc.1 + y, acc.2 + x * y, acc.3 + x * x)
    });

    let denom = n * sum_x2 - sum_x * sum_x;
    if denom.abs() < 1e-8 { return None; }

    let slope = (n * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / n;

    // Pearson correlation coefficient
    let r = pearson_corr(&vals)?;

    // y = mx+b and r
    Some((slope, intercept, r))
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