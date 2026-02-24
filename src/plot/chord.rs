/// A chord diagram: N nodes arranged around a circle, connected by ribbons
/// whose widths are proportional to flow magnitudes from an N×N matrix.
///
/// Each node occupies an arc segment on the outer ring. The arc length is
/// proportional to the node's total flow (row sum of the matrix). Ribbons
/// connect pairs of nodes; ribbon width at each end is proportional to the
/// flow in that direction.
///
/// # Symmetric vs asymmetric matrices
///
/// - **Symmetric** (`matrix[i][j] == matrix[j][i]`): each ribbon has equal
///   width at both ends. Use this for undirected relationships such as
///   co-occurrence, correlation, or shared membership.
/// - **Asymmetric** (`matrix[i][j] != matrix[j][i]`): each ribbon is wider
///   at the end of the stronger sender. Use this for directed flows such as
///   migration, regulation, or transition probabilities.
///
/// # Pixel-space rendering
///
/// The chord diagram is rendered entirely in pixel space — it does not use
/// the standard x/y axis system. Pass the plot inside `render_multiple` as
/// usual; `Layout::auto_from_plots` skips axis computation for chord plots.
/// A title set on the `Layout` is still rendered.
///
/// # Example
///
/// ```rust,no_run
/// use visus::plot::ChordPlot;
/// use visus::backend::svg::SvgBackend;
/// use visus::render::render::render_multiple;
/// use visus::render::layout::Layout;
/// use visus::render::plots::Plot;
///
/// let matrix = vec![
///     vec![ 0.0, 120.0,  70.0],
///     vec![120.0,   0.0,  88.0],
///     vec![ 70.0,  88.0,   0.0],
/// ];
///
/// let chord = ChordPlot::new()
///     .with_matrix(matrix)
///     .with_labels(["CD4 T", "CD8 T", "NK"])
///     .with_legend("Cell types");
///
/// let plots = vec![Plot::Chord(chord)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Cell Type Co-clustering");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("chord.svg", svg).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct ChordPlot {
    /// N×N flow matrix. `matrix[i][j]` is the flow from node i to node j.
    /// Diagonal entries (`matrix[i][i]`) are typically zero.
    pub matrix: Vec<Vec<f64>>,
    /// Node labels — one per row/column of the matrix.
    pub labels: Vec<String>,
    /// Per-node fill colors. Must have the same length as `labels` when set.
    /// When empty, the `category10` palette is used as a fallback.
    pub colors: Vec<String>,
    /// Gap between adjacent arc segments in degrees (default `2.0`).
    pub gap_degrees: f64,
    /// Controls arc thickness: `inner_r = outer_r * pad_fraction` (default `0.85`).
    pub pad_fraction: f64,
    /// Ribbon fill opacity — `0.0` (transparent) to `1.0` (opaque) (default `0.7`).
    pub ribbon_opacity: f64,
    /// When `Some`, a legend box is rendered showing one color-coded entry per node.
    pub legend_label: Option<String>,
}

impl ChordPlot {
    /// Create a chord plot with default settings.
    ///
    /// Defaults: gap 2°, pad fraction 0.85, ribbon opacity 0.7, no legend,
    /// category10 color palette.
    pub fn new() -> Self {
        Self {
            matrix: vec![],
            labels: vec![],
            colors: vec![],
            gap_degrees: 2.0,
            pad_fraction: 0.85,
            ribbon_opacity: 0.7,
            legend_label: None,
        }
    }

