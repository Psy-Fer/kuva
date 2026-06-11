use clap::Args;

use kuva::plot::{QuiverPivot, QuiverPlot};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{parse_colormap, ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_axis_args, apply_base_args, AxisArgs, BaseArgs};
use crate::output::write_output;

#[derive(clap::ValueEnum, Clone, Default, Debug)]
pub enum CliPivot {
    #[default]
    Tail,
    Middle,
    Tip,
}

impl From<CliPivot> for QuiverPivot {
    fn from(p: CliPivot) -> Self {
        match p {
            CliPivot::Tail => QuiverPivot::Tail,
            CliPivot::Middle => QuiverPivot::Middle,
            CliPivot::Tip => QuiverPivot::Tip,
        }
    }
}

/// Quiver plot — 2-D vector field rendered as arrows.
#[derive(Args, Debug)]
pub struct QuiverArgs {
    /// Tail X column (0-based index or header name; default: 0).
    #[arg(long)]
    pub x_col: Option<ColSpec>,

    /// Tail Y column (0-based index or header name; default: 1).
    #[arg(long)]
    pub y_col: Option<ColSpec>,

    /// Vector U (x-component) column (0-based index or header name; default: 2).
    #[arg(long)]
    pub u_col: Option<ColSpec>,

    /// Vector V (y-component) column (0-based index or header name; default: 3).
    #[arg(long)]
    pub v_col: Option<ColSpec>,

    /// Arrow color (CSS color string).
    #[arg(long)]
    pub color: Option<String>,

    /// Scale multiplier applied to `(u, v)` before axis mapping.
    #[arg(long)]
    pub arrow_scale: Option<f64>,

    /// Fraction of the nearest-neighbor distance for the longest arrow.
    /// Default `0.9` (auto-scaling is on). Mutually exclusive with `--arrow-scale`.
    #[arg(long, conflicts_with = "arrow_scale")]
    pub auto_scale: Option<f64>,

    /// Shaft stroke width in pixels.
    #[arg(long)]
    pub shaft_width: Option<f64>,

    /// Arrow head length in pixels.
    #[arg(long)]
    pub head_length: Option<f64>,

    /// Arrow head half-width in pixels.
    #[arg(long)]
    pub head_width: Option<f64>,

    /// Colormap name — arrows are colored by magnitude.
    /// See `parse_colormap` in data.rs for accepted names.
    #[arg(long)]
    pub colormap: Option<String>,

    /// Colorbar label (only applies when `--colormap` is set).
    #[arg(long)]
    pub colorbar_label: Option<String>,

    /// Legend label for the series.
    #[arg(long)]
    pub legend: Option<String>,

    /// Derive axis bounds from arrow tails only (arrows may overflow the plot box).
    #[arg(long)]
    pub tight_bounds: bool,

    /// Force arrow clipping to the plot rectangle regardless of bounds mode.
    #[arg(long, conflicts_with = "no_clip")]
    pub clip: bool,

    /// Disable arrow clipping even when --tight-bounds is set.
    #[arg(long)]
    pub no_clip: bool,

    /// Where (x, y) sits on each arrow: `tail` (default), `middle`, or `tip`.
    #[arg(long, value_enum)]
    pub pivot: Option<CliPivot>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,

    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: QuiverArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let x_col = args.x_col.unwrap_or(ColSpec::Index(0));
    let y_col = args.y_col.unwrap_or(ColSpec::Index(1));
    let u_col = args.u_col.unwrap_or(ColSpec::Index(2));
    let v_col = args.v_col.unwrap_or(ColSpec::Index(3));

    let xs = table.col_f64(&x_col)?;
    let ys = table.col_f64(&y_col)?;
    let us = table.col_f64(&u_col)?;
    let vs = table.col_f64(&v_col)?;

    let n = xs.len();
    if ys.len() != n || us.len() != n || vs.len() != n {
        return Err(format!(
            "column length mismatch: x={}, y={}, u={}, v={}",
            n,
            ys.len(),
            us.len(),
            vs.len()
        ));
    }

    let mut plot = QuiverPlot::new().with_arrows(
        xs.iter()
            .zip(&ys)
            .zip(&us)
            .zip(&vs)
            .map(|(((x, y), u), v)| (*x, *y, *u, *v)),
    );

    if let Some(c) = args.color {
        plot = plot.with_color(c);
    }
    if let Some(s) = args.arrow_scale {
        plot = plot.with_scale(s);
    }
    if let Some(f) = args.auto_scale {
        plot = plot.with_auto_scale(f);
    }
    if let Some(w) = args.shaft_width {
        plot = plot.with_shaft_width(w);
    }
    if let Some(l) = args.head_length {
        plot = plot.with_head_length(l);
    }
    if let Some(w) = args.head_width {
        plot = plot.with_head_width(w);
    }

    if let Some(name) = args.colormap {
        plot = plot.with_color_map(parse_colormap(&name));
    }
    if let Some(label) = args.colorbar_label {
        plot = plot.with_color_legend_label(label);
    }
    if let Some(s) = args.legend {
        plot = plot.with_legend(s);
    }
    if args.tight_bounds {
        plot = plot.with_tight_bounds();
    }
    if args.clip {
        plot = plot.with_clip_to_plot_area();
    }
    if args.no_clip {
        plot = plot.with_no_clip();
    }
    if let Some(p) = args.pivot {
        plot = plot.with_pivot(p.into());
    }

    let plots = vec![Plot::Quiver(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
