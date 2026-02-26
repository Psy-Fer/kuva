use clap::Args;

use visus::plot::{DotPlot, ColorMap};
use visus::render::layout::Layout;
use visus::render::plots::Plot;
use visus::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, apply_base_args, apply_axis_args};
use crate::output::write_output;

/// Dot plot with size and colour encoding.
#[derive(Args, Debug)]
pub struct DotArgs {
    /// X-category column (0-based index or header name; default: 0).
    #[arg(long)]
    pub x_col: Option<ColSpec>,

    /// Y-category column (0-based index or header name; default: 1).
    #[arg(long)]
    pub y_col: Option<ColSpec>,

    /// Size-encoding column (0-based index or header name; default: 2).
    #[arg(long)]
    pub size_col: Option<ColSpec>,

    /// Color-encoding column (0-based index or header name; default: 3).
    #[arg(long)]
    pub color_col: Option<ColSpec>,

    /// Color map: viridis (default), inferno, grayscale.
    #[arg(long, default_value = "viridis")]
    pub colormap: String,

    /// Maximum dot radius in pixels (default: 12.0).
    #[arg(long)]
    pub max_radius: Option<f64>,

    /// Show a size legend with this label.
    #[arg(long)]
    pub size_legend: Option<String>,

    /// Show a color bar with this label.
    #[arg(long)]
    pub colorbar: Option<String>,

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

pub fn run(args: DotArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let x_col = args.x_col.unwrap_or(ColSpec::Index(0));
    let y_col = args.y_col.unwrap_or(ColSpec::Index(1));
    let size_col = args.size_col.unwrap_or(ColSpec::Index(2));
    let color_col = args.color_col.unwrap_or(ColSpec::Index(3));

    let xs = table.col_str(&x_col)?;
    let ys = table.col_str(&y_col)?;
    let sizes = table.col_f64(&size_col)?;
    let colors = table.col_f64(&color_col)?;

    let pts: Vec<(String, String, f64, f64)> = xs.into_iter()
        .zip(ys)
        .zip(sizes)
        .zip(colors)
        .map(|(((x, y), s), c)| (x, y, s, c))
        .collect();

    let mut plot = DotPlot::new()
        .with_data(pts)
        .with_color_map(parse_colormap(&args.colormap));

    if let Some(r) = args.max_radius {
        plot = plot.with_max_radius(r);
    }
    if let Some(ref label) = args.size_legend {
        plot = plot.with_size_legend(label.clone());
    }
    if let Some(ref label) = args.colorbar {
        plot = plot.with_colorbar(label.clone());
    }

    let plots = vec![Plot::DotPlot(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let layout = layout.with_x_tick_rotate(-45.0);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
