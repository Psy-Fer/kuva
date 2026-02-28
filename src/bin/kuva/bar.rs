use clap::Args;

use kuva::plot::bar::BarPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, apply_base_args, apply_axis_args};
use crate::output::write_output;

/// Bar chart from label and value columns.
#[derive(Args, Debug)]
pub struct BarArgs {
    /// Label column (0-based index or header name; default: 0).
    #[arg(long)]
    pub label_col: Option<ColSpec>,

    /// Value column (0-based index or header name; default: 1).
    #[arg(long)]
    pub value_col: Option<ColSpec>,

    /// Bar fill color (CSS string; default: "steelblue").
    #[arg(long)]
    pub color: Option<String>,

    /// Bar width as a fraction of the slot (default: 0.8).
    #[arg(long)]
    pub bar_width: Option<f64>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: BarArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let label_col = args.label_col.unwrap_or(ColSpec::Index(0));
    let value_col = args.value_col.unwrap_or(ColSpec::Index(1));
    let color = args.color.unwrap_or_else(|| "steelblue".to_string());

    let labels = table.col_str(&label_col)?;
    let values = table.col_f64(&value_col)?;

    let pairs: Vec<(String, f64)> = labels.into_iter().zip(values).collect();

    let mut plot = BarPlot::new()
        .with_bars(pairs)
        .with_color(&color);

    if let Some(w) = args.bar_width {
        plot = plot.with_width(w);
    }

    let plots = vec![Plot::Bar(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let layout = layout.with_x_tick_rotate(-45.0);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
