/// A raincloud plot combines a half-violin (cloud), box-and-whisker (box),
/// and jittered raw points (rain) for each group.
///
/// Based on Allen et al. 2019 — a more transparent visualisation of raw data,
/// distribution shape, and summary statistics simultaneously.
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::RaincloudPlot;
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let plot = RaincloudPlot::new()
///     .with_group("Control", vec![4.1, 5.0, 5.3, 5.8, 6.2, 7.0, 5.5, 4.8])
///     .with_group("Treated", vec![5.5, 6.1, 6.4, 7.2, 7.8, 8.5, 6.9, 7.0]);
///
/// let plots = vec![Plot::Raincloud(plot)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Raincloud Plot")
///     .with_x_label("Group")
///     .with_y_label("Value");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("raincloud.svg", svg).unwrap();
/// ```
#[derive(Debug)]
pub struct RaincloudPlot {
    pub groups: Vec<RaincloudGroup>,

    // Color
    pub color: String,
    pub group_colors: Option<Vec<String>>,

    // Cloud (half-violin)
    pub cloud_width: f64,
    pub bandwidth: Option<f64>,
    /// Multiplicative scale applied to the Silverman bandwidth (default `1.0`).
    /// Values < 1.0 produce a sharper, more data-sensitive curve; > 1.0 smoother.
    /// Ignored when `bandwidth` is set explicitly.
    pub bandwidth_scale: f64,
    pub kde_samples: usize,
    pub cloud_alpha: f64,
    pub show_cloud: bool,

    // Box
    pub box_width: f64,
    pub show_box: bool,

    // Rain
    pub rain_size: f64,
    pub rain_jitter: f64,
    pub rain_alpha: f64,
    pub show_rain: bool,

    // Layout
    pub flip: bool,
    pub rain_offset: f64,
    pub cloud_offset: f64,

    // Misc
    pub seed: u64,
    pub legend_label: Option<String>,
}

/// A single group (one cloud+box+rain set) with a category label and raw values.
#[derive(Debug)]
pub struct RaincloudGroup {
    pub label: String,
    pub values: Vec<f64>,
}

impl Default for RaincloudPlot {
    fn default() -> Self {
        Self {
            groups: Vec::new(),
            color: "steelblue".to_string(),
            group_colors: None,
            cloud_width: 30.0,
            bandwidth: None,
            bandwidth_scale: 1.0,
            kde_samples: 200,
            cloud_alpha: 0.7,
            show_cloud: true,
            box_width: 0.08,
            show_box: true,
            rain_size: 3.0,
            rain_jitter: 0.05,
            rain_alpha: 0.7,
            show_rain: true,
            flip: false,
            rain_offset: 0.20,
            cloud_offset: 0.15,
            seed: 42,
            legend_label: None,
        }
    }
}

impl RaincloudPlot {
    /// Create a raincloud plot with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a group (one cloud/box/rain triplet) with a label and raw values.
    ///
    /// Groups are rendered left-to-right in the order added.
    pub fn with_group(mut self, label: impl Into<String>, values: Vec<f64>) -> Self {
        self.groups.push(RaincloudGroup { label: label.into(), values });
        self
    }

    /// Add multiple groups at once from an iterator of `(label, values)` pairs.
    pub fn with_groups(
        mut self,
        groups: impl IntoIterator<Item = (impl Into<String>, Vec<f64>)>,
    ) -> Self {
        for (label, values) in groups {
            self.groups.push(RaincloudGroup { label: label.into(), values });
        }
        self
    }

    /// Set the uniform fill color (CSS color string, e.g. `"steelblue"`).
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = color.into();
        self
    }

    /// Set per-group fill colors matched by position.
    pub fn with_group_colors(
        mut self,
        colors: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.group_colors = Some(colors.into_iter().map(|c| c.into()).collect());
        self
    }

    /// Set the maximum pixel half-width of the cloud (half-violin) shape (default `30.0`).
    pub fn with_cloud_width(mut self, w: f64) -> Self {
        self.cloud_width = w;
        self
    }

    /// Set the KDE bandwidth manually (default: Silverman's rule-of-thumb).
    pub fn with_bandwidth(mut self, bw: f64) -> Self {
        self.bandwidth = Some(bw);
        self
    }

    /// Scale the auto-computed Silverman bandwidth by a multiplier (default `1.0`).
    ///
    /// Equivalent to ggplot2's `adjust` parameter. Values below `1.0` produce a
    /// sharper, more data-sensitive cloud; values above `1.0` produce a smoother
    /// cloud. Has no effect when an explicit bandwidth is set via `with_bandwidth`.
    pub fn with_bandwidth_scale(mut self, scale: f64) -> Self {
        self.bandwidth_scale = scale;
        self
    }

    /// Set the number of KDE evaluation points (default `200`).
    pub fn with_kde_samples(mut self, n: usize) -> Self {
        self.kde_samples = n;
        self
    }

    /// Set the cloud fill opacity (default `0.7`).
    pub fn with_cloud_alpha(mut self, a: f64) -> Self {
        self.cloud_alpha = a;
        self
    }

    /// Show or hide the cloud (half-violin) element (default `true`).
    pub fn with_cloud(mut self, show: bool) -> Self {
        self.show_cloud = show;
        self
    }

    /// Set the box half-width as a fraction of the slot width (default `0.08`).
    pub fn with_box_width(mut self, w: f64) -> Self {
        self.box_width = w;
        self
    }

    /// Show or hide the box-and-whisker element (default `true`).
    pub fn with_box(mut self, show: bool) -> Self {
        self.show_box = show;
        self
    }

    /// Set the radius of rain (jitter) points in pixels (default `3.0`).
    pub fn with_rain_size(mut self, s: f64) -> Self {
        self.rain_size = s;
        self
    }

    /// Set the horizontal jitter spread in data-axis units (default `0.05`).
    pub fn with_rain_jitter(mut self, j: f64) -> Self {
        self.rain_jitter = j;
        self
    }

    /// Set the rain point fill opacity (default `0.7`).
    pub fn with_rain_alpha(mut self, a: f64) -> Self {
        self.rain_alpha = a;
        self
    }

    /// Show or hide the rain (jitter points) element (default `true`).
    pub fn with_rain(mut self, show: bool) -> Self {
        self.show_rain = show;
        self
    }

    /// Flip the direction of cloud and rain (default `false` — cloud on the right, rain on the left).
    pub fn with_flip(mut self, flip: bool) -> Self {
        self.flip = flip;
        self
    }

    /// Set the data-axis offset of rain points from the group centre (default `0.20`).
    pub fn with_rain_offset(mut self, offset: f64) -> Self {
        self.rain_offset = offset;
        self
    }

    /// Set the data-axis offset of the cloud centre from the group centre (default `0.15`).
    pub fn with_cloud_offset(mut self, offset: f64) -> Self {
        self.cloud_offset = offset;
        self
    }

    /// Set the random seed for reproducible jitter (default `42`).
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Attach a legend label to this plot.
    pub fn with_legend(mut self, label: impl Into<String>) -> Self {
        self.legend_label = Some(label.into());
        self
    }
}
