use clap::Args;

use visus::plot::histogram::Histogram;
use visus::render::layout::Layout;
use visus::render::plots::Plot;
use visus::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, LogArgs, apply_base_args, apply_axis_args, apply_log_args};
use crate::output::write_output;

#[derive(Args, Debug)]
pub struct HistogramArgs {
    /// Value column (0-based index or header name; default: 0).
    #[arg(long)]
    pub value_col: Option<ColSpec>,

    /// Bar fill color (CSS string; default: "steelblue").
    #[arg(long)]
    pub color: Option<String>,

    /// Number of bins (default: 10).
    #[arg(long)]
    pub bins: Option<usize>,

    /// Normalize counts to a probability density (area = 1).
    #[arg(long)]
    pub normalize: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
    #[command(flatten)]
    pub log: LogArgs,
}

pub fn run(args: HistogramArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let value_col = args.value_col.unwrap_or(ColSpec::Index(0));
    let color = args.color.unwrap_or_else(|| "steelblue".to_string());
    let bins = args.bins.unwrap_or(10);

    let values = table.col_f64(&value_col)?;
    if values.is_empty() {
        return Err("No data values found".to_string());
    }

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let mut plot = Histogram::new()
        .with_data(values)
        .with_bins(bins)
        .with_range((min, max))
        .with_color(&color);

    if args.normalize {
        plot = plot.with_normalize();
    }

    let plots = vec![Plot::Histogram(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let layout = apply_log_args(layout, &args.log);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
