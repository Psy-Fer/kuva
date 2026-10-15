/// One group of survival observations for a Kaplan-Meier curve.
///
/// Each subject contributes one `time` (time to event or last follow-up) and
/// one `event` flag (`true` = event occurred, `false` = censored/right-truncated).
pub struct KMGroup {
    pub label: String,
    pub times: Vec<f64>,
    pub events: Vec<bool>,
    /// Per-group color override. `None` falls back to the plot palette.
    pub color: Option<String>,
}

/// Builder for a Kaplan-Meier survival plot.
///
/// Each group produces a step-function survival curve, optional confidence
/// bands (Greenwood's formula, linear scale), and optional censoring tick
/// marks. When multiple groups are present a log-rank p-value can be
/// annotated via [`with_pvalue_text`](Self::with_pvalue_text).
///
/// # Example
///
/// ```rust,no_run
/// use kuva::prelude::*;
///
/// let plot = SurvivalPlot::new()
///     .with_group("Treatment A", vec![5.0,8.0,12.0,15.0,20.0], vec![true,true,false,true,false])
///     .with_group("Treatment B", vec![3.0,6.0,9.0,14.0,18.0], vec![true,false,true,true,false])
///     .with_ci(true)
///     .with_legend("Group");
///
/// let plots = vec![Plot::from(plot)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Overall Survival")
///     .with_x_label("Time (months)")
///     .with_y_label("Survival probability");
/// ```
pub struct SurvivalPlot {
    pub groups: Vec<KMGroup>,
    /// Fallback line color when no per-group colors are set and no palette is active.
    pub color: String,
    /// Per-group color overrides (indexed by group order).
    pub group_colors: Option<Vec<String>>,
    /// Line stroke width in pixels. Default `2.0`.
    pub line_width: f64,
    /// Draw Greenwood 95% CI bands. Default `false`.
    pub show_ci: bool,
    /// Opacity of CI bands. Default `0.2`.
    pub ci_alpha: f64,
    /// Draw censoring tick marks on the curves. Default `true`.
    pub show_censoring: bool,
    /// Half-height of censoring ticks in pixels. Default `4.0`.
    pub censoring_size: f64,
    /// Optional p-value / annotation text rendered in the upper-right of the plot area.
    pub pvalue_text: Option<String>,
    pub legend_label: Option<String>,
    /// Draw a "Number at risk" table below the plot, aligned to the x-axis ticks.
    /// Default `false`.
    pub risk_table: bool,
    /// Draw median-survival reference lines (horizontal at S=0.5 to each curve's
    /// median time, then vertical down to the axis). Default `false`.
    pub median_lines: bool,
    /// Compute the log-rank test p-value from the raw data and annotate it (overrides
    /// [`pvalue_text`](Self::pvalue_text) when set). Default `false`.
    pub logrank_pvalue: bool,
}

impl Default for SurvivalPlot {
    fn default() -> Self {
        Self::new()
    }
}

impl SurvivalPlot {
    /// Create a survival plot with default settings.
    pub fn new() -> Self {
        Self {
            groups: vec![],
            color: "steelblue".into(),
            group_colors: None,
            line_width: 2.0,
            show_ci: false,
            ci_alpha: 0.2,
            show_censoring: true,
            censoring_size: 4.0,
            pvalue_text: None,
            legend_label: None,
            risk_table: false,
            median_lines: false,
            logrank_pvalue: false,
        }
    }

    /// Add a group with separate time and event vectors.
    ///
    /// `times`: time to event or censoring. `events`: `true` = event occurred.
    pub fn with_group(
        mut self,
        label: impl Into<String>,
        times: Vec<f64>,
        events: Vec<bool>,
    ) -> Self {
        self.groups.push(KMGroup {
            label: label.into(),
            times,
            events,
            color: None,
        });
        self
    }

    /// Add a group with a per-group color override.
    pub fn with_colored_group(
        mut self,
        label: impl Into<String>,
        times: Vec<f64>,
        events: Vec<bool>,
        color: impl Into<String>,
    ) -> Self {
        self.groups.push(KMGroup {
            label: label.into(),
            times,
            events,
            color: Some(color.into()),
        });
        self
    }

