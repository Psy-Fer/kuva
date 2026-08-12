/// Builder for a histogram.
///
/// Bins a 1-D dataset and renders each bin as a vertical bar. The bin
/// boundaries are computed from the data range (or an explicit range)
/// and the requested bin count.
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::Histogram;
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let data = vec![1.1, 2.3, 2.7, 3.2, 3.8, 3.9, 4.0, 1.5, 2.1, 3.5];
///
/// let hist = Histogram::new()
///     .with_data(data)
///     .with_bins(10)
///     .with_color("steelblue");
///
/// let plots = vec![Plot::Histogram(hist)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Histogram")
///     .with_x_label("Value")
///     .with_y_label("Count");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("histogram.svg", svg).unwrap();
/// ```
/// Automatic bin-count rule, selected via [`Histogram::with_bin_method`].
///
/// When set, the rule computes the bin count from the data and overrides the
/// value passed to [`Histogram::with_bins`]. Rules that need a spread estimate
/// (Scott, Freedman-Diaconis) fall back to Sturges on degenerate data (zero
/// variance or IQR).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinMethod {
    /// Sturges' rule: `ceil(log2(n)) + 1`. Assumes roughly Gaussian data; the
    /// classic default for small samples.
    Sturges,
    /// Scott's rule: bin width `3.49 * sd / n^(1/3)`. Good for smooth data.
    Scott,
    /// Freedman-Diaconis: bin width `2 * IQR / n^(1/3)`. Robust to outliers.
    FreedmanDiaconis,
}

impl BinMethod {
    /// Parse a CLI-friendly name (`sturges`, `scott`, `fd` / `freedman-diaconis`).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().replace('_', "-").as_str() {
            "sturges" => Some(BinMethod::Sturges),
            "scott" => Some(BinMethod::Scott),
            "fd" | "freedman-diaconis" | "freedman" => Some(BinMethod::FreedmanDiaconis),
            _ => None,
        }
    }

    /// Compute the bin count for `data` over `range`. Always returns at least 1.
    pub fn bin_count(&self, data: &[f64], range: (f64, f64)) -> usize {
        let n = data.len();
        let span = range.1 - range.0;
        if n < 2 || !span.is_finite() || span <= 0.0 {
            return 1;
        }
        let sturges = || ((n as f64).log2().ceil() as i64 + 1).max(1) as f64;
        let k = match self {
            BinMethod::Sturges => sturges(),
            BinMethod::Scott => {
                let sd = std_dev(data);
                if sd <= 0.0 {
                    sturges()
                } else {
                    (span / (3.49 * sd / (n as f64).cbrt())).ceil()
                }
            }
            BinMethod::FreedmanDiaconis => {
                let iqr = interquartile_range(data);
                if iqr <= 0.0 {
                    sturges()
                } else {
                    (span / (2.0 * iqr / (n as f64).cbrt())).ceil()
                }
            }
        };
        (k as usize).clamp(1, 10_000)
    }
}

fn std_dev(data: &[f64]) -> f64 {
    let n = data.len() as f64;
    if n < 2.0 {
        return 0.0;
    }
    let mean = data.iter().sum::<f64>() / n;
    let var = data.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0);
    var.sqrt()
}

fn interquartile_range(data: &[f64]) -> f64 {
    let mut v: Vec<f64> = data.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    let pct = |p: f64| -> f64 {
        // Linear-interpolation percentile (matches render_utils::percentile).
        if v.is_empty() {
            return 0.0;
        }
        let rank = p / 100.0 * (v.len() as f64 - 1.0);
        let lo = rank.floor() as usize;
        let hi = rank.ceil() as usize;
        if lo == hi {
            v[lo]
        } else {
            v[lo] + (rank - lo as f64) * (v[hi] - v[lo])
        }
    };
    pct(75.0) - pct(25.0)
}

