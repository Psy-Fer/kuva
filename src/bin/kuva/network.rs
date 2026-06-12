use clap::Args;

use kuva::plot::network::{NetworkLayout, NetworkPlot};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_base_args, BaseArgs};
use crate::output::write_output;

/// Network / graph diagram from an edge list or adjacency matrix.
#[derive(Args, Debug)]
pub struct NetworkArgs {
    /// Read input as an N×N adjacency matrix instead of an edge list.
    #[arg(long)]
    pub matrix: bool,

    /// Source node column (edge-list mode; 0-based index or header name; default: 0).
    #[arg(long)]
    pub source_col: Option<ColSpec>,

    /// Target node column (edge-list mode; 0-based index or header name; default: 1).
    #[arg(long)]
    pub target_col: Option<ColSpec>,

    /// Edge weight column (edge-list mode; 0-based index or header name; optional).
    #[arg(long)]
    pub weight_col: Option<ColSpec>,

    /// Source node group column for colouring (edge-list mode; 0-based index or header name).
    /// Each row assigns the group to the source node only.
    #[arg(long)]
    pub group_col: Option<ColSpec>,

    /// Draw directed edges with arrowheads.
    #[arg(long)]
    pub directed: bool,

    /// Layout algorithm: "force" (default), "kk" (Kamada-Kawai), or "circle".
    #[arg(long, default_value = "force")]
    pub layout: String,

    /// Node radius in pixels (default: 8.0).
    #[arg(long)]
    pub node_radius: Option<f64>,

    /// Edge opacity 0.0–1.0 (default: 0.6).
    #[arg(long)]
    pub opacity: Option<f64>,

    /// Show node labels.
    #[arg(long)]
    pub labels: bool,

    /// Push overlapping labels apart.
    #[arg(long)]
    pub repel_labels: bool,

    /// Show a legend with this label.
    #[arg(long)]
    pub legend: Option<String>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
}

pub fn run(args: NetworkArgs) -> Result<(), String> {
    let proj: Vec<ColSpec> = if args.matrix {
        vec![]  // matrix mode reads all
    } else {
        let mut v = vec![
            args.source_col.clone().unwrap_or(ColSpec::Index(0)),
            args.target_col.clone().unwrap_or(ColSpec::Index(1)),
        ];
        if let Some(ref c) = args.weight_col { v.push(c.clone()); }
        if let Some(ref c) = args.group_col { v.push(c.clone()); }
        v
    };
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;

    let mut plot = NetworkPlot::new();

    if args.directed {
        plot = plot.with_directed();
    }
    if args.labels {
        plot = plot.with_labels();
    }
    if args.repel_labels {
        plot = plot.with_repel_labels();
    }

    match args.layout.as_str() {
        "force" => {}
        "kk" | "kamada-kawai" => {
            plot = plot.with_layout(NetworkLayout::KamadaKawai);
        }
        "circle" => {
            plot = plot.with_layout(NetworkLayout::Circle);
        }
        other => {
            return Err(format!(
                "unknown layout '{other}'; expected 'force', 'kk', or 'circle'"
            ))
        }
    }

    if let Some(r) = args.node_radius {
        plot = plot.with_node_radius(r);
    }
    if let Some(op) = args.opacity {
        plot = plot.with_edge_opacity(op);
    }
    if let Some(ref l) = args.legend {
        plot = plot.with_legend(l.clone());
    }

    if args.matrix {
        // ── Adjacency matrix mode ─────────────────────────────────────
        if table.rows.is_empty() {
            return Err("network matrix input has no data rows".into());
        }
        let labels: Vec<String> = if let Some(ref hdr) = table.header {
            hdr[1..].to_vec()
        } else {
            table.rows.iter().map(|r| r[0].clone()).collect()
        };

        let matrix: Vec<Vec<f64>> = table
            .rows
            .iter()
            .enumerate()
            .map(|(r, row)| {
                row[1..]
                    .iter()
                    .enumerate()
                    .map(|(c, cell)| {
                        cell.trim().parse::<f64>().map_err(|_| {
                            format!("row {r}, col {}: '{}' is not a number", c + 1, cell)
                        })
                    })
                    .collect::<Result<Vec<f64>, String>>()
            })
            .collect::<Result<Vec<_>, _>>()?;

        plot = plot.with_matrix(matrix, labels);
    } else {
        // ── Edge-list mode ────────────────────────────────────────────
        let source_col = args.source_col.unwrap_or(ColSpec::Index(0));
        let target_col = args.target_col.unwrap_or(ColSpec::Index(1));

        let sources = table.col_str(&source_col)?;
        let targets = table.col_str(&target_col)?;

        let weights: Vec<f64> = if let Some(ref wc) = args.weight_col {
            table.col_f64(wc)?
        } else {
            vec![1.0; sources.len()]
        };

        for ((src, tgt), w) in sources.iter().zip(targets.iter()).zip(weights.iter()) {
            plot = plot.with_edge(src.clone(), tgt.clone(), *w);
        }

        // Apply group column if provided.
        if let Some(ref gc) = args.group_col {
            let groups = table.col_str(gc)?;
            // Assign the group to the source node only; this avoids silently
            // overwriting a node's group when it appears as a target in a
            // later row with a different group value.
            for (src, grp) in sources.iter().zip(groups.iter()) {
                plot = plot.with_node_group(src.clone(), grp.clone());
            }
        }
    }

    let plots = vec![Plot::Network(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
