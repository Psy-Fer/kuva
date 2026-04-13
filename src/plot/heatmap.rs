use std::sync::Arc;
use colorous::{
    // Sequential multi-hue (perceptual)
    TURBO, VIRIDIS, INFERNO, MAGMA, PLASMA, CIVIDIS, WARM, COOL, CUBEHELIX,
    // Sequential multi-hue (ColorBrewer)
    BLUE_GREEN, BLUE_PURPLE, GREEN_BLUE, ORANGE_RED,
    PURPLE_BLUE_GREEN, PURPLE_BLUE, PURPLE_RED, RED_PURPLE,
    YELLOW_GREEN_BLUE, YELLOW_GREEN, YELLOW_ORANGE_BROWN, YELLOW_ORANGE_RED,
    // Sequential single-hue
    BLUES, GREENS, GREYS, ORANGES, PURPLES, REDS,
    // Diverging
    BROWN_GREEN, PURPLE_GREEN, PINK_GREEN, PURPLE_ORANGE,
    RED_BLUE, RED_GREY, RED_YELLOW_BLUE, RED_YELLOW_GREEN, SPECTRAL,
    // Cyclical
    RAINBOW, SINEBOW,
};

const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// Convert an RGB triplet to a 7-byte hex color string (`#rrggbb`).
/// Avoids `format!` overhead in hot loops (heatmaps, 2D histograms).
#[inline]
fn rgb_hex(r: u8, g: u8, b: u8) -> String {
    let bytes = [
        b'#',
        HEX_DIGITS[(r >> 4) as usize],
        HEX_DIGITS[(r & 0xf) as usize],
        HEX_DIGITS[(g >> 4) as usize],
        HEX_DIGITS[(g & 0xf) as usize],
        HEX_DIGITS[(b >> 4) as usize],
        HEX_DIGITS[(b & 0xf) as usize],
    ];
    // SAFETY: all bytes are ASCII
    unsafe { String::from_utf8_unchecked(bytes.to_vec()) }
}

fn cmap_str(gradient: colorous::Gradient, value: f64) -> String {
    let rgb = gradient.eval_continuous(value.clamp(0.0, 1.0));
    rgb_hex(rgb.r, rgb.g, rgb.b)
}

fn cmap_rgb(gradient: colorous::Gradient, value: f64) -> (u8, u8, u8) {
    let rgb = gradient.eval_continuous(value.clamp(0.0, 1.0));
    (rgb.r, rgb.g, rgb.b)
}

/// Color map used to encode numeric cell values as colors.
///
/// Values are normalized to `[0.0, 1.0]` relative to the data min/max before
/// the map is applied. The same `ColorMap` type is shared by [`Heatmap`],
/// [`Histogram2D`](crate::plot::Histogram2D), and [`CalendarPlot`](crate::plot::CalendarPlot).
///
/// # Choosing a color map
///
/// | Category | Variants | Use when |
/// |----------|----------|----------|
/// | Sequential (perceptual) | `Viridis`, `Inferno`, `Magma`, `Plasma`, `Cividis`, `Turbo`, `Warm`, `Cool`, `Cubehelix` | General-purpose continuous data; colorblind-safe options |
/// | Sequential (ColorBrewer) | `BlueGreen`, `BluePurple`, `GreenBlue`, `OrangeRed`, `PurpleBlue`, `PurpleBlueGreen`, `PurpleRed`, `RedPurple`, `YellowGreen`, `YellowGreenBlue`, `YellowOrangeBrown`, `YellowOrangeRed` | Themed sequential scales from [ColorBrewer](https://colorbrewer2.org/) |
/// | Sequential (single-hue) | `Blues`, `Greens`, `Grayscale`, `Oranges`, `Purples`, `Reds` | Monochromatic; print-friendly |
/// | Diverging | `BrownGreen`, `PinkGreen`, `PurpleGreen`, `PurpleOrange`, `RedBlue`, `RedGrey`, `RedYellowBlue`, `RedYellowGreen`, `Spectral` | Data with a meaningful midpoint (e.g. fold-change, correlation) |
/// | Cyclical | `Rainbow`, `Sinebow` | Periodic data (phase, angle, time-of-day) |
/// | Custom | `Custom` | Full control over color encoding |
#[derive(Clone)]
pub enum ColorMap {
    // ── Sequential multi-hue (perceptual) ──────────────────────────────────
    /// Improved rainbow; perceptually uniform; colorblind-safe.
    Turbo,
    /// Blue → green → yellow; perceptually uniform; default.
    Viridis,
    /// Black → purple → yellow; high-contrast; works in greyscale.
    Inferno,
    /// Black → purple → orange; similar to Inferno.
    Magma,
    /// Blue → purple → yellow; bright and perceptually uniform.
    Plasma,
    /// Blue → grey → yellow; optimized for color-vision deficiency.
    Cividis,
    /// Warm perceptual rainbow (180° rotation of Cool).
    Warm,
    /// Cool perceptual rainbow.
    Cool,
    /// Green's default Cubehelix spiral.
    Cubehelix,

