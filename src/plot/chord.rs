/// A chord diagram: N nodes arranged around a circle, connected by ribbons
/// whose widths are proportional to flow magnitudes from an N×N matrix.
#[derive(Debug, Clone)]
pub struct ChordPlot {
    /// N×N flow matrix; `matrix[i][j]` = flow from node i to node j.
    pub matrix: Vec<Vec<f64>>,
    /// Node labels (one per row/column).
    pub labels: Vec<String>,
    /// Per-node colors. Empty = use category10 palette fallback.
    pub colors: Vec<String>,
    /// Gap between arc segments in degrees (default 2.0).
    pub gap_degrees: f64,
    /// `inner_r = outer_r * pad_fraction` (default 0.85).
    pub pad_fraction: f64,
    /// Ribbon fill opacity (default 0.7).
    pub ribbon_opacity: f64,
    /// If set, adds one legend entry per node.
    pub legend_label: Option<String>,
}

impl ChordPlot {
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

    pub fn with_matrix(mut self, matrix: Vec<Vec<f64>>) -> Self {
        self.matrix = matrix;
        self
    }

    pub fn with_labels<S: Into<String>>(mut self, labels: impl IntoIterator<Item = S>) -> Self {
        self.labels = labels.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_colors<S: Into<String>>(mut self, colors: impl IntoIterator<Item = S>) -> Self {
        self.colors = colors.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_gap(mut self, degrees: f64) -> Self {
        self.gap_degrees = degrees;
        self
    }

    pub fn with_opacity(mut self, f: f64) -> Self {
        self.ribbon_opacity = f;
        self
    }

    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    /// Return the number of nodes (inferred from matrix dimensions).
    pub fn n_nodes(&self) -> usize {
        self.matrix.len()
    }
}
