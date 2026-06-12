use clap::Args;

use kuva::plot::HorizonPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_axis_args, apply_base_args, AxisArgs, BaseArgs};
use crate::output::write_output;

/// Horizon chart — stacked, folded area chart for dense multi-series time series.
#[derive(Args, Debug)]
pub struct HorizonArgs {
    /// X (time) column (0-based index or header name; default: 0).
    #[arg(long)]
    pub x_col: Option<ColSpec>,

    /// Value column (0-based index or header name; default: 1).
    #[arg(long)]
    pub value_col: Option<ColSpec>,

    /// Group by this column — one series per unique value.
    #[arg(long)]
    pub group_col: Option<ColSpec>,

    /// Number of color bands (default: 3).
    #[arg(long)]
    pub n_bands: Option<usize>,

    /// Per-row pixel height.
    #[arg(long)]
    pub row_height: Option<f64>,

    /// Baseline value separating positive from negative regions (default: 0.0).
    #[arg(long)]
    pub baseline: Option<f64>,

    /// Show the full-scale value at the right end of each row.
    #[arg(long)]
    pub value_labels: bool,

    /// Show a legend entry per series.
    #[arg(long)]
    pub legend: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: HorizonArgs) -> Result<(), String> {
    let mut proj: Vec<ColSpec> = vec![
        args.x_col.clone().unwrap_or(ColSpec::Index(0)),
        args.value_col.clone().unwrap_or(ColSpec::Index(1)),
    ];
    if let Some(ref c) = args.group_col { proj.push(c.clone()); }
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;

    let x_col = args.x_col.unwrap_or(ColSpec::Index(0));
    let value_col = args.value_col.unwrap_or(ColSpec::Index(1));

    let mut plot = HorizonPlot::new();

    if let Some(n) = args.n_bands {
        plot = plot.with_n_bands(n);
    }
    if let Some(h) = args.row_height {
        plot = plot.with_row_height(h);
    }
    if let Some(b) = args.baseline {
        plot = plot.with_baseline(b);
    }
    if args.value_labels {
        plot = plot.with_value_labels(true);
    }
    if args.legend {
        plot = plot.with_legend(true);
    }

    if let Some(ref gc) = args.group_col {
        let groups = table.group_by(gc)?;
        for (name, subtable) in groups {
            let x_vals = subtable.col_f64(&x_col)?;
            let y_vals = subtable.col_f64(&value_col)?;
            plot = plot.with_series(name, x_vals, y_vals);
        }
    } else {
        let x_vals = table.col_f64(&x_col)?;
        let y_vals = table.col_f64(&value_col)?;
        plot = plot.with_series("", x_vals, y_vals);
    }

    let plots = vec![Plot::Horizon(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
