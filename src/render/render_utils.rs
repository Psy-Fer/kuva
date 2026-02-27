

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

/// Generate x-axis ticks for a histogram so every tick falls exactly on a bin
/// boundary.
///
/// Finds the smallest integer multiplier `n` such that `n` divides `total_bins`
/// evenly and the resulting tick count stays within `target_ticks`.  The tick
/// step is then `n * bin_width`, guaranteeing alignment with bar edges.
pub fn generate_ticks_bin_aligned(x_min: f64, x_max: f64, bin_width: f64, target_ticks: usize) -> Vec<f64> {
    if bin_width <= 0.0 || x_max <= x_min {
        return generate_ticks(x_min, x_max, target_ticks);
    }

    let total_bins = ((x_max - x_min) / bin_width).round() as usize;
    if total_bins == 0 {
        return vec![x_min, x_max];
    }

    // Maximum number of tick intervals that keeps labels readable.
    let target_intervals = (target_ticks.saturating_sub(1)).max(2);

    // Find the smallest n that divides total_bins evenly and gives ≤ target_intervals.
    let n = (1..=total_bins)
        .find(|&n| total_bins % n == 0 && total_bins / n <= target_intervals)
        .unwrap_or(total_bins);

    let step = n as f64 * bin_width;
    let num_steps = total_bins / n;

    (0..=num_steps)
        .map(|k| {
            let v = x_min + k as f64 * step;
            (v * 1e9).round() / 1e9 // round to suppress float noise
        })
        .collect()
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
    order.sort_by(|&a, &b| y_screen[a].partial_cmp(&y_screen[b]).unwrap_or(std::cmp::Ordering::Equal));

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
                if k % 2 == 1 { magnitude } else { -magnitude }
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
    let mut nodes: Vec<PhyloNode> = (0..n).map(|i| PhyloNode {
        id: i,
        label: Some(labels[i].to_string()),
        parent: None,
        children: Vec::new(),
        branch_length: 0.0,
        support: None,
    }).collect();

    if n == 1 { return (nodes, 0); }

    // Working distance matrix (extended to hold internal nodes)
    let total = 2 * n - 1;
    let mut dm = vec![vec![0.0f64; total]; total];
    for i in 0..n {
        for j in 0..n {
            dm[i][j] = dist[i][j];
        }
    }

    let mut active: Vec<usize> = (0..n).collect();
    let mut size:   Vec<usize> = vec![1; total];
    let mut height: Vec<f64>   = vec![0.0; total];
    let mut next_id = n;

    while active.len() > 1 {
        // Find the pair with minimum distance
        let mut min_d = f64::INFINITY;
        let mut best  = (0usize, 1usize); // indices into `active`
        for ai in 0..active.len() {
            for aj in (ai + 1)..active.len() {
                let d = dm[active[ai]][active[aj]];
                if d < min_d {
                    min_d = d;
                    best  = (ai, aj);
                }
            }
        }
        let (ai, aj) = best;
        let ci = active[ai];
        let cj = active[aj];

        let h_new  = min_d / 2.0;
        let bl_i   = (h_new - height[ci]).max(0.0);
        let bl_j   = (h_new - height[cj]).max(0.0);

        nodes[ci].branch_length = bl_i;
        nodes[cj].branch_length = bl_j;

        let new_id   = next_id;
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

        size[new_id]   = new_size;
        height[new_id] = h_new;

        // Update distances for the new cluster
        for &ck in &active {
            if ck == ci || ck == cj { continue; }
            let d_new = (dm[ck][ci] * size[ci] as f64
                       + dm[ck][cj] * size[cj] as f64)
                      / new_size as f64;
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
    for (i, node) in nodes.iter_mut().enumerate() { node.id = i; }
    (nodes, root)
}

/// Convert a scipy / R linkage matrix into a `PhyloNode` tree.
///
/// Each row is `[left_idx, right_idx, distance, n_leaves]`.
/// Original leaf indices are `0..n`; internal nodes get indices `n..`.
pub fn linkage_to_nodes(
    labels:  &[&str],
    linkage: &[[f64; 4]],
) -> (Vec<crate::plot::phylo::PhyloNode>, usize) {
    use crate::plot::phylo::PhyloNode;

    let n = labels.len();

    let mut nodes: Vec<PhyloNode> = (0..n).map(|i| PhyloNode {
        id: i,
        label: Some(labels[i].to_string()),
        parent: None,
        children: Vec::new(),
        branch_length: 0.0,
        support: None,
    }).collect();

    for (row_idx, row) in linkage.iter().enumerate() {
        let left  = row[0] as usize;
        let right = row[1] as usize;
        let dist  = row[2];
        let new_id = n + row_idx;

        // Height of a cluster = half its merge distance
        let height_left  = if left  < n { 0.0 } else { linkage[left  - n][2] / 2.0 };
        let height_right = if right < n { 0.0 } else { linkage[right - n][2] / 2.0 };
        let h_new        = dist / 2.0;

        let bl_left  = (h_new - height_left ).max(0.0);
        let bl_right = (h_new - height_right).max(0.0);

        // Apply branch lengths to the children that already exist in nodes
        if left < nodes.len() {
            nodes[left].branch_length = bl_left;
            nodes[left].parent        = Some(new_id);
        }
        if right < nodes.len() {
            nodes[right].branch_length = bl_right;
            nodes[right].parent        = Some(new_id);
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