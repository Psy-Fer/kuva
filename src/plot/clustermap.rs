use crate::plot::colormap::ColorMap;
use crate::plot::phylo::PhyloTree;

/// Normalization applied to the data matrix before color mapping.
#[derive(Clone, Debug, Default)]
pub enum ClustermapNorm {
    /// No normalization — raw values are color-mapped.
    #[default]
    None,
    /// Each row is z-score normalized (mean 0, std 1).
    RowZScore,
    /// Each column is z-score normalized (mean 0, std 1).
    ColZScore,
}

/// A colored strip of cells alongside the heatmap body, used to annotate
/// rows or columns with categorical metadata (e.g. sample group, treatment).
#[derive(Clone, Debug)]
pub struct AnnotationTrack {
    /// One CSS color string per row/column, in the **original data order**
    /// (before clustering). The renderer reorders them automatically.
    pub colors: Vec<String>,
    /// Optional label displayed above (column tracks) or to the left (row tracks).
    pub label: Option<String>,
    /// Width (row tracks) or height (col tracks) of the strip in pixels. Default 15.
    pub width: f64,
}

impl AnnotationTrack {
    /// Create a new annotation track from an iterable of CSS color strings.
    pub fn new(colors: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            colors: colors.into_iter().map(|s| s.into()).collect(),
            label: None,
            width: 15.0,
        }
    }

    /// Set a label for this track.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the width (row tracks) or height (col tracks) of the strip in pixels.
    pub fn with_width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }
}

/// A clustermap: a heatmap with integrated hierarchical clustering dendrograms.
///
/// Unlike `Heatmap + PhyloTree` in a `Figure`, `Clustermap` guarantees
/// pixel-perfect alignment between dendrogram leaves and heatmap cell centres
/// because both share the same internal layout computation.
///
/// Rows and columns are clustered automatically via UPGMA (Euclidean distance)
/// unless clustering is disabled or a pre-built `PhyloTree` is supplied.
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::clustermap::{Clustermap, ClustermapNorm, AnnotationTrack};
/// use kuva::plot::ColorMap;
/// use kuva::render::plots::Plot;
/// use kuva::render::layout::Layout;
/// use kuva::render::render::render_multiple;
/// use kuva::backend::svg::SvgBackend;
///
/// let data = vec![
///     vec![5.0, 1.0, 0.5],
///     vec![0.1, 4.0, 0.2],
///     vec![0.3, 0.2, 6.0],
/// ];
///
/// let cm = Clustermap::new()
///     .with_data(data)
///     .with_row_labels(["A", "B", "C"])
///     .with_col_labels(["X", "Y", "Z"])
///     .with_legend("Expression");
///
/// let plots = vec![Plot::Clustermap(cm)];
/// let layout = Layout::auto_from_plots(&plots).with_title("Clustermap");
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("clustermap.svg", svg).unwrap();
/// ```
#[derive(Clone)]
pub struct Clustermap {
    /// Rows × columns grid of values. All rows must have the same length.
    pub data: Vec<Vec<f64>>,
    /// Optional row labels in original data order.
    pub row_labels: Option<Vec<String>>,
    /// Optional column labels in original data order.
    pub col_labels: Option<Vec<String>>,

    /// Whether to cluster rows (default `true`).
    pub cluster_rows: bool,
    /// Whether to cluster columns (default `true`).
    pub cluster_cols: bool,

    /// Pre-computed row tree. Overrides auto-clustering when provided.
    pub row_tree: Option<PhyloTree>,
    /// Pre-computed column tree. Overrides auto-clustering when provided.
    pub col_tree: Option<PhyloTree>,

    /// Color map applied to normalized cell values. Default: `Viridis`.
    pub color_map: ColorMap,
    /// When `true`, display the numeric value inside each cell.
    pub show_values: bool,
    /// Normalization applied before color mapping.
    pub normalization: ClustermapNorm,
    /// Branch color for both dendrograms. Default: `"black"`.
    pub branch_color: String,

    /// Width in pixels of the row dendrogram panel. Default: `100.0`.
    pub row_dendrogram_width: f64,
    /// Height in pixels of the column dendrogram panel. Default: `80.0`.
    pub col_dendrogram_height: f64,

    /// Annotation tracks displayed to the right of the row dendrogram.
    pub row_annotations: Vec<AnnotationTrack>,
    /// Annotation tracks displayed below the column dendrogram.
    pub col_annotations: Vec<AnnotationTrack>,

