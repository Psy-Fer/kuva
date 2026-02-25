use clap::Args;

use visus::plot::PiePlot;
use visus::plot::pie::PieLabelPosition;
use visus::render::layout::Layout;
use visus::render::plots::Plot;
use visus::render::render::render_multiple;
use visus::render::palette::Palette;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, apply_base_args};
use crate::output::write_output;

#[derive(Args, Debug)]
pub struct PieArgs {
    /// Label column (0-based index or header name; default: 0).
    #[arg(long)]
    pub label_col: Option<ColSpec>,

    /// Value column (0-based index or header name; default: 1).
    #[arg(long)]
    pub value_col: Option<ColSpec>,

    /// Optional color column (CSS colors). If omitted, uses the category10 palette.
    #[arg(long)]
    pub color_col: Option<ColSpec>,

    /// Render as a donut chart (inner radius in pixels; default 80 when flag is present).
    #[arg(long)]
    pub donut: bool,

    /// Inner radius for the donut hole in pixels (requires --donut; default: 80).
    #[arg(long)]
    pub inner_radius: Option<f64>,

    /// Append percentage to each slice label.
    #[arg(long)]
    pub percent: bool,

    /// Label placement: inside, outside, none (default: auto).
    #[arg(long)]
    pub label_position: Option<String>,

    /// Show a legend.
    #[arg(long)]
    pub legend: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
}

pub fn run(args: PieArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let label_col = args.label_col.unwrap_or(ColSpec::Index(0));
    let value_col = args.value_col.unwrap_or(ColSpec::Index(1));

    let labels = table.col_str(&label_col)?;
    let values = table.col_f64(&value_col)?;

    let colors: Vec<String> = if let Some(ref cc) = args.color_col {
        table.col_str(cc)?
    } else {
        let palette = Palette::category10();
        (0..labels.len()).map(|i| palette[i].to_string()).collect()
    };

    let label_pos = match args.label_position.as_deref() {
        Some("inside") => PieLabelPosition::Inside,
        Some("outside") => PieLabelPosition::Outside,
        Some("none") => PieLabelPosition::None,
        _ => PieLabelPosition::Auto,
    };

    let inner_radius = if args.donut {
        args.inner_radius.unwrap_or(80.0)
    } else {
        args.inner_radius.unwrap_or(0.0)
    };

    let mut plot = PiePlot::new()
        .with_inner_radius(inner_radius)
        .with_label_position(label_pos);

    if args.percent {
        plot = plot.with_percent();
    }
    if args.legend {
        plot = plot.with_legend("Slices");
    }

    for ((label, value), color) in labels.into_iter().zip(values).zip(colors) {
        plot = plot.with_slice(label, value, color);
    }

    let plots = vec![Plot::Pie(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
