use clap::Args;

use visus::plot::StripPlot;
use visus::render::layout::Layout;
use visus::render::plots::Plot;
use visus::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, apply_base_args, apply_axis_args};
use crate::output::write_output;

#[derive(Args, Debug)]
pub struct StripArgs {
    /// Group column (0-based index or header name; default: 0).
    #[arg(long)]
    pub group_col: Option<ColSpec>,

    /// Value column (0-based index or header name; default: 1).
    #[arg(long)]
    pub value_col: Option<ColSpec>,

    /// Point fill color (CSS string; default: "steelblue").
    #[arg(long)]
    pub color: Option<String>,

    /// Point radius in pixels (default: 4.0).
    #[arg(long)]
    pub point_size: Option<f64>,

    /// Use beeswarm (non-overlapping) layout instead of jitter.
    #[arg(long)]
    pub swarm: bool,

    /// Place all points at the group center (no horizontal spread).
    #[arg(long)]
    pub center: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: StripArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let group_col = args.group_col.unwrap_or(ColSpec::Index(0));
    let value_col = args.value_col.unwrap_or(ColSpec::Index(1));
    let color = args.color.unwrap_or_else(|| "steelblue".to_string());

    let groups = table.group_by(&group_col)?;

    let mut plot = StripPlot::new().with_color(&color);

    if let Some(size) = args.point_size {
        plot = plot.with_point_size(size);
    }

    if args.swarm {
        plot = plot.with_swarm();
    } else if args.center {
        plot = plot.with_center();
    }
    // default: jitter 0.3 (already the StripPlot default)

    for (name, subtable) in groups {
        let values = subtable.col_f64(&value_col)?;
        plot = plot.with_group(name, values);
    }

    let plots = vec![Plot::Strip(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
