/// A single ROC curve group (one classifier or one class).
pub struct RocGroup {
    pub label: String,
    /// Raw (score, label) pairs. The curve is computed internally.
    pub raw_predictions: Option<Vec<(f64, bool)>>,
    /// Pre-computed (fpr, tpr) points. AUC estimated via trapezoidal rule.
    pub precomputed_points: Option<Vec<(f64, f64)>>,
    pub color: Option<String>,
    pub show_ci: bool,
    pub ci_alpha: f64,
    /// Restrict AUC to FPR in [lo, hi].
    pub pauc_range: Option<(f64, f64)>,
    pub show_optimal_point: bool,
    pub show_auc_label: bool,
    pub line_width: f64,
    pub dasharray: Option<String>,
}

impl RocGroup {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            raw_predictions: None,
            precomputed_points: None,
            color: None,
            show_ci: false,
            ci_alpha: 0.15,
            pauc_range: None,
            show_optimal_point: false,
            show_auc_label: true,
            line_width: 2.0,
            dasharray: None,
        }
    }

    pub fn with_raw(mut self, predictions: impl IntoIterator<Item = (f64, bool)>) -> Self {
        self.raw_predictions = Some(predictions.into_iter().collect());
        self
    }

    pub fn with_points(mut self, pts: impl IntoIterator<Item = (f64, f64)>) -> Self {
        self.precomputed_points = Some(pts.into_iter().collect());
        self
    }

    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn with_ci(mut self, show: bool) -> Self {
        self.show_ci = show;
        self
    }

    pub fn with_ci_alpha(mut self, alpha: f64) -> Self {
        self.ci_alpha = alpha;
        self
    }

    pub fn with_pauc(mut self, lo: f64, hi: f64) -> Self {
        self.pauc_range = Some((lo, hi));
        self
    }

    pub fn with_optimal_point(mut self) -> Self {
        self.show_optimal_point = true;
        self
    }

    pub fn with_auc_label(mut self, show: bool) -> Self {
        self.show_auc_label = show;
        self
    }

    pub fn with_line_width(mut self, w: f64) -> Self {
        self.line_width = w;
        self
    }

    pub fn with_dasharray(mut self, d: impl Into<String>) -> Self {
        self.dasharray = Some(d.into());
        self
    }
}

/// A Receiver Operating Characteristic (ROC) plot supporting multiple classifiers,
/// DeLong confidence intervals, partial AUC, and Youden's J optimal threshold.
pub struct RocPlot {
    pub groups: Vec<RocGroup>,
    pub color: String,
    pub show_diagonal: bool,
    pub diagonal_color: String,
    pub diagonal_dasharray: String,
    pub legend_label: Option<String>,
}

impl Default for RocPlot {
    fn default() -> Self {
        Self::new()
    }
}

impl RocPlot {
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            color: "steelblue".to_string(),
            show_diagonal: true,
            diagonal_color: "#aaaaaa".to_string(),
            diagonal_dasharray: "5,3".to_string(),
            legend_label: None,
        }
    }

    pub fn with_group(mut self, group: RocGroup) -> Self {
        self.groups.push(group);
        self
    }

    pub fn with_groups(mut self, groups: impl IntoIterator<Item = RocGroup>) -> Self {
        self.groups.extend(groups);
        self
    }

    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_diagonal(mut self, show: bool) -> Self {
        self.show_diagonal = show;
        self
    }

    pub fn with_legend(mut self, label: impl Into<String>) -> Self {
        self.legend_label = Some(label.into());
        self
    }
}

// ── Internal computation types ─────────────────────────────────────────────────

#[derive(Clone)]
pub struct RocPoint {
    pub fpr: f64,
    pub tpr: f64,
    pub threshold: f64,
}

pub struct RocComputed {
    pub points: Vec<RocPoint>,
    pub auc: f64,
    pub pauc: Option<f64>,
    pub ci_lo: f64,
    pub ci_hi: f64,
    pub optimal_idx: Option<usize>,
}

/// Sort descending by score; walk thresholds to produce ROC points.
pub fn compute_roc_points(predictions: &[(f64, bool)]) -> Vec<RocPoint> {
    if predictions.is_empty() {
        return Vec::new();
    }
    let mut sorted = predictions.to_vec();
    sorted.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let n_pos = sorted.iter().filter(|p| p.1).count();
    let n_neg = sorted.len() - n_pos;
    if n_pos == 0 || n_neg == 0 {
        return Vec::new();
    }

    let mut points = vec![RocPoint { fpr: 0.0, tpr: 0.0, threshold: f64::INFINITY }];
    let mut tp = 0usize;
    let mut fp = 0usize;
    let mut i = 0usize;
    while i < sorted.len() {
        let thresh = sorted[i].0;
        // Consume all items at this threshold
        while i < sorted.len() && (sorted[i].0 - thresh).abs() < f64::EPSILON * 100.0 {
            if sorted[i].1 { tp += 1; } else { fp += 1; }
            i += 1;
        }
        points.push(RocPoint {
            fpr: fp as f64 / n_neg as f64,
            tpr: tp as f64 / n_pos as f64,
            threshold: thresh,
        });
    }
    // Ensure we end at (1,1)
    let last = points.last().unwrap();
    if (last.fpr - 1.0).abs() > 1e-9 || (last.tpr - 1.0).abs() > 1e-9 {
        points.push(RocPoint { fpr: 1.0, tpr: 1.0, threshold: f64::NEG_INFINITY });
    }
    points
}

