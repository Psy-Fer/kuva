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

/// Which side of the body the column labels go on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColLabelSide {
    /// Above the body, between the column annotations and the cells.
    Top,
    /// Below the body (default).
    #[default]
    Bottom,
}

/// Which side of the body the row labels go on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RowLabelSide {
    /// Left of the body, between the row annotations and the cells.
    Left,
    /// Right of the body (default).
    #[default]
    Right,
}

/// Colour for a missing (empty) category value, in strips, cells and the legend.
pub const NA_COLOR: &str = "#cccccc";

/// Category settings shared by categorical annotation tracks and categorical cells: how
/// category values map to colours and legend text, and the order they are listed in.
#[derive(Clone, Debug, Default)]
pub struct CategoryStyle {
    /// Palette the categories take colours from, in category order. `None` = category10.
    pub palette: Option<CategoryPalette>,
    /// Explicit `(category, colour)` choices; these win over the palette.
    pub colors: Vec<(String, String)>,
    /// Legend text for a category, `(category, text)`; default is the category itself.
    pub labels: Vec<(String, String)>,
    /// Categories to list first, in this order. The rest follow in order of first appearance.
    pub order: Vec<String>,
    /// Legend text for missing values (default `"NA"`).
    pub na_label: Option<String>,
}

/// The colours of a palette, for categories. Built from a [`Palette`](crate::Palette)
/// (`Palette::okabe_ito().into()`, or pass the palette straight to `with_palette`) or from a
/// list of CSS colours.
#[derive(Clone, Debug, PartialEq)]
pub struct CategoryPalette(pub Vec<String>);

impl<S: Into<String>> From<Vec<S>> for CategoryPalette {
    fn from(colors: Vec<S>) -> Self {
        CategoryPalette(colors.into_iter().map(Into::into).collect())
    }
}

/// One resolved category: its value, display text and colour.
#[derive(Clone, Debug, PartialEq)]
pub struct CategoryEntry {
    pub value: String,
    pub label: String,
    pub color: String,
}

/// Whether a category value counts as missing: empty or only whitespace.
pub fn is_na(value: &str) -> bool {
    value.trim().is_empty()
}

/// Colour of `value` among resolved categories (NA colour for a missing value).
pub(crate) fn category_color<'a>(entries: &'a [CategoryEntry], value: &str) -> &'a str {
    if is_na(value) {
        return NA_COLOR;
    }
    entries
        .iter()
        .find(|e| e.value == value)
        .map_or(NA_COLOR, |e| e.color.as_str())
}

/// Where annotation strips sit relative to the labels on the same side of the body. Only
/// matters when they share a side: column strips with column labels on top, row strips with
/// row labels on the left.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AnnotationPlacement {
    /// Labels against the cells, strips outside them (default).
    #[default]
    OutsideLabels,
    /// Strips against the cells, labels outside them.
    NextToCells,
}

/// A colored strip of cells alongside the heatmap body, used to annotate
/// rows or columns with metadata (e.g. sample group, treatment, batch).
///
/// Build it either from **categories** with [`AnnotationTrack::categorical`], which picks
/// colours for you and adds a labelled legend, or from raw colours with
/// [`AnnotationTrack::new`].
#[derive(Clone, Debug)]
pub struct AnnotationTrack {
    /// One CSS color string per row/column, in the **original data order**
    /// (before clustering). The renderer reorders them automatically. Unused for a
    /// categorical track, whose colours come from `categories`.
    pub colors: Vec<String>,
    /// Optional label displayed above (column tracks) or to the left (row tracks).
    /// For a categorical track it is also the legend group's title.
    pub label: Option<String>,
    /// Width (row tracks) or height (col tracks) of the strip in pixels. Default 15.
    pub width: f64,
    /// Category value per row/column (original data order), for a categorical track.
    pub categories: Option<Vec<String>>,
    /// How categories map to colours and legend text.
    pub style: CategoryStyle,
}

