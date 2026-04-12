/// Re-export the shared `ColorMap` type from the histogram2d module.
pub use crate::plot::histogram2d::ColorMap;

/// How to derive the fill color of each treemap cell.
#[derive(Clone, Default)]
pub enum TreemapColorMode {
    /// Each top-level group (root node) gets a distinct category color; its
    /// descendants inherit that color (default).
    #[default]
    ByParent,
    /// Color leaves by their value (or by the parallel `color_values` vector
    /// when present) using the given colormap.  Parent cells are drawn as
    /// neutral `#e0e0e0`.
    ByValue(ColorMap),
    /// Use the `color` field on each `TreemapNode`.  Nodes without an explicit
    /// color fall back to `"#888888"`.
    Explicit,
}

/// Layout algorithm used to partition space among sibling nodes.
#[derive(Debug, Clone, Default)]
pub enum TreemapLayout {
    /// Squarified tiling (Bruls et al. 2000) — minimises worst aspect ratio.
    /// **Default.**
    #[default]
    Squarify,
    /// Alternating horizontal / vertical splits by depth level.  Simple and
    /// fast, but produces thin slivers for unequal values.
    SliceDice,
    /// Balanced binary splits: partition the sorted node list into two halves
    /// that minimise `|left_sum − right_sum|`, then recurse alternating H/V.
    Binary,
}

/// A single node in the treemap hierarchy.
///
/// `value` may be `0.0` (the default) for inner nodes — the renderer will
/// auto-sum it from the node's `children`.  Set `value` explicitly when you
/// want to override the auto-sum (e.g. to reserve extra whitespace).
#[derive(Debug, Clone)]
pub struct TreemapNode {
    /// Display label for the node.
    pub label: String,
    /// Numeric value that determines area.  `0.0` means "auto-sum from children".
    pub value: f64,
    /// Child nodes.  Empty for leaf nodes.
    pub children: Vec<TreemapNode>,
    /// Optional CSS fill color used in [`TreemapColorMode::Explicit`] mode.
    pub color: Option<String>,
}

impl TreemapNode {
    /// Create a leaf node with a given value and no children.
    pub fn leaf(label: impl Into<String>, value: impl Into<f64>) -> Self {
        TreemapNode {
            label: label.into(),
            value: value.into(),
            children: vec![],
            color: None,
        }
    }

    /// Create an inner node whose value is auto-summed from its children.
    pub fn new(label: impl Into<String>, children: Vec<TreemapNode>) -> Self {
        TreemapNode {
            label: label.into(),
            value: 0.0,
            children,
            color: None,
        }
    }

    /// Create an inner node with an explicit value **and** children.
    pub fn with_value(label: impl Into<String>, value: impl Into<f64>, children: Vec<TreemapNode>) -> Self {
        TreemapNode {
            label: label.into(),
            value: value.into(),
            children,
            color: None,
        }
    }

    /// Create a leaf node with an explicit CSS fill color (used in [`TreemapColorMode::Explicit`]).
    pub fn leaf_colored(label: impl Into<String>, value: impl Into<f64>, color: impl Into<String>) -> Self {
        TreemapNode {
            label: label.into(),
            value: value.into(),
            children: vec![],
            color: Some(color.into()),
        }
    }

    /// The effective value used for area calculations.
    ///
    /// If this node has children and `value == 0.0`, the effective value is the
    /// recursive sum of the children's effective values.  Otherwise `value.max(0.0)`.
    pub(crate) fn resolved_value(&self) -> f64 {
        if !self.children.is_empty() && self.value == 0.0 {
            self.children.iter().map(|c| c.resolved_value()).sum()
        } else {
            self.value.max(0.0)
        }
    }

}

