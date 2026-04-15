use crate::render::layout::TickFormat;
use crate::render::palette::Palette;

/// How ribbon colors are assigned in a Sankey diagram.
#[derive(Debug, Clone)]
pub enum SankeyLinkColor {
    /// Ribbon inherits source node color (default).
    Source,
    /// SVG linearGradient from source to target color.
    Gradient,
    /// Use `SankeyLink.color` field per link.
    PerLink,
}

/// How nodes are ordered within each Sankey column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SankeyNodeOrder {
    /// Preserve insertion order within each column.
    Input,
    /// Reduce weighted link crossings using a wompwomp-style TSP cycle and
    /// Fenwick-scored objective evaluation.
    CrossingReduction,
    /// Use wompwomp's neighbornet backend for node/column cycle generation.
    Neighbornet,
}

/// How Sankey node colors are assigned when no explicit per-node colors exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SankeyNodeColoring {
    /// Reuse one palette color per visible label.
    Label,
    /// Match wompwomp's left-to-right parent-propagation rule.
    Left,
}

/// A node in a Sankey diagram.
#[derive(Debug, Clone)]
pub struct SankeyNode {
    pub id: String,
    pub label: String,
    pub color: Option<String>,
    pub column: Option<usize>,
}

/// A directed flow link between two nodes.
#[derive(Debug, Clone)]
pub struct SankeyLink {
    /// Index into the nodes vec.
    pub source: usize,
    /// Index into the nodes vec.
    pub target: usize,
    pub value: f64,
    /// Used when `SankeyLinkColor::PerLink`.
    pub color: Option<String>,
}

/// A weighted alluvium spanning multiple ordered axes.
#[derive(Debug, Clone)]
pub struct SankeyAlluvium {
    pub nodes: Vec<usize>,
    pub value: f64,
}

/// A Sankey diagram: nodes arranged in columns, connected by tapered ribbons.
#[derive(Debug, Clone)]
pub struct SankeyPlot {
    pub nodes: Vec<SankeyNode>,
    pub links: Vec<SankeyLink>,
    pub alluvia: Vec<SankeyAlluvium>,
    pub axis_names: Option<Vec<String>>,
    pub link_color: SankeyLinkColor,
    pub node_order: SankeyNodeOrder,
    pub node_coloring: SankeyNodeColoring,
    pub node_order_seed: u64,
    pub palette: Option<Vec<String>>,
    pub left_color_cutoff: f64,
    /// Ribbon fill opacity (default 0.5).
    pub link_opacity: f64,
    /// Node rectangle width in pixels (default 20.0).
    pub node_width: f64,
    /// Minimum gap between nodes in a column in pixels (default 8.0).
    pub node_gap: f64,
    /// If set, adds one legend entry per node.
    pub legend_label: Option<String>,
    /// Show the absolute flow value on each ribbon (default false).
    pub flow_labels: bool,
    /// Show each flow as a percentage of its source node's total outflow (default false).
    /// Takes priority over `flow_labels` when both are set.
    pub flow_percent: bool,
    /// Number format for absolute flow labels (default `Auto`).
    pub flow_label_format: TickFormat,
    /// Optional unit suffix appended to absolute labels, e.g. `"reads"` → `"1 200 reads"`.
    pub flow_label_unit: Option<String>,
    /// Minimum ribbon height in pixels required to render a label.
    /// Set to `0.0` to always show labels regardless of ribbon size (default `8.0`).
    pub flow_label_min_height: f64,
}

impl Default for SankeyPlot {
    fn default() -> Self {
        Self::new()
    }
}

