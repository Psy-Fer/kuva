use clap::Args;

use kuva::plot::line::LinePlot;
use kuva::plot::scatter::ScatterPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_twin_y;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{
    apply_axis_args, apply_base_args, apply_y2_axis_args, AxisArgs, BaseArgs, Y2AxisArgs,
};
use crate::output::write_output;

/// Twin-Y (dual-axis) plot: two series sharing the same X column but plotted
/// against independent Y axes (primary on the left, secondary on the right)
/// — for two related series with incompatible scales or units.
///
/// Currently supports `line` and `scatter` as the plot type on either axis.
/// For anything more elaborate (multiple series per axis, bar/histogram/box/etc.
/// on an axis, mixed plot types), build the two `Vec<Plot>` directly with the
/// Rust API and call `render_twin_y` — see `docs/src/plots/twin_y.md`.
#[derive(Args, Debug)]
pub struct TwinYArgs {
    /// X-axis column, shared by both series (0-based index or header name; default: 0).
    #[arg(long)]
    pub x: Option<ColSpec>,

    /// Primary (left-axis) Y column (default: 1).
    #[arg(long)]
    pub y: Option<ColSpec>,

    /// Secondary (right-axis) Y column (default: 2).
    #[arg(long)]
    pub y2: Option<ColSpec>,

    /// Plot type for the primary series: line or scatter.
    #[arg(long, value_name = "TYPE", default_value = "line")]
    pub primary_type: String,

    /// Plot type for the secondary series: line or scatter.
    #[arg(long, value_name = "TYPE", default_value = "line")]
    pub secondary_type: String,

    /// Primary series color (CSS string).
    #[arg(long)]
    pub primary_color: Option<String>,

    /// Secondary series color (CSS string).
    #[arg(long)]
    pub secondary_color: Option<String>,

    /// Legend label for the primary series (used when --legend is set; defaults
    /// to the primary Y column's header/name).
    #[arg(long)]
    pub primary_legend: Option<String>,

    /// Legend label for the secondary series (used when --legend is set;
    /// defaults to the secondary Y column's header/name).
    #[arg(long)]
    pub secondary_legend: Option<String>,

    /// Show a legend identifying the primary and secondary series.
    #[arg(long)]
    pub legend: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
    #[command(flatten)]
    pub y2_axis: Y2AxisArgs,
}

pub fn run(args: TwinYArgs) -> Result<(), String> {
    let x_col = args.x.clone().unwrap_or(ColSpec::Index(0));
    let y_col = args.y.clone().unwrap_or(ColSpec::Index(1));
    let y2_col = args.y2.clone().unwrap_or(ColSpec::Index(2));

    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.header_mode(),
        args.input.delimiter,
        &[x_col.clone(), y_col.clone(), y2_col.clone()],
    )?;

    let xs = table.col_f64(&x_col)?;
    let ys = table.col_f64(&y_col)?;
    let y2s = table.col_f64(&y2_col)?;
    let primary_data: Vec<(f64, f64)> = xs.iter().copied().zip(ys).collect();
    let secondary_data: Vec<(f64, f64)> = xs.into_iter().zip(y2s).collect();

    let primary_color = args
        .primary_color
        .unwrap_or_else(|| "steelblue".to_string());
    let secondary_color = args
        .secondary_color
        .unwrap_or_else(|| "firebrick".to_string());

    let primary_legend = args
        .legend
        .then(|| {
            args.primary_legend
                .clone()
                .unwrap_or_else(|| col_display_name(&table, &y_col))
        })
        .as_deref()
        .map(str::to_string);
    let secondary_legend = args
        .legend
        .then(|| {
            args.secondary_legend
                .clone()
                .unwrap_or_else(|| col_display_name(&table, &y2_col))
        })
        .as_deref()
        .map(str::to_string);

    let primary_plot = build_xy_plot(
        &args.primary_type,
        primary_data,
        &primary_color,
        primary_legend.as_deref(),
    )?;
    let secondary_plot = build_xy_plot(
        &args.secondary_type,
        secondary_data,
        &secondary_color,
        secondary_legend.as_deref(),
    )?;

    let primary = vec![primary_plot];
    let secondary = vec![secondary_plot];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let layout = apply_y2_axis_args(layout, &args.y2_axis);

    let scene = render_twin_y(primary, secondary, layout);
    write_output(scene, &args.base)
}

/// Build a `Plot::Line` or `Plot::Scatter` from `(x, y)` data — the two plot
/// types currently supported per axis (see `TwinYArgs` doc comment for why).
fn build_xy_plot(
    kind: &str,
    data: Vec<(f64, f64)>,
    color: &str,
    legend: Option<&str>,
) -> Result<Plot, String> {
    match kind {
        "line" => {
            let mut plot = LinePlot::new().with_data(data).with_color(color);
            if let Some(label) = legend {
                plot = plot.with_legend(label);
            }
            Ok(Plot::Line(plot))
        }
        "scatter" => {
            let mut plot = ScatterPlot::new().with_data(data).with_color(color);
            if let Some(label) = legend {
                plot = plot.with_legend(label);
            }
            Ok(Plot::Scatter(plot))
        }
        other => Err(format!(
            "Unknown plot type '{other}' for --primary-type/--secondary-type: \
             expected 'line' or 'scatter'"
        )),
    }
}

/// Return a human-readable name for a column: the header name when available,
/// or "col_N" for index-based specs with no header.
fn col_display_name(table: &DataTable, col: &ColSpec) -> String {
    // Delegate to the shared resolver so an all-numeric column name (e.g. "2024")
    // shows its header label rather than "col_2024" (issue #109).
    table.col_display_name(col)
}