    /// Label for the colorbar in the right margin.
    pub legend_label: Option<String>,
    /// Enable SVG tooltip overlays on heatmap cells.
    pub show_tooltips: bool,
}

impl Default for Clustermap {
    fn default() -> Self { Self::new() }
}

impl Clustermap {
    /// Create a clustermap with default settings.
    pub fn new() -> Self {
        Self {
            data: vec![],
            row_labels: None,
            col_labels: None,
            cluster_rows: true,
            cluster_cols: true,
            row_tree: None,
            col_tree: None,
            color_map: ColorMap::Viridis,
            show_values: false,
            normalization: ClustermapNorm::None,
            branch_color: "black".to_string(),
            row_dendrogram_width: 100.0,
            col_dendrogram_height: 80.0,
            row_annotations: vec![],
            col_annotations: vec![],
            legend_label: None,
            show_tooltips: false,
        }
    }

    /// Set the data matrix (rows × columns). Accepts any nested numeric iterables.
    pub fn with_data<U, T, I>(mut self, data: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: IntoIterator<Item = U>,
        U: Into<f64>,
    {
        let mut row: Vec<f64> = vec![];
        for r in data.into_iter() {
            for v in r {
                row.push(v.into());
            }
            self.data.push(row);
            row = vec![];
        }
        self
    }

    /// Set row labels in original data order (top to bottom).
    pub fn with_row_labels(mut self, labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.row_labels = Some(labels.into_iter().map(|s| s.into()).collect());
        self
    }

    /// Set column labels in original data order (left to right).
    pub fn with_col_labels(mut self, labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.col_labels = Some(labels.into_iter().map(|s| s.into()).collect());
        self
    }

    /// Enable or disable row clustering (default `true`).
    pub fn with_cluster_rows(mut self, v: bool) -> Self {
        self.cluster_rows = v;
        self
    }

    /// Enable or disable column clustering (default `true`).
    pub fn with_cluster_cols(mut self, v: bool) -> Self {
        self.cluster_cols = v;
        self
    }

    /// Provide a pre-built row tree instead of auto-clustering.
    ///
    /// The tree's leaf labels must match the row labels set via `with_row_labels`.
    pub fn with_row_tree(mut self, tree: PhyloTree) -> Self {
        self.row_tree = Some(tree);
        self
    }

    /// Provide a pre-built column tree instead of auto-clustering.
    ///
    /// The tree's leaf labels must match the column labels set via `with_col_labels`.
    pub fn with_col_tree(mut self, tree: PhyloTree) -> Self {
        self.col_tree = Some(tree);
        self
    }

    /// Set the color map (default [`ColorMap::Viridis`]).
    pub fn with_color_map(mut self, map: ColorMap) -> Self {
        self.color_map = map;
        self
    }

    /// Overlay raw numeric values inside each cell.
    pub fn with_values(mut self) -> Self {
        self.show_values = true;
        self
    }

    /// Set the normalization applied before color mapping.
    pub fn with_normalization(mut self, norm: ClustermapNorm) -> Self {
        self.normalization = norm;
        self
    }

    /// Set the branch color for both dendrograms (default `"black"`).
    pub fn with_branch_color(mut self, color: impl Into<String>) -> Self {
        self.branch_color = color.into();
        self
    }

    /// Set the pixel width of the row dendrogram panel (default `100.0`).
    pub fn with_row_dendrogram_width(mut self, w: f64) -> Self {
        self.row_dendrogram_width = w;
        self
    }

    /// Set the pixel height of the column dendrogram panel (default `80.0`).
    pub fn with_col_dendrogram_height(mut self, h: f64) -> Self {
        self.col_dendrogram_height = h;
        self
    }

    /// Add a row annotation track (displayed between the row dendrogram and the heatmap body).
    pub fn with_row_annotation(mut self, track: AnnotationTrack) -> Self {
        self.row_annotations.push(track);
        self
    }

    /// Add a column annotation track (displayed between the column dendrogram and the heatmap body).
    pub fn with_col_annotation(mut self, track: AnnotationTrack) -> Self {
        self.col_annotations.push(track);
        self
    }

    /// Set the colorbar legend label.
    pub fn with_legend(mut self, label: impl Into<String>) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    /// Enable SVG tooltip overlays showing cell values on hover.
    pub fn with_tooltips(mut self) -> Self {
        self.show_tooltips = true;
        self
    }
}
