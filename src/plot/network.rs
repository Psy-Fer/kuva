/// A network / graph diagram: nodes connected by edges, laid out with
/// force-directed (Fruchterman–Reingold), Kamada–Kawai, or circular placement.
///
/// Supports both edge-list and adjacency-matrix input. Edges can be directed
/// (arrowheads) or undirected (plain lines). Self-loops are rendered as small
/// arcs. Edge weight controls stroke width and opacity.
///
/// # Pixel-space rendering
///
/// Like chord and sankey diagrams, the network plot is rendered entirely in
/// pixel space — it does not use the standard x/y axis system. A title set on
/// the `Layout` is still rendered.
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::NetworkPlot;
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let net = NetworkPlot::new()
///     .with_edge("A", "B", 1.0)
///     .with_edge("A", "C", 2.0)
///     .with_edge("B", "C", 1.5)
///     .with_labels();
///
/// let plots = vec![Plot::Network(net)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("My Network");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("network.svg", svg).unwrap();
/// ```
use std::collections::HashMap;

/// Layout algorithm for node placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkLayout {
    /// Fruchterman–Reingold force-directed layout (default).
    ForceDirected,
    /// Kamada–Kawai stress-based layout.  Produces cleaner results for
    /// small–medium graphs where edge lengths should reflect graph distance.
    KamadaKawai,
    /// Nodes evenly spaced on a circle.
    Circle,
}

/// Node marker shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeShape {
    Circle,
    Square,
    Triangle,
    Diamond,
}

impl NodeShape {
    /// Ratio of the shape's circumradius to the base radius `r`.
    /// Used to offset edge endpoints so they clear the shape boundary.
    pub(crate) fn circumradius_factor(self) -> f64 {
        match self {
            Self::Circle => 1.0,
            Self::Square => std::f64::consts::SQRT_2,
            Self::Diamond => 1.2,
            Self::Triangle => 1.4,
        }
    }
}

/// A node in the network graph.
#[derive(Debug, Clone)]
pub struct NetworkNode {
    pub label: String,
    pub color: Option<String>,
    pub size: Option<f64>,
    pub group: Option<String>,
    pub shape: NodeShape,
    /// Fixed position in normalised \[0, 1\] space. When `Some`, the layout
    /// algorithm will not move this node.
    pub position: Option<(f64, f64)>,
}

/// An edge connecting two nodes.
#[derive(Debug, Clone)]
pub struct NetworkEdge {
    /// Index into `NetworkPlot::nodes`.
    pub source: usize,
    /// Index into `NetworkPlot::nodes`.
    pub target: usize,
    pub weight: f64,
    pub color: Option<String>,
    pub label: Option<String>,
    /// Signed perpendicular offset as a fraction of edge length for a
    /// quadratic-bezier arc. Positive bends left of travel direction
    /// (perp = (-uy, ux)); negative bends right. None = straight/antiparallel default.
    pub curve: Option<f64>,
}

/// A network / graph diagram.
#[derive(Debug, Clone)]
pub struct NetworkPlot {
    pub nodes: Vec<NetworkNode>,
    pub edges: Vec<NetworkEdge>,
    /// Draw arrowheads on edges (default `false`).
    pub directed: bool,
    /// Node placement algorithm (default [`NetworkLayout::ForceDirected`]).
    pub layout: NetworkLayout,
    /// Base node radius in pixels (default 8.0).
    pub node_radius: f64,
    /// Edge stroke opacity 0.0–1.0 (default 0.6).
    pub edge_opacity: f64,
    /// Render node labels (default `false`).
    pub show_labels: bool,
    /// If set, generate a legend with one entry per unique group.
    pub legend_label: Option<String>,
    /// Override label font size (pixels).
    pub label_size: Option<u32>,
    /// Apply label repulsion to avoid overlap (default `false`).
    pub repel_labels: bool,
    /// Render node labels centred inside the node circle instead of beside it
    /// (default `false`). Font size is auto-scaled to fit within the node
    /// radius; white text is used for contrast.
    pub label_inside: bool,
    /// Deferred adjacency matrix (expanded into edges by `resolve_matrix`).
    pending_matrix: Option<(Vec<Vec<f64>>, Vec<usize>)>,
    /// O(1) label→index lookup, kept in sync with `nodes`.
    node_map: HashMap<String, usize>,
}