    // ── Sequential multi-hue (ColorBrewer) ────────────────────────────────
    /// White → blue-green.
    BlueGreen,
    /// White → blue-purple.
    BluePurple,
    /// White → green-blue.
    GreenBlue,
    /// White → orange-red.
    OrangeRed,
    /// White → purple-blue-green.
    PurpleBlueGreen,
    /// White → purple-blue.
    PurpleBlue,
    /// White → purple-red.
    PurpleRed,
    /// White → red-purple.
    RedPurple,
    /// White → yellow-green-blue.
    YellowGreenBlue,
    /// White → yellow-green.
    YellowGreen,
    /// White → yellow-orange-brown.
    YellowOrangeBrown,
    /// White → yellow-orange-red.
    YellowOrangeRed,

    // ── Sequential single-hue ─────────────────────────────────────────────
    /// White → blue.
    Blues,
    /// White → green.
    Greens,
    /// White → black; print-friendly.
    Grayscale,
    /// White → orange.
    Oranges,
    /// White → purple.
    Purples,
    /// White → red.
    Reds,

    // ── Diverging ─────────────────────────────────────────────────────────
    /// Brown ← 0 → green.
    BrownGreen,
    /// Pink ← 0 → green.
    PinkGreen,
    /// Purple ← 0 → green.
    PurpleGreen,
    /// Purple ← 0 → orange.
    PurpleOrange,
    /// Red ← 0 → blue.
    RedBlue,
    /// Red ← 0 → grey.
    RedGrey,
    /// Red ← 0 → yellow → blue.
    RedYellowBlue,
    /// Red ← 0 → yellow → green.
    RedYellowGreen,
    /// Red → orange → yellow → green → blue → purple.
    Spectral,

    // ── Cyclical ──────────────────────────────────────────────────────────
    /// Less-angry rainbow; suitable for cyclical data.
    Rainbow,
    /// Smooth sinusoidal rainbow.
    Sinebow,

    // ── Custom ────────────────────────────────────────────────────────────
    /// User-defined mapping from a normalized `[0.0, 1.0]` value to a CSS
    /// color string. Wrap the function in `Arc` for cloneability.
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use kuva::plot::ColorMap;
    ///
    /// // Custom blue-to-red diverging scale
    /// let cmap = ColorMap::Custom(Arc::new(|t: f64| {
    ///     let r = (t * 255.0) as u8;
    ///     let b = ((1.0 - t) * 255.0) as u8;
    ///     format!("rgb({r},0,{b})")
    /// }));
    /// ```
    Custom(Arc<dyn Fn(f64) -> String + Send + Sync>),
}