    /// Set the fallback line color. Default `"steelblue"`.
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = color.into();
        self
    }

    /// Set per-group colors (indexed by group order). Falls back to category10.
    pub fn with_group_colors(
        mut self,
        colors: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.group_colors = Some(colors.into_iter().map(|c| c.into()).collect());
        self
    }

    /// Set line stroke width in pixels. Default `2.0`.
    pub fn with_line_width(mut self, w: f64) -> Self {
        self.line_width = w;
        self
    }

    /// Show 95% confidence bands (Greenwood's formula). Default `false`.
    pub fn with_ci(mut self, show: bool) -> Self {
        self.show_ci = show;
        self
    }

    /// Set confidence band opacity. Default `0.2`.
    pub fn with_ci_alpha(mut self, alpha: f64) -> Self {
        self.ci_alpha = alpha;
        self
    }

    /// Show censoring tick marks on curves. Default `true`.
    pub fn with_censoring(mut self, show: bool) -> Self {
        self.show_censoring = show;
        self
    }

    /// Set the half-height of censoring tick marks in pixels. Default `4.0`.
    pub fn with_censoring_size(mut self, size: f64) -> Self {
        self.censoring_size = size;
        self
    }

    /// Add a p-value or annotation string rendered in the upper-right corner.
    ///
    /// Typical use: `with_pvalue_text("p = 0.023")` or `"log-rank p < 0.001"`.
    pub fn with_pvalue_text(mut self, text: impl Into<String>) -> Self {
        self.pvalue_text = Some(text.into());
        self
    }

    /// Attach a legend to this plot (shows one entry per group).
    pub fn with_legend(mut self, label: impl Into<String>) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    /// Draw a "Number at risk" table below the plot, one row per group, with counts
    /// at each x-axis tick (survminer `risk.table = TRUE`).
    pub fn with_risk_table(mut self, show: bool) -> Self {
        self.risk_table = show;
        self
    }

    /// Draw median-survival reference lines (survminer `surv.median.line`).
    pub fn with_median_lines(mut self, show: bool) -> Self {
        self.median_lines = show;
        self
    }

    /// Compute and annotate the log-rank test p-value from the raw data. Overrides
    /// any [`with_pvalue_text`](Self::with_pvalue_text) when enabled.
    pub fn with_logrank_pvalue(mut self, show: bool) -> Self {
        self.logrank_pvalue = show;
        self
    }
}

// ── Internal KM computation (pub(crate) for use by the renderer) ──────────────

/// One KM step: (time, survival, ci_lo, ci_hi).
pub(crate) struct KMPoint {
    pub t: f64,
    pub s: f64,
    pub lo: f64,
    pub hi: f64,
}

/// Compute the Kaplan-Meier curve with Greenwood 95% CI for one group.
///
/// Returns a sorted list of `KMPoint` starting at `(0, 1, 1, 1)`.
pub(crate) fn km_curve(times: &[f64], events: &[bool]) -> Vec<KMPoint> {
    let mut result = vec![KMPoint {
        t: 0.0,
        s: 1.0,
        lo: 1.0,
        hi: 1.0,
    }];
    if times.is_empty() {
        return result;
    }

    let mut pairs: Vec<(f64, bool)> = times.iter().zip(events).map(|(&t, &e)| (t, e)).collect();
    pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let n_total = pairs.len();
    let mut survival = 1.0_f64;
    let mut greenwood = 0.0_f64;
    let mut at_risk = n_total;
    let mut i = 0;

    while i < n_total {
        let t = pairs[i].0;
        let mut j = i;
        while j < n_total && pairs[j].0 == t {
            j += 1;
        }

        let n_events = pairs[i..j].iter().filter(|&&(_, e)| e).count();

        if n_events > 0 {
            let ni = at_risk;
            survival *= 1.0 - n_events as f64 / ni as f64;
            let denom = ni * (ni - n_events);
            if denom > 0 {
                greenwood += n_events as f64 / denom as f64;
            }
            let se = (survival * survival * greenwood).sqrt();
            result.push(KMPoint {
                t,
                s: survival,
                lo: (survival - 1.96 * se).max(0.0),
                hi: (survival + 1.96 * se).min(1.0),
            });
        }

        at_risk -= j - i;
        i = j;
    }

    result
}

/// Return (time, survival_at_t) for each censored observation, using the
/// last KM step at or before `t_censor`.
pub(crate) fn censoring_levels(times: &[f64], events: &[bool], km: &[KMPoint]) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for (&t, &ev) in times.iter().zip(events) {
        if ev {
            continue;
        }
        // Find survival at t: last KM point with km.t <= t
        let s = km.iter().rev().find(|p| p.t <= t).map_or(1.0, |p| p.s);
        out.push((t, s));
    }
    out
}

