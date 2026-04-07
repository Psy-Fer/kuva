use kuva::plot::{Heatmap, ColorMap, PhyloTree};
use kuva::backend::svg::SvgBackend;
use kuva::render::figure::Figure;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;


#[test]
fn test_heatmap_colorbar_values() {
    let data = vec![
        vec![10.0, 20.0, 30.0],
        vec![4.0, 50.0, 6.0],
        vec![7.0, 8.0, 90.0],
    ];

    let heatmap = Heatmap::new()
                        .with_data(data)
                        .with_values()
                        // .with_color_map(ColorMap::Grayscale);
                        .with_color_map(ColorMap::Viridis);
                        // .with_color_map(ColorMap::Inferno);

    let plots = vec![Plot::Heatmap(heatmap)];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Heatmap");
        // .with_x_categories(x_labels);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/heatmap_values.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}

#[test]
fn test_heatmap_colorbar() {
    let data = vec![
        vec![10.0, 20.0, 30.0],
        vec![4.0, 50.0, 6.0],
        vec![7.0, 8.0, 90.0],
    ];

    let heatmap = Heatmap::new()
        .with_data(data)
        .with_color_map(ColorMap::Viridis);

    let plots = vec![Plot::Heatmap(heatmap)];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Heatmap with Colorbar");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/heatmap_colorbar.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<rect")); // colorbar rects
}

