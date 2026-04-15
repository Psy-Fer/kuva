use clap::Args;

use kuva::plot::{SankeyNodeColoring, SankeyNodeOrder, SankeyPlot};
use kuva::render::layout::{Layout, TickFormat};
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_base_args, palette_from_name, BaseArgs};
use crate::output::write_output;

/// Sankey flow diagram from source, target, and value columns.
#[derive(Args, Debug)]
pub struct SankeyArgs {
    /// Ordered axis columns for wide alluvium input. Repeat once per axis.
    /// When provided, Sankey is built from full alluvia rather than source-target edges.
    #[arg(long = "axis-col")]
    pub axis_cols: Vec<ColSpec>,

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

    /// Node ordering within columns: input (default) or crossings
    /// (wompwomp-style TSP/Fenwick ordering).
    #[arg(long, default_value = "input")]
    pub node_order: String,

    /// RNG seed for crossing-reduction ordering (default: 42).
    #[arg(long, default_value_t = 42)]
    pub node_order_seed: u64,

    /// Node coloring mode: label (default) or left (wompwomp-style propagation).
    #[arg(long, default_value = "label")]
    pub coloring: String,

    /// Show the absolute flow value on each ribbon.
    #[arg(long)]
    pub flow_labels: bool,

    /// Show each flow as a percentage of its source node's total outflow.
    #[arg(long)]
    pub flow_percent: bool,

    /// Number format for flow labels: auto (default), sci, integer, fixed2.
    #[arg(long, default_value = "auto")]
    pub flow_label_format: String,

    /// Unit suffix appended to each absolute flow label, e.g. "reads".
    #[arg(long)]
    pub flow_label_unit: Option<String>,

    /// Minimum ribbon height in pixels required to show a label (default 8.0; 0 = always show).
    #[arg(long)]
    pub flow_label_min_height: Option<f64>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
}

fn parse_flow_label_format(name: &str) -> TickFormat {
    match name {
        "sci" => TickFormat::Sci,
        "integer" => TickFormat::Integer,
        "fixed2" => TickFormat::Fixed(2),
        _ => TickFormat::Auto,
    }
}

fn parse_node_order(name: &str) -> Result<SankeyNodeOrder, String> {
    match name {
        "input" => Ok(SankeyNodeOrder::Input),
        "crossings" | "crossing-reduction" => Ok(SankeyNodeOrder::CrossingReduction),
        "neighbornet" | "nn" => Ok(SankeyNodeOrder::Neighbornet),
        other => Err(format!(
            "invalid node order '{other}'; expected 'input', 'crossings', or 'neighbornet'"
        )),
    }
}

fn parse_node_coloring(name: &str) -> Result<SankeyNodeColoring, String> {
    match name {
        "label" => Ok(SankeyNodeColoring::Label),
        "left" => Ok(SankeyNodeColoring::Left),
        other => Err(format!(
            "invalid coloring mode '{other}'; expected 'label' or 'left'"
        )),
    }
}

pub fn run(args: SankeyArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let mut plot = SankeyPlot::new()
        .with_node_order(parse_node_order(&args.node_order)?)
        .with_node_coloring(parse_node_coloring(&args.coloring)?)
        .with_node_order_seed(args.node_order_seed);

    if let Some(ref name) = args.base.palette {
        if let Some(pal) = palette_from_name(name) {
            plot = plot.with_palette(pal);
        }
    }

    if args.link_gradient {
        plot = plot.with_gradient_links();
    }
    if let Some(op) = args.opacity {
        plot = plot.with_link_opacity(op);
    }
    if let Some(ref label) = args.legend {
        plot = plot.with_legend(label.clone());
    }
    if args.flow_labels {
        plot = plot.with_flow_labels();
    }
    if args.flow_percent {
        plot = plot.with_flow_percent();
    }
    if args.flow_label_format != "auto" {
        plot = plot.with_flow_label_format(parse_flow_label_format(&args.flow_label_format));
    }
    if let Some(ref unit) = args.flow_label_unit {
        plot = plot.with_flow_label_unit(unit.clone());
    }
    if let Some(min_h) = args.flow_label_min_height {
        plot = plot.with_flow_label_min_height(min_h);
    }

    if !args.axis_cols.is_empty() {
        if args.axis_cols.len() < 2 {
            return Err(
                "at least two --axis-col values are required for alluvium input".to_string(),
            );
        }
        let axis_indices: Vec<usize> = args
            .axis_cols
            .iter()
            .map(|c| table.resolve(c))
            .collect::<Result<_, _>>()?;
        let axis_names: Vec<String> = axis_indices
            .iter()
            .map(|&idx| {
                table
                    .header
                    .as_ref()
                    .and_then(|header| header.get(idx))
                    .cloned()
                    .unwrap_or_else(|| format!("axis{}", idx + 1))
            })
            .collect();
        plot = plot.with_axis_names(axis_names);
        let weight_idx = match &args.value_col {
            Some(c) => Some(table.resolve(c)?),
            None => None,
        };
        for (row_i, row) in table.rows.iter().enumerate() {
            let strata: Vec<String> = axis_indices
                .iter()
                .map(|&idx| {
                    row.get(idx)
                        .cloned()
                        .ok_or_else(|| format!("Row {row_i}: no column at index {idx}"))
                })
                .collect::<Result<_, _>>()?;
            let value = match weight_idx {
                Some(idx) => row
                    .get(idx)
                    .ok_or_else(|| format!("Row {row_i}: no column at index {idx}"))?
                    .parse::<f64>()
                    .map_err(|_| format!("Row {row_i}: cannot parse '{}' as a number", row[idx]))?,
                None => 1.0,
            };
            plot = plot.with_alluvium(strata, value);
        }
    } else {
        let source_col = args.source_col.unwrap_or(ColSpec::Index(0));
        let target_col = args.target_col.unwrap_or(ColSpec::Index(1));
        let value_col = args.value_col.unwrap_or(ColSpec::Index(2));

        let sources = table.col_str(&source_col)?;
        let targets = table.col_str(&target_col)?;
        let values = table.col_f64(&value_col)?;

        for ((source, target), value) in sources.iter().zip(targets.iter()).zip(values.iter()) {
            plot = plot.with_link(source.clone(), target.clone(), *value);
        }
    }

    let plots = vec![Plot::Sankey(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
