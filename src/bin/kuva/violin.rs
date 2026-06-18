use clap::Args;

use kuva::plot::ViolinPlot;
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_axis_args, apply_base_args, AxisArgs, BaseArgs};
use crate::output::write_output;

/// Violin plot grouped by a column.
#[derive(Args, Debug)]
pub struct ViolinArgs {
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

    /// Violin fill color (CSS string; default: "steelblue").
    #[arg(long)]
    pub color: Option<String>,

    /// Per-group fill colors (comma-separated CSS colors, e.g. "steelblue,tomato,seagreen").
    /// Colors are matched to groups in the order they appear in the data.
    #[arg(long, value_delimiter = ',')]
    pub group_colors: Option<Vec<String>>,

    /// KDE bandwidth (Silverman's rule-of-thumb if omitted).
    #[arg(long)]
    pub bandwidth: Option<f64>,

    /// Overlay individual data points as a jittered strip.
    #[arg(long)]
    pub overlay_points: bool,

    /// Overlay individual data points as a non-overlapping beeswarm.
    #[arg(long)]
    pub overlay_swarm: bool,

    /// Render groups on the Y-axis and values on the X-axis.
    #[arg(long)]
    pub horizontal: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: ViolinArgs) -> Result<(), String> {
    let color = args.color.clone().unwrap_or_else(|| "steelblue".to_string());

    // Multi-column --y mode: each column is a group
    if args.y.len() > 1 {
        let table = DataTable::parse(
            args.input.input.as_deref(),
            args.input.no_header,
            args.input.delimiter,
            &args.y,
        )?;
        let mut plot = ViolinPlot::new().with_color(&color);
        if let Some(bw) = args.bandwidth {
            plot = plot.with_bandwidth(bw);
        }
        for col in &args.y {
            let name = table.col_display_name(col);
            let values = table.col_f64(col)?;
            plot = plot.with_group(name, values);
        }
        if let Some(colors) = args.group_colors {
            plot = plot.with_group_colors(colors);
        } else {
            let pal = Palette::category10();
            let colors: Vec<String> = (0..args.y.len()).map(|i| pal[i].to_string()).collect();
            plot = plot.with_group_colors(colors);
        }
        if args.overlay_swarm {
            plot = plot.with_swarm_overlay();
        } else if args.overlay_points {
            plot = plot.with_strip(0.3);
        }
        if args.horizontal {
            plot = plot.with_horizontal(true);
        }
        let plots = vec![Plot::Violin(plot)];
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

    let mut plot = ViolinPlot::new().with_color(&color);

    if let Some(bw) = args.bandwidth {
        plot = plot.with_bandwidth(bw);
    }

    for (name, subtable) in groups {
        let values = subtable.col_f64(&value_col)?;
        plot = plot.with_group(name, values);
    }

    if let Some(colors) = args.group_colors {
        plot = plot.with_group_colors(colors);
    }

    if args.overlay_swarm {
        plot = plot.with_swarm_overlay();
    } else if args.overlay_points {
        plot = plot.with_strip(0.3);
    }

    if args.horizontal {
        plot = plot.with_horizontal(true);
    }

    let plots = vec![Plot::Violin(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
