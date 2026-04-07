use crate::plot::scatter::{ScatterPlot, MarkerShape, TrendLine};

/// Marginal distribution style for `JointPlot`.
#[derive(Clone, Debug, Default)]
pub enum MarginalType {
    /// Histogram bars. **Default.**
    #[default]
    Histogram,
    /// Kernel density estimate curve (filled area).
    Density,
}

/// One data group within a `JointPlot`.
///
/// Wraps a [`ScatterPlot`] so every scatter feature (error bars, trend lines,
/// confidence bands, per-point colors/sizes, marker shapes, tooltips, …) is
/// available. The x/y values are accessed via [`x_values`](Self::x_values) /
/// [`y_values`](Self::y_values) for marginal computation.
#[derive(Clone, Debug)]
pub struct JointGroup {
    pub scatter: ScatterPlot,
}

impl JointGroup {
    /// Create a group from x and y data iterables.
    pub fn new(
        x: impl IntoIterator<Item = impl Into<f64>>,
        y: impl IntoIterator<Item = impl Into<f64>>,
    ) -> Self {
        let xv: Vec<f64> = x.into_iter().map(|v| v.into()).collect();
        let yv: Vec<f64> = y.into_iter().map(|v| v.into()).collect();
        let scatter = ScatterPlot::new()
            .with_data(xv.iter().copied().zip(yv.iter().copied()));
        Self { scatter }
    }

    /// Create a group from a pre-built [`ScatterPlot`].
    pub fn from_scatter(scatter: ScatterPlot) -> Self {
        Self { scatter }
    }

    /// X values (for marginal histogram/density computation).
    pub fn x_values(&self) -> Vec<f64> {
        self.scatter.data.iter().map(|p| p.x).collect()
    }

    /// Y values (for marginal histogram/density computation).
    pub fn y_values(&self) -> Vec<f64> {
        self.scatter.data.iter().map(|p| p.y).collect()
    }

    // ── Forwarding builders (delegate to ScatterPlot) ──────────────────────

    /// Set the group label (shown in the legend).
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.scatter = self.scatter.with_legend(label);
        self
    }

    /// Set the marker color.
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.scatter = self.scatter.with_color(color);
        self
    }

    /// Set the marker shape.
    pub fn with_marker(mut self, marker: MarkerShape) -> Self {
        self.scatter = self.scatter.with_marker(marker);
        self
    }

    /// Set a uniform marker size (radius in pixels).
    pub fn with_marker_size(mut self, size: f64) -> Self {
        self.scatter = self.scatter.with_size(size);
        self
    }

    /// Set the marker fill opacity (0.0–1.0).
    pub fn with_marker_opacity(mut self, opacity: f64) -> Self {
        self.scatter = self.scatter.with_marker_opacity(opacity);
        self
    }

    /// Set the marker stroke (outline) width in pixels.
    pub fn with_marker_stroke_width(mut self, width: f64) -> Self {
        self.scatter = self.scatter.with_marker_stroke_width(width);
        self
    }

    /// Set per-point sizes. Must have the same length as the data.
    pub fn with_sizes(
        mut self,
        sizes: impl IntoIterator<Item = impl Into<f64>>,
    ) -> Self {
        self.scatter = self.scatter.with_sizes(sizes);
        self
    }

    /// Set per-point colors. Must have the same length as the data.
    pub fn with_colors(
        mut self,
        colors: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.scatter = self.scatter.with_colors(colors);
        self
    }

    /// Add symmetric X error bars (±value).
    pub fn with_x_err(
        mut self,
        errors: impl IntoIterator<Item = impl Into<f64> + Copy>,
    ) -> Self {
        self.scatter = self.scatter.with_x_err(errors);
        self
    }

    /// Add asymmetric X error bars `(negative_arm, positive_arm)`.
    pub fn with_x_err_asymmetric(
        mut self,
        errors: impl IntoIterator<Item = (impl Into<f64>, impl Into<f64>)>,
    ) -> Self {
        self.scatter = self.scatter.with_x_err_asymmetric(errors);
        self
    }

    /// Add symmetric Y error bars (±value).
    pub fn with_y_err(
        mut self,
        errors: impl IntoIterator<Item = impl Into<f64> + Copy>,
    ) -> Self {
        self.scatter = self.scatter.with_y_err(errors);
        self
    }

    /// Add asymmetric Y error bars `(negative_arm, positive_arm)`.
    pub fn with_y_err_asymmetric(
        mut self,
        errors: impl IntoIterator<Item = (impl Into<f64>, impl Into<f64>)>,
    ) -> Self {
        self.scatter = self.scatter.with_y_err_asymmetric(errors);
        self
    }

    /// Add a trend line (`TrendLine::Linear`).
    pub fn with_trend(mut self, trend: TrendLine) -> Self {
        self.scatter = self.scatter.with_trend(trend);
        self
    }

    /// Set the trend line color.
    pub fn with_trend_color(mut self, color: impl Into<String>) -> Self {
        self.scatter = self.scatter.with_trend_color(color);
        self
    }

    /// Show the regression equation on the plot.
    pub fn with_equation(mut self) -> Self {
        self.scatter = self.scatter.with_equation();
        self
    }

    /// Show the Pearson correlation coefficient on the plot.
    pub fn with_correlation(mut self) -> Self {
        self.scatter = self.scatter.with_correlation();
        self
    }

    /// Set the trend line stroke width.
    pub fn with_trend_width(mut self, width: f64) -> Self {
        self.scatter = self.scatter.with_trend_width(width);
        self
    }

    /// Add a confidence band. `y_lower` and `y_upper` must align with the data points.
    pub fn with_band(
        mut self,
        y_lower: impl IntoIterator<Item = impl Into<f64>>,
        y_upper: impl IntoIterator<Item = impl Into<f64>>,
    ) -> Self {
        self.scatter = self.scatter.with_band(y_lower, y_upper);
        self
    }

    /// Enable SVG tooltip overlays showing `(x, y)` on hover.
    pub fn with_tooltips(mut self) -> Self {
        self.scatter = self.scatter.with_tooltips();
        self
    }

    /// Set custom per-point tooltip labels.
    pub fn with_tooltip_labels(
        mut self,
        labels: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.scatter = self.scatter.with_tooltip_labels(labels);
        self
    }
}

