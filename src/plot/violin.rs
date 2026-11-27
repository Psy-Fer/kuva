use crate::plot::strip::StripStyle;

/// Builder for a violin plot.
///
/// Estimates the probability density of each group using kernel density
/// estimation (KDE) and renders the result as a symmetric shape — wider
/// where data is dense, narrower where it is sparse. Unlike box plots,
/// violins reveal multi-modal and skewed distributions.
///
/// Bandwidth defaults to Silverman's rule-of-thumb. Individual data
/// points can be overlaid as a jittered strip or beeswarm.
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::ViolinPlot;
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let plot = ViolinPlot::new()
///     .with_group("Control", vec![4.1, 5.0, 5.3, 5.8, 6.2, 7.0, 5.5, 4.8])
///     .with_group("Treated", vec![5.5, 6.1, 6.4, 7.2, 7.8, 8.5, 6.9, 7.0])
///     .with_color("steelblue")
///     .with_width(0.8);
///
/// let plots = vec![Plot::Violin(plot)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Control vs. Treated")
///     .with_x_label("Group")
///     .with_y_label("Value");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("violin.svg", svg).unwrap();
/// ```
pub struct ViolinPlot {
    pub groups: Vec<ViolinGroup>,
    pub color: String,
    /// Width as a fraction of the category slot width (default `0.8`).
    /// A value of `1.0` fills the slot edge-to-edge; `0.8` leaves a 10 %
    /// gap on each side.  Equivalent to `1.0 - gap`.
    pub width: f64,
    pub legend_label: Option<String>,
    /// KDE bandwidth. `None` uses Silverman's rule-of-thumb.
    pub bandwidth: Option<f64>,
    /// Number of KDE evaluation points (default `200`).
    pub kde_samples: usize,
    pub group_colors: Option<Vec<String>>,
    pub overlay: Option<StripStyle>,
    pub overlay_color: String,
    pub overlay_size: f64,
    pub overlay_seed: u64,
    pub horizontal: bool,
    /// When `true`, each category renders as a split violin: the regular
    /// `groups` KDE on one half, `split_groups` (paired by index) mirrored on
    /// the other half — e.g. comparing two conditions within each category.
    pub split: bool,
    /// Right-half (vertical) / bottom-half (horizontal) values, paired by
    /// index with `groups`. Only used when `split` is `true`.
    pub split_groups: Vec<ViolinGroup>,
    pub split_color: String,
    pub split_group_colors: Option<Vec<String>>,
    pub split_legend_label: Option<String>,
}

/// A single group (one violin) with a category label and raw values.
pub struct ViolinGroup {
    pub label: String,
    pub values: Vec<f64>,
}

impl Default for ViolinPlot {
    fn default() -> Self {
        Self::new()
    }
}

impl ViolinPlot {
    /// Create a violin plot with default settings.
    ///
    /// Defaults: color `"black"`, width `0.8` (slot fraction), Silverman bandwidth,
    /// 200 KDE evaluation points, no overlay.
    pub fn new() -> Self {
        Self {
            groups: vec![],
            color: "black".into(),
            width: 0.8,
            legend_label: None,
            bandwidth: None,
            kde_samples: 200,
            group_colors: None,
            overlay: None,
            overlay_color: "rgba(0,0,0,0.45)".into(),
            overlay_size: 3.0,
            overlay_seed: 42,
            horizontal: false,
            split: false,
            split_groups: vec![],
            split_color: "tomato".into(),
            split_group_colors: None,
            split_legend_label: None,
        }
    }

    /// Add a group (one violin) with a label and raw values.
    ///
    /// Groups are rendered left-to-right in the order they are added.
    /// More data points produce a smoother, more accurate density estimate.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::ViolinPlot;
    /// let plot = ViolinPlot::new()
    ///     .with_group("A", vec![1.0, 2.5, 3.0, 3.5, 4.0, 5.0])
    ///     .with_group("B", vec![2.0, 3.0, 3.8, 4.2, 4.8, 6.0]);
    /// ```
    pub fn with_group<T, U, I>(mut self, label: T, values: I) -> Self
    where
        T: Into<String>,
        I: IntoIterator<Item = U>,
        U: Into<f64>,
    {
        self.groups.push(ViolinGroup {
            label: label.into(),
            values: values.into_iter().map(|x| x.into()).collect(),
        });
        self
    }

    /// Set the violin fill color (CSS color string, e.g. `"steelblue"`).
    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    /// Set per-group fill colors.
    ///
    /// Colors are matched to groups by position. If the list is shorter than
    /// the number of groups, the uniform color from [`with_color`](Self::with_color)
    /// is used as a fallback.
    pub fn with_group_colors<S, I>(mut self, colors: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.group_colors = Some(colors.into_iter().map(|s| s.into()).collect());
        self
    }

    /// Set the violin width as a fraction of the category slot (default `0.8`).
    ///
    /// `1.0` fills the slot edge-to-edge; `0.8` leaves a 10 % gap on each side.
    /// Equivalent to `with_gap(1.0 - width)`.
    pub fn with_width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// Set the gap between adjacent violins as a fraction of the slot width
    /// (default `0.2`).  Equivalent to `with_width(1.0 - gap)`.
    pub fn with_gap(mut self, gap: f64) -> Self {
        self.width = 1.0 - gap;
        self
    }

