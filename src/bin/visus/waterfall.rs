use clap::Args;
use std::collections::HashSet;

use visus::plot::WaterfallPlot;
use visus::render::layout::Layout;
use visus::render::plots::Plot;
use visus::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, apply_base_args, apply_axis_args};
use crate::output::write_output;

/// Waterfall chart from label and delta-value columns.
#[derive(Args, Debug)]
pub struct WaterfallArgs {
    /// Label column (0-based index or header name; default: 0).
    #[arg(long)]
    pub label_col: Option<ColSpec>,

    /// Value column (0-based index or header name; default: 1).
    #[arg(long)]
    pub value_col: Option<ColSpec>,

    /// Labels that should be rendered as summary/total bars (repeatable).
    #[arg(long = "total")]
    pub totals: Vec<String>,

    /// Draw dashed connector lines between consecutive bars.
    #[arg(long)]
    pub connectors: bool,

    /// Print numeric values on each bar.
    #[arg(long)]
    pub values: bool,

    /// Color for positive delta bars (default: "rgb(68,170,68)").
    #[arg(long)]
    pub color_pos: Option<String>,

    /// Color for negative delta bars (default: "rgb(204,68,68)").
    #[arg(long)]
    pub color_neg: Option<String>,

    /// Color for total/subtotal bars (default: "steelblue").
    #[arg(long)]
    pub color_total: Option<String>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: WaterfallArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let label_col = args.label_col.unwrap_or(ColSpec::Index(0));
    let value_col = args.value_col.unwrap_or(ColSpec::Index(1));

    let labels = table.col_str(&label_col)?;
    let values = table.col_f64(&value_col)?;

    let totals_set: HashSet<&str> = args.totals.iter().map(|s| s.as_str()).collect();

    let mut plot = WaterfallPlot::new();

    if let Some(ref c) = args.color_pos {
        plot = plot.with_color_positive(c.clone());
    }
    if let Some(ref c) = args.color_neg {
        plot = plot.with_color_negative(c.clone());
    }
    if let Some(ref c) = args.color_total {
        plot = plot.with_color_total(c.clone());
    }
    if args.connectors {
        plot = plot.with_connectors();
    }
    if args.values {
        plot = plot.with_values();
    }

    for (label, value) in labels.iter().zip(values.iter()) {
        if totals_set.contains(label.as_str()) {
            plot = plot.with_total(label.clone());
        } else {
            plot = plot.with_delta(label.clone(), *value);
        }
    }

    let plots = vec![Plot::Waterfall(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let layout = layout.with_x_tick_rotate(-45.0);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