/// One additional data series stacked (or overlaid) on top of a histogram's
/// primary series. See [`Histogram::with_group`].
#[derive(Debug, Clone)]
pub struct HistGroup {
    pub data: Vec<f64>,
    pub color: String,
    pub label: Option<String>,
}

/// Binned result shared by the renderer and `bounds()` so both agree on bin
/// count, per-layer heights, and the y-extent for every mode. Produced by
/// [`Histogram::compute_bins`].
#[derive(Debug, Clone)]
pub struct BinnedHistogram {
    pub range: (f64, f64),
    pub bin_width: f64,
    pub bins: usize,
    /// One heights vector per layer, bottom first. Weights and the cumulative
    /// transform are already applied; the peak-normalize factor is `norm`.
    pub layers: Vec<Vec<f64>>,
    pub colors: Vec<String>,
    pub labels: Vec<Option<String>>,
    /// Multiply every height by this to apply peak-normalization (1.0 otherwise).
    pub norm: f64,
    /// Whether layers stack (baseline accumulates) or overlay (each from zero).
    pub stacked: bool,
    /// Final y-extent (already includes `norm`): 1.0 when normalized, else the peak.
    pub max_y: f64,
}

#[derive(Debug, Clone)]
pub struct Histogram {
    pub data: Vec<f64>,
    pub bins: usize,
    pub range: Option<(f64, f64)>,
    pub color: String,
    pub normalize: bool,
    pub legend_label: Option<String>,
    pub precomputed: Option<(Vec<f64>, Vec<f64>)>,
    pub show_tooltips: bool,
    pub tooltip_labels: Option<Vec<String>>,
    /// Overlay a Gaussian KDE density curve, scaled to the histogram's own
    /// bar-height units (default `false`). Ignored for precomputed histograms.
    pub show_kde: bool,
    /// KDE stroke color. Defaults to a darker shade when unset.
    pub kde_color: Option<String>,
    /// KDE bandwidth. `None` uses Silverman's rule-of-thumb.
    pub kde_bandwidth: Option<f64>,
    /// Number of KDE evaluation points (default `200`).
    pub kde_samples: usize,
    /// Draw outline-only staircases instead of filled bars (default `false`).
    pub step: bool,
    /// Accumulate counts left-to-right into a cumulative histogram (default `false`).
    pub cumulative: bool,
    /// Stack `groups` on top of the primary series instead of overlaying them
    /// (default `false`; only meaningful when `groups` is non-empty).
    pub stacked: bool,
    /// Per-sample weights for the primary series. Length must match `data`;
    /// otherwise ignored. Groups are always unweighted.
    pub weights: Option<Vec<f64>>,
    /// Automatic bin-count rule; overrides `bins` when set.
    pub bin_method: Option<BinMethod>,
    /// Extra series stacked (or overlaid) on the primary one.
    pub groups: Vec<HistGroup>,
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}

impl Histogram {
    /// Create a histogram with default settings.
    ///
    /// Defaults: 10 bins, color `"black"`, no normalization.
    pub fn new() -> Self {
        Self {
            data: vec![],
            bins: 10,
            range: None,
            color: "black".to_string(),
            normalize: false,
            legend_label: None,
            precomputed: None,
            show_tooltips: false,
            tooltip_labels: None,
            show_kde: false,
            kde_color: None,
            kde_bandwidth: None,
            kde_samples: 200,
            step: false,
            cumulative: false,
            stacked: false,
            weights: None,
            bin_method: None,
            groups: Vec::new(),
        }
    }

    /// Create a histogram from precomputed bin edges and counts.
    ///
    /// `edges` must have length `counts.len() + 1`. Use `f64` counts to support
    /// fractional values (density estimates, normalized inputs from R/numpy).
    /// `range` and `with_data` / `with_bins` are ignored when precomputed bins are set.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::Histogram;
    /// let edges = vec![0.0, 1.0, 2.0, 3.0];
    /// let counts = vec![5.0, 12.0, 8.0];
    /// let hist = Histogram::from_bins(edges, counts).with_color("steelblue");
    /// ```
    pub fn from_bins(edges: Vec<f64>, counts: Vec<f64>) -> Self {
        Self {
            precomputed: Some((edges, counts)),
            ..Self::new()
        }
    }

