use visus::plot::{PhyloTree, TreeBranchStyle, TreeOrientation};
use visus::render::{plots::Plot, layout::Layout, render::render_multiple};
use visus::backend::svg::SvgBackend;
use visus::render_phylo_tree;

fn svg_with_title(tree: PhyloTree, title: Option<&str>) -> String {
    let plots = vec![Plot::PhyloTree(tree.clone())];
    let mut layout = Layout::auto_from_plots(&plots);
    if let Some(t) = title {
        layout = layout.with_title(t);
    }
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

fn write(name: &str, svg: &str) {
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write(format!("test_outputs/{name}.svg"), svg).unwrap();
}

/// 1. Newick, Left, Rectangular (default) — 5 leaves, with title
#[test]
fn test_phylo_newick_basic() {
    let tree = PhyloTree::from_newick(
        "((TaxonA:1.0,TaxonB:2.0)95:1.0,(TaxonC:0.5,TaxonD:0.5)88:1.5,TaxonE:3.0);"
    ).with_support_threshold(80.0);

    let svg = svg_with_title(tree, Some("Rectangular tree (Left) — 5 leaves"));
    write("phylo_newick_basic", &svg);

    assert!(svg.contains("<svg"), "output should be SVG");
    assert!(svg.contains("TaxonA"), "leaf label should appear");
    assert!(svg.contains("TaxonE"), "leaf label should appear");
    // Support value 95 should appear (above threshold 80)
    assert!(svg.contains("95"), "support value should appear");
}

/// 2. Same tree, Slanted branches, Right orientation
#[test]
fn test_phylo_slanted() {
    let tree = PhyloTree::from_newick("((A:1,B:2):1,C:3);")
        .with_branch_style(TreeBranchStyle::Slanted)
        .with_orientation(TreeOrientation::Right)
        .with_branch_color("#555");

    let svg = svg_with_title(tree, Some("Slanted branches (Right)"));
    write("phylo_slanted", &svg);
    assert!(svg.contains("<svg"), "output should be SVG");
}

/// 3. Circular style, with title
#[test]
fn test_phylo_circular() {
    let tree = PhyloTree::from_newick("((A:1,B:2):1,(C:0.5,D:0.5):1,(E:1.5,F:0.8):0.5);")
        .with_branch_style(TreeBranchStyle::Circular);

    let svg = svg_with_title(tree, Some("Circular / radial layout"));
    write("phylo_circular", &svg);
    assert!(svg.contains("<svg"), "circular should render without panic");
}

/// 4. Top orientation, phylogram mode, with title
#[test]
fn test_phylo_top_phylogram() {
    let tree = PhyloTree::from_newick("((A:1.0,B:3.0):1.0,(C:2.0,D:0.5):2.0);")
        .with_orientation(TreeOrientation::Top)
        .with_phylogram();

    let svg = svg_with_title(tree, Some("Phylogram — Top orientation"));
    write("phylo_top_phylogram", &svg);
    assert!(svg.contains("<svg"), "top orientation should render");
}

/// 5. Bottom orientation
#[test]
fn test_phylo_bottom() {
    let tree = PhyloTree::from_newick("((A:1,B:2):1,(C:0.5,D:0.5):1);")
        .with_orientation(TreeOrientation::Bottom);

    let svg = svg_with_title(tree, Some("Bottom orientation"));
    write("phylo_bottom", &svg);
    assert!(svg.contains("<svg"));
}

/// 6. from_distance_matrix — UPGMA clustering of 4 taxa
#[test]
fn test_phylo_upgma() {
    let labels = ["Wolf", "Cat", "Whale", "Human"];
    let dist = vec![
        vec![0.0, 0.5, 0.9, 0.8],
        vec![0.5, 0.0, 0.9, 0.8],
        vec![0.9, 0.9, 0.0, 0.7],
        vec![0.8, 0.8, 0.7, 0.0],
    ];
    let tree = PhyloTree::from_distance_matrix(&labels, &dist);

    assert_eq!(
        tree.nodes.iter().filter(|n| n.children.is_empty()).count(),
        4,
        "should have 4 leaves"
    );

    let svg = svg_with_title(tree, Some("UPGMA tree"));
    write("phylo_upgma", &svg);
    assert!(svg.contains("<svg"), "UPGMA tree should render");
    assert!(svg.contains("Wolf") || svg.contains("Cat"), "leaf labels should appear");
}

/// 7. from_edges with clade coloring and legend
#[test]
fn test_phylo_clade_color() {
    let edges: Vec<(&str, &str, f64)> = vec![
        ("root", "Bacteria", 1.5),
        ("root", "Eukarya", 2.0),
        ("Bacteria", "E. coli", 0.5),
        ("Bacteria", "B. subtilis", 0.7),
        ("Eukarya", "Yeast", 1.0),
        ("Eukarya", "Human", 0.8),
    ];
    // node id 1 = "Bacteria", id 2 = "Eukarya"
    let tree = PhyloTree::from_edges(&edges)
        .with_clade_color(1, "#e41a1c")
        .with_clade_color(2, "#377eb8")
        .with_legend("Domains");

    let svg = svg_with_title(tree, Some("Clade coloring by domain"));
    write("phylo_clade_color", &svg);
    assert!(svg.contains("<svg"), "clade color tree should render");
    assert!(svg.contains("#e41a1c"), "red clade color should appear in SVG");
    assert!(svg.contains("#377eb8"), "blue clade color should appear in SVG");
}

/// 8. render_phylo_tree standalone function
#[test]
fn test_phylo_render_standalone() {
    let tree = PhyloTree::from_newick("(Alpha:1,(Beta:1,Gamma:1):1);");
    let plots = vec![Plot::PhyloTree(tree.clone())];
    let layout = Layout::auto_from_plots(&plots).with_title("Standalone render");
    let svg = SvgBackend.render_scene(&render_phylo_tree(&tree, &layout));
    write("phylo_standalone", &svg);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Alpha"));
}

/// 9. Big tree — 20 leaves, Left rectangular, with title
#[test]
fn test_phylo_big_tree() {
    let newick = concat!(
        "(((((Sp_A:0.05,Sp_B:0.08):0.12,(Sp_C:0.07,Sp_D:0.06):0.10):0.15,",
        "((Sp_E:0.09,Sp_F:0.11):0.08,(Sp_G:0.06,Sp_H:0.10):0.13):0.12):0.20,",
        "(((Sp_I:0.08,Sp_J:0.12):0.10,(Sp_K:0.05,Sp_L:0.09):0.11):0.15,",
        "((Sp_M:0.07,Sp_N:0.08):0.09,(Sp_O:0.10,Sp_P:0.06):0.12):0.14):0.18):0.10,",
        "((Sp_Q:0.15,Sp_R:0.12):0.20,(Sp_S:0.08,Sp_T:0.10):0.18):0.25);"
    );

    let tree = PhyloTree::from_newick(newick);
    let leaf_count = tree.nodes.iter().filter(|n| n.children.is_empty()).count();
    assert_eq!(leaf_count, 20, "should have 20 leaves");

    let svg = svg_with_title(tree, Some("20-taxon phylogenetic tree (phylogram)"));
    write("phylo_big_tree", &svg);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Sp_A"));
    assert!(svg.contains("Sp_T"));
}

/// 10. Big circular tree — 20 leaves
#[test]
fn test_phylo_big_circular() {
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

    let svg = svg_with_title(tree, Some("20-taxon radial tree"));
    write("phylo_big_circular", &svg);
    assert!(svg.contains("<svg"));
}

/// 11. leaf_labels_top_to_bottom matches render order
#[test]
fn test_phylo_leaf_order() {
    let tree = PhyloTree::from_newick("((A:1,B:2):1,C:3);");
    let labels = tree.leaf_labels_top_to_bottom();
    assert_eq!(labels, vec!["A", "B", "C"]);
}