/// Median survival time for a KM curve: the smallest event time at which the
/// survival estimate drops to `0.5` or below. Returns `None` when the curve never
/// reaches 0.5 (more than half the group survives to the end of follow-up).
pub(crate) fn median_survival(km: &[KMPoint]) -> Option<f64> {
    km.iter().find(|p| p.s <= 0.5).map(|p| p.t)
}

/// Number of subjects still at risk (event or censoring time `>= t`) in a group.
pub(crate) fn n_at_risk(times: &[f64], t: f64) -> usize {
    times.iter().filter(|&&x| x >= t).count()
}

/// Log-rank test across `groups`, returning `(chi_square, degrees_of_freedom)`.
///
/// Two groups use the exact Mantel-Haenszel statistic with hypergeometric
/// variance (df = 1). For three or more groups the classic `Σ (O−E)²/E`
/// approximation is used (df = k−1). Returns `None` for fewer than two groups
/// or when no events occur.
pub(crate) fn logrank_chi2(groups: &[KMGroup]) -> Option<(f64, usize)> {
    let k = groups.len();
    if k < 2 {
        return None;
    }
    // Distinct times at which at least one event occurs, across all groups.
    let mut event_times: Vec<f64> = groups
        .iter()
        .flat_map(|g| {
            g.times
                .iter()
                .zip(&g.events)
                .filter(|(_, &e)| e)
                .map(|(&t, _)| t)
        })
        .collect();
    event_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    event_times.dedup();
    if event_times.is_empty() {
        return None;
    }

    let mut observed = vec![0.0_f64; k];
    let mut expected = vec![0.0_f64; k];
    let mut var = 0.0_f64; // variance of O_1 - E_1 (two-group case)

    for &t in &event_times {
        // At-risk and events at time t, overall and per group.
        let n_g: Vec<f64> = groups
            .iter()
            .map(|g| n_at_risk(&g.times, t) as f64)
            .collect();
        let d_g: Vec<f64> = groups
            .iter()
            .map(|g| {
                g.times
                    .iter()
                    .zip(&g.events)
                    .filter(|(&x, &e)| e && x == t)
                    .count() as f64
            })
            .collect();
        let n_total: f64 = n_g.iter().sum();
        let d_total: f64 = d_g.iter().sum();
        if n_total <= 0.0 || d_total <= 0.0 {
            continue;
        }
        for g in 0..k {
            observed[g] += d_g[g];
            expected[g] += d_total * n_g[g] / n_total;
        }
        if k == 2 && n_total > 1.0 {
            let p1 = n_g[0] / n_total;
            var += d_total * (n_total - d_total) / (n_total - 1.0) * p1 * (1.0 - p1);
        }
    }

    let (chi2, df) = if k == 2 {
        if var <= 0.0 {
            return None;
        }
        ((observed[0] - expected[0]).powi(2) / var, 1)
    } else {
        let chi2: f64 = (0..k)
            .filter(|&g| expected[g] > 0.0)
            .map(|g| (observed[g] - expected[g]).powi(2) / expected[g])
            .sum();
        (chi2, k - 1)
    };
    Some((chi2, df))
}

/// Log-rank p-value formatted for display, e.g. `"log-rank p = 0.023"` or
/// `"log-rank p < 0.001"`. `None` when the test is undefined.
pub(crate) fn logrank_pvalue_text(groups: &[KMGroup]) -> Option<String> {
    let (chi2, df) = logrank_chi2(groups)?;
    let p = chi_square_sf(chi2, df);
    Some(if p < 0.001 {
        "log-rank p < 0.001".to_string()
    } else {
        format!("log-rank p = {p:.3}")
    })
}

/// Upper-tail probability of the chi-square distribution: `P(X > x)` for `df`
/// degrees of freedom. Implemented via the regularized upper incomplete gamma
/// function `Q(df/2, x/2)`.
pub(crate) fn chi_square_sf(x: f64, df: usize) -> f64 {
    if x <= 0.0 {
        return 1.0;
    }
    gammq(df as f64 / 2.0, x / 2.0)
}

/// Regularized upper incomplete gamma `Q(a, x) = 1 - P(a, x)` (Numerical Recipes).
fn gammq(a: f64, x: f64) -> f64 {
    if x < 0.0 || a <= 0.0 {
        return 1.0;
    }
    if x < a + 1.0 {
        1.0 - gser(a, x)
    } else {
        gcf(a, x)
    }
}

/// Series expansion for the regularized lower incomplete gamma `P(a, x)`.
fn gser(a: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let mut ap = a;
    let mut sum = 1.0 / a;
    let mut del = sum;
    for _ in 0..200 {
        ap += 1.0;
        del *= x / ap;
        sum += del;
        if del.abs() < sum.abs() * 1e-12 {
            break;
        }
    }
    sum * (-x + a * x.ln() - ln_gamma(a)).exp()
}

