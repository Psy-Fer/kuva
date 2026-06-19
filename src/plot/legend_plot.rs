use crate::plot::legend::LegendEntry;

/// A plot cell that renders only a legend — no axes, no data.
///
/// `LegendPlot` is designed for two situations:
///
/// 1. **Shared legend in a figure.** Place it in any adjacent cell so multiple
///    data panels can share one legend without duplicating it.
/// 2. **Standalone use.** Render a freestanding legend key outside of any figure,
///    e.g. to composite with an externally produced image.
///
/// Entries can be supplied directly or collected from a set of plots via
/// [`collect_legend_entries`](crate::render::render::collect_legend_entries).
///
/// # Column layout
///
/// By default the number of columns is chosen automatically: the renderer
/// estimates each entry's pixel width from the longest label (using a 0.68
/// character-width factor at the current font size) and packs as many columns
/// as fit across the available cell width.  Call [`with_cols`](Self::with_cols)
/// to pin the column count instead.
///
/// # Examples
///
/// ## Shared legend below a scatter plot
///
/// ```rust
/// use kuva::prelude::*;
/// use kuva::render::render::collect_legend_entries;
///
/// let scatter = ScatterPlot::new()
///     .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
///     .with_color("steelblue");
///
/// // Collect entries from the data plots, then hand them to a LegendPlot cell.
/// let data_plots: Vec<Plot> = vec![scatter.into()];
/// let entries = collect_legend_entries(&data_plots);
///
/// let legend_cell = LegendPlot::from_entries(entries);
///
/// let fig = Figure::new(2, 1)          // two rows, one column
///     .with_cell_size(600.0, 400.0)
///     .with_plots(vec![
///         data_plots,                   // row 0 — the chart
///         vec![legend_cell.into()],     // row 1 — the legend
///     ]);
/// let svg = fig.render();
/// # let _ = svg;
/// ```
///
/// ## Legend to the right of the chart
///
/// ```rust
/// use kuva::prelude::*;
/// use kuva::render::render::collect_legend_entries;
///
/// let scatter = ScatterPlot::new()
///     .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
///     .with_color("steelblue");
///
/// let data_plots: Vec<Plot> = vec![scatter.into()];
/// let entries = collect_legend_entries(&data_plots);
///
/// let legend_cell = LegendPlot::from_entries(entries).with_cols(1);
///
/// let fig = Figure::new(1, 2)          // one row, two columns
///     .with_cell_size(500.0, 400.0)
///     .with_plots(vec![
///         data_plots,
///         vec![legend_cell.into()],
///     ]);
/// # let _ = fig.render();
/// ```
///
/// ## Manual entries with a title
///
/// ```rust
/// use kuva::prelude::*;
///
/// let entries = vec![
///     LegendEntry { label: "Treatment".into(), color: "#4477AA".into(),
///                   shape: LegendShape::Rect, dasharray: None },
///     LegendEntry { label: "Control".into(),   color: "#EE6677".into(),
///                   shape: LegendShape::Rect, dasharray: None },
/// ];
///
/// let lp = LegendPlot::from_entries(entries)
///     .with_title("Groups")
///     .with_cols(2);
/// # let _ = lp;
/// ```
pub struct LegendPlot {
    pub entries: Vec<LegendEntry>,
    /// Fixed column count. `None` = auto-detect from available width.
    pub cols: Option<usize>,
    /// Maximum number of columns. `None` = no cap. Applied after auto-detection
    /// and the vertical-fit bump-up loop, so labels never get squeezed below
    /// this column budget.
    pub max_cols: Option<usize>,
    /// Maximum number of entries to display. Entries beyond this are replaced
    /// with a "… (+N more)" line. `None` = show all.
    pub max_entries: Option<usize>,
    pub title: Option<String>,
    pub show_box: bool,
}

impl LegendPlot {
    /// Create an empty `LegendPlot`. Add entries with [`with_entry`](Self::with_entry).
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            cols: None,
            max_cols: None,
            max_entries: None,
            title: None,
            show_box: true,
        }
    }

    /// Create a `LegendPlot` pre-populated with `entries`.
    ///
    /// Use [`collect_legend_entries`](crate::render::render::collect_legend_entries)
    /// to derive entries automatically from a set of plots.
    pub fn from_entries(entries: Vec<LegendEntry>) -> Self {
        Self {
            entries,
            cols: None,
            max_cols: None,
            max_entries: None,
            title: None,
            show_box: true,
        }
    }

    /// Append a single entry.
    pub fn with_entry(mut self, entry: LegendEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Fix the number of columns.
    ///
    /// When omitted, columns are auto-computed from the cell width and the
    /// widest label using a conservative all-caps character-width estimate.
    pub fn with_cols(mut self, n: usize) -> Self {
        self.cols = Some(n);
        self
    }

    /// Cap the maximum number of columns used during auto-layout.
    ///
    /// The auto-layout may bump the column count upward to make all rows fit
    /// within the cell height. This cap prevents that bump from squeezing
    /// long labels into columns that are too narrow.
    pub fn with_max_cols(mut self, n: usize) -> Self {
        self.max_cols = Some(n);
        self
    }

    /// Limit the number of entries displayed. Entries beyond `n` are replaced
    /// with a "… (+M more)" line in the next grid slot.
    pub fn with_max_entries(mut self, n: usize) -> Self {
        self.max_entries = Some(n);
        self
    }

    /// Add a bold title above the entries.
    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Hide the background box and border drawn around the legend.
    pub fn without_box(mut self) -> Self {
        self.show_box = false;
        self
    }
}

impl Default for LegendPlot {
    fn default() -> Self {
        Self::new()
    }
}