    /// Set precomputed bin edges and counts via the builder chain.
    ///
    /// Equivalent to `Histogram::from_bins(edges, counts)` but usable when
    /// constructing conditionally after other options are set.
    pub fn with_precomputed(mut self, edges: Vec<f64>, counts: Vec<f64>) -> Self {
        self.precomputed = Some((edges, counts));
        self
    }

    /// Set the input data.
    ///
    /// Accepts any iterator of values implementing `Into<f64>`. Values
    /// outside the active range are silently ignored.
    ///
    /// > **Note:** [`with_range`](Self::with_range) must also be called.
    /// > Without an explicit range, [`Layout::auto_from_plots`](crate::render::layout::Layout::auto_from_plots)
    /// > cannot determine the axis extent and the chart will be empty.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::Histogram;
    /// let data = vec![1.1, 2.3, 2.7, 3.2, 3.8];
    /// let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
    /// let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    /// let hist = Histogram::new()
    ///     .with_data(data)
    ///     .with_range((min, max));
    /// ```
    pub fn with_data<T, I>(mut self, data: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<f64>,
    {
        self.data = data.into_iter().map(|x| x.into()).collect();
        self
    }

    /// Set the number of equal-width bins (default `10`).
    ///
    /// The bin edges span from `range.min` to `range.max`. Choose a
    /// value that balances resolution against noise for your sample size.
    pub fn with_bins(mut self, bins: usize) -> Self {
        self.bins = bins;
        self
    }

    /// Set the bin range — **required** for `Layout::auto_from_plots` to work.
    ///
    /// Without an explicit range, `bounds()` returns `None` and
    /// [`Layout::auto_from_plots`](crate::render::layout::Layout::auto_from_plots)
    /// cannot determine the axis extent, resulting in an empty chart.
    ///
    /// Typically pass the data min/max. For overlapping histograms, pass the
    /// same combined range to both so their x-axes align.
    ///
    /// Values outside the range are silently ignored during binning.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::Histogram;
    /// let data = vec![0.1, 0.5, 1.2, 2.8, 3.0];
    /// let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
    /// let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    /// let hist = Histogram::new()
    ///     .with_data(data)
    ///     .with_range((min, max));
    /// ```
    pub fn with_range(mut self, range: (f64, f64)) -> Self {
        self.range = Some(range);
        self
    }

    /// Set the bar fill color (CSS color string, e.g. `"steelblue"`, `"#4682b4"`).
    ///
    /// For overlapping histograms, use an 8-digit hex color with an alpha
    /// channel (`#RRGGBBAA`) so bars from different series show through:
    ///
    /// ```rust,no_run
    /// # use kuva::plot::Histogram;
    /// let hist = Histogram::new()
    ///     .with_data(vec![1.0, 2.0, 3.0])
    ///     .with_color("#4682b480");  // steelblue at 50% opacity
    /// ```
    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    /// Normalize bar heights so the tallest bar equals `1.0`.
    ///
    /// This is a peak-normalization — not a probability density. The
    /// y-axis represents relative frequency (tallest bin = 1), not
    /// counts or probability per unit width.
    pub fn with_normalize(mut self) -> Self {
        self.normalize = true;
        self
    }

    /// Attach a legend label to this histogram.
    ///
    /// A legend is rendered automatically when at least one plot in the
    /// `Vec<Plot>` has a label.
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    pub fn with_tooltips(mut self) -> Self {
        self.show_tooltips = true;
        self
    }

    pub fn with_tooltip_labels(
        mut self,
        labels: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.tooltip_labels = Some(labels.into_iter().map(|s| s.into()).collect());
        self
    }

    /// Overlay a Gaussian KDE density curve on top of the bars (default `false`).
    ///
    /// The curve is scaled to the histogram's own bar-height units (expected
    /// count per bin, or peak-normalized to `1` when
    /// [`.with_normalize()`](Self::with_normalize) is also set) so it reads
    /// directly against the bars rather than a separate density scale.
    /// Ignored for precomputed histograms ([`from_bins`](Self::from_bins)),
    /// which have no raw samples to estimate a density from.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::Histogram;
    /// let hist = Histogram::new()
    ///     .with_data(vec![1.1, 2.3, 2.7, 3.2, 3.8, 3.9, 4.0, 1.5, 2.1, 3.5])
    ///     .with_bins(8)
    ///     .with_kde(true);
    /// ```
    pub fn with_kde(mut self, show: bool) -> Self {
        self.show_kde = show;
        self
    }

    /// Set the KDE curve's stroke color (default: a darker shade of the bar color).
    pub fn with_kde_color<S: Into<String>>(mut self, color: S) -> Self {
        self.kde_color = Some(color.into());
        self
    }

    /// Set the KDE bandwidth manually. `None` (the default) uses Silverman's rule-of-thumb.
    pub fn with_kde_bandwidth(mut self, h: f64) -> Self {
        self.kde_bandwidth = Some(h);
        self
    }

    /// Set the number of points at which the KDE curve is evaluated (default `200`).
    pub fn with_kde_samples(mut self, n: usize) -> Self {
        self.kde_samples = n;
        self
    }

    /// Draw the histogram as an outline-only staircase instead of filled bars
    /// (matplotlib `histtype='step'`). Clean for overlaying several distributions.
    pub fn with_step(mut self, step: bool) -> Self {
        self.step = step;
        self
    }

    /// Accumulate bin counts left-to-right so each bar includes everything to
    /// its left (matplotlib `cumulative=True`). The final bar equals the total.
    pub fn with_cumulative(mut self, cumulative: bool) -> Self {
        self.cumulative = cumulative;
        self
    }

    /// Stack [`groups`](Self::with_group) on top of the primary series rather
    /// than overlaying them from a shared baseline (matplotlib `stacked=True`).
    pub fn with_stacked(mut self, stacked: bool) -> Self {
        self.stacked = stacked;
        self
    }

    /// Set per-sample weights for the primary series (matplotlib `weights=`).
    /// Each sample contributes its weight to its bin instead of `1`. The vector
    /// length must match the data; a mismatch is ignored (falls back to counts).
    pub fn with_weights(mut self, weights: Vec<f64>) -> Self {
        self.weights = Some(weights);
        self
    }

    /// Choose an automatic bin-count rule; overrides [`with_bins`](Self::with_bins).
    pub fn with_bin_method(mut self, method: BinMethod) -> Self {
        self.bin_method = Some(method);
        self
    }

    /// Add an extra data series, drawn stacked on top of (or, without
    /// [`with_stacked`](Self::with_stacked), overlaid on) the primary series.
    pub fn with_group(
        mut self,
        data: Vec<f64>,
        color: impl Into<String>,
        label: Option<String>,
    ) -> Self {
        self.groups.push(HistGroup {
            data,
            color: color.into(),
            label,
        });
        self
    }

    /// Bin the raw data for the auto-binning (non-precomputed) path, applying
    /// weights, the cumulative transform, stacking, and normalization uniformly.
    /// Returns `None` when there is nothing to bin. Shared by the renderer and
    /// `bounds()` so they never disagree.
    pub fn compute_bins(&self) -> Option<BinnedHistogram> {
        // All samples across the primary series and every group, for range/method.
        let all: Vec<f64> = self
            .data
            .iter()
            .copied()
            .chain(self.groups.iter().flat_map(|g| g.data.iter().copied()))
            .collect();
        if all.is_empty() {
            return None;
        }

        let range = self.range.unwrap_or_else(|| {
            let min = all.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = all.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            (min, max)
        });

        let bins = self
            .bin_method
            .map(|m| m.bin_count(&all, range))
            .unwrap_or(self.bins)
            .max(1);
        let span = range.1 - range.0;
        let bin_width = if span > 0.0 { span / bins as f64 } else { 1.0 };

        // Bin one series into `bins` buckets with optional per-sample weights.
        let bin_series = |data: &[f64], weights: Option<&Vec<f64>>| -> Vec<f64> {
            let mut counts = vec![0.0_f64; bins];
            for (i, &value) in data.iter().enumerate() {
                if value < range.0 || value > range.1 || span <= 0.0 {
                    continue;
                }
                let mut b = ((value - range.0) / bin_width).floor() as usize;
                if b >= bins {
                    b = bins - 1;
                }
                let w = weights.and_then(|w| w.get(i).copied()).unwrap_or(1.0);
                counts[b] += w;
            }
            counts
        };

        let cumsum = |mut c: Vec<f64>| -> Vec<f64> {
            let mut acc = 0.0;
            for v in c.iter_mut() {
                acc += *v;
                *v = acc;
            }
            c
        };

        let weights = self.weights.as_ref().filter(|w| w.len() == self.data.len());

        let mut layers: Vec<Vec<f64>> = Vec::with_capacity(1 + self.groups.len());
        let mut counts = bin_series(&self.data, weights);
        if self.cumulative {
            counts = cumsum(counts);
        }
        layers.push(counts);
        for g in &self.groups {
            let mut c = bin_series(&g.data, None);
            if self.cumulative {
                c = cumsum(c);
            }
            layers.push(c);
        }

        let mut colors = vec![self.color.clone()];
        colors.extend(self.groups.iter().map(|g| g.color.clone()));
        let mut labels = vec![self.legend_label.clone()];
        labels.extend(self.groups.iter().map(|g| g.label.clone()));

        // Peak (pre-normalize): stacked -> tallest per-bin total; else tallest single bar.
        let stacked = self.stacked && !self.groups.is_empty();
        let peak = if stacked {
            (0..bins)
                .map(|b| layers.iter().map(|l| l[b]).sum::<f64>())
                .fold(0.0_f64, f64::max)
        } else {
            layers
                .iter()
                .flat_map(|l| l.iter().copied())
                .fold(0.0_f64, f64::max)
        };
        let norm = if self.normalize && peak > 0.0 {
            1.0 / peak
        } else {
            1.0
        };
        let max_y = if self.normalize { 1.0 } else { peak }.max(0.0);

        Some(BinnedHistogram {
            range,
            bin_width,
            bins,
            layers,
            colors,
            labels,
            norm,
            stacked,
            max_y,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hist(data: Vec<f64>, bins: usize) -> Histogram {
        Histogram::new()
            .with_data(data)
            .with_bins(bins)
            .with_range((0.0, 10.0))
    }

    #[test]
    fn plain_counts() {
        let b = hist(vec![0.5, 1.5, 1.6, 9.9], 10).compute_bins().unwrap();
        assert_eq!(b.bins, 10);
        assert_eq!(b.layers.len(), 1);
        assert_eq!(b.layers[0][0], 1.0); // [0,1)
        assert_eq!(b.layers[0][1], 2.0); // [1,2)
        assert_eq!(b.layers[0][9], 1.0); // last bin catches 9.9
        assert_eq!(b.max_y, 2.0);
        assert_eq!(b.norm, 1.0);
    }

    #[test]
    fn cumulative_is_monotone_and_totals() {
        let b = hist(vec![0.5, 1.5, 1.6, 9.9], 10)
            .with_cumulative(true)
            .compute_bins()
            .unwrap();
        // Non-decreasing across bins.
        for w in b.layers[0].windows(2) {
            assert!(w[1] >= w[0]);
        }
        // Final bin equals the total sample count.
        assert_eq!(*b.layers[0].last().unwrap(), 4.0);
        assert_eq!(b.max_y, 4.0);
    }

    #[test]
    fn weights_scale_bin_heights() {
        let b = hist(vec![1.5, 1.6, 5.5], 10)
            .with_weights(vec![2.0, 3.0, 10.0])
            .compute_bins()
            .unwrap();
        assert_eq!(b.layers[0][1], 5.0); // 2 + 3 in [1,2)
        assert_eq!(b.layers[0][5], 10.0); // weight 10 in [5,6)
        assert_eq!(b.max_y, 10.0);
    }

    #[test]
    fn mismatched_weights_are_ignored() {
        let b = hist(vec![1.5, 1.6], 10)
            .with_weights(vec![2.0]) // wrong length
            .compute_bins()
            .unwrap();
        assert_eq!(b.layers[0][1], 2.0); // falls back to unit counts
    }

    #[test]
    fn normalize_peaks_at_one() {
        let b = hist(vec![1.5, 1.6, 1.7, 5.5], 10)
            .with_normalize()
            .compute_bins()
            .unwrap();
        assert_eq!(b.max_y, 1.0);
        // Tallest raw bin is 3 -> norm scales it to 1.
        assert!((b.layers[0][1] * b.norm - 1.0).abs() < 1e-12);
    }

    #[test]
    fn stacked_totals_bins_across_layers() {
        let b = Histogram::new()
            .with_data(vec![1.5, 1.6])
            .with_bins(10)
            .with_range((0.0, 10.0))
            .with_group(vec![1.7], "#f00", None)
            .with_stacked(true)
            .compute_bins()
            .unwrap();
        assert_eq!(b.layers.len(), 2);
        assert!(b.stacked);
        // Bin [1,2): 2 from primary + 1 from group = 3 total height.
        assert_eq!(b.max_y, 3.0);
    }

    #[test]
    fn groups_without_stacked_flag_do_not_stack() {
        let b = Histogram::new()
            .with_data(vec![1.5, 1.6])
            .with_bins(10)
            .with_range((0.0, 10.0))
            .with_group(vec![1.7], "#f00", None)
            .compute_bins()
            .unwrap();
        assert!(!b.stacked);
        // Peak is the tallest single bar (2), not the sum.
        assert_eq!(b.max_y, 2.0);
    }

    #[test]
    fn sturges_bin_count() {
        // n = 8 -> ceil(log2(8)) + 1 = 3 + 1 = 4.
        let data: Vec<f64> = (0..8).map(|i| i as f64).collect();
        assert_eq!(BinMethod::Sturges.bin_count(&data, (0.0, 7.0)), 4);
    }

    #[test]
    fn bin_method_overrides_bins() {
        let data: Vec<f64> = (0..8).map(|i| i as f64).collect();
        let b = Histogram::new()
            .with_data(data)
            .with_bins(99)
            .with_range((0.0, 7.0))
            .with_bin_method(BinMethod::Sturges)
            .compute_bins()
            .unwrap();
        assert_eq!(b.bins, 4); // Sturges wins over with_bins(99)
    }

    #[test]
    fn bin_method_parses_aliases() {
        assert_eq!(BinMethod::parse("FD"), Some(BinMethod::FreedmanDiaconis));
        assert_eq!(
            BinMethod::parse("freedman-diaconis"),
            Some(BinMethod::FreedmanDiaconis)
        );
        assert_eq!(BinMethod::parse("scott"), Some(BinMethod::Scott));
        assert_eq!(BinMethod::parse("nope"), None);
    }

    #[test]
    fn degenerate_data_gives_one_bin() {
        assert_eq!(BinMethod::Scott.bin_count(&[1.0, 1.0, 1.0], (1.0, 1.0)), 1);
        assert_eq!(BinMethod::FreedmanDiaconis.bin_count(&[], (0.0, 1.0)), 1);
    }
}
