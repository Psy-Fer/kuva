use clap::Args;

use kuva::plot::streamgraph::{StreamBaseline, StreamOrder, StreamgraphPlot};
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_axis_args, apply_base_args, palette_from_name, AxisArgs, BaseArgs};
use crate::output::write_output;

/// Streamgraph (flowing stacked area with displaced baseline).
#[derive(Args, Debug)]
pub struct StreamgraphArgs {
    /// X-axis column (0-based index or header name; default: 0).
    #[arg(long)]
    pub x_col: Option<ColSpec>,

    /// Group / category column (default: 1).
    #[arg(long)]
    pub group_col: Option<ColSpec>,

    /// Y-axis value column (default: 2).
    #[arg(long)]
    pub y_col: Option<ColSpec>,

    /// Baseline algorithm: wiggle (default), symmetric, zero.
    #[arg(long, default_value = "wiggle")]
    pub baseline: String,

    /// Layer ordering: inside-out (default), by-total, original.
    #[arg(long, default_value = "inside-out")]
    pub order: String,

    /// Use straight line segments instead of Catmull-Rom splines.
    #[arg(long)]
    pub linear: bool,

    /// Normalise each column to 100 %.
    #[arg(long)]
    pub normalize: bool,

    /// Draw thin white strokes between adjacent streams.
    #[arg(long)]
    pub stroke: bool,

    /// Hide inline stream labels.
    #[arg(long)]
    pub no_labels: bool,

    /// Minimum pixel height before an inline label is drawn (default: 14).
    #[arg(long)]
    pub min_label_height: Option<f64>,

    /// Fill opacity for each band (default: 0.85).
    #[arg(long)]
    pub fill_opacity: Option<f64>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: StreamgraphArgs) -> Result<(), String> {
    let proj: Vec<ColSpec> = vec![
        args.x_col.clone().unwrap_or(ColSpec::Index(0)),
        args.group_col.clone().unwrap_or(ColSpec::Index(1)),
        args.y_col.clone().unwrap_or(ColSpec::Index(2)),
    ];
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;

    let x_col = args.x_col.unwrap_or(ColSpec::Index(0));
    let group_col = args.group_col.unwrap_or(ColSpec::Index(1));
    let y_col = args.y_col.unwrap_or(ColSpec::Index(2));

    let groups = table.group_by(&group_col)?;

    let palette = args
        .base
        .palette
        .as_deref()
        .and_then(palette_from_name)
        .unwrap_or_else(Palette::category10);

    let baseline = match args.baseline.as_str() {
        "symmetric" | "sym" => StreamBaseline::Symmetric,
        "zero" => StreamBaseline::Zero,
        _ => StreamBaseline::Wiggle,
    };
    let order = match args.order.as_str() {
        "by-total" | "total" => StreamOrder::ByTotal,
        "original" | "orig" => StreamOrder::Original,
        _ => StreamOrder::InsideOut,
    };

    let mut plot = StreamgraphPlot::new()
        .with_baseline(baseline)
        .with_order(order);

    if args.linear {
        plot = plot.with_linear();
    }
    if args.normalize {
        plot = plot.with_normalized();
    }
    if args.stroke {
        plot = plot.with_stroke();
    }
    if args.no_labels {
        plot = plot.with_stream_labels(false);
    }
    if let Some(h) = args.min_label_height {
        plot = plot.with_min_label_height(h);
    }
    if let Some(op) = args.fill_opacity {
        plot = plot.with_fill_opacity(op);
    }

    let mut x_set = false;
    for (i, (name, subtable)) in groups.into_iter().enumerate() {
        let xs = subtable.col_f64(&x_col)?;
        let ys = subtable.col_f64(&y_col)?;

        if !x_set {
            plot = plot.with_x(xs);
            x_set = true;
        }

        let color = palette[i].to_string();
        plot = plot.with_series(ys).with_color(color).with_label(name);
    }

    // Auto-set axis labels
    let x_label = args.axis.x_label.as_deref().unwrap_or("").to_string();
    let y_label = args.axis.y_label.as_deref().unwrap_or("").to_string();

    let plots = vec![Plot::Streamgraph(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let mut axis_args = args.axis;
    if x_label.is_empty() {
        axis_args.x_label = None;
    }
    if y_label.is_empty() {
        axis_args.y_label = None;
    }
    let layout = apply_axis_args(layout, &axis_args);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