/// Builder for a treemap plot.
///
/// A treemap tiles a rectangular area with nested rectangles whose sizes are
/// proportional to node values.  The default squarified layout minimises
/// worst-case aspect ratios.
///
/// # Basic usage
///
/// ```rust,no_run
/// use kuva::plot::treemap::{TreemapPlot, TreemapNode};
/// use kuva::render::plots::Plot;
/// use kuva::render::layout::Layout;
/// use kuva::render::render::render_multiple;
/// use kuva::backend::svg::SvgBackend;
///
/// let root = TreemapNode::new("Languages", vec![
///     TreemapNode::leaf("Rust",   40.0),
///     TreemapNode::leaf("Python", 35.0),
///     TreemapNode::leaf("Go",     25.0),
/// ]);
/// let plot = TreemapPlot::new().with_node(root);
///
/// let plots = vec![Plot::Treemap(plot)];
/// let layout = Layout::auto_from_plots(&plots).with_title("Language usage");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("treemap.svg", svg).unwrap();
/// ```
#[derive(Clone)]
pub struct TreemapPlot {
    /// Top-level root nodes (forests are supported — multiple roots).
    pub roots: Vec<TreemapNode>,
    /// Optional parallel vector of scalar values for each **leaf** (depth-first
    /// order) used when `color_mode == ByValue(_)`.  When present, the area of
    /// each leaf is still determined by its `value`, but its color is driven by
    /// the corresponding entry here (e.g. p-value independent of gene count).
    pub color_values: Option<Vec<f64>>,
    /// How to derive cell fill colors.  Default: [`TreemapColorMode::ByParent`].
    pub color_mode: TreemapColorMode,
    /// Layout algorithm.  Default: [`TreemapLayout::Squarify`].
    pub layout_algo: TreemapLayout,
    /// Show leaf labels.  Default: `true`.
    pub show_labels: bool,
    /// Show parent / group labels.  Default: `true`.
    pub show_parent_labels: bool,
    /// Suppress the cell label when the cell area in pixels² is below this
    /// threshold.  Default: `1200.0`.
    pub min_label_area: f64,
    /// Padding in pixels between a parent's border and its children.
    /// Decreases with depth (see `effective_padding`).  Default: `4.0`.
    pub padding: f64,
    /// Border width (px) for leaf and non-root inner cells.  Default: `0.5`.
    pub border_width: f64,
    /// Border width (px) for top-level root cells.  Default: `2.0`.
    pub root_border_width: f64,
    /// Clamp the colorbar scale to `(lo, hi)` — used in `ByValue` mode.
    pub color_range: Option<(f64, f64)>,
    /// Show a colorbar in `ByValue` mode.  Default: `false` (auto-enabled when
    /// the mode is set via `.with_color_mode()`).
    pub show_colorbar: bool,
    /// Override the auto-derived colorbar label.
    pub colorbar_label: Option<String>,
    /// Limit render depth (root = 0).  `None` = unlimited.
    pub max_depth: Option<usize>,
    /// Emit SVG `<title>` tooltip on each cell showing the full path and value.
    /// Default: `true`.
    pub show_tooltips: bool,
}

impl Default for TreemapPlot {
    fn default() -> Self { Self::new() }
}

impl TreemapPlot {
    /// Create a new `TreemapPlot` with default settings.
    pub fn new() -> Self {
        TreemapPlot {
            roots: vec![],
            color_values: None,
            color_mode: TreemapColorMode::ByParent,
            layout_algo: TreemapLayout::Squarify,
            show_labels: true,
            show_parent_labels: true,
            min_label_area: 1200.0,
            padding: 4.0,
            border_width: 0.5,
            root_border_width: 2.0,
            color_range: None,
            show_colorbar: false,
            colorbar_label: None,
            max_depth: None,
            show_tooltips: true,
        }
    }

    /// Add a root node to the treemap.
    pub fn with_node(mut self, node: TreemapNode) -> Self {
        self.roots.push(node);
        self
    }

    /// Convenience: add a parent node with given children.
    pub fn with_children(mut self, label: impl Into<String>, children: Vec<TreemapNode>) -> Self {
        self.roots.push(TreemapNode::new(label, children));
        self
    }

    /// Set the color mode.
    ///
    /// When `ByValue(_)` is set, `show_colorbar` is automatically enabled.
    pub fn with_color_mode(mut self, mode: TreemapColorMode) -> Self {
        if matches!(mode, TreemapColorMode::ByValue(_)) {
            self.show_colorbar = true;
        }
        self.color_mode = mode;
        self
    }

