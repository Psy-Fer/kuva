//! Polars DataFrame integration for kuva plot types.
//!
//! Provides ergonomic methods to build plots directly from DataFrame columns
//! without manual data extraction. Feature-gated behind `polars`.
//!
//! # Example
//!
//! ```rust,no_run
//! use polars::prelude::*;
//! use kuva::prelude::*;
//! use kuva::dataframe::DataFrameExt;
//!
//! let df = df! {
//!     "x" => &[1.0, 2.0, 3.0, 4.0, 5.0],
//!     "y" => &[2.3, 3.1, 2.8, 4.5, 3.9],
//! }.unwrap();
//!
//! let scatter = ScatterPlot::new()
//!     .with_xy(&df, "x", "y")
//!     .unwrap()
//!     .with_color("steelblue");
//!
//! let svg = kuva::render_to_svg(
//!     vec![scatter.into()],
//!     Layout::auto_from_plots(&[]),
//! );
//! ```

use polars::prelude::*;

/// Error returned when a DataFrame column can't be used for plotting.
#[derive(Debug)]
pub enum PlotDataError {
    ColumnNotFound(String),
    DtypeMismatch { column: String, expected: &'static str, found: String },
    NullValues { column: String },
    LengthMismatch { col_a: String, len_a: usize, col_b: String, len_b: usize },
}

impl std::fmt::Display for PlotDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlotDataError::ColumnNotFound(name) =>
                write!(f, "column '{name}' not found in DataFrame"),
            PlotDataError::DtypeMismatch { column, expected, found } =>
                write!(f, "column '{column}': expected {expected}, found {found}"),
            PlotDataError::NullValues { column } =>
                write!(f, "column '{column}' contains null values"),
            PlotDataError::LengthMismatch { col_a, len_a, col_b, len_b } =>
                write!(f, "columns '{col_a}' ({len_a} rows) and '{col_b}' ({len_b} rows) have different lengths"),
        }
    }
}

impl std::error::Error for PlotDataError {}

// ── Column extraction helpers ───────────────────────────────────────────────

/// Extract a column as `Vec<f64>`, casting integers if needed.
fn col_f64(df: &DataFrame, name: &str) -> Result<Vec<f64>, PlotDataError> {
    let series = df.column(name)
        .map_err(|_| PlotDataError::ColumnNotFound(name.into()))?;
    let ca = series.cast(&DataType::Float64)
        .map_err(|_| PlotDataError::DtypeMismatch {
            column: name.into(),
            expected: "numeric (integer or float)",
            found: format!("{}", series.dtype()),
        })?;
    let ca = ca.f64()
        .map_err(|_| PlotDataError::DtypeMismatch {
            column: name.into(),
            expected: "Float64",
            found: format!("{}", series.dtype()),
        })?;
    ca.into_iter()
        .map(|opt| opt.ok_or_else(|| PlotDataError::NullValues { column: name.into() }))
        .collect()
}

/// Extract a column as `Vec<String>`.
fn col_string(df: &DataFrame, name: &str) -> Result<Vec<String>, PlotDataError> {
    let series = df.column(name)
        .map_err(|_| PlotDataError::ColumnNotFound(name.into()))?;
    let ca = series.str()
        .map_err(|_| PlotDataError::DtypeMismatch {
            column: name.into(),
            expected: "String/Utf8",
            found: format!("{}", series.dtype()),
        })?;
    ca.into_iter()
        .map(|opt| opt.map(|s| s.to_string()).ok_or_else(|| PlotDataError::NullValues { column: name.into() }))
        .collect()
}

/// Zip two f64 columns into `Vec<(f64, f64)>`.
fn zip_xy(df: &DataFrame, x: &str, y: &str) -> Result<Vec<(f64, f64)>, PlotDataError> {
    let xs = col_f64(df, x)?;
    let ys = col_f64(df, y)?;
    if xs.len() != ys.len() {
        return Err(PlotDataError::LengthMismatch {
            col_a: x.into(), len_a: xs.len(),
            col_b: y.into(), len_b: ys.len(),
        });
    }
    Ok(xs.into_iter().zip(ys).collect())
}

// ── Extension trait for DataFrame ───────────────────────────────────────────

/// Convenience methods for building common plots from a DataFrame.
///
/// These are standalone functions rather than methods on the plot types so
/// they don't require the `polars` feature to be enabled to compile the
/// plot module itself.
pub trait DataFrameExt {
    /// Build a scatter plot from x/y columns.
    fn scatter(&self, x: &str, y: &str) -> Result<crate::plot::ScatterPlot, PlotDataError>;

    /// Build a line plot from x/y columns.
    fn line(&self, x: &str, y: &str) -> Result<crate::plot::LinePlot, PlotDataError>;

    /// Build a histogram from a single numeric column.
    fn histogram(&self, col: &str, bins: usize) -> Result<crate::plot::Histogram, PlotDataError>;

    /// Build a bar plot from label and value columns.
    fn bar(&self, labels: &str, values: &str) -> Result<crate::plot::BarPlot, PlotDataError>;
}