impl Default for NetworkPlot {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkPlot {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            edges: vec![],
            directed: false,
            layout: NetworkLayout::ForceDirected,
            node_radius: 8.0,
            edge_opacity: 0.6,
            show_labels: false,
            legend_label: None,
            label_size: None,
            repel_labels: false,
            label_inside: false,
            pending_matrix: None,
            node_map: HashMap::new(),
        }
    }

    /// Find an existing node by label, or insert a new one and return its index.
    fn node_index(&mut self, label: &str) -> usize {
        if let Some(&idx) = self.node_map.get(label) {
            return idx;
        }
        let idx = self.nodes.len();
        self.nodes.push(NetworkNode {
            label: label.to_string(),
            color: None,
            size: None,
            group: None,
            shape: NodeShape::Circle,
            position: None,
        });
        self.node_map.insert(label.to_string(), idx);
        idx
    }

    fn push_edge(
        &mut self,
        source: String,
        target: String,
        weight: f64,
        color: Option<String>,
        label: Option<String>,
        curve: Option<f64>,
    ) {
        let si = self.node_index(&source);
        let ti = self.node_index(&target);
        self.edges.push(NetworkEdge {
            source: si,
            target: ti,
            weight,
            color,
            label,
            curve,
        });
    }

    /// Add an edge, auto-creating source and target nodes by label if needed.
    pub fn with_edge<S: Into<String>>(
        mut self,
        source: S,
        target: S,
        weight: impl Into<f64>,
    ) -> Self {
        self.push_edge(source.into(), target.into(), weight.into(), None, None, None);
        self
    }

    /// Add an edge with an explicit colour.
    pub fn with_edge_color<S: Into<String>, C: Into<String>>(
        mut self,
        source: S,
        target: S,
        weight: impl Into<f64>,
        color: C,
    ) -> Self {
        self.push_edge(
            source.into(),
            target.into(),
            weight.into(),
            Some(color.into()),
            None,
            None,
        );
        self
    }

    /// Add an edge with a text label rendered at its midpoint.
    pub fn with_edge_label<S: Into<String>, L: Into<String>>(
        mut self,
        source: S,
        target: S,
        weight: impl Into<f64>,
        label: L,
    ) -> Self {
        self.push_edge(
            source.into(),
            target.into(),
            weight.into(),
            None,
            Some(label.into()),
            None,
        );
        self
    }

    /// Add an edge with both an explicit colour and a text label.
    pub fn with_edge_styled<S: Into<String>, C: Into<String>, L: Into<String>>(
        mut self,
        source: S,
        target: S,
        weight: impl Into<f64>,
        color: C,
        label: L,
    ) -> Self {
        self.push_edge(
            source.into(),
            target.into(),
            weight.into(),
            Some(color.into()),
            Some(label.into()),
            None,
        );
        self
    }

    /// Add a curved edge via quadratic-bezier arc.
    ///
    /// `curve` is a signed perpendicular offset as a fraction of edge length.
    /// Positive bends left of the travel direction (perp = (−uy, ux));
    /// negative bends right.
    pub fn with_edge_curved<S: Into<String>>(
        mut self,
        source: S,
        target: S,
        weight: impl Into<f64>,
        curve: f64,
    ) -> Self {
        self.push_edge(
            source.into(),
            target.into(),
            weight.into(),
            None,
            None,
            Some(curve),
        );
        self
    }

    /// Bulk-add edges from an iterator of `(source, target, weight)`.
    pub fn with_edges<S, V, I>(mut self, edges: I) -> Self
    where
        S: Into<String>,
        V: Into<f64>,
        I: IntoIterator<Item = (S, S, V)>,
    {
        for (src, tgt, w) in edges {
            self.push_edge(src.into(), tgt.into(), w.into(), None, None, None);
        }
        self
    }

    /// Build a network from an N×N adjacency matrix.
    ///
    /// Non-zero entries become edges; the value is used as the weight.
    /// The matrix is stored and edges are expanded by
    /// `resolve_matrix` (called automatically by `render_multiple`),
    /// so `.with_directed()` can be called before or after this method.
    ///
    /// If you call `compute_positions` directly (e.g. to inspect the
    /// layout), call `resolve_matrix()` first.
    pub fn with_matrix<S, L>(mut self, matrix: Vec<Vec<f64>>, labels: L) -> Self
    where
        S: Into<String>,
        L: IntoIterator<Item = S>,
    {
        let labels: Vec<String> = labels.into_iter().map(Into::into).collect();
        let indices: Vec<usize> = labels.iter().map(|l| self.node_index(l)).collect();
        self.pending_matrix = Some((matrix, indices));
        self
    }

    /// Expand a pending adjacency matrix into edges.  Called automatically
    /// by `render_multiple`; safe to call multiple times (no-op after the
    /// first).
    pub fn resolve_matrix(&mut self) {
        if let Some((matrix, indices)) = self.pending_matrix.take() {
            let n = indices.len();
            for i in 0..n {
                if i >= matrix.len() {
                    continue;
                }
                let j_start = if self.directed { 0 } else { i + 1 };
                for j in j_start..n {
                    if j == i || j >= matrix[i].len() {
                        continue;
                    }
                    let w = matrix[i][j];
                    if w.abs() < f64::EPSILON {
                        continue;
                    }
                    self.edges.push(NetworkEdge {
                        source: indices[i],
                        target: indices[j],
                        weight: w,
                        color: None,
                        label: None,
                        curve: None,
                    });
                }
                // Self-loops from diagonal: only in directed mode.
                // For undirected graphs the diagonal is meaningless (a node
                // connected to itself has no physical interpretation when edges
                // are symmetric), so we intentionally skip it.
                if self.directed && i < matrix[i].len() {
                    let w = matrix[i][i];
                    if w.abs() >= f64::EPSILON {
                        self.edges.push(NetworkEdge {
                            source: indices[i],
                            target: indices[i],
                            weight: w,
                            color: None,
                            label: None,
                            curve: None,
                        });
                    }
                }
            }
        }
    }

    /// Declare a node explicitly (no-op if it already exists).
    pub fn with_node<S: Into<String>>(mut self, label: S) -> Self {
        let label = label.into();
        self.node_index(&label);
        self
    }

    /// Set the colour for a node, creating it if absent.
    pub fn with_node_color<S: Into<String>, C: Into<String>>(mut self, label: S, color: C) -> Self {
        let label = label.into();
        let idx = self.node_index(&label);
        self.nodes[idx].color = Some(color.into());
        self
    }

    /// Set the size for a node, creating it if absent.
    pub fn with_node_size<S: Into<String>>(mut self, label: S, size: f64) -> Self {
        let label = label.into();
        let idx = self.node_index(&label);
        self.nodes[idx].size = Some(size);
        self
    }

    /// Set the group for a node, creating it if absent.
    pub fn with_node_group<S: Into<String>, G: Into<String>>(mut self, label: S, group: G) -> Self {
        let label = label.into();
        let idx = self.node_index(&label);
        self.nodes[idx].group = Some(group.into());
        self
    }

    /// Set the marker shape for a node, creating it if absent.
    pub fn with_node_shape<S: Into<String>>(mut self, label: S, shape: NodeShape) -> Self {
        let label = label.into();
        let idx = self.node_index(&label);
        self.nodes[idx].shape = shape;
        self
    }

    /// Fix a node's position in normalised \[0, 1\] space.
    pub fn with_node_position<S: Into<String>>(mut self, label: S, x: f64, y: f64) -> Self {
        let label = label.into();
        let idx = self.node_index(&label);
        self.nodes[idx].position = Some((x, y));
        self
    }

    /// Draw directed edges with arrowheads.
    pub fn with_directed(mut self) -> Self {
        self.directed = true;
        self
    }

    /// Set the layout algorithm.
    pub fn with_layout(mut self, layout: NetworkLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Set the base node radius in pixels.
    pub fn with_node_radius(mut self, r: f64) -> Self {
        self.node_radius = r;
        self
    }

    /// Set edge stroke opacity (0.0–1.0).
    pub fn with_edge_opacity(mut self, opacity: f64) -> Self {
        self.edge_opacity = opacity;
        self
    }

    /// Show node labels beside each node.
    pub fn with_labels(mut self) -> Self {
        self.show_labels = true;
        self
    }

    /// Enable label repulsion so overlapping labels push apart.
    pub fn with_repel_labels(mut self) -> Self {
        self.repel_labels = true;
        self
    }

    /// Render node labels centred inside each node.  Also enables
    /// `show_labels`.  Font size is auto-scaled to fit the node radius;
    /// white text is used for contrast against the node fill.
    pub fn with_labels_inside(mut self) -> Self {
        self.show_labels = true;
        self.label_inside = true;
        self
    }

    /// Show a legend; one entry per unique group.  If no groups have been
    /// assigned the legend falls back to one entry per node, which can be
    /// large — prefer using `with_node_group` to keep the legend compact.
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    /// Override the label font size.
    pub fn with_label_size(mut self, size: u32) -> Self {
        self.label_size = Some(size);
        self
    }

    // ── Layout algorithms ─────────────────────────────────────────────

    /// Compute node positions in \[0, 1\] × \[0, 1\] space.
    ///
    /// Disconnected components are laid out independently, then tiled
    /// side-by-side (like igraph's `component_wise` approach).
    ///
    /// Call `resolve_matrix` first if a matrix was provided via
    /// `with_matrix`, or this will only see edges added via
    /// `with_edge`/`with_edges`.
    pub fn compute_positions(&self) -> Vec<(f64, f64)> {
        let n = self.nodes.len();
        if n == 0 {
            return vec![];
        }
        if n == 1 {
            return vec![(0.5, 0.5)];
        }

        // Circle layout doesn't benefit from component-wise — it places
        // all nodes on one circle regardless.
        if self.layout == NetworkLayout::Circle {
            return self.circle_layout();
        }

        let components = self.connected_components();
        if components.len() == 1 {
            // Single component — run layout directly on the full graph.
            return match self.layout {
                NetworkLayout::ForceDirected => self.fruchterman_reingold(),
                NetworkLayout::KamadaKawai => self.kamada_kawai(),
                _ => unreachable!(),
            };
        }

        // Multiple components: lay out each independently, then tile.
        self.layout_component_wise(&components)
    }

    fn circle_layout(&self) -> Vec<(f64, f64)> {
        let n = self.nodes.len();
        if n == 0 {
            return vec![];
        }
        if n == 1 {
            return vec![(0.5, 0.5)];
        }
        (0..n)
            .map(|i| {
                let angle = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64)
                    - std::f64::consts::FRAC_PI_2; // start at top
                let x = 0.5 + 0.5 * angle.cos();
                let y = 0.5 + 0.5 * angle.sin();
                (x, y)
            })
            .collect()
    }

    // ── Connected components & tiling ──────────────────────────────────

    /// Find connected components via BFS.  Returns a Vec of Vec<usize>,
    /// each inner Vec being the node indices of one component.
    fn connected_components(&self) -> Vec<Vec<usize>> {
        let n = self.nodes.len();
        let adj = self.adjacency();
        let mut visited = vec![false; n];
        let mut components = Vec::new();
        for start in 0..n {
            if visited[start] {
                continue;
            }
            let mut comp = Vec::new();
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(start);
            visited[start] = true;
            while let Some(u) = queue.pop_front() {
                comp.push(u);
                for &(v, _) in &adj[u] {
                    if !visited[v] {
                        visited[v] = true;
                        queue.push_back(v);
                    }
                }
            }
            components.push(comp);
        }
        components
    }

    /// Lay out each connected component independently, then tile them
    /// into the [0, 1] × [0, 1] space using a simple row packing.
    fn layout_component_wise(&self, components: &[Vec<usize>]) -> Vec<(f64, f64)> {
        let n = self.nodes.len();
        let mut pos = vec![(0.0, 0.0); n];

        // Build a sub-NetworkPlot for each component and lay it out.
        struct CompLayout {
            indices: Vec<usize>,
            positions: Vec<(f64, f64)>, // in [0,1] space per component
            node_count: usize,
        }

        let mut comp_layouts: Vec<CompLayout> = Vec::new();
        for comp in components {
            if comp.len() == 1 {
                // Single isolated node — place at centre.
                comp_layouts.push(CompLayout {
                    indices: comp.clone(),
                    positions: vec![(0.5, 0.5)],
                    node_count: 1,
                });
                continue;
            }

            // Build sub-graph: remap node indices to 0..comp.len()
            let mut index_map = HashMap::new();
            for (new_i, &old_i) in comp.iter().enumerate() {
                index_map.insert(old_i, new_i);
            }

            let mut sub = NetworkPlot::new();
            sub.directed = self.directed;
            sub.layout = self.layout;
            // Add nodes
            for &old_i in comp {
                let node = &self.nodes[old_i];
                sub.nodes.push(NetworkNode {
                    label: node.label.clone(),
                    color: node.color.clone(),
                    size: node.size,
                    group: node.group.clone(),
                    shape: node.shape,
                    position: node.position,
                });
            }
            // Add edges (remapped)
            for edge in &self.edges {
                if let (Some(&new_s), Some(&new_t)) =
                    (index_map.get(&edge.source), index_map.get(&edge.target))
                {
                    sub.edges.push(NetworkEdge {
                        source: new_s,
                        target: new_t,
                        weight: edge.weight,
                        color: edge.color.clone(),
                        label: edge.label.clone(),
                        curve: edge.curve,
                    });
                }
            }

            let positions = match self.layout {
                NetworkLayout::ForceDirected => sub.fruchterman_reingold(),
                NetworkLayout::KamadaKawai => sub.kamada_kawai(),
                _ => sub.circle_layout(),
            };

            comp_layouts.push(CompLayout {
                indices: comp.clone(),
                positions,
                node_count: comp.len(),
            });
        }

        // Sort components by size (largest first) for better packing.
        comp_layouts.sort_by_key(|b| std::cmp::Reverse(b.node_count));

        // Tile: allocate horizontal width proportional to sqrt(node_count).
        let total_weight: f64 = comp_layouts
            .iter()
            .map(|c| (c.node_count as f64).sqrt())
            .sum();
        let gap = 0.03; // gap between components
        let total_gap = gap * (comp_layouts.len().saturating_sub(1)) as f64;
        let usable = (1.0 - total_gap).max(0.1);

        let mut x_cursor = 0.0;
        for cl in &comp_layouts {
            let weight = (cl.node_count as f64).sqrt();
            let width = usable * weight / total_weight;
            // Map component positions [0,1] into [x_cursor, x_cursor+width] × [0,1]
            for (local_i, &old_i) in cl.indices.iter().enumerate() {
                let (lx, ly) = cl.positions[local_i];
                pos[old_i] = (x_cursor + lx * width, ly);
            }
            x_cursor += width + gap;
        }

        pos
    }

    // ── Shared helpers ────────────────────────────────────────────────

    /// Build an adjacency list from the current edges (ignoring self-loops).
    fn adjacency(&self) -> Vec<Vec<(usize, f64)>> {
        let n = self.nodes.len();
        let mut adj = vec![vec![]; n];
        for e in &self.edges {
            if e.source == e.target {
                continue;
            }
            adj[e.source].push((e.target, e.weight));
            if !self.directed {
                adj[e.target].push((e.source, e.weight));
            }
        }
        adj
    }

    /// All-pairs shortest-path distances (BFS-like using Dijkstra with
    /// `weight = 1/edge_weight` so stronger edges mean shorter distance).
    /// Returns `dist[i][j]`; disconnected pairs get `f64::INFINITY`.
    fn all_pairs_distances(&self) -> Vec<Vec<f64>> {
        let n = self.nodes.len();
        let adj = self.adjacency();
        let mut dist = vec![vec![f64::INFINITY; n]; n];
        for s in 0..n {
            dist[s][s] = 0.0;
            // Dijkstra from s
            let mut visited = vec![false; n];
            let mut d = vec![f64::INFINITY; n];
            d[s] = 0.0;
            for _ in 0..n {
                // Find unvisited node with smallest distance
                let mut u = n;
                let mut best = f64::INFINITY;
                for v in 0..n {
                    if !visited[v] && d[v] < best {
                        best = d[v];
                        u = v;
                    }
                }
                if u == n {
                    break;
                }
                visited[u] = true;
                for &(v, w) in &adj[u] {
                    let edge_dist = 1.0 / w.max(1e-9);
                    if d[u] + edge_dist < d[v] {
                        d[v] = d[u] + edge_dist;
                    }
                }
            }
            dist[s] = d;
        }
        dist
    }

    /// Deterministic initial grid placement with perturbation.
    fn initial_positions(&self) -> Vec<(f64, f64)> {
        let n = self.nodes.len();
        let cols = (n as f64).sqrt().ceil() as usize;
        let mut pos: Vec<(f64, f64)> = (0..n)
            .map(|i| {
                let row = i / cols;
                let col = i % cols;
                let x = (col as f64 + 0.5) / cols as f64;
                let y = (row as f64 + 0.5) / cols as f64;
                let hash = ((i as u64).wrapping_mul(2654435761) & 0xFFFF) as f64 / 65536.0;
                (x + 0.01 * hash, y + 0.01 * (1.0 - hash))
            })
            .collect();
        // Honour user-supplied positions
        for (i, node) in self.nodes.iter().enumerate() {
            if let Some((px, py)) = node.position {
                pos[i] = (px, py);
            }
        }
        pos
    }

    /// Normalise unpinned node positions to \[0, 1\] with uniform scaling.
    fn normalise_positions(&self, pos: &mut [(f64, f64)]) {
        let n = pos.len();
        let free: Vec<usize> = (0..n)
            .filter(|&i| self.nodes[i].position.is_none())
            .collect();
        if free.is_empty() {
            return;
        }
        let (mut xmin, mut xmax) = (f64::INFINITY, f64::NEG_INFINITY);
        let (mut ymin, mut ymax) = (f64::INFINITY, f64::NEG_INFINITY);
        for &i in &free {
            xmin = xmin.min(pos[i].0);
            xmax = xmax.max(pos[i].0);
            ymin = ymin.min(pos[i].1);
            ymax = ymax.max(pos[i].1);
        }
        let xrange = xmax - xmin;
        let yrange = ymax - ymin;
        let scale = xrange.max(yrange).max(1e-6);
        let x_offset = (1.0 - xrange / scale) / 2.0;
        let y_offset = (1.0 - yrange / scale) / 2.0;
        for &i in &free {
            pos[i].0 = (pos[i].0 - xmin) / scale + x_offset;
            pos[i].1 = (pos[i].1 - ymin) / scale + y_offset;
        }
    }

    // ── Fruchterman–Reingold (with Barnes–Hut for n > 256) ────────────

    fn fruchterman_reingold(&self) -> Vec<(f64, f64)> {
        let n = self.nodes.len();
        if n == 0 {
            return vec![];
        }
        if n == 1 {
            return vec![(0.5, 0.5)];
        }

        let area = 4.0_f64;
        let k = (area / n as f64).sqrt();
        let iterations = 100;
        let mut temp = 0.2 * (n as f64).sqrt();
        let cooling = temp / iterations as f64;

        let mut pos = self.initial_positions();

        let fa = |d: f64| -> f64 { d * d / k }; // attractive
        let fr_force = |d: f64| -> f64 { k * k / (d + 1e-6) }; // repulsive

        let use_bh = n > 256;

        for _ in 0..iterations {
            let mut disp = vec![(0.0_f64, 0.0_f64); n];

            if use_bh {
                // Barnes–Hut: build quadtree, approximate distant repulsion
                let tree = QuadTree::build(&pos);
                for i in 0..n {
                    let (fx, fy) = tree.repulsive_force(pos[i].0, pos[i].1, k, 0.8);
                    disp[i].0 += fx;
                    disp[i].1 += fy;
                }
            } else {
                // Exact O(n²) repulsion
                for i in 0..n {
                    for j in (i + 1)..n {
                        let dx = pos[i].0 - pos[j].0;
                        let dy = pos[i].1 - pos[j].1;
                        let dist = (dx * dx + dy * dy).sqrt().max(1e-6);
                        let force = fr_force(dist);
                        let fx = dx / dist * force;
                        let fy = dy / dist * force;
                        disp[i].0 += fx;
                        disp[i].1 += fy;
                        disp[j].0 -= fx;
                        disp[j].1 -= fy;
                    }
                }
            }

            // Attractive forces along edges
            for edge in &self.edges {
                let (si, ti) = (edge.source, edge.target);
                if si == ti {
                    continue;
                }
                let dx = pos[si].0 - pos[ti].0;
                let dy = pos[si].1 - pos[ti].1;
                let dist = (dx * dx + dy * dy).sqrt().max(1e-6);
                let force = fa(dist);
                let fx = dx / dist * force;
                let fy = dy / dist * force;
                disp[si].0 -= fx;
                disp[si].1 -= fy;
                disp[ti].0 += fx;
                disp[ti].1 += fy;
            }

            // Apply displacement capped by temperature
            for i in 0..n {
                if self.nodes[i].position.is_some() {
                    continue;
                }
                let dx = disp[i].0;
                let dy = disp[i].1;
                let mag = (dx * dx + dy * dy).sqrt().max(1e-6);
                let cap = mag.min(temp);
                pos[i].0 += dx / mag * cap;
                pos[i].1 += dy / mag * cap;
            }

            temp -= cooling;
            if temp < 0.0 {
                temp = 0.0;
            }
        }

        self.normalise_positions(&mut pos);
        pos
    }

    // ── Kamada–Kawai ──────────────────────────────────────────────────

    fn kamada_kawai(&self) -> Vec<(f64, f64)> {
        let n = self.nodes.len();
        if n == 0 {
            return vec![];
        }
        if n == 1 {
            return vec![(0.5, 0.5)];
        }

        let dist = self.all_pairs_distances();

        // Ideal distances: d_ij * L / max_dist, where L ≈ 1.0
        let max_dist = dist
            .iter()
            .flatten()
            .filter(|&&d| d.is_finite())
            .cloned()
            .fold(0.0_f64, f64::max)
            .max(1.0);
        let l_factor = 1.0 / max_dist;

        // Spring strengths: k_ij = 1 / d_ij^2
        // Start from circle layout for stability
        let mut pos = self.circle_layout();
        for (i, node) in self.nodes.iter().enumerate() {
            if let Some((px, py)) = node.position {
                pos[i] = (px, py);
            }
        }

        let iterations = 200;
        let epsilon = 1e-4;

        for _ in 0..iterations {
            // Find node with largest partial derivative (most stress)
            let mut max_delta = 0.0_f64;
            let mut m = 0;
            for i in 0..n {
                if self.nodes[i].position.is_some() {
                    continue;
                }
                let (mut dx, mut dy) = (0.0, 0.0);
                for j in 0..n {
                    if i == j || !dist[i][j].is_finite() {
                        continue;
                    }
                    let xd = pos[i].0 - pos[j].0;
                    let yd = pos[i].1 - pos[j].1;
                    let actual = (xd * xd + yd * yd).sqrt().max(1e-9);
                    let ideal = dist[i][j] * l_factor;
                    let k_ij = 1.0 / (dist[i][j] * dist[i][j]).max(1e-9);
                    dx += k_ij * (xd - ideal * xd / actual);
                    dy += k_ij * (yd - ideal * yd / actual);
                }
                let delta = (dx * dx + dy * dy).sqrt();
                if delta > max_delta {
                    max_delta = delta;
                    m = i;
                }
            }
            if max_delta < epsilon {
                break;
            }

            // Move node m to reduce its stress (inner loop)
            for _ in 0..5 {
                let (mut dx, mut dy) = (0.0, 0.0);
                let (mut dxx, mut dxy, mut dyy) = (0.0, 0.0, 0.0);
                for j in 0..n {
                    if m == j || !dist[m][j].is_finite() {
                        continue;
                    }
                    let xd = pos[m].0 - pos[j].0;
                    let yd = pos[m].1 - pos[j].1;
                    let actual = (xd * xd + yd * yd).sqrt().max(1e-9);
                    let ideal = dist[m][j] * l_factor;
                    let k_ij = 1.0 / (dist[m][j] * dist[m][j]).max(1e-9);
                    dx += k_ij * (xd - ideal * xd / actual);
                    dy += k_ij * (yd - ideal * yd / actual);
                    let actual3 = actual * actual * actual;
                    dxx += k_ij * (1.0 - ideal * yd * yd / actual3);
                    dxy += k_ij * (ideal * xd * yd / actual3);
                    dyy += k_ij * (1.0 - ideal * xd * xd / actual3);
                }
                let det = (dxx * dyy - dxy * dxy).max(1e-9);
                let step_x = (dyy * dx - dxy * dy) / det;
                let step_y = (dxx * dy - dxy * dx) / det;
                pos[m].0 -= step_x;
                pos[m].1 -= step_y;
            }
        }

        self.normalise_positions(&mut pos);
        pos
    }
}

