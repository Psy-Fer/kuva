use crate::render::palette::Palette;

/// A single data cell in a mosaic (Marimekko) plot.
#[derive(Debug, Clone)]
pub struct MosaicCell {
    pub col: String,
    pub row: String,
    pub value: f64,
}

/// A mosaic / Marimekko chart.
///
/// Encodes two categorical variables simultaneously:
/// column widths are proportional to column totals, and segment heights within
/// each column represent the row breakdown of that column's total.
/// Each cell's *area* is proportional to its joint frequency.
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::mosaic::MosaicPlot;
/// use kuva::render::plots::Plot;
/// use kuva::render::layout::Layout;
/// use kuva::render::render::render_multiple;
/// use kuva::backend::svg::SvgBackend;
///
/// let plot = MosaicPlot::new()
///     .with_cell("Control", "Positive", 30.0)
///     .with_cell("Control", "Negative", 70.0)
///     .with_cell("Treated", "Positive", 60.0)
///     .with_cell("Treated", "Negative", 40.0)
///     .with_legend("Response");
///
/// let plots = vec![Plot::Mosaic(plot)];
/// let layout = Layout::auto_from_plots(&plots).with_title("Treatment vs Response");
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("mosaic.svg", svg).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct MosaicPlot {
    pub cells: Vec<MosaicCell>,
    /// Explicit column ordering. Empty = first-seen order from cells.
    pub col_order: Vec<String>,
    /// Explicit row/segment ordering. Empty = first-seen order from cells.
    pub row_order: Vec<String>,
    /// Per-row-category colors. Falls back to category10 palette.
    pub group_colors: Option<Vec<String>>,
    /// Pixel gap between columns and between segments (default: `2.0`).
    pub gap: f64,
    /// Show percentage labels inside cells (default: `true`).
    pub show_percents: bool,
    /// Show raw value labels inside cells (default: `false`).
    pub show_values: bool,
    /// Suppress labels when cell height is below this many pixels (default: `18.0`).
    pub min_label_height: f64,
    /// Suppress labels when cell width is below this many pixels (default: `30.0`).
    pub min_label_width: f64,
    /// Normalize each column to full plot height (default: `true`).
    /// When `false`, column heights are proportional to their share of the grand total.
    pub normalize: bool,
    /// Legend group title.
    pub legend_label: Option<String>,
}

impl Default for MosaicPlot {
    fn default() -> Self {
        Self::new()
    }
}

impl MosaicPlot {
    pub fn new() -> Self {
        Self {
            cells: vec![],
            col_order: vec![],
            row_order: vec![],
            group_colors: None,
            gap: 2.0,
            show_percents: true,
            show_values: false,
            min_label_height: 18.0,
            min_label_width: 30.0,
            normalize: true,
            legend_label: None,
        }
    }

    /// Add a single cell (col × row = value).
    pub fn with_cell(mut self, col: impl Into<String>, row: impl Into<String>, value: f64) -> Self {
        self.cells.push(MosaicCell { col: col.into(), row: row.into(), value });
        self
    }

    /// Add multiple cells at once.
    pub fn with_cells(
        mut self,
        cells: impl IntoIterator<Item = (impl Into<String>, impl Into<String>, f64)>,
    ) -> Self {
        for (col, row, val) in cells {
            self.cells.push(MosaicCell { col: col.into(), row: row.into(), value: val });
        }
        self
    }

    /// Set explicit column ordering.
    pub fn with_col_order(mut self, order: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.col_order = order.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Set explicit row/segment ordering.
    pub fn with_row_order(mut self, order: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.row_order = order.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Set per-row-category colors (indexed by row order).
    pub fn with_group_colors(mut self, colors: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.group_colors = Some(colors.into_iter().map(|c| c.into()).collect());
        self
    }

    /// Set the pixel gap between columns and between segments (default: `2.0`).
    pub fn with_gap(mut self, px: f64) -> Self {
        self.gap = px.max(0.0);
        self
    }

    /// Show percentage labels inside cells (default: `true`).
    pub fn with_percents(mut self, v: bool) -> Self {
        self.show_percents = v;
        self
    }

    /// Show raw value labels inside cells (default: `false`).
    pub fn with_values(mut self, v: bool) -> Self {
        self.show_values = v;
        self
    }

    /// Minimum cell height in pixels before labels are suppressed (default: `18.0`).
    pub fn with_min_label_height(mut self, px: f64) -> Self {
        self.min_label_height = px.max(0.0);
        self
    }

    /// Normalize each column to full plot height (default: `true`).
    pub fn with_normalize(mut self, v: bool) -> Self {
        self.normalize = v;
        self
    }

    /// Attach a legend with the given title (one entry per row category).
    pub fn with_legend(mut self, label: impl Into<String>) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    pub(crate) fn color_for_row_idx(&self, i: usize) -> String {
        if let Some(ref cv) = self.group_colors {
            if let Some(c) = cv.get(i) {
                if !c.is_empty() {
                    return c.clone();
                }
            }
        }
        let pal = Palette::category10();
        pal[i % pal.len()].to_string()
    }

    /// Column order: user-specified or first-seen from cells.
    pub(crate) fn effective_col_order(&self) -> Vec<String> {
        if !self.col_order.is_empty() {
            return self.col_order.clone();
        }
        let mut seen = std::collections::HashSet::new();
        let mut order = vec![];
        for cell in &self.cells {
            if seen.insert(cell.col.clone()) {
                order.push(cell.col.clone());
            }
        }
        order
    }

    /// Row order: user-specified or first-seen from cells.
    pub(crate) fn effective_row_order(&self) -> Vec<String> {
        if !self.row_order.is_empty() {
            return self.row_order.clone();
        }
        let mut seen = std::collections::HashSet::new();
        let mut order = vec![];
        for cell in &self.cells {
            if seen.insert(cell.row.clone()) {
                order.push(cell.row.clone());
            }
        }
        order
    }

    /// Sum of all values in a column.
    pub(crate) fn col_total(&self, col: &str) -> f64 {
        self.cells.iter()
            .filter(|c| c.col == col)
            .map(|c| c.value)
            .sum()
    }

    /// Value for a specific (col, row) pair (sums duplicates).
    pub(crate) fn cell_value(&self, col: &str, row: &str) -> f64 {
        self.cells.iter()
            .filter(|c| c.col == col && c.row == row)
            .map(|c| c.value)
            .sum()
    }
}
