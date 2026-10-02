/// A single Precision-Recall curve group (one classifier or one class).
pub struct PrGroup {
    pub label: String,
    /// Raw (score, label) pairs. The curve is computed internally.
    pub raw_predictions: Option<Vec<(f64, bool)>>,
    /// Pre-computed (recall, precision) points.
    pub precomputed_points: Option<Vec<(f64, f64)>>,
    /// Prevalence override for pre-computed data (used for baseline).
    pub prevalence: Option<f64>,
    pub color: Option<String>,
    pub show_optimal_point: bool,
    pub show_auc_label: bool,
    pub line_width: f64,
    pub dasharray: Option<String>,
}

impl PrGroup {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            raw_predictions: None,
            precomputed_points: None,
            prevalence: None,
            color: None,
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

    pub fn with_prevalence(mut self, p: f64) -> Self {
        self.prevalence = Some(p);
        self
    }

    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
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

/// A Precision-Recall (PR) curve plot — the standard companion to ROC for
/// imbalanced classification. Supports multiple classifiers, AUC-PR, a
/// no-skill prevalence baseline, and an optimal F1 threshold marker.
pub struct PrPlot {
    pub groups: Vec<PrGroup>,
    pub color: String,
    pub show_baseline: bool,
    pub baseline_color: String,
    pub baseline_dasharray: String,
    pub legend_label: Option<String>,
}

impl Default for PrPlot {
    fn default() -> Self {
        Self::new()
    }
}

impl PrPlot {
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            color: "steelblue".to_string(),
            show_baseline: true,
            baseline_color: "#aaaaaa".to_string(),
            baseline_dasharray: "5,3".to_string(),
            legend_label: None,
        }
    }

    pub fn with_group(mut self, group: PrGroup) -> Self {
        self.groups.push(group);
        self
    }

    pub fn with_groups(mut self, groups: impl IntoIterator<Item = PrGroup>) -> Self {
        self.groups.extend(groups);
        self
    }

    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_baseline(mut self, show: bool) -> Self {
        self.show_baseline = show;
        self
    }

    pub fn with_legend(mut self, label: impl Into<String>) -> Self {
        self.legend_label = Some(label.into());
        self
    }
}

// ── Internal computation types ─────────────────────────────────────────────────

#[derive(Clone)]
pub struct PrPoint {
    pub recall: f64,
    pub precision: f64,
    pub threshold: f64,
}

pub struct PrComputed {
    pub points: Vec<PrPoint>,
    pub auc: f64,
    /// Prevalence = n_pos / n_total (used for the no-skill baseline).
    pub prevalence: f64,
    pub optimal_idx: Option<usize>,
}

/// Sort descending by score; walk thresholds to produce PR points.
/// Returns `(points, prevalence)`.
/// Only equal scores share a threshold; distinct scores keep their order.
pub fn compute_pr_points(predictions: &[(f64, bool)]) -> (Vec<PrPoint>, f64) {
    if predictions.is_empty() {
        return (Vec::new(), 0.0);
    }
    let mut sorted = predictions.to_vec();
    sorted.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let n_pos = sorted.iter().filter(|p| p.1).count();
    let n_total = sorted.len();
    let prevalence = n_pos as f64 / n_total as f64;

    if n_pos == 0 {
        return (Vec::new(), prevalence);
    }

    // Anchor: (recall=0, precision=1.0) at threshold=+∞
    let mut points = vec![PrPoint {
        recall: 0.0,
        precision: 1.0,
        threshold: f64::INFINITY,
    }];
    let mut tp = 0usize;
    let mut fp = 0usize;
    let mut i = 0usize;

    while i < sorted.len() {
        let thresh = sorted[i].0;
        // Consume all items at this threshold
        while i < sorted.len() && sorted[i].0 == thresh {
            if sorted[i].1 {
                tp += 1;
            } else {
                fp += 1;
            }
            i += 1;
        }
        let precision = if tp + fp > 0 {
            tp as f64 / (tp + fp) as f64
        } else {
            1.0
        };
        let recall = tp as f64 / n_pos as f64;
        points.push(PrPoint {
            recall,
            precision,
            threshold: thresh,
        });
    }

    (points, prevalence)
}

/// Trapezoidal AUC-PR over the (recall, precision) curve.
pub fn auc_pr_trapz(points: &[PrPoint]) -> f64 {
    let mut auc = 0.0;
    for w in points.windows(2) {
        let dr = w[1].recall - w[0].recall;
        let avg_p = (w[0].precision + w[1].precision) / 2.0;
        auc += dr * avg_p;
    }
    auc.abs()
}

/// Index of the point that maximises F1 = 2·P·R / (P+R).
pub fn optimal_f1_idx(points: &[PrPoint]) -> usize {
    points
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            let f1a = f1(a.precision, a.recall);
            let f1b = f1(b.precision, b.recall);
            f1a.partial_cmp(&f1b).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
        .unwrap_or(0)
}

fn f1(precision: f64, recall: f64) -> f64 {
    let denom = precision + recall;
    if denom > 0.0 {
        2.0 * precision * recall / denom
    } else {
        0.0
    }
}

/// Full pipeline: compute everything for a `PrGroup`.
pub fn compute_pr_group(group: &PrGroup) -> PrComputed {
    let (points, prevalence) = if let Some(raw) = &group.raw_predictions {
        compute_pr_points(raw)
    } else if let Some(pts) = &group.precomputed_points {
        let converted = pts
            .iter()
            .map(|&(r, p)| PrPoint {
                recall: r,
                precision: p,
                threshold: f64::NAN,
            })
            .collect();
        (converted, group.prevalence.unwrap_or(0.5))
    } else {
        (Vec::new(), 0.0)
    };

    if points.is_empty() {
        return PrComputed {
            points,
            auc: 0.0,
            prevalence,
            optimal_idx: None,
        };
    }

    let auc = auc_pr_trapz(&points);
    let optimal_idx = if group.show_optimal_point {
        Some(optimal_f1_idx(&points))
    } else {
        None
    };

    PrComputed {
        points,
        auc,
        prevalence,
        optimal_idx,
    }
}
