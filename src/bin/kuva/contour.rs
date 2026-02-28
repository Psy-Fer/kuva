use clap::Args;

use kuva::plot::{ContourPlot, ColorMap};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, apply_base_args, apply_axis_args};
use crate::output::write_output;

/// Contour or filled-contour plot from scattered x/y/z data.
#[derive(Args, Debug)]
pub struct ContourArgs {
    /// X column (0-based index or header name; default: 0).
    #[arg(long)]
    pub x: Option<ColSpec>,

    /// Y column (0-based index or header name; default: 1).
    #[arg(long)]
    pub y: Option<ColSpec>,

    /// Z (value) column (0-based index or header name; default: 2).
    #[arg(long)]
    pub z: Option<ColSpec>,

    /// Number of contour levels (default: 8).
    #[arg(long, default_value_t = 8)]
    pub levels: usize,

    /// Fill between contour levels with color.
    #[arg(long)]
    pub filled: bool,

    /// Color map for filled contours: viridis (default), inferno, grayscale.
    #[arg(long, default_value = "viridis")]
    pub colormap: String,

    /// Contour line color (overrides colormap for unfilled mode).
    #[arg(long)]
    pub line_color: Option<String>,

    /// Show a legend entry with this label.
    #[arg(long)]
    pub legend: Option<String>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

fn parse_colormap(name: &str) -> ColorMap {
    match name {
        "inferno" => ColorMap::Inferno,
        "grayscale" | "grey" | "gray" => ColorMap::Grayscale,
        _ => ColorMap::Viridis,
    }
}

pub fn run(args: ContourArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let x_col = args.x.unwrap_or(ColSpec::Index(0));
    let y_col = args.y.unwrap_or(ColSpec::Index(1));
    let z_col = args.z.unwrap_or(ColSpec::Index(2));

    let xs = table.col_f64(&x_col)?;
    let ys = table.col_f64(&y_col)?;
    let zs = table.col_f64(&z_col)?;

    let pts: Vec<(f64, f64, f64)> = xs.into_iter().zip(ys).zip(zs)
        .map(|((x, y), z)| (x, y, z))
        .collect();

    let mut plot = ContourPlot::new()
        .with_points(pts)
        .with_n_levels(args.levels)
        .with_colormap(parse_colormap(&args.colormap));

    if args.filled {
        plot = plot.with_filled();
    }
    if let Some(ref c) = args.line_color {
        plot = plot.with_line_color(c.clone());
    }
    if let Some(ref label) = args.legend {
        plot = plot.with_legend(label.clone());
    }

    let plots = vec![Plot::Contour(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
