use clap::Args;

use kuva::plot::hexbin::{HexbinPlot, ZReduce, ColorMap};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, LogArgs, apply_base_args, apply_axis_args, apply_log_args};
use crate::output::write_output;

/// Hexbin density plot — aggregate scatter points into hexagonal bins.
#[derive(Args, Debug)]
pub struct HexbinArgs {
    /// X-axis column (name or 0-based index; default: 0).
    #[arg(long)]
    pub x: Option<ColSpec>,

    /// Y-axis column (name or 0-based index; default: 1).
    #[arg(long)]
    pub y: Option<ColSpec>,

    /// Optional third variable column for aggregation-based coloring.
    #[arg(long)]
    pub z: Option<ColSpec>,

    /// Aggregation function for the z variable.
    #[arg(long, default_value = "count", value_enum)]
    pub reduce: CliZReduce,

    /// Target number of hex bins across the x-axis (default: 20).
    #[arg(long, default_value_t = 20)]
    pub n_bins: usize,

    /// Apply log₁₀ scaling to bin values before color mapping.
    #[arg(long)]
    pub log_color: bool,

    /// Minimum number of points per bin to render (default: 1).
    #[arg(long, default_value_t = 1)]
    pub min_count: usize,

    /// Divide counts by total points (fractional density).
    #[arg(long)]
    pub normalize: bool,

    /// Use flat-top hex orientation instead of pointy-top.
    #[arg(long)]
    pub flat_top: bool,

    /// Hex outline stroke color (CSS string; e.g. "#333333").
    #[arg(long)]
    pub stroke: Option<String>,

    /// Color map: viridis (default), inferno, turbo, grayscale.
    #[arg(long, value_enum)]
    pub colormap: Option<CliColorMap>,

    /// Hide the colorbar.
    #[arg(long)]
    pub no_colorbar: bool,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
    #[command(flatten)]
    pub log: LogArgs,
    #[command(flatten)]
    pub input: InputArgs,
}

/// Aggregation function for the z variable.
#[derive(clap::ValueEnum, Clone, Default, Debug)]
pub enum CliZReduce {
    #[default]
    Count,
    Mean,
    Sum,
    Median,
    Min,
    Max,
}

/// Color map for the hexbin color scale.
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum CliColorMap {
    Viridis,
    Inferno,
    Turbo,
    Grayscale,
}

fn cli_to_z_reduce(c: &CliZReduce) -> ZReduce {
    match c {
        CliZReduce::Count  => ZReduce::Count,
        CliZReduce::Mean   => ZReduce::Mean,
        CliZReduce::Sum    => ZReduce::Sum,
        CliZReduce::Median => ZReduce::Median,
        CliZReduce::Min    => ZReduce::Min,
        CliZReduce::Max    => ZReduce::Max,
    }
}

fn cli_to_color_map(c: Option<&CliColorMap>) -> ColorMap {
    match c {
        Some(CliColorMap::Inferno)   => ColorMap::Inferno,
        Some(CliColorMap::Turbo)     => ColorMap::Turbo,
        Some(CliColorMap::Grayscale) => ColorMap::Grayscale,
        _                            => ColorMap::Viridis,
    }
}

pub fn run(args: HexbinArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let x_col = args.x.unwrap_or(ColSpec::Index(0));
    let y_col = args.y.unwrap_or(ColSpec::Index(1));

    let xs = table.col_f64(&x_col)?;
    let ys = table.col_f64(&y_col)?;

    if xs.is_empty() {
        return Err("hexbin: input has no data".into());
    }

    let mut plot = HexbinPlot::new()
        .with_data(xs, ys)
        .with_n_bins(args.n_bins)
        .with_log_color(args.log_color)
        .with_min_count(args.min_count)
        .with_normalize(args.normalize)
        .with_flat_top(args.flat_top)
        .with_colorbar(!args.no_colorbar)
        .with_color_map(cli_to_color_map(args.colormap.as_ref()));

    if let Some(ref stroke) = args.stroke {
        plot = plot.with_stroke(stroke.clone());
    }

    if let Some(ref z_col) = args.z {
        let zs = table.col_f64(z_col)?;
        plot = plot.with_z(zs, cli_to_z_reduce(&args.reduce));
    }

    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let layout = apply_log_args(layout, &args.log);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