impl std::fmt::Debug for ColorMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ColorMap::Turbo            => "Turbo",
            ColorMap::Viridis          => "Viridis",
            ColorMap::Inferno          => "Inferno",
            ColorMap::Magma            => "Magma",
            ColorMap::Plasma           => "Plasma",
            ColorMap::Cividis          => "Cividis",
            ColorMap::Warm             => "Warm",
            ColorMap::Cool             => "Cool",
            ColorMap::Cubehelix        => "Cubehelix",
            ColorMap::BlueGreen        => "BlueGreen",
            ColorMap::BluePurple       => "BluePurple",
            ColorMap::GreenBlue        => "GreenBlue",
            ColorMap::OrangeRed        => "OrangeRed",
            ColorMap::PurpleBlueGreen  => "PurpleBlueGreen",
            ColorMap::PurpleBlue       => "PurpleBlue",
            ColorMap::PurpleRed        => "PurpleRed",
            ColorMap::RedPurple        => "RedPurple",
            ColorMap::YellowGreenBlue  => "YellowGreenBlue",
            ColorMap::YellowGreen      => "YellowGreen",
            ColorMap::YellowOrangeBrown => "YellowOrangeBrown",
            ColorMap::YellowOrangeRed  => "YellowOrangeRed",
            ColorMap::Blues            => "Blues",
            ColorMap::Greens           => "Greens",
            ColorMap::Grayscale        => "Grayscale",
            ColorMap::Oranges          => "Oranges",
            ColorMap::Purples          => "Purples",
            ColorMap::Reds             => "Reds",
            ColorMap::BrownGreen       => "BrownGreen",
            ColorMap::PinkGreen        => "PinkGreen",
            ColorMap::PurpleGreen      => "PurpleGreen",
            ColorMap::PurpleOrange     => "PurpleOrange",
            ColorMap::RedBlue          => "RedBlue",
            ColorMap::RedGrey          => "RedGrey",
            ColorMap::RedYellowBlue    => "RedYellowBlue",
            ColorMap::RedYellowGreen   => "RedYellowGreen",
            ColorMap::Spectral         => "Spectral",
            ColorMap::Rainbow          => "Rainbow",
            ColorMap::Sinebow          => "Sinebow",
            ColorMap::Custom(_)        => return write!(f, "ColorMap::Custom(<fn>)"),
        };
        write!(f, "ColorMap::{name}")
    }
}

impl ColorMap {
    /// Map a normalized value in `[0.0, 1.0]` to a CSS color string.
    pub fn map(&self, value: f64) -> String {
        match self {
            ColorMap::Turbo            => cmap_str(TURBO,              value),
            ColorMap::Viridis          => cmap_str(VIRIDIS,            value),
            ColorMap::Inferno          => cmap_str(INFERNO,            value),
            ColorMap::Magma            => cmap_str(MAGMA,              value),
            ColorMap::Plasma           => cmap_str(PLASMA,             value),
            ColorMap::Cividis          => cmap_str(CIVIDIS,            value),
            ColorMap::Warm             => cmap_str(WARM,               value),
            ColorMap::Cool             => cmap_str(COOL,               value),
            ColorMap::Cubehelix        => cmap_str(CUBEHELIX,          value),
            ColorMap::BlueGreen        => cmap_str(BLUE_GREEN,         value),
            ColorMap::BluePurple       => cmap_str(BLUE_PURPLE,        value),
            ColorMap::GreenBlue        => cmap_str(GREEN_BLUE,         value),
            ColorMap::OrangeRed        => cmap_str(ORANGE_RED,         value),
            ColorMap::PurpleBlueGreen  => cmap_str(PURPLE_BLUE_GREEN,  value),
            ColorMap::PurpleBlue       => cmap_str(PURPLE_BLUE,        value),
            ColorMap::PurpleRed        => cmap_str(PURPLE_RED,         value),
            ColorMap::RedPurple        => cmap_str(RED_PURPLE,         value),
            ColorMap::YellowGreenBlue  => cmap_str(YELLOW_GREEN_BLUE,  value),
            ColorMap::YellowGreen      => cmap_str(YELLOW_GREEN,       value),
            ColorMap::YellowOrangeBrown => cmap_str(YELLOW_ORANGE_BROWN, value),
            ColorMap::YellowOrangeRed  => cmap_str(YELLOW_ORANGE_RED,  value),
            ColorMap::Blues            => cmap_str(BLUES,              value),
            ColorMap::Greens           => cmap_str(GREENS,             value),
            ColorMap::Grayscale        => cmap_str(GREYS,              value),
            ColorMap::Oranges          => cmap_str(ORANGES,            value),
            ColorMap::Purples          => cmap_str(PURPLES,            value),
            ColorMap::Reds             => cmap_str(REDS,               value),
            ColorMap::BrownGreen       => cmap_str(BROWN_GREEN,        value),
            ColorMap::PinkGreen        => cmap_str(PINK_GREEN,         value),
            ColorMap::PurpleGreen      => cmap_str(PURPLE_GREEN,       value),
            ColorMap::PurpleOrange     => cmap_str(PURPLE_ORANGE,      value),
            ColorMap::RedBlue          => cmap_str(RED_BLUE,           value),
            ColorMap::RedGrey          => cmap_str(RED_GREY,           value),
            ColorMap::RedYellowBlue    => cmap_str(RED_YELLOW_BLUE,    value),
            ColorMap::RedYellowGreen   => cmap_str(RED_YELLOW_GREEN,   value),
            ColorMap::Spectral         => cmap_str(SPECTRAL,           value),
            ColorMap::Rainbow          => cmap_str(RAINBOW,            value),
            ColorMap::Sinebow          => cmap_str(SINEBOW,            value),
            ColorMap::Custom(f)        => f(value),
        }
    }

