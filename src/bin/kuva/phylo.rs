use clap::Args;

use kuva::plot::{PhyloTree, TreeOrientation, TreeBranchStyle};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, apply_base_args};
use crate::output::write_output;

/// Phylogenetic tree from a Newick string or edge list.
#[derive(Args, Debug)]
pub struct PhyloArgs {
    /// Newick string input (overrides file input).
    #[arg(long)]
    pub newick: Option<String>,

    /// Parent column in edge-list TSV (default: 0).
    #[arg(long)]
    pub parent_col: Option<ColSpec>,

    /// Child column in edge-list TSV (default: 1).
    #[arg(long)]
    pub child_col: Option<ColSpec>,

    /// Branch-length column in edge-list TSV (default: 2).
    #[arg(long)]
    pub length_col: Option<ColSpec>,

    /// Tree orientation: left (default), right, top, bottom.
    #[arg(long, default_value = "left")]
    pub orientation: String,

    /// Branch style: rectangular (default), slanted, circular.
    #[arg(long, default_value = "rectangular")]
    pub branch_style: String,

    /// Draw a phylogram (branches scaled by length).
    #[arg(long)]
    pub phylogram: bool,

    /// Branch line color (CSS color string, e.g. "white", "#aaaaaa").
    #[arg(long)]
    pub branch_color: Option<String>,

    /// Show a legend with this label.
    #[arg(long)]
    pub legend: Option<String>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
}

pub fn run(args: PhyloArgs) -> Result<(), String> {
    let orientation = match args.orientation.as_str() {
        "right" => TreeOrientation::Right,
        "top" => TreeOrientation::Top,
        "bottom" => TreeOrientation::Bottom,
        _ => TreeOrientation::Left,
    };

    let branch_style = match args.branch_style.as_str() {
        "slanted" => TreeBranchStyle::Slanted,
        "circular" => TreeBranchStyle::Circular,
        _ => TreeBranchStyle::Rectangular,
    };

    let mut tree = if let Some(ref nwk) = args.newick {
        PhyloTree::from_newick(nwk)
    } else {
        let table = DataTable::parse(
            args.input.input.as_deref(),
            args.input.no_header,
            args.input.delimiter,
        )?;

        let parent_col = args.parent_col.unwrap_or(ColSpec::Index(0));
        let child_col = args.child_col.unwrap_or(ColSpec::Index(1));
        let length_col = args.length_col.unwrap_or(ColSpec::Index(2));

        let parents = table.col_str(&parent_col)?;
        let children = table.col_str(&child_col)?;
        let lengths = table.col_f64(&length_col)?;

        let edges: Vec<(String, String, f64)> = parents.into_iter()
            .zip(children)
            .zip(lengths)
            .map(|((p, c), l)| (p, c, l))
            .collect();

        let edge_refs: Vec<(&str, &str, f64)> = edges.iter()
            .map(|(p, c, l)| (p.as_str(), c.as_str(), *l))
            .collect();

        PhyloTree::from_edges(&edge_refs)
    };

    tree = tree.with_orientation(orientation).with_branch_style(branch_style);

    if let Some(ref color) = args.branch_color {
        tree = tree.with_branch_color(color.clone());
    }
    if args.phylogram {
        tree = tree.with_phylogram();
    }
    if let Some(ref label) = args.legend {
        tree = tree.with_legend(label.clone());
    }

    let plots = vec![Plot::PhyloTree(tree)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