impl AnnotationTrack {
    /// Create a track from an iterable of CSS color strings, one per row/column. It adds no
    /// legend entries; for that, use [`categorical`](Self::categorical).
    pub fn new(colors: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            colors: colors.into_iter().map(|s| s.into()).collect(),
            label: None,
            width: 15.0,
            categories: None,
            style: CategoryStyle::default(),
        }
    }

    /// Create a track from category values, one per row/column in original data order
    /// (`"ctrl"`, `"treat"`, ...). Each category gets a colour (category10 by default, see
    /// [`with_palette`](Self::with_palette)), and the clustermap adds a legend group, titled
    /// with the track's [`label`](Self::with_label), listing every category. An empty value is
    /// missing: grey, and listed as `NA` in the legend.
    pub fn categorical(values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            categories: Some(values.into_iter().map(|s| s.into()).collect()),
            ..Self::new(Vec::<String>::new())
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

    /// Palette for a categorical track's colours (default category10). Categories take
    /// palette colours in legend order.
    pub fn with_palette(mut self, palette: impl Into<CategoryPalette>) -> Self {
        self.style.palette = Some(palette.into());
        self
    }

    /// Fix the colour of one category in a categorical track.
    pub fn with_category_color(
        mut self,
        category: impl Into<String>,
        color: impl Into<String>,
    ) -> Self {
        self.style.colors.push((category.into(), color.into()));
        self
    }

    /// Legend text for one category in a categorical track (default: the category itself),
    /// e.g. `("treat", "Treated (10 uM)")`.
    pub fn with_category_label(
        mut self,
        category: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        self.style.labels.push((category.into(), label.into()));
        self
    }

    /// Legend text for missing (empty) values in this track, instead of `NA`.
    pub fn with_na_label(mut self, label: impl Into<String>) -> Self {
        self.style.na_label = Some(label.into());
        self
    }

    /// Order categories are listed in (and take palette colours in). Categories not named
    /// here follow in order of first appearance.
    pub fn with_category_order(
        mut self,
        order: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.style.order = order.into_iter().map(|s| s.into()).collect();
        self
    }
}

/// One bin of a [`DiscreteColorMap`]: values from `from` up to the next bin's `from`.
#[derive(Clone, Debug)]
pub struct DiscreteBin {
    pub from: f64,
    pub color: String,
    pub label: String,
}

/// Colours numeric cells by bin instead of along a gradient, with a labelled swatch legend:
/// e.g. copy number 0 / 1 / 2 / 3 / 4+.
///
/// Each bin covers values from its `from` up to the next bin's `from`; the last bin is open
/// ended. Values below the first bin, and NaN, are missing (grey, `NA` in the legend).
///
/// ```rust
/// use kuva::plot::clustermap::DiscreteColorMap;
/// let copy_number = DiscreteColorMap::new()
///     .with_bin(0.0, "#2166ac", "0 (deletion)")
///     .with_bin(1.0, "#92c5de", "1")
///     .with_bin(2.0, "#f7f7f7", "2 (normal)")
///     .with_bin(3.0, "#f4a582", "3")
///     .with_bin(4.0, "#b2182b", "4+");
/// assert_eq!(copy_number.bin_of(2.0).unwrap().label, "2 (normal)");
/// assert_eq!(copy_number.bin_of(7.0).unwrap().label, "4+");
/// assert!(copy_number.bin_of(-1.0).is_none());
/// ```
#[derive(Clone, Debug, Default)]
pub struct DiscreteColorMap {
    pub bins: Vec<DiscreteBin>,
}

impl DiscreteColorMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a bin starting at `from`. Bins may be added in any order.
    pub fn with_bin(
        mut self,
        from: f64,
        color: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        self.bins.push(DiscreteBin {
            from,
            color: color.into(),
            label: label.into(),
        });
        self.bins.sort_by(|a, b| a.from.total_cmp(&b.from));
        self
    }

    /// The bin `value` falls in, or `None` if it is below every bin or NaN.
    pub fn bin_of(&self, value: f64) -> Option<&DiscreteBin> {
        if value.is_nan() {
            return None;
        }
        self.bins.iter().rev().find(|b| value >= b.from)
    }

    /// Bins that at least one of `values` falls in, in bin order, as legend entries (plus
    /// `NA` last if any value is missing).
    pub fn entries<'a>(&self, values: impl IntoIterator<Item = &'a f64>) -> Vec<CategoryEntry> {
        let mut used = vec![false; self.bins.len()];
        let mut has_na = false;
        for &v in values {
            match self.bin_of(v) {
                Some(b) => {
                    let i = self.bins.iter().position(|x| std::ptr::eq(x, b)).unwrap();
                    used[i] = true;
                }
                None => has_na = true,
            }
        }
        let mut out: Vec<CategoryEntry> = self
            .bins
            .iter()
            .zip(used)
            .filter(|(_, u)| *u)
            .map(|(b, _)| CategoryEntry {
                value: b.label.clone(),
                label: b.label.clone(),
                color: b.color.clone(),
            })
            .collect();
        if has_na {
            out.push(CategoryEntry {
                value: String::new(),
                label: "NA".to_string(),
                color: NA_COLOR.to_string(),
            });
        }
        out
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

    /// Side of the body the column labels are drawn on. Default: [`ColLabelSide::Bottom`].
    pub col_label_side: ColLabelSide,
    /// Side of the body the row labels are drawn on. Default: [`RowLabelSide::Right`].
    pub row_label_side: RowLabelSide,
    /// Slant of the column labels in degrees from horizontal, `0` to `90`. Default `45`.
    pub col_label_rotation: f64,
    /// Add a legend group for each categorical annotation track. Default `true`.
    pub annotation_legend: bool,
    /// Strips outside the labels (default) or between the labels and the cells.
    pub annotation_placement: AnnotationPlacement,

    /// Categorical cell values (rows x columns), set by
    /// [`with_categorical_data`](Self::with_categorical_data). When set, cells are coloured by
    /// category and `data` is not used.
    pub categorical_data: Option<Vec<Vec<String>>>,
    /// How categorical cell values map to colours and legend text.
    pub cell_style: CategoryStyle,
    /// Bins for colouring numeric cells, instead of the gradient colour map.
    pub discrete_color_map: Option<DiscreteColorMap>,
    /// Pixel size of one cell `(width, height)`; when set, the canvas is sized to fit.
    pub cell_size: Option<(f64, f64)>,
    /// Gap between cells in pixels. `None` (default) leaves a hairline (cells fill 99%).
    pub cell_gap: Option<f64>,
}

