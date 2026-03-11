use clap::Args;

use kuva::plot::ternary::TernaryPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, apply_base_args};
use crate::output::write_output;

/// Ternary (simplex) scatter plot with barycentric coordinate system.
#[derive(Args, Debug)]
pub struct TernaryArgs {
    /// Column for the top-vertex (A) component (0-based index or header name).
    #[arg(long, default_value = "0")]
    pub a: ColSpec,

    /// Column for the bottom-left (B) component (0-based index or header name).
    #[arg(long, default_value = "1")]
    pub b: ColSpec,

    /// Column for the bottom-right (C) component (0-based index or header name).
    #[arg(long, default_value = "2")]
    pub c: ColSpec,

    /// Group by this column for colored series.
    #[arg(long)]
    pub color_by: Option<ColSpec>,

    /// Label for the top (A) vertex.
    #[arg(long, default_value = "A")]
    pub a_label: String,

    /// Label for the bottom-left (B) vertex.
    #[arg(long, default_value = "B")]
    pub b_label: String,

    /// Label for the bottom-right (C) vertex.
    #[arg(long, default_value = "C")]
    pub c_label: String,

    /// Normalize each row so a+b+c=1.
    #[arg(long)]
    pub normalize: bool,

    /// Number of grid lines per axis (default: 5).
    #[arg(long, default_value_t = 5)]
    pub grid_lines: usize,

    /// Show legend.
    #[arg(long)]
    pub legend: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
}

pub fn run(args: TernaryArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let mut plot = TernaryPlot::new()
        .with_corner_labels(args.a_label.clone(), args.b_label.clone(), args.c_label.clone())
        .with_normalize(args.normalize)
        .with_grid_lines(args.grid_lines)
        .with_legend(args.legend);

    if let Some(ref cb) = args.color_by {
        let groups = table.group_by(cb)?;
        for (name, subtable) in groups {
            let a_vals = subtable.col_f64(&args.a)?;
            let b_vals = subtable.col_f64(&args.b)?;
            let c_vals = subtable.col_f64(&args.c)?;
            for ((a, b), c) in a_vals.iter().zip(b_vals.iter()).zip(c_vals.iter()) {
                plot = plot.with_point_group(*a, *b, *c, name.clone());
            }
        }
    } else {
        let a_vals = table.col_f64(&args.a)?;
        let b_vals = table.col_f64(&args.b)?;
        let c_vals = table.col_f64(&args.c)?;
        for ((a, b), c) in a_vals.iter().zip(b_vals.iter()).zip(c_vals.iter()) {
            plot = plot.with_point(*a, *b, *c);
        }
    }

    let plots = vec![Plot::Ternary(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
