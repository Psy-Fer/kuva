use clap::Args;

use kuva::plot::ridgeline::RidgelinePlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, apply_base_args, apply_axis_args};
use crate::output::write_output;

/// Ridgeline (joyplot) — stacked KDE density curves, one per group.
#[derive(Args, Debug)]
pub struct RidgelineArgs {
    /// Column containing numeric values (0-based index or header name; default: 0).
    #[arg(long, default_value = "0")]
    pub value: ColSpec,

    /// Column to group by (one ridge per unique value).
    #[arg(long)]
    pub group_by: Option<ColSpec>,

    /// Fill the area under each ridge curve (default: true).
    #[arg(long, default_value_t = true)]
    pub filled: bool,

    /// Fill opacity (0.0–1.0; default: 0.7).
    #[arg(long, default_value_t = 0.7)]
    pub opacity: f64,

    /// Ridge overlap factor (0 = no overlap, 1 = full cell height; default: 0.5).
    #[arg(long, default_value_t = 0.5)]
    pub overlap: f64,

    /// KDE bandwidth (default: Silverman's rule-of-thumb).
    #[arg(long)]
    pub bandwidth: Option<f64>,

    #[command(next_help_heading = "Input")]
    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,

    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: RidgelineArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let mut plot = RidgelinePlot::new()
        .with_filled(args.filled)
        .with_opacity(args.opacity)
        .with_overlap(args.overlap);
    if let Some(bw) = args.bandwidth {
        plot = plot.with_bandwidth(bw);
    }

    if let Some(ref gb) = args.group_by {
        let mut groups = table.group_by(gb)?;
        groups.sort_by(|(a, _), (b, _)| a.cmp(b));
        for (name, subtable) in groups {
            let vals = subtable.col_f64(&args.value)?;
            plot = plot.with_group(name, vals);
        }
    } else {
        let vals = table.col_f64(&args.value)?;
        plot = plot.with_group("data", vals);
    }

    let plots = vec![Plot::Ridgeline(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