/// A scatter plot with marginal distribution panels on the top and/or right edges.
///
/// Render with [`render_jointplot`](crate::render::render::render_jointplot).
///
/// # Example
/// ```rust,no_run
/// use kuva::prelude::*;
/// let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let y = vec![2.0, 3.5, 2.5, 4.0, 3.0];
/// let joint = JointPlot::new()
///     .with_xy(x, y)
///     .with_x_label("Feature A")
///     .with_y_label("Feature B");
/// let layout = Layout::new((0.5, 5.5), (1.5, 4.5)).with_title("Joint Plot");
/// let scene = render_jointplot(joint, layout);
/// ```
#[derive(Clone, Debug)]
pub struct JointPlot {
    /// Data groups (one or more). Each wraps a full `ScatterPlot`.
    pub groups: Vec<JointGroup>,
    /// Marginal distribution style (Histogram or Density). Default: `Histogram`.
    pub marginal_type: MarginalType,
    /// Show a marginal panel above the scatter plot. Default: `true`.
    pub show_top: bool,
    /// Show a marginal panel to the right of the scatter plot. Default: `true`.
    pub show_right: bool,
    /// Pixel height (top) or width (right) of each marginal panel. Default: `80.0`.
    pub marginal_size: f64,
    /// Gap in pixels between the marginal panel and the scatter plot. Default: `4.0`.
    pub marginal_gap: f64,
    /// Number of histogram bins. Default: `20`.
    pub bins: usize,
    /// KDE bandwidth. `None` = Silverman's rule of thumb.
    pub bandwidth: Option<f64>,
    /// Opacity of marginal bars/fill. Default: `0.6`.
    pub marginal_alpha: f64,
    /// X-axis label.
    pub x_label: Option<String>,
    /// Y-axis label.
    pub y_label: Option<String>,
    /// Default scatter marker radius applied by `with_xy` / `with_group`. Default: `4.0`.
    pub marker_size: f64,
    /// Default scatter marker opacity applied by `with_xy` / `with_group`. Default: `0.8`.
    pub marker_opacity: f64,
}

impl Default for JointPlot {
    fn default() -> Self { Self::new() }
}

