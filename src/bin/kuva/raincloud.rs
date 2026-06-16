use clap::Args;

use kuva::plot::RaincloudPlot;
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_axis_args, apply_base_args, AxisArgs, BaseArgs};
use crate::output::write_output;

/// Raincloud plot — cloud (KDE), box, and jittered rain per group.
#[derive(Args, Debug)]
pub struct RaincloudArgs {
    /// Group column (0-based index or header name; default: 0).
    #[arg(long)]
    pub group_col: Option<ColSpec>,

    /// Value column (0-based index or header name; default: 1).
    #[arg(long)]
    pub value_col: Option<ColSpec>,

    /// Default color for single-group data (CSS color string).
    #[arg(long)]
    pub color: Option<String>,

    /// KDE bandwidth (default: Silverman's rule-of-thumb).
    #[arg(long)]
    pub bandwidth: Option<f64>,

    /// Hide the cloud (KDE density shape).
    #[arg(long)]
    pub no_cloud: bool,

    /// Hide the box-plot summary.
    #[arg(long)]
    pub no_box: bool,

    /// Hide the rain (jittered raw data points).
    #[arg(long)]
    pub no_rain: bool,

    /// Flip orientation (horizontal layout).
    #[arg(long)]
    pub flip: bool,

    /// Legend title (enables legend display).
    #[arg(long)]
    pub legend: Option<String>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,

    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: RaincloudArgs) -> Result<(), String> {
    let proj: Vec<ColSpec> = vec![
        args.group_col.clone().unwrap_or(ColSpec::Index(0)),
        args.value_col.clone().unwrap_or(ColSpec::Index(1)),
    ];
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;

    let group_col = args.group_col.unwrap_or(ColSpec::Index(0));
    let value_col = args.value_col.unwrap_or(ColSpec::Index(1));

    let groups = table.group_by(&group_col)?;

    let mut plot = RaincloudPlot::new();

    if args.no_cloud {
        plot = plot.with_cloud(false);
    }
    if args.no_box {
        plot = plot.with_box(false);
    }
    if args.no_rain {
        plot = plot.with_rain(false);
    }
    if args.flip {
        plot = plot.with_flip(true);
    }
    if let Some(v) = args.bandwidth {
        plot = plot.with_bandwidth(v);
    }
    if let Some(s) = args.legend {
        plot = plot.with_legend(s);
    }

    let palette = Palette::category10();
    let mut colors: Vec<String> = Vec::new();

    for (i, (name, subtable)) in groups.iter().enumerate() {
        let vals = subtable.col_f64(&value_col)?;
        plot = plot.with_group(name.clone(), vals);
        let color = if groups.len() == 1 {
            args.color.clone().unwrap_or_else(|| palette[0].to_string())
        } else {
            palette[i % palette.len()].to_string()
        };
        colors.push(color);
    }

    plot = plot.with_group_colors(colors);

    let plots = vec![Plot::Raincloud(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