impl SankeyPlot {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            links: vec![],
            alluvia: vec![],
            axis_names: None,
            link_color: SankeyLinkColor::Source,
            node_order: SankeyNodeOrder::Input,
            node_coloring: SankeyNodeColoring::Label,
            node_order_seed: 42,
            palette: None,
            left_color_cutoff: 0.5,
            link_opacity: 0.5,
            node_width: 20.0,
            node_gap: 8.0,
            legend_label: None,
            flow_labels: false,
            flow_percent: false,
            flow_label_format: TickFormat::Auto,
            flow_label_unit: None,
            flow_label_min_height: 8.0,
        }
    }

    /// Find an existing node by internal id, or insert a new one and return its index.
    fn node_index_internal(&mut self, id: &str, label: &str, column: Option<usize>) -> usize {
        if let Some(idx) = self.nodes.iter().position(|n| n.id == id) {
            return idx;
        }
        let idx = self.nodes.len();
        self.nodes.push(SankeyNode {
            id: id.to_string(),
            label: label.to_string(),
            color: None,
            column,
        });
        idx
    }

    /// Declare a node explicitly (no-op if it already exists).
    pub fn with_node<S: Into<String>>(mut self, label: S) -> Self {
        let label = label.into();
        self.node_index_internal(&label, &label, None);
        self
    }

    /// Set the color for a node, creating it if absent.
    pub fn with_node_color<S: Into<String>, C: Into<String>>(mut self, label: S, color: C) -> Self {
        let label = label.into();
        let idx = self.node_index_internal(&label, &label, None);
        self.nodes[idx].color = Some(color.into());
        self
    }

    /// Pin a node to a specific column, creating it if absent.
    pub fn with_node_column<S: Into<String>>(mut self, label: S, col: usize) -> Self {
        let label = label.into();
        let idx = self.node_index_internal(&label, &label, Some(col));
        self.nodes[idx].column = Some(col);
        self
    }

    /// Add a link, auto-creating nodes by label if needed.
    pub fn with_link<S: Into<String>>(mut self, source: S, target: S, value: f64) -> Self {
        let src_label = source.into();
        let tgt_label = target.into();
        let src = self.node_index_internal(&src_label, &src_label, None);
        let tgt = self.node_index_internal(&tgt_label, &tgt_label, None);
        self.links.push(SankeyLink {
            source: src,
            target: tgt,
            value,
            color: None,
        });
        self
    }

    /// Add a link with an explicit per-link color.
    pub fn with_link_colored<S: Into<String>, C: Into<String>>(
        mut self,
        source: S,
        target: S,
        value: f64,
        color: C,
    ) -> Self {
        let src_label = source.into();
        let tgt_label = target.into();
        let src = self.node_index_internal(&src_label, &src_label, None);
        let tgt = self.node_index_internal(&tgt_label, &tgt_label, None);
        self.links.push(SankeyLink {
            source: src,
            target: tgt,
            value,
            color: Some(color.into()),
        });
        self
    }

    /// Bulk add links from an iterator of `(source_label, target_label, value)`.
    pub fn with_links<S, I>(mut self, links: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = (S, S, f64)>,
    {
        for (src, tgt, val) in links {
            self = self.with_link(src, tgt, val);
        }
        self
    }

    /// Set display/canonical names for alluvium axes in input order.
    pub fn with_axis_names<S, I>(mut self, axis_names: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.axis_names = Some(axis_names.into_iter().map(Into::into).collect());
        self
    }

    fn add_or_accumulate_link(&mut self, source: usize, target: usize, value: f64) {
        if let Some(link) = self
            .links
            .iter_mut()
            .find(|l| l.source == source && l.target == target && l.color.is_none())
        {
            link.value += value;
        } else {
            self.links.push(SankeyLink {
                source,
                target,
                value,
                color: None,
            });
        }
    }

    /// Add a weighted alluvium spanning multiple ordered axes.
    pub fn with_alluvium<S, I>(mut self, strata: I, value: f64) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        let mut nodes = Vec::new();
        for (axis_idx, raw) in strata.into_iter().enumerate() {
            let label = raw.into();
            let id = format!("{axis_idx}~~{label}");
            let node_idx = self.node_index_internal(&id, &label, Some(axis_idx));
            self.nodes[node_idx].column = Some(axis_idx);
            nodes.push(node_idx);
        }
        if nodes.len() >= 2 {
            for pair in nodes.windows(2) {
                self.add_or_accumulate_link(pair[0], pair[1], value);
            }
        }
        self.alluvia.push(SankeyAlluvium { nodes, value });
        self
    }

    /// Bulk add weighted alluvia from an iterator of `(strata, value)`.
    pub fn with_alluvia<S, I, J>(mut self, alluvia: J) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
        J: IntoIterator<Item = (I, f64)>,
    {
        for (strata, value) in alluvia {
            self = self.with_alluvium(strata, value);
        }
        self
    }

    /// Use gradient ribbons (linearGradient from source to target color).
    pub fn with_gradient_links(mut self) -> Self {
        self.link_color = SankeyLinkColor::Gradient;
        self
    }

    /// Use per-link colors (falls back to source color if link.color is None).
    pub fn with_per_link_colors(mut self) -> Self {
        self.link_color = SankeyLinkColor::PerLink;
        self
    }

    /// Choose how nodes are ordered within each column.
    pub fn with_node_order(mut self, order: SankeyNodeOrder) -> Self {
        self.node_order = order;
        self
    }

    /// Choose how node colors are assigned when explicit node colors are absent.
    pub fn with_node_coloring(mut self, coloring: SankeyNodeColoring) -> Self {
        self.node_coloring = coloring;
        self
    }

    /// Set the RNG seed used by crossing-reduction ordering.
    pub fn with_node_order_seed(mut self, seed: u64) -> Self {
        self.node_order_seed = seed;
        self
    }

    /// Minimize weighted crossings within columns using a wompwomp-style TSP
    /// cycle with Fenwick-scored objective evaluation.
    pub fn with_crossing_reduction(mut self) -> Self {
        self.node_order = SankeyNodeOrder::CrossingReduction;
        self
    }

    /// Use the neighbornet backend for alluvium/Sankey ordering.
    pub fn with_neighbornet(mut self) -> Self {
        self.node_order = SankeyNodeOrder::Neighbornet;
        self
    }

    /// Match wompwomp's left-to-right color propagation.
    pub fn with_left_coloring(mut self) -> Self {
        self.node_coloring = SankeyNodeColoring::Left;
        self
    }

    /// Override the palette used for fallback Sankey node colors.
    pub fn with_palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette.colors().to_vec());
        self
    }

    /// Set the parent-share threshold used by left coloring. Default: 0.5.
    pub fn with_left_color_cutoff(mut self, cutoff: f64) -> Self {
        self.left_color_cutoff = cutoff;
        self
    }

    pub fn with_link_opacity(mut self, opacity: f64) -> Self {
        self.link_opacity = opacity;
        self
    }

    pub fn with_node_width(mut self, width: f64) -> Self {
        self.node_width = width;
        self
    }

    pub fn with_node_gap(mut self, gap: f64) -> Self {
        self.node_gap = gap;
        self
    }

    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    /// Show the absolute flow value on each ribbon.
    /// Combine with [`with_flow_label_format`](Self::with_flow_label_format) and
    /// [`with_flow_label_unit`](Self::with_flow_label_unit) as needed.
    pub fn with_flow_labels(mut self) -> Self {
        self.flow_labels = true;
        self
    }

    /// Show each flow as a percentage of its source node's total outflow.
    /// Takes priority over [`with_flow_labels`](Self::with_flow_labels) when both are set.
    pub fn with_flow_percent(mut self) -> Self {
        self.flow_percent = true;
        self
    }

    /// Number format for absolute flow labels (default: [`TickFormat::Auto`]).
    /// Has no effect when [`with_flow_percent`](Self::with_flow_percent) is active.
    pub fn with_flow_label_format(mut self, fmt: TickFormat) -> Self {
        self.flow_label_format = fmt;
        self
    }

    /// Append a unit string to each absolute flow label, e.g. `"reads"` → `"1200 reads"`.
    /// Has no effect in percent mode.
    pub fn with_flow_label_unit<S: Into<String>>(mut self, unit: S) -> Self {
        self.flow_label_unit = Some(unit.into());
        self
    }

    /// Minimum ribbon height in pixels required to render a flow label.
    /// Set to `0.0` to always show labels regardless of ribbon size (default: `8.0`).
    pub fn with_flow_label_min_height(mut self, min_h: f64) -> Self {
        self.flow_label_min_height = min_h;
        self
    }
}