    /// Supply a parallel `color_values` vector for `ByValue` coloring.
    ///
    /// Values must be in leaf depth-first order.  If the length mismatches the
    /// leaf count, extras are silently ignored and missing entries use `None`.
    pub fn with_color_values(mut self, vals: impl IntoIterator<Item = impl Into<f64>>) -> Self {
        self.color_values = Some(vals.into_iter().map(|v| v.into()).collect());
        self
    }

    /// Choose the layout algorithm.
    pub fn with_layout(mut self, algo: TreemapLayout) -> Self {
        self.layout_algo = algo;
        self
    }

    /// Set padding (px) between parent borders and children.
    pub fn with_padding(mut self, px: f64) -> Self {
        self.padding = px;
        self
    }

    /// Set border width (px) for leaf and non-root inner cells.
    pub fn with_border_width(mut self, px: f64) -> Self {
        self.border_width = px;
        self
    }

    /// Set border width (px) for top-level root cells.
    pub fn with_root_border_width(mut self, px: f64) -> Self {
        self.root_border_width = px;
        self
    }

    /// Minimum cell area (px²) for a label to be rendered.
    pub fn with_min_label_area(mut self, area: f64) -> Self {
        self.min_label_area = area;
        self
    }

    /// Show / hide leaf labels.
    pub fn with_show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    /// Show / hide parent / group labels.
    pub fn with_show_parent_labels(mut self, show: bool) -> Self {
        self.show_parent_labels = show;
        self
    }

    /// Show / hide the colorbar (only visible in `ByValue` mode).
    pub fn with_colorbar(mut self, show: bool) -> Self {
        self.show_colorbar = show;
        self
    }

    /// Override the colorbar label.
    pub fn with_colorbar_label(mut self, label: impl Into<String>) -> Self {
        self.colorbar_label = Some(label.into());
        self
    }

    /// Clamp the colorbar scale to a fixed `[lo, hi]` interval.
    pub fn with_color_range(mut self, lo: f64, hi: f64) -> Self {
        self.color_range = Some((lo, hi));
        self
    }

    /// Limit how many depth levels are rendered.
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    /// Enable / disable SVG hover tooltips on each cell.
    pub fn with_tooltips(mut self, show: bool) -> Self {
        self.show_tooltips = show;
        self
    }

    /// Bioinformatics convenience builder for GO enrichment results.
    ///
    /// Accepts an iterator of `(label, description, gene_count, p_value)` tuples.
    /// Each entry becomes a leaf node with `value = gene_count`.  The `p_value`
    /// values are stored in `color_values`, and `color_mode` is set to
    /// `ByValue(ColorMap::Viridis)` so larger (worse) p-values map to the
    /// brighter end of the colormap.
    ///
    /// Call `.with_colorbar_label("p-value")` after this to label the colorbar.
    pub fn with_go_terms(
        mut self,
        terms: impl IntoIterator<Item = (impl Into<String>, impl Into<String>, impl Into<f64>, impl Into<f64>)>,
    ) -> Self {
        let mut leaves = Vec::new();
        let mut pvalues = Vec::new();
        for (term_id, description, count, pvalue) in terms {
            let label = format!("{} — {}", term_id.into(), description.into());
            let cnt: f64 = count.into();
            let pv: f64 = pvalue.into();
            leaves.push(TreemapNode::leaf(label, cnt));
            pvalues.push(pv);
        }
        self.roots.extend(leaves);
        self.color_values = Some(pvalues);
        self.color_mode = TreemapColorMode::ByValue(ColorMap::Viridis);
        self.show_colorbar = true;
        self
    }

    /// Count total number of nodes (for `estimated_primitives`).
    pub(crate) fn node_count(&self) -> usize {
        fn count(nodes: &[TreemapNode]) -> usize {
            nodes.iter().map(|n| 1 + count(&n.children)).sum()
        }
        count(&self.roots)
    }
}