// ── Barnes–Hut quadtree for O(n log n) repulsive forces ──────────────────

struct QuadTree {
    // Bounding box
    cx: f64,
    cy: f64,
    half: f64,
    // Centre of mass and total count
    com_x: f64,
    com_y: f64,
    count: usize,
    // Children: NW, NE, SW, SE (None = empty)
    children: [Option<Box<QuadTree>>; 4],
}

impl QuadTree {
    fn build(points: &[(f64, f64)]) -> Self {
        let (mut xmin, mut xmax) = (f64::INFINITY, f64::NEG_INFINITY);
        let (mut ymin, mut ymax) = (f64::INFINITY, f64::NEG_INFINITY);
        for &(x, y) in points {
            xmin = xmin.min(x);
            xmax = xmax.max(x);
            ymin = ymin.min(y);
            ymax = ymax.max(y);
        }
        let half = ((xmax - xmin).max(ymax - ymin) / 2.0).max(1e-6);
        let cx = (xmin + xmax) / 2.0;
        let cy = (ymin + ymax) / 2.0;
        let mut tree = Self {
            cx,
            cy,
            half,
            com_x: 0.0,
            com_y: 0.0,
            count: 0,
            children: Default::default(),
        };
        for &(x, y) in points {
            tree.insert(x, y);
        }
        tree
    }