    /// Attach a legend label to this violin plot.
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    /// Set the KDE bandwidth manually.
    ///
    /// Bandwidth controls the smoothness of the density estimate. Smaller
    /// values reveal finer structure (but may be noisy); larger values
    /// produce a smoother shape (but may hide modes). When not set,
    /// Silverman's rule-of-thumb is applied automatically — a good
    /// starting point for unimodal, roughly normal data.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::ViolinPlot;
    /// let plot = ViolinPlot::new()
    ///     .with_group("A", vec![1.0, 2.0, 3.0, 4.0, 5.0])
    ///     .with_bandwidth(0.5);  // tighter than the default
    /// ```
    pub fn with_bandwidth(mut self, h: f64) -> Self {
        self.bandwidth = Some(h);
        self
    }

    /// Set the number of points at which the KDE is evaluated (default `200`).
    ///
    /// Higher values produce a smoother curve at the cost of slightly more
    /// computation. The default is adequate for most use cases.
    pub fn with_kde_samples(mut self, n: usize) -> Self {
        self.kde_samples = n;
        self
    }

    /// Overlay individual data points as a jittered strip.
    ///
    /// `jitter` controls the horizontal spread (in data-axis units).
    /// A value of `0.15`–`0.2` is typical. Use a semi-transparent
    /// [`with_overlay_color`](Self::with_overlay_color) so the violin
    /// shape remains visible underneath.
    pub fn with_strip(mut self, jitter: f64) -> Self {
        self.overlay = Some(StripStyle::Strip { jitter });
        self
    }

    /// Overlay individual data points as a beeswarm.
    ///
    /// Points are spread horizontally to avoid overlap, giving a clearer
    /// picture of density than a jittered strip. Works best with smaller
    /// datasets (roughly N < 200 per group).
    ///
    /// A point with no free spot within 1000 point radii of the centre is
    /// drawn on the centre line, overlapping. Layout tries at most
    /// [`MAX_SWARM_CANDIDATE_CHECKS`](crate::render::render_utils::MAX_SWARM_CANDIDATE_CHECKS)
    /// (20,000,000) positions per group, which
    /// ordinary data, even 10,000 densely packed points, stays well under;
    /// the points left after that are drawn on the centre line too, and
    /// rendering adds a `swarm.budget` warning.
    pub fn with_swarm_overlay(mut self) -> Self {
        self.overlay = Some(StripStyle::Swarm);
        self
    }

    /// Set the fill color for overlay points (default `"rgba(0,0,0,0.45)"`).
    ///
    /// A semi-transparent color is strongly recommended so the KDE shape
    /// behind the points remains legible.
    pub fn with_overlay_color<S: Into<String>>(mut self, color: S) -> Self {
        self.overlay_color = color.into();
        self
    }

    /// Set the radius of overlay points in pixels (default `3.0`).
    pub fn with_overlay_size(mut self, size: f64) -> Self {
        self.overlay_size = size;
        self
    }

    /// Render groups along the Y-axis and data values along the X-axis (default `false`).
    pub fn with_horizontal(mut self, h: bool) -> Self {
        self.horizontal = h;
        self
    }

    /// Enable split-violin rendering (default `false`).
    ///
    /// Each category shows a full-width violin split down the middle: the
    /// regular [`.with_group()`](Self::with_group) KDE on one half, and the
    /// matching [`.with_split_group()`](Self::with_split_group) KDE
    /// (paired by index) mirrored on the other half. Typical use: comparing
    /// two conditions (e.g. male/female) within each category without
    /// doubling the number of violins.
    ///
    /// The point/swarm overlay ([`.with_strip()`](Self::with_strip) /
    /// [`.with_swarm_overlay()`](Self::with_swarm_overlay)) is not supported
    /// in split mode and is ignored when `split` is `true`.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::ViolinPlot;
    /// let plot = ViolinPlot::new()
    ///     .with_group("Mon", vec![4.1, 5.0, 5.3, 5.8, 6.2, 7.0])
    ///     .with_split_group(vec![5.5, 6.1, 6.4, 7.2, 7.8, 8.5])
    ///     .with_split(true)
    ///     .with_legend("Male")
    ///     .with_split_legend("Female");
    /// ```
    pub fn with_split(mut self, split: bool) -> Self {
        self.split = split;
        self
    }

    /// Add the "other half" of a split violin, paired by index with the
    /// `groups` added so far via [`.with_group()`](Self::with_group).
    ///
    /// Only rendered when [`.with_split(true)`](Self::with_split) is set.
    pub fn with_split_group<U, I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = U>,
        U: Into<f64>,
    {
        let idx = self.split_groups.len();
        let label = self
            .groups
            .get(idx)
            .map(|g| g.label.clone())
            .unwrap_or_default();
        self.split_groups.push(ViolinGroup {
            label,
            values: values.into_iter().map(|x| x.into()).collect(),
        });
        self
    }

    /// Set the fill color for the split-violin's "other half" (default `"tomato"`).
    pub fn with_split_color<S: Into<String>>(mut self, color: S) -> Self {
        self.split_color = color.into();
        self
    }

    /// Set per-category fill colors for the split-violin's "other half".
    ///
    /// Colors are matched to categories by position, mirroring
    /// [`.with_group_colors()`](Self::with_group_colors) for the primary half.
    pub fn with_split_group_colors<S, I>(mut self, colors: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.split_group_colors = Some(colors.into_iter().map(|s| s.into()).collect());
        self
    }

    /// Attach a legend label for the split-violin's "other half".
    pub fn with_split_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.split_legend_label = Some(label.into());
        self
    }
}
