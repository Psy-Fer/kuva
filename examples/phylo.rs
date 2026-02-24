//! Phylogenetic tree documentation examples.
//!
//! Generates canonical SVG outputs used in the visus documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example phylo
//! ```
//!
//! SVGs are written to `docs/src/assets/phylo/`.

use visus::plot::{PhyloTree, TreeBranchStyle, TreeOrientation};
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

const OUT: &str = "docs/src/assets/phylo";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/phylo");

    basic();
    phylogram();
    circular();
    clade_color();
    upgma();

    println!("Phylo SVGs written to {OUT}/");
}

/// Basic rectangular tree from Newick, Left orientation, with support values.
fn basic() {
    let tree = PhyloTree::from_newick(
        "((TaxonA:1.0,TaxonB:2.0)95:1.0,(TaxonC:0.5,TaxonD:0.5)88:1.5,TaxonE:3.0);"
    )
    .with_support_threshold(80.0);

    let plots = vec![Plot::PhyloTree(tree)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Rectangular Tree");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Phylogram mode — branch lengths used for the depth axis, Top orientation.
fn phylogram() {
    let tree = PhyloTree::from_newick(
        "((A:1.0,B:3.0)90:1.0,(C:2.0,(D:0.5,E:1.5)85:1.0):2.0);"
    )
    .with_orientation(TreeOrientation::Top)
    .with_phylogram()
    .with_support_threshold(80.0);

    let plots = vec![Plot::PhyloTree(tree)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Phylogram (Top orientation)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/phylogram.svg"), svg).unwrap();
}

/// Circular / radial layout — 20-taxon tree.
fn circular() {
    let newick = concat!(
        "(((((Sp_A:0.05,Sp_B:0.08):0.12,(Sp_C:0.07,Sp_D:0.06):0.10):0.15,",
        "((Sp_E:0.09,Sp_F:0.11):0.08,(Sp_G:0.06,Sp_H:0.10):0.13):0.12):0.20,",
        "(((Sp_I:0.08,Sp_J:0.12):0.10,(Sp_K:0.05,Sp_L:0.09):0.11):0.15,",
        "((Sp_M:0.07,Sp_N:0.08):0.09,(Sp_O:0.10,Sp_P:0.06):0.12):0.14):0.18):0.10,",
        "((Sp_Q:0.15,Sp_R:0.12):0.20,(Sp_S:0.08,Sp_T:0.10):0.18):0.25);"
    );

    let tree = PhyloTree::from_newick(newick)
        .with_branch_style(TreeBranchStyle::Circular)
        .with_phylogram();

    let plots = vec![Plot::PhyloTree(tree)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Circular Tree — 20 Taxa");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/circular.svg"), svg).unwrap();
}

/// Clade coloring from an edge list, with a legend.
///
/// Node IDs in `from_edges` are assigned in order of first appearance:
///   0 = root, 1 = Bacteria, 2 = Eukarya, …
fn clade_color() {
    let edges: Vec<(&str, &str, f64)> = vec![
        ("root",     "Bacteria",    1.5),
        ("root",     "Eukarya",     2.0),
        ("Bacteria", "E. coli",     0.5),
        ("Bacteria", "B. subtilis", 0.7),
        ("Eukarya",  "Yeast",       1.0),
        ("Eukarya",  "Human",       0.8),
    ];

    // node 1 = Bacteria, node 2 = Eukarya
    let tree = PhyloTree::from_edges(&edges)
        .with_clade_color(1, "#e41a1c")   // Bacteria subtree → red
        .with_clade_color(2, "#377eb8")   // Eukarya subtree → blue
        .with_legend("Domains");

    let plots = vec![Plot::PhyloTree(tree)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Clade Coloring by Domain");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/clade_color.svg"), svg).unwrap();
}

/// UPGMA tree from a pairwise distance matrix.
fn upgma() {
    let labels = ["Wolf", "Cat", "Whale", "Human"];
    let dist = vec![
        vec![0.0, 0.5, 0.9, 0.8],
        vec![0.5, 0.0, 0.9, 0.8],
        vec![0.9, 0.9, 0.0, 0.7],
        vec![0.8, 0.8, 0.7, 0.0],
    ];

    let tree = PhyloTree::from_distance_matrix(&labels, &dist)
        .with_phylogram();

    let plots = vec![Plot::PhyloTree(tree)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("UPGMA Tree from Distance Matrix");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/upgma.svg"), svg).unwrap();
}