impl JointPlot {
    /// Create a `JointPlot` with default settings.
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            marginal_type: MarginalType::Histogram,
            show_top: true,
            show_right: true,
            marginal_size: 80.0,
            marginal_gap: 4.0,
            bins: 20,
            bandwidth: None,
            marginal_alpha: 0.6,
            x_label: None,
            y_label: None,
            marker_size: 4.0,
            marker_opacity: 0.8,
        }
    }

    /// Add a single (unlabeled) data group. Applies `marker_size` and `marker_opacity` defaults.
    pub fn with_xy(
        mut self,
        x: impl IntoIterator<Item = impl Into<f64>>,
        y: impl IntoIterator<Item = impl Into<f64>>,
    ) -> Self {
        let g = JointGroup::new(x, y)
            .with_marker_size(self.marker_size)
            .with_marker_opacity(self.marker_opacity);
        self.groups.push(g);
        self
    }

    /// Add a named and colored data group. Applies `marker_size` and `marker_opacity` defaults.
    pub fn with_group(
        mut self,
        label: impl Into<String>,
        x: impl IntoIterator<Item = impl Into<f64>>,
        y: impl IntoIterator<Item = impl Into<f64>>,
        color: impl Into<String>,
    ) -> Self {
        let g = JointGroup::new(x, y)
            .with_label(label)
            .with_color(color)
            .with_marker_size(self.marker_size)
            .with_marker_opacity(self.marker_opacity);
        self.groups.push(g);
        self
    }

    /// Add a fully-configured [`JointGroup`] (maximum control).
    ///
    /// Use this to pass a group with error bars, trend lines, per-point colors, etc.
    pub fn with_joint_group(mut self, group: JointGroup) -> Self {
        self.groups.push(group);
        self
    }

    /// Set the marginal distribution style.
    pub fn with_marginal_type(mut self, t: MarginalType) -> Self { self.marginal_type = t; self }
    /// Show/hide the top marginal panel (default `true`).
    pub fn with_top_marginal(mut self, v: bool) -> Self { self.show_top = v; self }
    /// Show/hide the right marginal panel (default `true`).
    pub fn with_right_marginal(mut self, v: bool) -> Self { self.show_right = v; self }
    /// Set marginal panel size in pixels (height for top, width for right). Default `80.0`.
    pub fn with_marginal_size(mut self, s: f64) -> Self { self.marginal_size = s; self }
    /// Set the gap in pixels between each marginal panel and the scatter. Default `4.0`.
    pub fn with_marginal_gap(mut self, g: f64) -> Self { self.marginal_gap = g; self }
    /// Set the number of histogram bins. Default `20`.
    pub fn with_bins(mut self, n: usize) -> Self { self.bins = n; self }
    /// Set the KDE bandwidth (`None` = Silverman's rule of thumb).
    pub fn with_bandwidth(mut self, bw: f64) -> Self { self.bandwidth = Some(bw); self }
    /// Set the opacity of marginal bars/density fill. Default `0.6`.
    pub fn with_marginal_alpha(mut self, a: f64) -> Self { self.marginal_alpha = a; self }
    /// Set the x-axis label.
    pub fn with_x_label(mut self, s: impl Into<String>) -> Self { self.x_label = Some(s.into()); self }
    /// Set the y-axis label.
    pub fn with_y_label(mut self, s: impl Into<String>) -> Self { self.y_label = Some(s.into()); self }
    /// Set the default scatter marker radius applied by `with_xy`/`with_group`. Default `4.0`.
    pub fn with_marker_size(mut self, r: f64) -> Self { self.marker_size = r; self }
    /// Set the default scatter marker opacity applied by `with_xy`/`with_group`. Default `0.8`.
    pub fn with_marker_opacity(mut self, a: f64) -> Self { self.marker_opacity = a; self }

    /// Returns `(x_min, x_max)` across all groups. Returns `(0.0, 1.0)` if no data.
    pub fn x_range(&self) -> (f64, f64) {
        let all_x: Vec<f64> = self.groups.iter()
            .flat_map(|g| g.x_values())
            .collect();
        if all_x.is_empty() { return (0.0, 1.0); }
        let min = all_x.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = all_x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if (max - min).abs() < 1e-12 { (min - 1.0, max + 1.0) } else { (min, max) }
    }

    /// Returns `(y_min, y_max)` across all groups. Returns `(0.0, 1.0)` if no data.
    pub fn y_range(&self) -> (f64, f64) {
        let all_y: Vec<f64> = self.groups.iter()
            .flat_map(|g| g.y_values())
            .collect();
        if all_y.is_empty() { return (0.0, 1.0); }
        let min = all_y.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = all_y.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if (max - min).abs() < 1e-12 { (min - 1.0, max + 1.0) } else { (min, max) }
    }
}
