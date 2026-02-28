use clap::Args;

use kuva::plot::BoxPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, apply_base_args, apply_axis_args};
use crate::output::write_output;

/// Box-and-whisker plot grouped by a column.
#[derive(Args, Debug)]
#[command(name = "box")]
pub struct BoxArgs {
    /// Group column (0-based index or header name; default: 0).
    #[arg(long)]
    pub group_col: Option<ColSpec>,

    /// Value column (0-based index or header name; default: 1).
    #[arg(long)]
    pub value_col: Option<ColSpec>,

    /// Box fill color (CSS string; default: "steelblue").
    #[arg(long)]
    pub color: Option<String>,

    /// Overlay individual data points as a jittered strip.
    #[arg(long)]
    pub overlay_points: bool,

    /// Overlay individual data points as a non-overlapping beeswarm.
    #[arg(long)]
    pub overlay_swarm: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: BoxArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let group_col = args.group_col.unwrap_or(ColSpec::Index(0));
    let value_col = args.value_col.unwrap_or(ColSpec::Index(1));
    let color = args.color.unwrap_or_else(|| "steelblue".to_string());

    let groups = table.group_by(&group_col)?;

    let mut plot = BoxPlot::new().with_color(&color);

    for (name, subtable) in groups {
        let values = subtable.col_f64(&value_col)?;
        plot = plot.with_group(name, values);
    }

    if args.overlay_swarm {
        plot = plot.with_swarm_overlay();
    } else if args.overlay_points {
        plot = plot.with_strip(0.3);
    }

    let plots = vec![Plot::Box(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
