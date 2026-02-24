use std::collections::HashMap;

/// Which axis the root appears on.
#[derive(Debug, Clone, PartialEq)]
pub enum TreeOrientation {
    /// Root at left, leaves at right (default).
    Left,
    /// Root at right, leaves at left.
    Right,
    /// Root at top, leaves at bottom.
    Top,
    /// Root at bottom, leaves at top.
    Bottom,
}

/// How branches are drawn between parent and child.
#[derive(Debug, Clone, PartialEq)]
pub enum TreeBranchStyle {
    /// Right-angle elbows at the parent depth (default).
    Rectangular,
    /// Single diagonal line from parent to child.
    Slanted,
    /// Polar/radial projection.
    Circular,
}

/// A single node in the phylogenetic tree.
#[derive(Debug, Clone)]
pub struct PhyloNode {
    pub id:            usize,
    /// Leaf label; internal nodes may store a support value string.
    pub label:         Option<String>,
    pub parent:        Option<usize>,
    pub children:      Vec<usize>,
    /// Edge length from parent to this node (0.0 if not given).
    pub branch_length: f64,
    /// Bootstrap / posterior support value.
    pub support:       Option<f64>,
}

/// A phylogenetic tree plot.
#[derive(Debug, Clone)]
pub struct PhyloTree {
    pub nodes:             Vec<PhyloNode>,
    pub root:              usize,
    pub orientation:       TreeOrientation,
    pub branch_style:      TreeBranchStyle,
    /// Use accumulated branch lengths for the depth axis (phylogram mode).
    pub phylogram:         bool,
    pub branch_color:      String,
    /// Text color for leaf labels.
    pub leaf_color:        String,
    /// Display support values >= this threshold (None = never show).
    pub support_threshold: Option<f64>,
    /// Per-clade colors: (node_id, color) — colors the entire subtree.
    pub clade_colors:      Vec<(usize, String)>,
    pub legend_label:      Option<String>,
}

// ── Newick parser ─────────────────────────────────────────────────────────────

struct NewickParser {
    input: Vec<u8>,
    pos:   usize,
}

impl NewickParser {
    fn new(s: &str) -> Self {
        Self { input: s.as_bytes().to_vec(), pos: 0 }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) {
        if self.pos < self.input.len() { self.pos += 1; }
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r')) {
            self.advance();
        }
    }

    /// Read a label token up to the next structural character.
    fn read_token(&mut self) -> String {
        self.skip_ws();
        let start = self.pos;
        while let Some(c) = self.peek() {
            if matches!(c, b'(' | b')' | b',' | b':' | b';') { break; }
            self.advance();
        }
        std::str::from_utf8(&self.input[start..self.pos])
            .unwrap_or("")
            .trim()
            .to_string()
    }

    /// Parse an optional floating-point number; rewind and return `None` if absent.
    fn read_number(&mut self) -> Option<f64> {
        self.skip_ws();
        let start = self.pos;
        if matches!(self.peek(), Some(b'-') | Some(b'+')) { self.advance(); }
        let mut saw_digit = false;
        while matches!(
            self.peek(),
            Some(b'0'..=b'9') | Some(b'.') | Some(b'e') | Some(b'E') | Some(b'+') | Some(b'-')
        ) {
            saw_digit = true;
            self.advance();
        }
        if !saw_digit { self.pos = start; return None; }
        std::str::from_utf8(&self.input[start..self.pos]).ok()?.parse().ok()
    }

    /// Recursively parse a subtree; returns the new node's id.
    fn parse_subtree(&mut self, nodes: &mut Vec<PhyloNode>, parent: Option<usize>) -> usize {
        self.skip_ws();
        let id = nodes.len();
        nodes.push(PhyloNode {
            id,
            label:         None,
            parent,
            children:      Vec::new(),
            branch_length: 0.0,
            support:       None,
        });

        if self.peek() == Some(b'(') {
            self.advance(); // consume '('
            loop {
                self.skip_ws();
                let child = self.parse_subtree(nodes, Some(id));
                nodes[id].children.push(child);
                self.skip_ws();
                match self.peek() {
                    Some(b',') => { self.advance(); }
                    Some(b')') => { self.advance(); break; }
                    _          => break,
                }
            }
            // Optional label / support value after ')'
            self.skip_ws();
            if !matches!(self.peek(), Some(b':') | Some(b';') | Some(b',') | Some(b')') | None) {
                let tok = self.read_token();
                if !tok.is_empty() {
                    if let Ok(v) = tok.parse::<f64>() {
                        nodes[id].support = Some(v);
                    } else {
                        nodes[id].label = Some(tok);
                    }
                }
            }
        } else {
            // Leaf: read its label
            let tok = self.read_token();
            if !tok.is_empty() {
                nodes[id].label = Some(tok);
            }
        }

        // Optional branch length after ':'
        self.skip_ws();
        if self.peek() == Some(b':') {
            self.advance();
            self.skip_ws();
            nodes[id].branch_length = self.read_number().unwrap_or(0.0);
        }

        id
    }

    fn parse(mut self) -> (Vec<PhyloNode>, usize) {
        let mut nodes = Vec::new();
        let root = self.parse_subtree(&mut nodes, None);
        // Consume trailing ';'
        self.skip_ws();
        if self.peek() == Some(b';') { self.advance(); }
        (nodes, root)
    }
}

// ── Constructors & builder ────────────────────────────────────────────────────

