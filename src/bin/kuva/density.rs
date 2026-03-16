use clap::Args;

use kuva::plot::DensityPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::render::palette::Palette;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, LogArgs, apply_base_args, apply_axis_args, apply_log_args};
use crate::output::write_output;

/// Kernel density estimate curve from a numeric column.
#[derive(Args, Debug)]
pub struct DensityArgs {
    /// Column containing numeric values (0-based index or header name; default: 0).
    #[arg(long, default_value = "0")]
    pub value: ColSpec,

    /// Group by this column — one density curve per unique value.
    #[arg(long)]
    pub color_by: Option<ColSpec>,

    /// Fill the area under the density curve.
    #[arg(long)]
    pub filled: bool,

    /// KDE bandwidth (default: Silverman's rule-of-thumb).
    #[arg(long)]
    pub bandwidth: Option<f64>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,

    #[command(flatten)]
    pub axis: AxisArgs,

    #[command(flatten)]
    pub log: LogArgs,
}

pub fn run(args: DensityArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let plots: Vec<Plot> = if let Some(ref cb) = args.color_by {
        let pal = Palette::category10();
        let groups = table.group_by(cb)?;
        groups
            .into_iter()
            .enumerate()
            .map(|(i, (name, subtable))| {
                let vals = subtable.col_f64(&args.value)?;
                let mut dp = DensityPlot::new()
                    .with_data(vals)
                    .with_color(pal[i].to_string())
                    .with_legend(name);
                if args.filled {
                    dp = dp.with_filled(true);
                }
                if let Some(bw) = args.bandwidth {
                    dp = dp.with_bandwidth(bw);
                }
                if let (Some(lo), Some(hi)) = (args.axis.x_min, args.axis.x_max) {
                    dp = dp.with_x_range(lo, hi);
                }
                Ok(Plot::Density(dp))
            })
            .collect::<Result<Vec<_>, String>>()?
    } else {
        let vals = table.col_f64(&args.value)?;
        let mut dp = DensityPlot::new().with_data(vals);
        if args.filled {
            dp = dp.with_filled(true);
        }
        if let Some(bw) = args.bandwidth {
            dp = dp.with_bandwidth(bw);
        }
        if let (Some(lo), Some(hi)) = (args.axis.x_min, args.axis.x_max) {
            dp = dp.with_x_range(lo, hi);
        }
        vec![Plot::Density(dp)]
    };

    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let layout = apply_log_args(layout, &args.log);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