/// Trapezoidal AUC.
pub fn auc_trapz(points: &[RocPoint]) -> f64 {
    let mut auc = 0.0;
    for w in points.windows(2) {
        let dx = w[1].fpr - w[0].fpr;
        let dy = (w[1].tpr + w[0].tpr) / 2.0;
        auc += dx * dy;
    }
    auc.abs()
}

/// DeLong AUC + variance for 95% CI.
/// Returns `(auc, variance)`. CI = `auc ± 1.96 * sqrt(variance)`.
pub fn delong_auc(predictions: &[(f64, bool)]) -> (f64, f64) {
    let pos: Vec<f64> = predictions.iter().filter(|p| p.1).map(|p| p.0).collect();
    let neg: Vec<f64> = predictions.iter().filter(|p| !p.1).map(|p| p.0).collect();
    let n_pos = pos.len();
    let n_neg = neg.len();
    if n_pos == 0 || n_neg == 0 {
        return (0.5, 0.0);
    }

    // V10[i] = fraction of negatives with score < pos[i]  (placement statistic)
    let v10: Vec<f64> = pos.iter().map(|&s| {
        let less = neg.iter().filter(|&&n| n < s).count();
        let tied = neg.iter().filter(|&&n| (n - s).abs() < f64::EPSILON * 100.0).count();
        (less as f64 + 0.5 * tied as f64) / n_neg as f64
    }).collect();

    // V01[j] = fraction of positives with score > neg[j]
    let v01: Vec<f64> = neg.iter().map(|&s| {
        let greater = pos.iter().filter(|&&p| p > s).count();
        let tied = pos.iter().filter(|&&p| (p - s).abs() < f64::EPSILON * 100.0).count();
        (greater as f64 + 0.5 * tied as f64) / n_pos as f64
    }).collect();

    let auc = v10.iter().sum::<f64>() / n_pos as f64;
    let var10 = variance(&v10);
    let var01 = variance(&v01);
    let auc_var = var10 / n_pos as f64 + var01 / n_neg as f64;
    (auc, auc_var)
}

fn variance(v: &[f64]) -> f64 {
    if v.len() < 2 {
        return 0.0;
    }
    let mean = v.iter().sum::<f64>() / v.len() as f64;
    v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (v.len() - 1) as f64
}

/// Partial AUC over FPR in `[fpr_lo, fpr_hi]`, normalised to the range width.
pub fn partial_auc(points: &[RocPoint], fpr_lo: f64, fpr_hi: f64) -> f64 {
    let mut clipped: Vec<(f64, f64)> = Vec::new();
    for w in points.windows(2) {
        let (f0, t0) = (w[0].fpr, w[0].tpr);
        let (f1, t1) = (w[1].fpr, w[1].tpr);
        if f1 <= fpr_lo || f0 >= fpr_hi {
            continue;
        }
        let fa = f0.max(fpr_lo);
        let fb = f1.min(fpr_hi);
        let interp = |f: f64| -> f64 {
            if (f1 - f0).abs() < 1e-12 { t0 }
            else { t0 + (t1 - t0) * (f - f0) / (f1 - f0) }
        };
        clipped.push((fa, interp(fa)));
        clipped.push((fb, interp(fb)));
    }
    clipped.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-12);
    let raw: f64 = clipped.windows(2).map(|w| {
        (w[1].0 - w[0].0) * (w[0].1 + w[1].1) / 2.0
    }).sum();
    let width = fpr_hi - fpr_lo;
    if width > 0.0 { raw / width } else { 0.0 }
}

/// Index of the optimal threshold point (maximises Youden J = TPR − FPR).
pub(crate) fn optimal_threshold_idx(points: &[RocPoint]) -> usize {
    points.iter().enumerate()
        .max_by(|(_, a), (_, b)| {
            let ja = a.tpr - a.fpr;
            let jb = b.tpr - b.fpr;
            ja.partial_cmp(&jb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Full pipeline: compute everything for a `RocGroup`.
pub fn compute_group(group: &RocGroup) -> RocComputed {
    let points = if let Some(raw) = &group.raw_predictions {
        compute_roc_points(raw)
    } else if let Some(pts) = &group.precomputed_points {
        pts.iter().map(|&(f, t)| RocPoint { fpr: f, tpr: t, threshold: f64::NAN }).collect()
    } else {
        Vec::new()
    };

    if points.is_empty() {
        return RocComputed { points, auc: 0.0, pauc: None, ci_lo: 0.0, ci_hi: 0.0, optimal_idx: None };
    }

    let (auc, ci_lo, ci_hi) = if let Some(raw) = &group.raw_predictions {
        let (a, var) = delong_auc(raw);
        let margin = 1.96 * var.sqrt();
        (a, (a - margin).max(0.0), (a + margin).min(1.0))
    } else {
        let a = auc_trapz(&points);
        (a, f64::NAN, f64::NAN)
    };

    let pauc = group.pauc_range.map(|(lo, hi)| partial_auc(&points, lo, hi));
    let optimal_idx = if group.show_optimal_point {
        Some(optimal_threshold_idx(&points))
    } else {
        None
    };

    RocComputed { points, auc, pauc, ci_lo, ci_hi, optimal_idx }
}
