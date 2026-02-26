use clap::Args;

use visus::plot::scatter::{ScatterPlot, TrendLine};
use visus::render::layout::Layout;
use visus::render::plots::Plot;
use visus::render::render::render_multiple;
use visus::render::palette::Palette;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, LogArgs, apply_base_args, apply_axis_args, apply_log_args};
use crate::output::write_output;

/// Scatter plot from two numeric columns.
#[derive(Args, Debug)]
pub struct ScatterArgs {
    /// X-axis column (0-based index or header name; default: 0).
    #[arg(long)]
    pub x: Option<ColSpec>,

    /// Y-axis column (0-based index or header name; default: 1).
    #[arg(long)]
    pub y: Option<ColSpec>,

    /// Group rows by this column and render each group as a separate series.
    #[arg(long)]
    pub color_by: Option<ColSpec>,

    /// Point color (CSS string). Ignored when --color-by is used.
    #[arg(long)]
    pub color: Option<String>,

    /// Point radius in pixels (default: 3.0).
    #[arg(long)]
    pub size: Option<f64>,

    /// Overlay a linear trend line.
    #[arg(long)]
    pub trend: bool,

    /// Annotate with the regression equation (requires --trend).
    #[arg(long)]
    pub equation: bool,

    /// Annotate with the Pearson R² value (requires --trend).
    #[arg(long)]
    pub correlation: bool,

    /// Show a legend for each series.
    #[arg(long)]
    pub legend: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
    #[command(flatten)]
    pub log: LogArgs,
}

pub fn run(args: ScatterArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let x_col = args.x.unwrap_or(ColSpec::Index(0));
    let y_col = args.y.unwrap_or(ColSpec::Index(1));
    let color = args.color.unwrap_or_else(|| "steelblue".to_string());
    let size = args.size.unwrap_or(3.0);
    let trend = args.trend;
    let equation = args.equation;
    let correlation = args.correlation;
    let legend = args.legend;

    let plots: Vec<Plot> = if let Some(color_by) = args.color_by {
        let groups = table.group_by(&color_by)?;
        let palette = Palette::category10();
        let colors: Vec<String> = (0..groups.len()).map(|i| palette[i].to_string()).collect();

        groups
            .into_iter()
            .zip(colors)
            .map(|((name, subtable), grp_color)| {
                let xs = subtable.col_f64(&x_col)?;
                let ys = subtable.col_f64(&y_col)?;
                let data: Vec<(f64, f64)> = xs.into_iter().zip(ys).collect();

                let mut plot = ScatterPlot::new()
                    .with_data(data)
                    .with_color(&grp_color)
                    .with_size(size);

                if trend {
                    plot = plot.with_trend(TrendLine::Linear);
                    if equation { plot = plot.with_equation(); }
                    if correlation { plot = plot.with_correlation(); }
                }
                if legend {
                    plot = plot.with_legend(name);
                }
                Ok(Plot::Scatter(plot))
            })
            .collect::<Result<Vec<_>, String>>()?
    } else {
        let xs = table.col_f64(&x_col)?;
        let ys = table.col_f64(&y_col)?;
        let data: Vec<(f64, f64)> = xs.into_iter().zip(ys).collect();

        let mut plot = ScatterPlot::new()
            .with_data(data)
            .with_color(&color)
            .with_size(size);

        if trend {
            plot = plot.with_trend(TrendLine::Linear);
            if equation { plot = plot.with_equation(); }
            if correlation { plot = plot.with_correlation(); }
        }

        vec![Plot::Scatter(plot)]
    };

    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let layout = apply_log_args(layout, &args.log);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