    fn insert(&mut self, x: f64, y: f64) {
        self.insert_depth(x, y, 0);
    }

    fn insert_depth(&mut self, x: f64, y: f64, depth: usize) {
        if self.count == 0 {
            self.com_x = x;
            self.com_y = y;
            self.count = 1;
            return;
        }

        // Stop subdividing at depth 50 to prevent infinite recursion when
        // two points share the same coordinates.
        if depth >= 50 {
            let total = self.count as f64 + 1.0;
            self.com_x = (self.com_x * self.count as f64 + x) / total;
            self.com_y = (self.com_y * self.count as f64 + y) / total;
            self.count += 1;
            return;
        }

        // If this is a leaf with one point, push the existing point down
        if self.count == 1 && self.children.iter().all(|c| c.is_none()) {
            let old_x = self.com_x;
            let old_y = self.com_y;
            self.push_down_depth(old_x, old_y, depth + 1);
        }

        // Update centre of mass
        let total = self.count as f64 + 1.0;
        self.com_x = (self.com_x * self.count as f64 + x) / total;
        self.com_y = (self.com_y * self.count as f64 + y) / total;
        self.count += 1;

        // Insert into appropriate quadrant
        self.push_down_depth(x, y, depth + 1);
    }

    fn push_down_depth(&mut self, x: f64, y: f64, depth: usize) {
        let qi = if x < self.cx {
            if y < self.cy {
                0
            } else {
                2
            }
        } else {
            if y < self.cy {
                1
            } else {
                3
            }
        };
        let child = self.children[qi].get_or_insert_with(|| {
            let h = self.half / 2.0;
            let ncx = if qi % 2 == 0 {
                self.cx - h
            } else {
                self.cx + h
            };
            let ncy = if qi < 2 { self.cy - h } else { self.cy + h };
            Box::new(QuadTree {
                cx: ncx,
                cy: ncy,
                half: h,
                com_x: 0.0,
                com_y: 0.0,
                count: 0,
                children: Default::default(),
            })
        });
        child.insert_depth(x, y, depth);
    }

