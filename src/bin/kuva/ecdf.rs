use clap::Args;

use kuva::plot::EcdfPlot;
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{
    apply_axis_args, apply_base_args, apply_log_args, AxisArgs, BaseArgs, LogArgs,
};
use crate::output::write_output;

/// Empirical cumulative distribution function (ECDF).
#[derive(Args, Debug)]
pub struct EcdfArgs {
    /// Column containing numeric values (0-based index or header name; default: 0).
    #[arg(long, default_value = "0")]
    pub value: ColSpec,

    /// Group by this column — one ECDF curve per unique value.
    #[arg(long)]
    pub color_by: Option<ColSpec>,

    /// Plot 1 - F(x) instead of F(x) (complementary / survival function).
    #[arg(long)]
    pub complementary: bool,

    /// Draw a DKW 95% confidence band around each curve.
    #[arg(long)]
    pub confidence_band: bool,

    /// Draw rug tick marks at each data point below the x-axis.
    #[arg(long)]
    pub rug: bool,

    /// Horizontal reference lines at these percentile levels (comma-separated, e.g. 0.25,0.5,0.75).
    #[arg(long, value_delimiter = ',')]
    pub percentile_lines: Vec<f64>,

    /// Show a dot marker at each step transition.
    #[arg(long)]
    pub markers: bool,

    /// Use a smooth KDE-integrated CDF instead of the step function.
    #[arg(long)]
    pub smooth: bool,

    /// Line stroke width (default: 1.5).
    #[arg(long)]
    pub stroke_width: Option<f64>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,

    #[command(flatten)]
    pub axis: AxisArgs,

    #[command(flatten)]
    pub log: LogArgs,
}

pub fn run(args: EcdfArgs) -> Result<(), String> {
    let mut proj: Vec<ColSpec> = vec![args.value.clone()];
    if let Some(ref c) = args.color_by { proj.push(c.clone()); }
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;

    let mut base_plot = EcdfPlot::new();
    if args.complementary {
        base_plot = base_plot.with_complementary();
    }
    if args.confidence_band {
        base_plot = base_plot.with_confidence_band();
    }
    if args.rug {
        base_plot = base_plot.with_rug();
    }
    if args.markers {
        base_plot = base_plot.with_markers();
    }
    if args.smooth {
        base_plot = base_plot.with_smooth();
    }
    if let Some(w) = args.stroke_width {
        base_plot = base_plot.with_stroke_width(w);
    }
    if !args.percentile_lines.is_empty() {
        base_plot = base_plot.with_percentile_lines(args.percentile_lines.clone());
    }

    let plots: Vec<Plot> = if let Some(ref cb) = args.color_by {
        let pal = Palette::category10();
        let groups = table.group_by(cb)?;
        let mut plot = base_plot.with_legend("");
        for (i, (name, subtable)) in groups.into_iter().enumerate() {
            let vals = subtable.col_f64(&args.value)?;
            plot = plot.with_data_colored(name, vals, pal[i].to_string());
        }
        vec![Plot::Ecdf(plot)]
    } else {
        let vals = table.col_f64(&args.value)?;
        let plot = base_plot.with_data("", vals);
        vec![Plot::Ecdf(plot)]
    };

    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let layout = apply_log_args(layout, &args.log);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
