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

/// A node in a Sankey diagram.
#[derive(Debug, Clone)]
pub struct SankeyNode {
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

/// A Sankey diagram: nodes arranged in columns, connected by tapered ribbons.
#[derive(Debug, Clone)]
pub struct SankeyPlot {
    pub nodes: Vec<SankeyNode>,
    pub links: Vec<SankeyLink>,
    pub link_color: SankeyLinkColor,
    /// Ribbon fill opacity (default 0.5).
    pub link_opacity: f64,
    /// Node rectangle width in pixels (default 20.0).
    pub node_width: f64,
    /// Minimum gap between nodes in a column in pixels (default 8.0).
    pub node_gap: f64,
    /// If set, adds one legend entry per node.
    pub legend_label: Option<String>,
}

impl Default for SankeyPlot {
    fn default() -> Self { Self::new() }
}

impl SankeyPlot {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            links: vec![],
            link_color: SankeyLinkColor::Source,
            link_opacity: 0.5,
            node_width: 20.0,
            node_gap: 8.0,
            legend_label: None,
        }
    }

    /// Find an existing node by label, or insert a new one and return its index.
    fn node_index(&mut self, label: &str) -> usize {
        if let Some(idx) = self.nodes.iter().position(|n| n.label == label) {
            return idx;
        }
        let idx = self.nodes.len();
        self.nodes.push(SankeyNode {
            label: label.to_string(),
            color: None,
            column: None,
        });
        idx
    }

    /// Declare a node explicitly (no-op if it already exists).
    pub fn with_node<S: Into<String>>(mut self, label: S) -> Self {
        let label = label.into();
        self.node_index(&label);
        self
    }

    /// Set the color for a node, creating it if absent.
    pub fn with_node_color<S: Into<String>, C: Into<String>>(mut self, label: S, color: C) -> Self {
        let label = label.into();
        let idx = self.node_index(&label);
        self.nodes[idx].color = Some(color.into());
        self
    }

    /// Pin a node to a specific column, creating it if absent.
    pub fn with_node_column<S: Into<String>>(mut self, label: S, col: usize) -> Self {
        let label = label.into();
        let idx = self.node_index(&label);
        self.nodes[idx].column = Some(col);
        self
    }

    /// Add a link, auto-creating nodes by label if needed.
    pub fn with_link<S: Into<String>>(mut self, source: S, target: S, value: f64) -> Self {
        let src_label = source.into();
        let tgt_label = target.into();
        let src = self.node_index(&src_label);
        let tgt = self.node_index(&tgt_label);
        self.links.push(SankeyLink { source: src, target: tgt, value, color: None });
        self
    }

    /// Add a link with an explicit per-link color.
    pub fn with_link_colored<S: Into<String>, C: Into<String>>(
        mut self, source: S, target: S, value: f64, color: C,
    ) -> Self {
        let src_label = source.into();
        let tgt_label = target.into();
        let src = self.node_index(&src_label);
        let tgt = self.node_index(&tgt_label);
        self.links.push(SankeyLink { source: src, target: tgt, value, color: Some(color.into()) });
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
}
