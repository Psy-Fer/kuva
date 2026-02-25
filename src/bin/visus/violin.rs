use clap::Args;

use visus::plot::ViolinPlot;
use visus::render::layout::Layout;
use visus::render::plots::Plot;
use visus::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, apply_base_args, apply_axis_args};
use crate::output::write_output;

#[derive(Args, Debug)]
pub struct ViolinArgs {
    #[command(flatten)]
    pub input: InputArgs,

    /// Group column (0-based index or header name; default: 0).
    #[arg(long)]
    pub group_col: Option<ColSpec>,

    /// Value column (0-based index or header name; default: 1).
    #[arg(long)]
    pub value_col: Option<ColSpec>,

    /// Violin fill color (CSS string; default: "steelblue").
    #[arg(long)]
    pub color: Option<String>,

    /// KDE bandwidth (Silverman's rule-of-thumb if omitted).
    #[arg(long)]
    pub bandwidth: Option<f64>,

    /// Overlay a jittered strip of individual data points.
    #[arg(long)]
    pub strip: bool,

    /// Overlay a beeswarm of individual data points.
    #[arg(long)]
    pub swarm: bool,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: ViolinArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let group_col = args.group_col.unwrap_or(ColSpec::Index(0));
    let value_col = args.value_col.unwrap_or(ColSpec::Index(1));
    let color = args.color.unwrap_or_else(|| "steelblue".to_string());

    let groups = table.group_by(&group_col)?;

    let mut plot = ViolinPlot::new().with_color(&color);

    if let Some(bw) = args.bandwidth {
        plot = plot.with_bandwidth(bw);
    }

    for (name, subtable) in groups {
        let values = subtable.col_f64(&value_col)?;
        plot = plot.with_group(name, values);
    }

    if args.swarm {
        plot = plot.with_swarm_overlay();
    } else if args.strip {
        plot = plot.with_strip(0.3);
    }

    let plots = vec![Plot::Violin(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