/// Continued-fraction expansion for the regularized upper incomplete gamma `Q(a, x)`.
fn gcf(a: f64, x: f64) -> f64 {
    let tiny = 1e-30;
    let mut b = x + 1.0 - a;
    let mut c = 1.0 / tiny;
    let mut d = 1.0 / b;
    let mut h = d;
    for i in 1..200 {
        let an = -(i as f64) * (i as f64 - a);
        b += 2.0;
        d = an * d + b;
        if d.abs() < tiny {
            d = tiny;
        }
        c = b + an / c;
        if c.abs() < tiny {
            c = tiny;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < 1e-12 {
            break;
        }
    }
    (-x + a * x.ln() - ln_gamma(a)).exp() * h
}

/// Natural log of the gamma function (Lanczos approximation, g = 7).
fn ln_gamma(x: f64) -> f64 {
    const C: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if x < 0.5 {
        // Reflection formula.
        (std::f64::consts::PI / (std::f64::consts::PI * x).sin()).ln() - ln_gamma(1.0 - x)
    } else {
        let x = x - 1.0;
        let mut a = C[0];
        let t = x + 7.5;
        for (i, &c) in C.iter().enumerate().skip(1) {
            a += c / (x + i as f64);
        }
        0.5 * (2.0 * std::f64::consts::PI).ln() + (x + 0.5) * t.ln() - t + a.ln()
    }
}

#[cfg(test)]
mod survival_tests {
    use super::*;

    #[test]
    fn median_survival_finds_half_crossing() {
        // 5 subjects, all events at increasing times -> S = .8, .6, .4, .2, 0.
        let km = km_curve(&[1.0, 2.0, 3.0, 4.0, 5.0], &[true, true, true, true, true]);
        // Median = first time with S <= 0.5 -> t = 3 (S = 0.4).
        assert_eq!(median_survival(&km), Some(3.0));
    }

    #[test]
    fn median_none_when_survival_stays_high() {
        // Only one event; S never reaches 0.5.
        let km = km_curve(
            &[1.0, 2.0, 3.0, 4.0, 5.0],
            &[true, false, false, false, false],
        );
        assert_eq!(median_survival(&km), None);
    }

    #[test]
    fn n_at_risk_counts_from_time() {
        let times = [1.0, 2.0, 2.0, 5.0];
        assert_eq!(n_at_risk(&times, 0.0), 4);
        assert_eq!(n_at_risk(&times, 2.0), 3);
        assert_eq!(n_at_risk(&times, 5.0), 1);
        assert_eq!(n_at_risk(&times, 6.0), 0);
    }

    #[test]
    fn chi_square_sf_matches_known_values() {
        // df=1: P(X>3.841) = 0.05; df=1: P(X>6.635) = 0.01.
        assert!((chi_square_sf(3.841, 1) - 0.05).abs() < 1e-3);
        assert!((chi_square_sf(6.635, 1) - 0.01).abs() < 1e-3);
        // df=2: P(X>5.991) = 0.05.
        assert!((chi_square_sf(5.991, 2) - 0.05).abs() < 1e-3);
        assert_eq!(chi_square_sf(0.0, 1), 1.0);
    }

    #[test]
    fn logrank_identical_groups_is_not_significant() {
        // Two identical groups -> chi2 ~ 0 -> p ~ 1.
        let t = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let e = vec![true; 6];
        let g = SurvivalPlot::new()
            .with_group("A", t.clone(), e.clone())
            .with_group("B", t, e);
        let (chi2, df) = logrank_chi2(&g.groups).unwrap();
        assert_eq!(df, 1);
        assert!(
            chi2 < 0.5,
            "identical groups should give tiny chi2, got {chi2}"
        );
        assert!(chi_square_sf(chi2, df) > 0.3);
    }

    #[test]
    fn logrank_separated_groups_is_significant() {
        // Group A dies early, group B dies late -> large chi2 -> small p.
        let a_t = vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0];
        let b_t = vec![8.0, 8.0, 9.0, 9.0, 10.0, 10.0];
        let g = SurvivalPlot::new()
            .with_group("A", a_t, vec![true; 6])
            .with_group("B", b_t, vec![true; 6]);
        let (chi2, df) = logrank_chi2(&g.groups).unwrap();
        let p = chi_square_sf(chi2, df);
        assert!(
            p < 0.01,
            "well-separated groups should be significant, p={p}"
        );
    }
}
