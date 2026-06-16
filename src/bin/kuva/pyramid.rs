use clap::Args;

use kuva::plot::PopulationPyramid;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_axis_args, apply_base_args, AxisArgs, BaseArgs};
use crate::output::write_output;

/// Population pyramid — mirrored horizontal bars for two groups across age/category bands.
#[derive(Args, Debug)]
pub struct PyramidArgs {
    /// Age/category label column (0-based index or header name; default: 0).
    #[arg(long)]
    pub label_col: Option<ColSpec>,

    /// Left-bar value column (0-based index or header name; default: 1).
    #[arg(long)]
    pub left_col: Option<ColSpec>,

    /// Right-bar value column (0-based index or header name; default: 2).
    #[arg(long)]
    pub right_col: Option<ColSpec>,

    /// Label for the left side (default: "Left").
    #[arg(long)]
    pub left_label: Option<String>,

    /// Label for the right side (default: "Right").
    #[arg(long)]
    pub right_label: Option<String>,

    /// Bar color for the left side (CSS color string).
    #[arg(long)]
    pub left_color: Option<String>,

    /// Bar color for the right side (CSS color string).
    #[arg(long)]
    pub right_color: Option<String>,

    /// Normalize bars to percentages of the total.
    #[arg(long)]
    pub normalize: bool,

    /// Show value labels on bars.
    #[arg(long)]
    pub show_values: bool,

    /// Show a legend.
    #[arg(long)]
    pub legend: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,

    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: PyramidArgs) -> Result<(), String> {
    let proj: Vec<ColSpec> = vec![
        args.label_col.clone().unwrap_or(ColSpec::Index(0)),
        args.left_col.clone().unwrap_or(ColSpec::Index(1)),
        args.right_col.clone().unwrap_or(ColSpec::Index(2)),
    ];
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;

    let label_col = args.label_col.unwrap_or(ColSpec::Index(0));
    let left_col = args.left_col.unwrap_or(ColSpec::Index(1));
    let right_col = args.right_col.unwrap_or(ColSpec::Index(2));

    let labels = table.col_str(&label_col)?;
    let lefts = table.col_f64(&left_col)?;
    let rights = table.col_f64(&right_col)?;

    let n = labels.len();
    if lefts.len() != n || rights.len() != n {
        return Err(format!(
            "column length mismatch: label={}, left={}, right={}",
            n,
            lefts.len(),
            rights.len()
        ));
    }

    let mut plot = PopulationPyramid::new();

    if let Some(s) = args.left_label {
        plot = plot.with_left_label(s);
    }
    if let Some(s) = args.right_label {
        plot = plot.with_right_label(s);
    }
    if let Some(s) = args.left_color {
        plot = plot.with_left_color(s);
    }
    if let Some(s) = args.right_color {
        plot = plot.with_right_color(s);
    }
    if args.normalize {
        plot = plot.with_normalize(true);
    }
    if args.show_values {
        plot = plot.with_show_values(true);
    }
    if args.legend {
        plot = plot.with_legend(true);
    }

    for ((label, left), right) in labels.iter().zip(lefts.iter()).zip(rights.iter()) {
        plot = plot.with_group(label.as_str(), *left, *right);
    }

    let plots = vec![Plot::Pyramid(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
