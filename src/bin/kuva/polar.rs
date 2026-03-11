use clap::Args;

use kuva::plot::polar::{PolarMode, PolarPlot};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::render::palette::Palette;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, apply_base_args};
use crate::output::write_output;

/// Polar coordinate scatter/line plot.
#[derive(Args, Debug)]
pub struct PolarArgs {
    /// Column containing radial values (0-based index or header name).
    #[arg(long, default_value = "0")]
    pub r: ColSpec,

    /// Column containing angle values in degrees (0-based index or header name).
    #[arg(long, default_value = "1")]
    pub theta: ColSpec,

    /// Group by this column — one series per unique value.
    #[arg(long)]
    pub color_by: Option<ColSpec>,

    /// Plot mode: scatter or line.
    #[arg(long, default_value = "scatter")]
    pub mode: String,

    /// Maximum radial extent (default: auto from data).
    #[arg(long)]
    pub r_max: Option<f64>,

    /// Angular divisions for grid spokes (default: 12 = every 30°).
    #[arg(long, default_value_t = 12)]
    pub theta_divisions: usize,

    /// Where θ=0 appears on canvas, degrees CW from north. Default 0 = north.
    #[arg(long, default_value_t = 0.0)]
    pub theta_start: f64,

    /// Show legend.
    #[arg(long)]
    pub legend: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
}

pub fn run(args: PolarArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let mode = if args.mode == "line" { PolarMode::Line } else { PolarMode::Scatter };

    let mut plot = PolarPlot::new()
        .with_theta_divisions(args.theta_divisions)
        .with_theta_start(args.theta_start)
        .with_legend(args.legend);

    if let Some(r_max) = args.r_max {
        plot = plot.with_r_max(r_max);
    }

    if let Some(ref cb) = args.color_by {
        let pal = Palette::category10();
        let groups = table.group_by(cb)?;
        for (i, (name, subtable)) in groups.into_iter().enumerate() {
            let r_vals = subtable.col_f64(&args.r)?;
            let theta_vals = subtable.col_f64(&args.theta)?;
            plot = plot.with_series_labeled(r_vals, theta_vals, name, mode.clone());
            if let Some(s) = plot.series.last_mut() {
                s.color = Some(pal[i % pal.len()].to_string());
            }
        }
    } else {
        let r_vals = table.col_f64(&args.r)?;
        let theta_vals = table.col_f64(&args.theta)?;
        plot = match mode {
            PolarMode::Scatter => plot.with_series(r_vals, theta_vals),
            PolarMode::Line => plot.with_series_line(r_vals, theta_vals),
        };
    }

    let plots = vec![Plot::Polar(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
