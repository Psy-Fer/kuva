use clap::Args;

use visus::plot::volcano::VolcanoPlot;
use visus::render::layout::Layout;
use visus::render::plots::Plot;
use visus::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, apply_base_args, apply_axis_args};
use crate::output::write_output;

/// Volcano plot for differential expression analysis.
#[derive(Args, Debug)]
pub struct VolcanoArgs {
    /// Name/gene column (0-based index or header name; default: 0).
    #[arg(long)]
    pub name_col: Option<ColSpec>,

    /// log2 fold-change column (0-based index or header name; default: 1).
    #[arg(long)]
    pub x_col: Option<ColSpec>,

    /// p-value column (0-based index or header name; default: 2).
    #[arg(long)]
    pub y_col: Option<ColSpec>,

    /// |log2FC| cutoff for up/down classification (default: 1.0).
    #[arg(long)]
    pub fc_cutoff: Option<f64>,

    /// p-value significance threshold (default: 0.05).
    #[arg(long)]
    pub p_cutoff: Option<f64>,

    /// Label this many most-significant points.
    #[arg(long)]
    pub top_n: Option<usize>,

    /// Color for up-regulated points (default: "firebrick").
    #[arg(long)]
    pub color_up: Option<String>,

    /// Color for down-regulated points (default: "steelblue").
    #[arg(long)]
    pub color_down: Option<String>,

    /// Color for not-significant points (default: "#aaaaaa").
    #[arg(long)]
    pub color_ns: Option<String>,

    /// Point radius in pixels (default: 3.0).
    #[arg(long)]
    pub point_size: Option<f64>,

    /// Show a legend for Up / Down / NS categories.
    #[arg(long)]
    pub legend: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: VolcanoArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let name_col = args.name_col.unwrap_or(ColSpec::Index(0));
    let x_col = args.x_col.unwrap_or(ColSpec::Index(1));
    let y_col = args.y_col.unwrap_or(ColSpec::Index(2));

    let names = table.col_str(&name_col)?;
    let fcs = table.col_f64(&x_col)?;
    let pvals = table.col_f64(&y_col)?;

    let points: Vec<(String, f64, f64)> = names.into_iter()
        .zip(fcs)
        .zip(pvals)
        .map(|((n, fc), p)| (n, fc, p))
        .collect();

    let mut plot = VolcanoPlot::new().with_points(points);

    if let Some(c) = args.fc_cutoff {
        plot = plot.with_fc_cutoff(c);
    }
    if let Some(c) = args.p_cutoff {
        plot = plot.with_p_cutoff(c);
    }
    if let Some(n) = args.top_n {
        plot = plot.with_label_top(n);
    }
    if let Some(ref c) = args.color_up {
        plot = plot.with_color_up(c.clone());
    }
    if let Some(ref c) = args.color_down {
        plot = plot.with_color_down(c.clone());
    }
    if let Some(ref c) = args.color_ns {
        plot = plot.with_color_ns(c.clone());
    }
    if let Some(s) = args.point_size {
        plot = plot.with_point_size(s);
    }
    if args.legend {
        plot = plot.with_legend("DEG status");
    }

    let plots = vec![Plot::Volcano(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