    /// Map a normalized value to `(r, g, b)` bytes, avoiding string allocation.
    /// Returns `None` for `Custom` colormaps (which must go through `map()`).
    pub fn map_rgb(&self, value: f64) -> Option<(u8, u8, u8)> {
        Some(match self {
            ColorMap::Turbo            => cmap_rgb(TURBO,              value),
            ColorMap::Viridis          => cmap_rgb(VIRIDIS,            value),
            ColorMap::Inferno          => cmap_rgb(INFERNO,            value),
            ColorMap::Magma            => cmap_rgb(MAGMA,              value),
            ColorMap::Plasma           => cmap_rgb(PLASMA,             value),
            ColorMap::Cividis          => cmap_rgb(CIVIDIS,            value),
            ColorMap::Warm             => cmap_rgb(WARM,               value),
            ColorMap::Cool             => cmap_rgb(COOL,               value),
            ColorMap::Cubehelix        => cmap_rgb(CUBEHELIX,          value),
            ColorMap::BlueGreen        => cmap_rgb(BLUE_GREEN,         value),
            ColorMap::BluePurple       => cmap_rgb(BLUE_PURPLE,        value),
            ColorMap::GreenBlue        => cmap_rgb(GREEN_BLUE,         value),
            ColorMap::OrangeRed        => cmap_rgb(ORANGE_RED,         value),
            ColorMap::PurpleBlueGreen  => cmap_rgb(PURPLE_BLUE_GREEN,  value),
            ColorMap::PurpleBlue       => cmap_rgb(PURPLE_BLUE,        value),
            ColorMap::PurpleRed        => cmap_rgb(PURPLE_RED,         value),
            ColorMap::RedPurple        => cmap_rgb(RED_PURPLE,         value),
            ColorMap::YellowGreenBlue  => cmap_rgb(YELLOW_GREEN_BLUE,  value),
            ColorMap::YellowGreen      => cmap_rgb(YELLOW_GREEN,       value),
            ColorMap::YellowOrangeBrown => cmap_rgb(YELLOW_ORANGE_BROWN, value),
            ColorMap::YellowOrangeRed  => cmap_rgb(YELLOW_ORANGE_RED,  value),
            ColorMap::Blues            => cmap_rgb(BLUES,              value),
            ColorMap::Greens           => cmap_rgb(GREENS,             value),
            ColorMap::Grayscale        => cmap_rgb(GREYS,              value),
            ColorMap::Oranges          => cmap_rgb(ORANGES,            value),
            ColorMap::Purples          => cmap_rgb(PURPLES,            value),
            ColorMap::Reds             => cmap_rgb(REDS,               value),
            ColorMap::BrownGreen       => cmap_rgb(BROWN_GREEN,        value),
            ColorMap::PinkGreen        => cmap_rgb(PINK_GREEN,         value),
            ColorMap::PurpleGreen      => cmap_rgb(PURPLE_GREEN,       value),
            ColorMap::PurpleOrange     => cmap_rgb(PURPLE_ORANGE,      value),
            ColorMap::RedBlue          => cmap_rgb(RED_BLUE,           value),
            ColorMap::RedGrey          => cmap_rgb(RED_GREY,           value),
            ColorMap::RedYellowBlue    => cmap_rgb(RED_YELLOW_BLUE,    value),
            ColorMap::RedYellowGreen   => cmap_rgb(RED_YELLOW_GREEN,   value),
            ColorMap::Spectral         => cmap_rgb(SPECTRAL,           value),
            ColorMap::Rainbow          => cmap_rgb(RAINBOW,            value),
            ColorMap::Sinebow          => cmap_rgb(SINEBOW,            value),
            ColorMap::Custom(_)        => return None,
        })
    }
}

