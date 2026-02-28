use clap::Args;

use kuva::plot::SankeyPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, apply_base_args};
use crate::output::write_output;

/// Sankey flow diagram from source, target, and value columns.
#[derive(Args, Debug)]
pub struct SankeyArgs {
    /// Source node column (0-based index or header name; default: 0).
    #[arg(long)]
    pub source_col: Option<ColSpec>,

    /// Target node column (0-based index or header name; default: 1).
    #[arg(long)]
    pub target_col: Option<ColSpec>,

    /// Flow value column (0-based index or header name; default: 2).
    #[arg(long)]
    pub value_col: Option<ColSpec>,

    /// Fill each link with a gradient from the source node colour to the target node colour.
    #[arg(long)]
    pub link_gradient: bool,

    /// Link opacity 0.0–1.0 (default: 0.5).
    #[arg(long)]
    pub opacity: Option<f64>,

    /// Show a legend with this label.
    #[arg(long)]
    pub legend: Option<String>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
}

pub fn run(args: SankeyArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let source_col = args.source_col.unwrap_or(ColSpec::Index(0));
    let target_col = args.target_col.unwrap_or(ColSpec::Index(1));
    let value_col = args.value_col.unwrap_or(ColSpec::Index(2));

    let sources = table.col_str(&source_col)?;
    let targets = table.col_str(&target_col)?;
    let values = table.col_f64(&value_col)?;

    let mut plot = SankeyPlot::new();

    if args.link_gradient {
        plot = plot.with_gradient_links();
    }
    if let Some(op) = args.opacity {
        plot = plot.with_link_opacity(op);
    }
    if let Some(ref label) = args.legend {
        plot = plot.with_legend(label.clone());
    }

    for ((source, target), value) in sources.iter().zip(targets.iter()).zip(values.iter()) {
        plot = plot.with_link(source.clone(), target.clone(), *value);
    }

    let plots = vec![Plot::Sankey(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
