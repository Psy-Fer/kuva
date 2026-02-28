use clap::Args;

use kuva::plot::histogram2d::{Histogram2D, ColorMap};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, LogArgs, apply_base_args, apply_axis_args, apply_log_args};
use crate::output::write_output;

/// 2-D density histogram from two numeric columns.
#[derive(Args, Debug)]
pub struct Hist2dArgs {
    /// X-axis column (0-based index or header name; default: 0).
    #[arg(long)]
    pub x: Option<ColSpec>,

    /// Y-axis column (0-based index or header name; default: 1).
    #[arg(long)]
    pub y: Option<ColSpec>,

    /// Number of bins on the X axis (default: 10).
    #[arg(long, default_value_t = 10)]
    pub bins_x: usize,

    /// Number of bins on the Y axis (default: 10).
    #[arg(long, default_value_t = 10)]
    pub bins_y: usize,

    /// Color map: viridis (default), inferno, grayscale.
    #[arg(long, default_value = "viridis")]
    pub colormap: String,

    /// Overlay the Pearson correlation coefficient.
    #[arg(long)]
    pub correlation: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
    #[command(flatten)]
    pub log: LogArgs,
}

fn parse_colormap(name: &str) -> ColorMap {
    match name {
        "inferno" => ColorMap::Inferno,
        "grayscale" | "grey" | "gray" => ColorMap::Grayscale,
        _ => ColorMap::Viridis,
    }
}

pub fn run(args: Hist2dArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let x_col = args.x.unwrap_or(ColSpec::Index(0));
    let y_col = args.y.unwrap_or(ColSpec::Index(1));

    let xs = table.col_f64(&x_col)?;
    let ys = table.col_f64(&y_col)?;

    let data: Vec<(f64, f64)> = xs.into_iter().zip(ys).collect();

    let x_min = data.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let x_max = data.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let y_min = data.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let y_max = data.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);

    if data.is_empty() {
        return Err("hist2d input has no data".into());
    }

    let mut plot = Histogram2D::new()
        .with_data(
            data,
            (x_min, x_max),
            (y_min, y_max),
            args.bins_x,
            args.bins_y,
        )
        .with_color_map(parse_colormap(&args.colormap));

    if args.correlation {
        plot = plot.with_correlation();
    }

    let plots = vec![Plot::Histogram2d(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let layout = apply_log_args(layout, &args.log);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