impl PhyloTree {
    pub(crate) fn new_from_nodes(nodes: Vec<PhyloNode>, root: usize) -> Self {
        Self {
            nodes,
            root,
            orientation:       TreeOrientation::Left,
            branch_style:      TreeBranchStyle::Rectangular,
            phylogram:         false,
            branch_color:      "black".to_string(),
            leaf_color:        "black".to_string(),
            support_threshold: None,
            clade_colors:      Vec::new(),
            legend_label:      None,
        }
    }

    /// Parse a Newick-format string into a `PhyloTree`.
    ///
    /// Supports branch lengths (`A:1.0`), support values on internal nodes,
    /// and arbitrarily nested subtrees.
    pub fn from_newick(s: &str) -> Self {
        let (nodes, root) = NewickParser::new(s).parse();
        Self::new_from_nodes(nodes, root)
    }

    /// Build a tree from `(parent_label, child_label, branch_length)` edges.
    ///
    /// Root = the node that never appears as a child.
    pub fn from_edges<S: AsRef<str>>(edges: &[(S, S, f64)]) -> Self {
        let mut label_to_id: HashMap<String, usize> = HashMap::new();
        let mut nodes: Vec<PhyloNode> = Vec::new();
        let mut child_ids: std::collections::HashSet<usize> = std::collections::HashSet::new();

        for (parent_lbl, child_lbl, branch_len) in edges {
            let parent_lbl = parent_lbl.as_ref();
            let child_lbl  = child_lbl.as_ref();

            // Get-or-create parent
            let parent_id = if let Some(&id) = label_to_id.get(parent_lbl) {
                id
            } else {
                let id = nodes.len();
                nodes.push(PhyloNode {
                    id, label: Some(parent_lbl.to_string()),
                    parent: None, children: Vec::new(), branch_length: 0.0, support: None,
                });
                label_to_id.insert(parent_lbl.to_string(), id);
                id
            };

            // Get-or-create child
            let child_id = if let Some(&id) = label_to_id.get(child_lbl) {
                id
            } else {
                let id = nodes.len();
                nodes.push(PhyloNode {
                    id, label: Some(child_lbl.to_string()),
                    parent: None, children: Vec::new(), branch_length: 0.0, support: None,
                });
                label_to_id.insert(child_lbl.to_string(), id);
                id
            };

            nodes[child_id].parent        = Some(parent_id);
            nodes[child_id].branch_length = *branch_len;
            nodes[parent_id].children.push(child_id);
            child_ids.insert(child_id);
        }

        let root = (0..nodes.len()).find(|id| !child_ids.contains(id)).unwrap_or(0);
        Self::new_from_nodes(nodes, root)
    }

    /// Build a tree by UPGMA clustering of a symmetric distance matrix.
    pub fn from_distance_matrix(labels: &[&str], dist: &[Vec<f64>]) -> Self {
        let (nodes, root) = crate::render::render_utils::upgma(labels, dist);
        Self::new_from_nodes(nodes, root)
    }

    /// Build a tree from a scipy / R linkage matrix.
    ///
    /// Each row is `[left_idx, right_idx, distance, n_leaves]`.
    /// Indices `0..n-1` are the original leaves; `n..` are internal nodes.
    pub fn from_linkage(labels: &[&str], linkage: &[[f64; 4]]) -> Self {
        let (nodes, root) = crate::render::render_utils::linkage_to_nodes(labels, linkage);
        Self::new_from_nodes(nodes, root)
    }

    // ── Builder ───────────────────────────────────────────────────────────────

    pub fn with_orientation(mut self, o: TreeOrientation) -> Self {
        self.orientation = o; self
    }

    pub fn with_branch_style(mut self, s: TreeBranchStyle) -> Self {
        self.branch_style = s; self
    }

    /// Enable phylogram mode: use branch lengths for the depth axis.
    pub fn with_phylogram(mut self) -> Self {
        self.phylogram = true; self
    }

    pub fn with_branch_color<S: Into<String>>(mut self, c: S) -> Self {
        self.branch_color = c.into(); self
    }

    pub fn with_leaf_color<S: Into<String>>(mut self, c: S) -> Self {
        self.leaf_color = c.into(); self
    }

    /// Show support values >= `threshold`.
    pub fn with_support_threshold(mut self, t: f64) -> Self {
        self.support_threshold = Some(t); self
    }

    /// Color the entire subtree rooted at `node_id` with `color`.
    pub fn with_clade_color<S: Into<String>>(mut self, node_id: usize, color: S) -> Self {
        self.clade_colors.push((node_id, color.into())); self
    }

    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into()); self
    }

    // ── Helper ────────────────────────────────────────────────────────────────

    /// Returns leaf labels in the top-to-bottom render order (post-order DFS,
    /// left children first).  Use this to set `y_categories` on a side-by-side
    /// `Heatmap` for row alignment.
    pub fn leaf_labels_top_to_bottom(&self) -> Vec<String> {
        post_order_dfs(self.root, &self.nodes)
            .into_iter()
            .filter(|&id| self.nodes[id].children.is_empty())
            .filter_map(|id| self.nodes[id].label.clone())
            .collect()
    }
}

/// Iterative post-order DFS; left children first (pushed in reverse onto stack).
pub(crate) fn post_order_dfs(root: usize, nodes: &[PhyloNode]) -> Vec<usize> {
    let mut result = Vec::new();
    let mut stack  = vec![(root, false)];
    while let Some((id, done)) = stack.pop() {
        if done {
            result.push(id);
        } else {
            stack.push((id, true));
            for &child in nodes[id].children.iter().rev() {
                stack.push((child, false));
            }
        }
    }
    result
}