impl DataFrameExt for DataFrame {
    fn scatter(&self, x: &str, y: &str) -> Result<crate::plot::ScatterPlot, PlotDataError> {
        let data = zip_xy(self, x, y)?;
        Ok(crate::plot::ScatterPlot::new().with_data(data))
    }

    fn line(&self, x: &str, y: &str) -> Result<crate::plot::LinePlot, PlotDataError> {
        let data = zip_xy(self, x, y)?;
        Ok(crate::plot::LinePlot::new().with_data(data))
    }

    fn histogram(&self, col: &str, bins: usize) -> Result<crate::plot::Histogram, PlotDataError> {
        let values = col_f64(self, col)?;
        Ok(crate::plot::Histogram::new().with_data(values).with_bins(bins))
    }

    fn bar(&self, labels: &str, values: &str) -> Result<crate::plot::BarPlot, PlotDataError> {
        let lab = col_string(self, labels)?;
        let val = col_f64(self, values)?;
        if lab.len() != val.len() {
            return Err(PlotDataError::LengthMismatch {
                col_a: labels.into(), len_a: lab.len(),
                col_b: values.into(), len_b: val.len(),
            });
        }
        let mut plot = crate::plot::BarPlot::new();
        for (label, value) in lab.into_iter().zip(val) {
            plot = plot.with_group(label, vec![(value, "steelblue")]);
        }
        Ok(plot)
    }
}

// ── Builder extensions on individual plot types ─────────────────────────────

impl crate::plot::ScatterPlot {
    /// Set scatter data from two DataFrame columns.
    #[cfg(feature = "polars")]
    pub fn with_xy(self, df: &DataFrame, x: &str, y: &str) -> Result<Self, PlotDataError> {
        let data = zip_xy(df, x, y)?;
        Ok(self.with_data(data))
    }
}

impl crate::plot::LinePlot {
    /// Set line data from two DataFrame columns.
    #[cfg(feature = "polars")]
    pub fn with_xy(self, df: &DataFrame, x: &str, y: &str) -> Result<Self, PlotDataError> {
        let data = zip_xy(df, x, y)?;
        Ok(self.with_data(data))
    }
}

impl crate::plot::Histogram {
    /// Set histogram data from a single DataFrame column.
    #[cfg(feature = "polars")]
    pub fn with_column(self, df: &DataFrame, col: &str) -> Result<Self, PlotDataError> {
        let values = col_f64(df, col)?;
        Ok(self.with_data(values))
    }
}

impl crate::plot::Heatmap {
    /// Build a heatmap from a DataFrame with numeric columns.
    /// Each column becomes a row of the heatmap grid.
    #[cfg(feature = "polars")]
    pub fn with_dataframe(self, df: &DataFrame) -> Result<Self, PlotDataError> {
        let mut grid: Vec<Vec<f64>> = Vec::with_capacity(df.width());
        for series in df.get_columns() {
            let ca = series.cast(&DataType::Float64)
                .map_err(|_| PlotDataError::DtypeMismatch {
                    column: series.name().to_string(),
                    expected: "numeric",
                    found: format!("{}", series.dtype()),
                })?;
            let row: Vec<f64> = ca.f64()
                .map_err(|_| PlotDataError::DtypeMismatch {
                    column: series.name().to_string(),
                    expected: "Float64",
                    found: format!("{}", series.dtype()),
                })?
                .into_iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect();
            grid.push(row);
        }
        Ok(self.with_data(grid))
    }
}

impl crate::plot::ManhattanPlot {
    /// Load Manhattan plot data from chromosome, position, and p-value columns.
    #[cfg(feature = "polars")]
    pub fn with_columns(
        self,
        df: &DataFrame,
        chrom: &str,
        pvalue: &str,
    ) -> Result<Self, PlotDataError> {
        let chroms = col_string(df, chrom)?;
        let pvals = col_f64(df, pvalue)?;
        if chroms.len() != pvals.len() {
            return Err(PlotDataError::LengthMismatch {
                col_a: chrom.into(), len_a: chroms.len(),
                col_b: pvalue.into(), len_b: pvals.len(),
            });
        }
        let data: Vec<(String, f64)> = chroms.into_iter().zip(pvals).collect();
        Ok(self.with_data(data))
    }
}

impl crate::plot::VolcanoPlot {
    /// Load volcano plot data from gene name, log2FC, and p-value columns.
    #[cfg(feature = "polars")]
    pub fn with_columns(
        self,
        df: &DataFrame,
        gene: &str,
        log2fc: &str,
        pvalue: &str,
    ) -> Result<Self, PlotDataError> {
        let names = col_string(df, gene)?;
        let fcs = col_f64(df, log2fc)?;
        let pvals = col_f64(df, pvalue)?;
        let data: Vec<(String, f64, f64)> = names.into_iter()
            .zip(fcs)
            .zip(pvals)
            .map(|((n, fc), p)| (n, fc, p))
            .collect();
        Ok(self.with_points(data))
    }
}