/// Builder for a heatmap.
///
/// Renders a two-dimensional grid of colored cells. Cell color encodes the
/// numeric value — each cell is mapped through a [`ColorMap`] after
/// normalizing values to `[0.0, 1.0]` relative to the data range. A colorbar
/// is always shown in the right margin.
///
/// ## Axis labels
///
/// To display axis tick labels, pass them to
/// [`Layout::with_x_categories`](crate::render::layout::Layout::with_x_categories)
/// (column labels) and
/// [`Layout::with_y_categories`](crate::render::layout::Layout::with_y_categories)
/// (row labels).
///
/// ## Row / column reordering (e.g. phylogenetic alignment)
///
/// Call [`with_labels`](Heatmap::with_labels) first to associate each row and
/// column with a name. Then call [`with_y_categories`](Heatmap::with_y_categories)
/// or [`with_x_categories`](Heatmap::with_x_categories) with the desired order
/// to **reorder the data matrix in-place** and update the stored labels.
///
/// ```rust,no_run
/// use kuva::plot::{Heatmap, PhyloTree};
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let labels: Vec<String> = ["A","B","C","D","E"].iter().map(|s| s.to_string()).collect();
/// let data = vec![
///     vec![0.0, 1.0, 1.0, 1.0, 1.0],
///     vec![1.0, 0.0, 0.4, 1.0, 1.0],
///     vec![1.0, 0.4, 0.0, 1.0, 1.0],
///     vec![1.0, 1.0, 1.0, 0.0, 1.0],
///     vec![1.0, 1.0, 1.0, 1.0, 0.0],
/// ];
///
/// let label_refs: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
/// let tree = PhyloTree::from_distance_matrix(&label_refs, &data);
/// let leaf_order = tree.leaf_labels_top_to_bottom();
///
/// let heatmap = Heatmap::new()
///     .with_data(data)
///     .with_labels(labels, vec![])    // record original row order
///     .with_y_categories(leaf_order); // first leaf → top of heatmap
///
/// // row_labels is stored bottom-to-top — pass to Layout directly
/// let layout_cats = heatmap.row_labels.clone().unwrap();
/// let plots: Vec<Plot> = vec![Plot::PhyloTree(tree), Plot::Heatmap(heatmap)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_y_categories(layout_cats); // axis tick labels in matching order
/// ```
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::{Heatmap, ColorMap};
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let data = vec![
///     vec![0.8, 0.3, 0.9],
///     vec![0.4, 0.7, 0.1],
///     vec![0.5, 0.9, 0.4],
/// ];
///
/// let heatmap = Heatmap::new()
///     .with_data(data)
///     .with_color_map(ColorMap::Viridis);
///
/// let plots = vec![Plot::Heatmap(heatmap)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Heatmap")
///     .with_x_categories(vec!["A".into(), "B".into(), "C".into()])
///     .with_y_categories(vec!["X".into(), "Y".into(), "Z".into()]);
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("heatmap.svg", svg).unwrap();
/// ```
#[derive(Clone)]
pub struct Heatmap {
    /// Rows × columns grid of values. All rows must have the same length.
    pub data: Vec<Vec<f64>>,
    /// Optional row labels — stored in the struct but rendered via
    /// `Layout::with_y_categories`.
    pub row_labels: Option<Vec<String>>,
    /// Optional column labels — stored in the struct but rendered via
    /// `Layout::with_x_categories`.
    pub col_labels: Option<Vec<String>>,
    /// Color map applied after normalizing values to `[0.0, 1.0]`.
    /// Defaults to [`ColorMap::Viridis`].
    pub color_map: ColorMap,
    /// When `true`, each cell displays its raw numeric value as text.
    pub show_values: bool,
    pub legend_label: Option<String>,
    pub show_tooltips: bool,
    pub tooltip_labels: Option<Vec<String>>,
}


impl Default for Heatmap {
    fn default() -> Self { Self::new() }
}

impl Heatmap {
    /// Create a heatmap with default settings.
    ///
    /// Defaults: Viridis color map, no value overlay, no labels.
    pub fn new() -> Self {
        Self {
            data: vec![],
            row_labels: None,
            col_labels: None,
            color_map: ColorMap::Viridis,
            show_values: false,
            legend_label: None,
            show_tooltips: false,
            tooltip_labels: None,
        }
    }

    /// Set the grid data.
    ///
    /// Accepts any iterable of iterables of numeric values. The outer iterator
    /// produces rows (top to bottom); the inner iterator produces columns
    /// (left to right). All rows must have the same number of columns.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::Heatmap;
    /// let heatmap = Heatmap::new().with_data(vec![
    ///     vec![1.0, 2.0, 3.0],
    ///     vec![4.0, 5.0, 6.0],
    /// ]);
    /// ```
    // accept data of any numerical type and push it to f64
    pub fn with_data<U, T, I>(mut self, data: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: IntoIterator<Item = U>,
        U: Into<f64>,
    {
        let mut a: Vec<f64> = vec![];
        for d in data.into_iter() {
            for v in d {
                a.push(v.into())
            }
            self.data.push(a);
            a = vec![];
        }
        self
    }

    /// Store row and column label strings in the struct.
    ///
    /// These labels are used for tooltip text and as the reference mapping for
    /// [`with_y_categories`](Heatmap::with_y_categories) /
    /// [`with_x_categories`](Heatmap::with_x_categories) row/column reordering.
    /// To display them as axis tick labels, also pass them to
    /// [`Layout::with_y_categories`](crate::render::layout::Layout::with_y_categories)
    /// and [`Layout::with_x_categories`](crate::render::layout::Layout::with_x_categories).
    pub fn with_labels(mut self, rows: Vec<String>, cols: Vec<String>) -> Self {
        self.row_labels = Some(rows);
        self.col_labels = Some(cols);
        self
    }

    /// Reorder heatmap rows so that `desired_order[0]` appears at the **top** of
    /// the rendered heatmap and `desired_order[N-1]` at the bottom.
    ///
    /// `desired_order` is interpreted as **top-to-bottom** — matching the convention
    /// of [`PhyloTree::leaf_labels_top_to_bottom`](crate::plot::PhyloTree::leaf_labels_top_to_bottom)
    /// so that passing its result here aligns heatmap rows with tree leaves.
    ///
    /// If row labels have already been set via [`with_labels`](Heatmap::with_labels),
    /// the data matrix rows are permuted accordingly. Any labels in `desired_order`
    /// not found in the current label set are silently skipped.
    ///
    /// After calling this method, pass `heatmap.row_labels.clone().unwrap()` (which
    /// is stored in **bottom-to-top** order to match the y-axis convention) to
    /// [`Layout::with_y_categories`](crate::render::layout::Layout::with_y_categories)
    /// to display the axis tick labels in the correct order.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::{Heatmap, PhyloTree};
    /// # use kuva::render::layout::Layout;
    /// # use kuva::render::plots::Plot;
    /// let labels = ["A", "B", "C"];
    /// let tree = PhyloTree::from_newick("((A:1,B:2):1,C:3);");
    /// let leaf_order = tree.leaf_labels_top_to_bottom(); // top-to-bottom
    ///
    /// let heatmap = Heatmap::new()
    ///     .with_data(vec![vec![1.0,2.0,3.0], vec![4.0,5.0,6.0], vec![7.0,8.0,9.0]])
    ///     .with_labels(labels.iter().map(|s| s.to_string()).collect(), vec![])
    ///     .with_y_categories(leaf_order); // first label → top row
    ///
    /// // row_labels is bottom-to-top — pass directly to Layout
    /// let layout_cats = heatmap.row_labels.clone().unwrap();
    /// let plots: Vec<Plot> = vec![Plot::Heatmap(heatmap)];
    /// let layout = Layout::auto_from_plots(&plots).with_y_categories(layout_cats);
    /// ```
    pub fn with_y_categories(mut self, desired_order: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let order: Vec<String> = desired_order.into_iter().map(|s| s.into()).collect();
        if let Some(ref current_labels) = self.row_labels.clone() {
            let label_to_idx: std::collections::HashMap<&str, usize> = current_labels
                .iter()
                .enumerate()
                .map(|(i, s)| (s.as_str(), i))
                .collect();
            // Build rows in desired order, then reverse so index 0 = bottom (matching
            // the heatmap renderer's convention) and the last row = top.
            let mut new_data: Vec<Vec<f64>> = order
                .iter()
                .filter_map(|label| label_to_idx.get(label.as_str()).map(|&i| self.data[i].clone()))
                .collect();
            new_data.reverse();
            self.data = new_data;
        }
        // Store labels in bottom-to-top order so they can be passed directly
        // to Layout::with_y_categories (which also uses bottom-to-top / index-0-at-bottom).
        let mut bottom_to_top = order;
        bottom_to_top.reverse();
        self.row_labels = Some(bottom_to_top);
        self
    }

    /// Reorder heatmap columns to match `desired_order` and store the new column labels.
    ///
    /// If column labels have already been set via [`with_labels`](Heatmap::with_labels),
    /// the data matrix columns are permuted so that each column's label matches the
    /// corresponding position in `desired_order`. Any labels in `desired_order`
    /// that are not found in the current label set are silently skipped.
    ///
    /// If no column labels have been set, the provided order is stored as-is (the
    /// caller is responsible for ensuring the data is already in this order).
    ///
    /// After calling this method, pass the same order to
    /// [`Layout::with_x_categories`](crate::render::layout::Layout::with_x_categories)
    /// to display the labels as axis tick marks.
    pub fn with_x_categories(mut self, desired_order: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let order: Vec<String> = desired_order.into_iter().map(|s| s.into()).collect();
        if let Some(ref current_labels) = self.col_labels.clone() {
            let label_to_idx: std::collections::HashMap<&str, usize> = current_labels
                .iter()
                .enumerate()
                .map(|(i, s)| (s.as_str(), i))
                .collect();
            self.data = self
                .data
                .iter()
                .map(|row| {
                    order
                        .iter()
                        .filter_map(|label| label_to_idx.get(label.as_str()).map(|&j| row[j]))
                        .collect()
                })
                .collect();
        }
        self.col_labels = Some(order);
        self
    }

    /// Set the color map used to encode cell values (default [`ColorMap::Viridis`]).
    ///
    /// ```rust,no_run
    /// # use kuva::plot::{Heatmap, ColorMap};
    /// let heatmap = Heatmap::new()
    ///     .with_data(vec![vec![1.0, 2.0], vec![3.0, 4.0]])
    ///     .with_color_map(ColorMap::Inferno);
    /// ```
    pub fn with_color_map(mut self, map: ColorMap) -> Self {
        self.color_map = map;
        self
    }

    /// Overlay numeric values inside each cell.
    ///
    /// Values are formatted to two decimal places and centered in the cell.
    /// Most useful for small grids where the text remains legible.
    pub fn with_values(mut self) -> Self {
        self.show_values = true;
        self
    }

    /// Attach a legend label to this heatmap.
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    pub fn with_tooltips(mut self) -> Self {
        self.show_tooltips = true;
        self
    }

    pub fn with_tooltip_labels(mut self, labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tooltip_labels = Some(labels.into_iter().map(|s| s.into()).collect());
        self
    }
}
