

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

// TODO: move helper
pub fn percentile(sorted: &[f64], p: f64) -> f64 {
    let rank = p / 100.0 * (sorted.len() - 1) as f64;
    let low = rank.floor() as usize;
    let high = rank.ceil() as usize;
    let weight = rank - low as f64;
    sorted[low] * (1.0 - weight) + sorted[high] * weight
}


/// gaussian bump kde
/// TODO: I can do better lol
pub fn simple_kde(values: &[f64], bandwidth: f64, samples: usize) -> Vec<(f64, f64)> {
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let step = (max - min) / samples as f64;

    (0..samples).map(|i| {
        let x = min + i as f64 * step;
        let y = values.iter().map(|v| {
            let u = (x - v) / bandwidth;
            (-0.5 * u * u).exp()
        }).sum::<f64>();
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
    let mean_x = sum_x / n;
    let mean_y = sum_y / n;
    let r_num: f64 = vals.iter().map(|(x, y)| (x - mean_x) * (y - mean_y)).sum();
    let r_den_x: f64 = vals.iter().map(|(x, _)| (x - mean_x).powi(2)).sum();
    let r_den_y: f64 = vals.iter().map(|(_, y)| (y - mean_y).powi(2)).sum();
    let r = r_num / (r_den_x.sqrt() * r_den_y.sqrt());

    // y = mx+b and r
    Some((slope, intercept, r))
}