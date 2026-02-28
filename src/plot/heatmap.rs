use std::sync::Arc;
use colorous::{VIRIDIS, INFERNO, GREYS};

// Map [0.0, 1.0] to color string
fn viridis(value: f64) -> String {
    let rgb = VIRIDIS.eval_continuous(value.clamp(0.0, 1.0));
    format!("rgb({},{},{})", rgb.r, rgb.g, rgb.b)
}

fn inferno(value: f64) -> String {
    let rgb = INFERNO.eval_continuous(value.clamp(0.0, 1.0));
    format!("rgb({},{},{})", rgb.r, rgb.g, rgb.b)
}

fn greyscale(value: f64) -> String {
    let rgb = GREYS.eval_continuous(value.clamp(0.0, 1.0));
    format!("rgb({},{},{})", rgb.r, rgb.g, rgb.b)
}

/// Color map used to encode numeric cell values as colors.
///
/// Values are normalized to `[0.0, 1.0]` relative to the data min/max before
/// the map is applied. The same `ColorMap` type is shared by [`Heatmap`] and
/// [`Histogram2D`](crate::plot::Histogram2D).
///
/// # Choosing a color map
///
/// | Variant | Character | Use when |
/// |---------|-----------|----------|
/// | `Viridis` | Blue → green → yellow | General purpose; perceptually uniform; colorblind-safe |
/// | `Inferno` | Black → purple → yellow | High-contrast; works in greyscale print |
/// | `Grayscale` | Black → white | Publication figures; print-friendly |
/// | `Custom` | User-defined | Full control over color encoding |
#[derive(Clone)]
pub enum ColorMap {
    /// Perceptually uniform blue-green-yellow scale (default).
    Grayscale,
    /// Perceptually uniform blue-green-yellow scale.
    Viridis,
    /// High-contrast black-purple-yellow scale.
    Inferno,
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

impl ColorMap {
    /// Map a normalized value in `[0.0, 1.0]` to a CSS color string.
    pub fn map(&self, value: f64) -> String {
        match self {
            ColorMap::Grayscale => greyscale(value),
            ColorMap::Viridis => viridis(value),
            ColorMap::Inferno => inferno(value),
            ColorMap::Custom(f) => f(value),
        }
    }
}

/// Builder for a heatmap.
///
/// Renders a two-dimensional grid of colored cells. Cell color encodes the
/// numeric value — each cell is mapped through a [`ColorMap`] after
/// normalizing values to `[0.0, 1.0]` relative to the data range. A colorbar
/// is always shown in the right margin.
///
/// Axis labels are set on the [`Layout`](crate::render::layout::Layout) via
/// [`with_x_categories`](crate::render::layout::Layout::with_x_categories)
/// (column labels) and
/// [`with_y_categories`](crate::render::layout::Layout::with_y_categories)
/// (row labels), not on the `Heatmap` struct directly.
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
    /// These labels are **not** rendered automatically. To display them on the
    /// axes, pass them to
    /// [`Layout::with_y_categories`](crate::render::layout::Layout::with_y_categories)
    /// (rows) and
    /// [`Layout::with_x_categories`](crate::render::layout::Layout::with_x_categories)
    /// (columns) when building the layout.
    pub fn with_labels(mut self, rows: Vec<String>, cols: Vec<String>) -> Self {
        self.row_labels = Some(rows);
        self.col_labels = Some(cols);
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
}