impl Default for Clustermap {
    fn default() -> Self {
        Self::new()
    }
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
            col_label_side: ColLabelSide::Bottom,
            row_label_side: RowLabelSide::Right,
            col_label_rotation: 45.0,
            annotation_legend: true,
            annotation_placement: AnnotationPlacement::OutsideLabels,
            categorical_data: None,
            cell_style: CategoryStyle::default(),
            discrete_color_map: None,
            cell_size: None,
            cell_gap: None,
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

    /// Draw the column labels above or below the body (default below).
    ///
    /// On top, the labels sit directly against the cells, between the body and any column
    /// annotations and dendrogram, so each label starts at its own column.
    pub fn with_col_labels_position(mut self, side: ColLabelSide) -> Self {
        self.col_label_side = side;
        self
    }

    /// Draw the row labels left or right of the body (default right).
    ///
    /// On the left, the labels sit directly against the cells, between the body and any row
    /// annotations and dendrogram.
    pub fn with_row_labels_position(mut self, side: RowLabelSide) -> Self {
        self.row_label_side = side;
        self
    }

    /// Use categorical cell values (rows x columns of text: genotype calls, mutation types,
    /// states) instead of numeric data. Each category gets a colour (category10 by default;
    /// see [`with_category_colors`](Self::with_category_colors)) and a swatch legend replaces
    /// the colorbar, titled by [`with_legend`](Self::with_legend). An empty cell is missing
    /// (grey, `NA`).
    ///
    /// Clustering is turned off, since distances between labels mean nothing; supply a tree
    /// with [`with_row_tree`](Self::with_row_tree) / [`with_col_tree`](Self::with_col_tree) to
    /// order rows or columns. Normalization does not apply. With
    /// [`with_values`](Self::with_values), cells show their category text.
    pub fn with_categorical_data(
        mut self,
        data: impl IntoIterator<Item = impl IntoIterator<Item = impl Into<String>>>,
    ) -> Self {
        self.categorical_data = Some(
            data.into_iter()
                .map(|r| r.into_iter().map(|v| v.into()).collect())
                .collect(),
        );
        self.cluster_rows = false;
        self.cluster_cols = false;
        self
    }

    /// Fix the colour of categorical cell values: `[("het", "#4c9be8"), ...]`.
    pub fn with_category_colors(
        mut self,
        colors: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.cell_style
            .colors
            .extend(colors.into_iter().map(|(k, v)| (k.into(), v.into())));
        self
    }

    /// Legend text for categorical cell values: `[("het", "Heterozygous"), ...]`.
    pub fn with_category_labels(
        mut self,
        labels: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.cell_style
            .labels
            .extend(labels.into_iter().map(|(k, v)| (k.into(), v.into())));
        self
    }