/// Verify that `with_y_categories` reorders data rows such that desired_order[0]
/// lands at the TOP of the rendered heatmap (= last data row, bottom-to-top convention).
/// Desired order [C, B, A] → top-to-bottom → stored internally as [A, B, C] (bottom-to-top).
#[test]
fn test_heatmap_with_y_categories_reorders_data() {
    // Row labels in original order: A, B, C
    // Row A is distinctive: first column value is 99.0
    let data = vec![
        vec![99.0, 1.0, 2.0],  // row A
        vec![3.0,  4.0, 5.0],  // row B
        vec![6.0,  7.0, 8.0],  // row C
    ];
    let row_labels: Vec<String> = ["A", "B", "C"].iter().map(|s| s.to_string()).collect();
    let col_labels: Vec<String> = ["x", "y", "z"].iter().map(|s| s.to_string()).collect();

    // Desired top-to-bottom order: C (top), B (mid), A (bottom)
    let desired_top_to_bottom: Vec<String> = ["C", "B", "A"].iter().map(|s| s.to_string()).collect();

    let heatmap = Heatmap::new()
        .with_data(data)
        .with_labels(row_labels, col_labels)
        .with_y_categories(desired_top_to_bottom);

    // Internally stored bottom-to-top: row 0 = A (bottom), row 1 = B, row 2 = C (top)
    assert_eq!(heatmap.data[0][0], 99.0, "data row 0 (bottom) should be A");
    assert_eq!(heatmap.data[1][0], 3.0,  "data row 1 should be B");
    assert_eq!(heatmap.data[2][0], 6.0,  "data row 2 (top) should be C");

    // row_labels is bottom-to-top — can be passed directly to Layout::with_y_categories
    let expected_row_labels: &[String] = &["A", "B", "C"].iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert_eq!(heatmap.row_labels.as_deref(), Some(expected_row_labels));

    // Render to SVG for visual inspection (C at top, A at bottom)
    let layout_cats = heatmap.row_labels.clone().unwrap();
    let plots = vec![Plot::Heatmap(heatmap)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Heatmap — C top, B mid, A bottom")
        .with_y_categories(layout_cats);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/heatmap_y_categories.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

/// Verify that `with_x_categories` reorders data columns to match the given label order.
#[test]
fn test_heatmap_with_x_categories_reorders_data() {
    // Column labels in original order: x, y, z
    // Column z (index 2) has distinctive values: 10, 20, 30
    let data = vec![
        vec![1.0, 2.0, 10.0],
        vec![3.0, 4.0, 20.0],
        vec![5.0, 6.0, 30.0],
    ];
    let row_labels: Vec<String> = ["A", "B", "C"].iter().map(|s| s.to_string()).collect();
    let col_labels: Vec<String> = ["x", "y", "z"].iter().map(|s| s.to_string()).collect();

    // Desired column order: z, x, y
    let desired: Vec<String> = ["z", "x", "y"].iter().map(|s| s.to_string()).collect();

    let heatmap = Heatmap::new()
        .with_data(data)
        .with_labels(row_labels, col_labels)
        .with_x_categories(desired.clone());

    // After reordering, column 0 should be z (10, 20, 30)
    assert_eq!(heatmap.data[0][0], 10.0, "col 0 row 0 should be z-value for A");
    assert_eq!(heatmap.data[1][0], 20.0, "col 0 row 1 should be z-value for B");
    assert_eq!(heatmap.data[2][0], 30.0, "col 0 row 2 should be z-value for C");
    assert_eq!(heatmap.col_labels.as_deref(), Some(desired.as_slice()));

    let plots = vec![Plot::Heatmap(heatmap)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Heatmap — cols reordered z, x, y")
        .with_x_categories(desired);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/heatmap_x_categories.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

/// Full phylo+heatmap alignment workflow: build a tree from a distance matrix,
/// get leaf order, reorder heatmap rows to match, and render both side by side
/// using Figure so the tree and heatmap appear in adjacent cells.
#[test]
fn test_phylo_heatmap_alignment() {
    let labels_str = ["Wolf", "Cat", "Whale", "Human"];
    let labels: Vec<String> = labels_str.iter().map(|s| s.to_string()).collect();

    // Pairwise distance matrix — Wolf/Cat close, Whale/Human close
    let dist = vec![
        vec![0.0, 0.5, 0.9, 0.8],
        vec![0.5, 0.0, 0.9, 0.8],
        vec![0.9, 0.9, 0.0, 0.7],
        vec![0.8, 0.8, 0.7, 0.0],
    ];

    let tree = PhyloTree::from_distance_matrix(&labels_str, &dist).with_phylogram();
    let leaf_order = tree.leaf_labels_top_to_bottom(); // top-to-bottom tree order

    // Build a heatmap using the same distance matrix, rows reordered to match tree.
    // with_y_categories takes top-to-bottom order; first leaf ends up at the top of
    // the heatmap. row_labels is stored bottom-to-top for Layout::with_y_categories.
    let heatmap = Heatmap::new()
        .with_data(dist)
        .with_labels(labels, vec![])
        .with_y_categories(leaf_order.clone());

    // The last leaf (bottom of tree) should be in data row 0 (bottom of heatmap).
    let last_leaf = leaf_order.last().unwrap().as_str();
    let last_leaf_idx_in_original = labels_str
        .iter().position(|&s| s == last_leaf).unwrap();
    assert_eq!(
        heatmap.data[0][last_leaf_idx_in_original], 0.0,
        "diagonal must be 0.0: bottom-of-tree leaf should be in data row 0"
    );

    // row_labels is bottom-to-top — pass directly to Layout::with_y_categories
    let layout_cats = heatmap.row_labels.clone().unwrap();

    let tree_plots = vec![Plot::PhyloTree(tree)];
    let heatmap_plots = vec![Plot::Heatmap(heatmap)];

    let tree_layout = Layout::auto_from_plots(&tree_plots)
        .with_title("UPGMA Tree");
    let heatmap_layout = Layout::auto_from_plots(&heatmap_plots)
        .with_title("Distance Matrix")
        .with_y_categories(layout_cats);

    // 1 row × 2 cols: tree on left, heatmap on right
    let figure = Figure::new(1, 2)
        .with_plots(vec![tree_plots, heatmap_plots])
        .with_layouts(vec![tree_layout, heatmap_layout])
        .with_title("Phylo + Heatmap — aligned leaf order");

    let svg = SvgBackend.render_scene(&figure.render());
    std::fs::write("test_outputs/heatmap_phylo_alignment.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}
