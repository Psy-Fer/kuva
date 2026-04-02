use clap::Args;

use kuva::plot::{Heatmap, ColorMap};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use std::collections::BTreeMap;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, apply_base_args, apply_axis_args};
use crate::output::write_output;

/// Heatmap from a wide matrix (first column as row labels).
#[derive(Args, Debug)]
pub struct HeatmapArgs {
    /// Color map: viridis (default), inferno, grayscale.
    #[arg(long, default_value = "viridis")]
    pub colormap: String,

    /// Print numeric values in each cell.
    #[arg(long)]
    pub values: bool,

    /// Show a color bar legend with this label.
    #[arg(long)]
    pub legend: Option<String>,

    /// Accept long-format input: (row, col, value) triples instead of a wide matrix.
    #[arg(long)]
    pub long_format: bool,

    /// Row-label column for --long-format (default: 0).
    #[arg(long)]
    pub row_col: Option<ColSpec>,

    /// Column-label column for --long-format (default: 1).
    #[arg(long)]
    pub col_col: Option<ColSpec>,

    /// Value column for --long-format (default: 2).
    #[arg(long)]
    pub value_col: Option<ColSpec>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

/// Parse colormap name → ColorMap enum.
fn parse_colormap(name: &str) -> ColorMap {
    match name {
        "inferno" => ColorMap::Inferno,
        "grayscale" | "grey" | "gray" => ColorMap::Grayscale,
        _ => ColorMap::Viridis,
    }
}

pub fn run(args: HeatmapArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    if table.rows.is_empty() {
        return Err("heatmap input has no data rows".into());
    }

    // ── Long-format pivot ─────────────────────────────────────────────────────
    let (row_labels, col_labels, matrix) = if args.long_format {
        let row_col   = args.row_col.unwrap_or(ColSpec::Index(0));
        let col_col   = args.col_col.unwrap_or(ColSpec::Index(1));
        let value_col = args.value_col.unwrap_or(ColSpec::Index(2));

        let rows_str = table.col_str(&row_col)?;
        let cols_str = table.col_str(&col_col)?;
        let vals     = table.col_f64(&value_col)?;

        // Collect unique row/col labels in insertion order.
        let mut row_order: Vec<String> = Vec::new();
        let mut col_order: Vec<String> = Vec::new();
        let mut seen_rows: BTreeMap<String, usize> = BTreeMap::new();
        let mut seen_cols: BTreeMap<String, usize> = BTreeMap::new();
        let mut cells: BTreeMap<(usize, usize), f64> = BTreeMap::new();

        for ((r, c), v) in rows_str.into_iter().zip(cols_str).zip(vals) {
            let ri = *seen_rows.entry(r.clone()).or_insert_with(|| {
                let i = row_order.len();
                row_order.push(r);
                i
            });
            let ci = *seen_cols.entry(c.clone()).or_insert_with(|| {
                let i = col_order.len();
                col_order.push(c);
                i
            });
            cells.insert((ri, ci), v);
        }

        let mat: Vec<Vec<f64>> = (0..row_order.len()).map(|ri| {
            (0..col_order.len()).map(|ci| {
                *cells.get(&(ri, ci)).unwrap_or(&0.0)
            }).collect()
        }).collect();

        (row_order, col_order, mat)
    } else {
        // ── Wide-matrix path ─────────────────────────────────────────────────
        let ncols = table.rows[0].len();
        if ncols < 2 {
            return Err("heatmap input needs at least 2 columns (row label + data)".into());
        }

        let nrows = table.rows.len();
        let ncells = nrows * (ncols - 1);
        const CELL_LIMIT: usize = 1_000_000;
        if ncells > CELL_LIMIT {
            return Err(format!(
                "heatmap has {ncells} cells ({nrows} rows × {} data columns), \
                 which exceeds the {CELL_LIMIT} cell limit for SVG/PDF/PNG output.\n\
                 \n\
                 Aggregate or downsample your data before plotting.  Examples:\n\
                 \n\
                 # Average every N rows with datamash:\n\
                 datamash -H groupby 1 mean 2,3,... < data.tsv | kuva heatmap\n\
                 \n\
                 # Keep only every Nth row with awk:\n\
                 awk 'NR==1 || NR%10==0' data.tsv | kuva heatmap\n\
                 \n\
                 # Pivot long-format data and aggregate in Python:\n\
                 # df.groupby('row').mean().to_csv('agg.tsv', sep='\\t')",
                ncols - 1,
            ));
        }

        let row_labels: Vec<String> = table.rows.iter().map(|r| r[0].clone()).collect();
        let col_labels: Vec<String> = if let Some(ref hdr) = table.header {
            hdr[1..].to_vec()
        } else {
            (1..ncols).map(|i| format!("col_{i}")).collect()
        };
        let matrix: Vec<Vec<f64>> = table.rows.iter().enumerate().map(|(r, row)| {
            row[1..].iter().enumerate().map(|(c, cell)| {
                cell.trim().parse::<f64>().map_err(|_| {
                    format!("row {r}, col {}: '{}' is not a number", c + 1, cell)
                })
            }).collect::<Result<Vec<f64>, String>>()
        }).collect::<Result<Vec<_>, _>>()?;

        (row_labels, col_labels, matrix)
    };

    let mut plot = Heatmap::new()
        .with_data(matrix)
        .with_labels(row_labels, col_labels)
        .with_color_map(parse_colormap(&args.colormap));

    if args.values {
        plot = plot.with_values();
    }
    if let Some(ref label) = args.legend {
        plot = plot.with_legend(label.clone());
    }

    let plots = vec![Plot::Heatmap(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