    /// Legend order for categorical cell values; others follow in order of first appearance.
    pub fn with_category_order(
        mut self,
        order: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.cell_style.order = order.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Legend text for missing cells (empty categorical values, or numbers outside every
    /// bin of a [`DiscreteColorMap`]), instead of `NA`: e.g. `"No call"`.
    pub fn with_na_label(mut self, label: impl Into<String>) -> Self {
        self.cell_style.na_label = Some(label.into());
        self
    }

    /// Palette for categorical cell values (default category10).
    pub fn with_category_palette(mut self, palette: impl Into<CategoryPalette>) -> Self {
        self.cell_style.palette = Some(palette.into());
        self
    }

    /// Colour numeric cells by bin, with a labelled swatch legend instead of the colorbar.
    /// Clustering and normalization still apply to the numbers.
    pub fn with_discrete_color_map(mut self, map: DiscreteColorMap) -> Self {
        self.discrete_color_map = Some(map);
        self
    }

    /// Size every cell to `width` x `height` pixels and fit the canvas around the table,
    /// instead of squeezing the table into a fixed canvas. Useful for big tables: the figure
    /// grows with the matrix and labels stay readable. Applies when the canvas size is left to
    /// [`Layout::auto_from_plots`](crate::render::layout::Layout::auto_from_plots); an explicit
    /// `with_width` / `with_height` on the layout still wins for that dimension.
    pub fn with_cell_size(mut self, width: f64, height: f64) -> Self {
        self.cell_size = Some((width.max(1.0), height.max(1.0)));
        self
    }

    /// Gap between neighbouring cells, in pixels (also applied to annotation strips so they
    /// line up). `0.0` makes cells touch; the default is a hairline.
    pub fn with_cell_gap(mut self, gap: f64) -> Self {
        self.cell_gap = Some(gap.max(0.0));
        self
    }

    /// Whether cells are coloured along a gradient with a colorbar (numeric data, no
    /// discrete map), as opposed to by category with a swatch legend.
    pub fn uses_colorbar(&self) -> bool {
        self.categorical_data.is_none() && self.discrete_color_map.is_none()
    }

    /// `(rows, cols)` of the cell matrix, from whichever data is in use.
    pub fn dims(&self) -> (usize, usize) {
        match &self.categorical_data {
            Some(d) => (d.len(), d.first().map_or(0, Vec::len)),
            None => (self.data.len(), self.data.first().map_or(0, Vec::len)),
        }
    }

    /// A copy whose rows all have the first row's length and whose label lists
    /// match the matrix, or `None` when the shape is already consistent.
    ///
    /// The fields are public and the CLI reads files that may be ragged, so the
    /// renderer can't assume the documented shape. Short rows are padded (NaN,
    /// or an empty category), long rows and extra labels are cut, and missing
    /// labels are filled with their index, as when no labels are given.
    pub(crate) fn shape_fixed(&self) -> Option<Clustermap> {
        let (n_rows, n_cols) = self.dims();
        let rows_ok = match &self.categorical_data {
            Some(d) => d.iter().all(|r| r.len() == n_cols),
            None => self.data.iter().all(|r| r.len() == n_cols),
        };
        let labels_ok = |l: &Option<Vec<String>>, n: usize| l.as_ref().is_none_or(|l| l.len() == n);
        if rows_ok && labels_ok(&self.row_labels, n_rows) && labels_ok(&self.col_labels, n_cols) {
            return None;
        }
        let mut cm = self.clone();
        match &mut cm.categorical_data {
            Some(d) => d.iter_mut().for_each(|r| r.resize(n_cols, String::new())),
            None => cm.data.iter_mut().for_each(|r| r.resize(n_cols, f64::NAN)),
        }
        let fit = |l: &mut Option<Vec<String>>, n: usize| {
            if let Some(l) = l {
                l.truncate(n);
                let have = l.len();
                l.extend((have..n).map(|i| i.to_string()));
            }
        };
        fit(&mut cm.row_labels, n_rows);
        fit(&mut cm.col_labels, n_cols);
        Some(cm)
    }

    /// Put annotation strips outside the labels (default) or between the labels and the
    /// cells. It applies where strips and labels share a side: column strips when column labels
    /// are on top, row strips when row labels are on the left.
    pub fn with_annotation_placement(mut self, placement: AnnotationPlacement) -> Self {
        self.annotation_placement = placement;
        self
    }

    /// Show (default) or hide the legend groups that categorical annotation tracks add.
    pub fn with_annotation_legend(mut self, show: bool) -> Self {
        self.annotation_legend = show;
        self
    }

    /// Slant of the column labels, in degrees from horizontal: `0` is flat, `45` (the
    /// default) slanted, `90` vertical, which suits dense tables. The sign is ignored and the
    /// value is clamped to `0..=90`; labels always read upward from their column.
    pub fn with_col_label_rotation(mut self, degrees: f64) -> Self {
        self.col_label_rotation = degrees;
        self
    }
}
