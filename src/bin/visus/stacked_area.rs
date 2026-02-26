use clap::Args;

use visus::plot::StackedAreaPlot;
use visus::render::layout::Layout;
use visus::render::palette::Palette;
use visus::render::plots::Plot;
use visus::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, apply_base_args, apply_axis_args, palette_from_name};
use crate::output::write_output;

#[derive(Args, Debug)]
pub struct StackedAreaArgs {
    /// X-axis column (0-based index or header name; default: 0).
    #[arg(long)]
    pub x_col: Option<ColSpec>,

    /// Group column (0-based index or header name; default: 1).
    #[arg(long)]
    pub group_col: Option<ColSpec>,

    /// Y-axis column (0-based index or header name; default: 2).
    #[arg(long)]
    pub y_col: Option<ColSpec>,

    /// Enable 100% normalized stacking.
    #[arg(long)]
    pub normalize: bool,

    /// Fill opacity for each band (default: 0.7).
    #[arg(long)]
    pub fill_opacity: Option<f64>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: StackedAreaArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let x_col = args.x_col.unwrap_or(ColSpec::Index(0));
    let group_col = args.group_col.unwrap_or(ColSpec::Index(1));
    let y_col = args.y_col.unwrap_or(ColSpec::Index(2));

    // group_by preserves insertion order
    let groups = table.group_by(&group_col)?;

    // Choose palette: use --palette arg if given, else category10
    let palette = args.base.palette.as_deref()
        .and_then(palette_from_name)
        .unwrap_or_else(Palette::category10);

    let mut plot = StackedAreaPlot::new();

    if args.normalize {
        plot = plot.with_normalized();
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
        plot = plot
            .with_series(ys)
            .with_color(color)
            .with_legend(name);
    }

    let plots = vec![Plot::StackedArea(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
