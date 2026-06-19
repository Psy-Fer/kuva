use clap::Args;

use kuva::plot::StripPlot;
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_axis_args, apply_base_args, AxisArgs, BaseArgs};
use crate::output::write_output;

/// Strip / beeswarm plot grouped by a column.
#[derive(Args, Debug)]
pub struct StripArgs {
    /// Group column (0-based index or header name; default: 0).
    #[arg(long)]
    pub group_col: Option<ColSpec>,

    /// Value column (0-based index or header name; default: 1).
    #[arg(long)]
    pub value_col: Option<ColSpec>,

    /// Value column(s). Comma-separated for multi-column mode: `--y A,B,C` treats each
    /// column as a separate group (column name = group label). Overrides --value-col.
    #[arg(long, value_delimiter = ',')]
    pub y: Vec<ColSpec>,

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

    /// Color groups by palette and show a legend.
    #[arg(long)]
    pub legend: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: StripArgs) -> Result<(), String> {
    let color = args
        .color
        .clone()
        .unwrap_or_else(|| "steelblue".to_string());

    // Multi-column --y mode: each column is a group
    if args.y.len() > 1 {
        let table = DataTable::parse(
            args.input.input.as_deref(),
            args.input.no_header,
            args.input.delimiter,
            &args.y,
        )?;
        let mut plot = StripPlot::new().with_color(&color);
        if let Some(size) = args.point_size {
            plot = plot.with_point_size(size);
        }
        if args.swarm {
            plot = plot.with_swarm();
        } else if args.center {
            plot = plot.with_center();
        }
        for col in &args.y {
            let name = table.col_display_name(col);
            let values = table.col_f64(col)?;
            plot = plot.with_group(name, values);
        }
        let pal = Palette::category10();
        let colors: Vec<String> = (0..args.y.len()).map(|i| pal[i].to_string()).collect();
        plot = plot.with_group_colors(colors).with_legend("");
        let plots = vec![Plot::Strip(plot)];
        let layout = Layout::auto_from_plots(&plots);
        let layout = apply_base_args(layout, &args.base);
        let layout = apply_axis_args(layout, &args.axis);
        let scene = render_multiple(plots, layout);
        return write_output(scene, &args.base);
    }

    let value_col = if args.y.len() == 1 {
        args.y[0].clone()
    } else {
        args.value_col.unwrap_or(ColSpec::Index(1))
    };
    let proj: Vec<ColSpec> = vec![
        args.group_col.clone().unwrap_or(ColSpec::Index(0)),
        value_col.clone(),
    ];
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;

    let group_col = args.group_col.unwrap_or(ColSpec::Index(0));
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

    for (name, subtable) in groups {
        let values = subtable.col_f64(&value_col)?;
        plot = plot.with_group(name, values);
    }

    if args.legend {
        let pal = Palette::category10();
        let colors: Vec<String> = (0..plot.groups.len()).map(|i| pal[i].to_string()).collect();
        plot = plot.with_group_colors(colors).with_legend("");
    }

    let plots = vec![Plot::Strip(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
