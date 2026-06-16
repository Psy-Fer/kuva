use clap::Args;

use kuva::plot::MosaicPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_axis_args, apply_base_args, AxisArgs, BaseArgs};
use crate::output::write_output;

/// Mosaic plot — tiled rectangles encoding two categorical dimensions and a value.
#[derive(Args, Debug)]
pub struct MosaicArgs {
    /// Column-category column (0-based index or header name; default: 0).
    #[arg(long)]
    pub col_col: Option<ColSpec>,

    /// Row-category column (0-based index or header name; default: 1).
    #[arg(long)]
    pub row_col: Option<ColSpec>,

    /// Value column (0-based index or header name; default: 2).
    #[arg(long)]
    pub value_col: Option<ColSpec>,

    /// Gap between tiles in pixels.
    #[arg(long)]
    pub gap: Option<f64>,

    /// Disable percentage labels on tiles.
    #[arg(long)]
    pub no_percents: bool,

    /// Show raw value labels on tiles.
    #[arg(long)]
    pub show_values: bool,

    /// Legend title (enables legend display).
    #[arg(long)]
    pub legend: Option<String>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,

    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: MosaicArgs) -> Result<(), String> {
    let proj: Vec<ColSpec> = vec![
        args.col_col.clone().unwrap_or(ColSpec::Index(0)),
        args.row_col.clone().unwrap_or(ColSpec::Index(1)),
        args.value_col.clone().unwrap_or(ColSpec::Index(2)),
    ];
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;

    let col_col = args.col_col.unwrap_or(ColSpec::Index(0));
    let row_col = args.row_col.unwrap_or(ColSpec::Index(1));
    let value_col = args.value_col.unwrap_or(ColSpec::Index(2));

    let col_cats = table.col_str(&col_col)?;
    let row_cats = table.col_str(&row_col)?;
    let values = table.col_f64(&value_col)?;

    let n = col_cats.len();
    if row_cats.len() != n || values.len() != n {
        return Err(format!(
            "column length mismatch: col_cat={}, row_cat={}, value={}",
            n,
            row_cats.len(),
            values.len()
        ));
    }

    let mut plot = MosaicPlot::new();

    if let Some(v) = args.gap {
        plot = plot.with_gap(v);
    }
    if args.no_percents {
        plot = plot.with_percents(false);
    }
    if args.show_values {
        plot = plot.with_values(true);
    }
    if let Some(s) = args.legend {
        plot = plot.with_legend(s);
    }

    for ((col_cat, row_cat), value) in col_cats.iter().zip(row_cats.iter()).zip(values.iter()) {
        plot = plot.with_cell(col_cat.as_str(), row_cat.as_str(), *value);
    }

    let plots = vec![Plot::Mosaic(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
