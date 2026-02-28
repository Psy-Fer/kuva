use clap::Args;

use kuva::plot::{Heatmap, ColorMap};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{DataTable, InputArgs};
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
    let ncols = table.rows[0].len();
    if ncols < 2 {
        return Err("heatmap input needs at least 2 columns (row label + data)".into());
    }

    // Row labels: first column of each data row.
    let row_labels: Vec<String> = table.rows.iter()
        .map(|r| r[0].clone())
        .collect();

    // Column labels: header columns [1..], or generate default names.
    let col_labels: Vec<String> = if let Some(ref hdr) = table.header {
        hdr[1..].to_vec()
    } else {
        (1..ncols).map(|i| format!("col_{i}")).collect()
    };

    // Build row-major matrix.
    let matrix: Vec<Vec<f64>> = table.rows.iter().enumerate().map(|(r, row)| {
        row[1..].iter().enumerate().map(|(c, cell)| {
            cell.trim().parse::<f64>().map_err(|_| {
                format!("row {r}, col {}: '{}' is not a number", c + 1, cell)
            })
        }).collect::<Result<Vec<f64>, String>>()
    }).collect::<Result<Vec<_>, _>>()?;

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
