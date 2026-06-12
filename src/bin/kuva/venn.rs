use clap::Args;

use kuva::plot::venn::VennPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_base_args, BaseArgs};
use crate::output::write_output;

/// Venn diagram — set membership overlap from element/set columns.
#[derive(Args, Debug)]
pub struct VennArgs {
    /// Element column (the item; 0-based index or header name; default: 0).
    #[arg(long)]
    pub element_col: Option<ColSpec>,

    /// Set column (which set the element belongs to; default: 1).
    #[arg(long)]
    pub set_col: Option<ColSpec>,

    /// Scale circle areas proportional to set sizes.
    #[arg(long)]
    pub proportional: bool,

    /// Hide set name labels.
    #[arg(long)]
    pub no_set_labels: bool,

    /// Fill opacity for circles/ellipses (default: 0.25).
    #[arg(long)]
    pub fill_opacity: Option<f64>,

    /// Legend title.
    #[arg(long)]
    pub legend: Option<String>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
}

pub fn run(args: VennArgs) -> Result<(), String> {
    let proj: Vec<ColSpec> = vec![
        args.element_col.clone().unwrap_or(ColSpec::Index(0)),
        args.set_col.clone().unwrap_or(ColSpec::Index(1)),
    ];
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;

    let element_col = args.element_col.unwrap_or(ColSpec::Index(0));
    let set_col = args.set_col.unwrap_or(ColSpec::Index(1));

    let mut plot = VennPlot::new();

    if args.proportional {
        plot = plot.with_proportional(true);
    }
    if args.no_set_labels {
        plot = plot.with_set_labels(false);
    }
    if let Some(fo) = args.fill_opacity {
        plot = plot.with_fill_opacity(fo);
    }
    if let Some(legend) = args.legend {
        plot = plot.with_legend(legend);
    }

    let groups = table.group_by(&set_col)?;
    for (name, subtable) in groups {
        let elements = subtable.col_str(&element_col)?;
        plot = plot.with_set(name, elements);
    }

    let plots = vec![Plot::Venn(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