    /// Compute repulsive force on point (px, py) from all points in the tree.
    /// `theta` is the Barnes-Hut opening angle (0.8 is typical).
    fn repulsive_force(&self, px: f64, py: f64, k: f64, theta: f64) -> (f64, f64) {
        if self.count == 0 {
            return (0.0, 0.0);
        }

        let dx = px - self.com_x;
        let dy = py - self.com_y;
        let dist = (dx * dx + dy * dy).sqrt();

        // If sufficiently far away, treat cluster as single point
        if self.half * 2.0 / dist.max(1e-9) < theta || self.count == 1 {
            if dist < 1e-6 {
                return (0.0, 0.0);
            }
            let force = k * k / (dist + 1e-6) * self.count as f64;
            return (dx / dist * force, dy / dist * force);
        }

        // Otherwise recurse into children
        let (mut fx, mut fy) = (0.0, 0.0);
        for c in self.children.iter().flatten() {
            let (cfx, cfy) = c.repulsive_force(px, py, k, theta);
            fx += cfx;
            fy += cfy;
        }
        (fx, fy)
    }
}

impl Default for QuadTree {
    fn default() -> Self {
        Self {
            cx: 0.0,
            cy: 0.0,
            half: 1.0,
            com_x: 0.0,
            com_y: 0.0,
            count: 0,
            children: Default::default(),
        }
    }
}