    /// Set the N×N flow matrix.
    ///
    /// `matrix[i][j]` is the flow (or connection strength) from node i to
    /// node j. The matrix must be square. Diagonal entries are typically `0.0`
    /// (self-loops are not rendered).
    ///
    /// Arc lengths on the outer ring are proportional to each node's total
    /// outgoing flow (row sum). Ribbon widths at each end reflect the flow
    /// in each direction independently — so an asymmetric matrix produces
    /// ribbons that are thicker at the stronger source.
    ///
    /// ```rust,no_run
    /// # use visus::plot::ChordPlot;
    /// let chord = ChordPlot::new()
    ///     .with_matrix(vec![
    ///         vec![ 0.0, 50.0, 30.0],
    ///         vec![50.0,  0.0, 40.0],
    ///         vec![30.0, 40.0,  0.0],
    ///     ]);
    /// ```
    pub fn with_matrix(mut self, matrix: Vec<Vec<f64>>) -> Self {
        self.matrix = matrix;
        self
    }

    /// Set the node labels shown outside the arc segments.
    ///
    /// Must provide one label per row/column of the matrix. Labels are
    /// rendered as text outside the outer ring.
    ///
    /// ```rust,no_run
    /// # use visus::plot::ChordPlot;
    /// let chord = ChordPlot::new()
    ///     .with_matrix(vec![vec![0.0,1.0],vec![1.0,0.0]])
    ///     .with_labels(["Group A", "Group B"]);
    /// ```
    pub fn with_labels<S: Into<String>>(mut self, labels: impl IntoIterator<Item = S>) -> Self {
        self.labels = labels.into_iter().map(Into::into).collect();
        self
    }

    /// Set explicit per-node fill colors.
    ///
    /// Provide one color per node. Accepts any CSS color string (named,
    /// hex, `rgb(…)`). When not called, the `category10` palette is used
    /// automatically, cycling if there are more than ten nodes.
    ///
    /// ```rust,no_run
    /// # use visus::plot::ChordPlot;
    /// let chord = ChordPlot::new()
    ///     .with_matrix(vec![vec![0.0,1.0],vec![1.0,0.0]])
    ///     .with_labels(["Alpha", "Beta"])
    ///     .with_colors(["#e41a1c", "#377eb8"]);
    /// ```
    pub fn with_colors<S: Into<String>>(mut self, colors: impl IntoIterator<Item = S>) -> Self {
        self.colors = colors.into_iter().map(Into::into).collect();
        self
    }

    /// Set the gap between adjacent arc segments in degrees (default `2.0`).
    ///
    /// Larger values increase the white space between nodes, making individual
    /// arcs easier to distinguish. Total gap = `n_nodes * gap_degrees`; very
    /// large values compress the arc lengths.
    ///
    /// ```rust,no_run
    /// # use visus::plot::ChordPlot;
    /// # let chord = ChordPlot::new();
    /// let chord = chord.with_gap(5.0);  // wider separation between arcs
    /// ```
    pub fn with_gap(mut self, degrees: f64) -> Self {
        self.gap_degrees = degrees;
        self
    }

    /// Set the ribbon fill opacity (default `0.7`).
    ///
    /// Valid range is `0.0` (fully transparent) to `1.0` (fully opaque).
    /// Reducing opacity makes overlapping ribbons in dense diagrams easier
    /// to read by letting the arcs and other ribbons show through.
    ///
    /// ```rust,no_run
    /// # use visus::plot::ChordPlot;
    /// # let chord = ChordPlot::new();
    /// let chord = chord.with_opacity(0.5);
    /// ```
    pub fn with_opacity(mut self, f: f64) -> Self {
        self.ribbon_opacity = f;
        self
    }

    /// Enable a node legend.
    ///
    /// When set, a legend box is rendered showing one color-coded entry per
    /// node using the node labels and colors.
    ///
    /// ```rust,no_run
    /// # use visus::plot::ChordPlot;
    /// let chord = ChordPlot::new()
    ///     .with_matrix(vec![vec![0.0,1.0],vec![1.0,0.0]])
    ///     .with_labels(["A", "B"])
    ///     .with_legend("Nodes");
    /// ```
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    /// Return the number of nodes (inferred from matrix dimensions).
    pub fn n_nodes(&self) -> usize {
        self.matrix.len()
    }
}
