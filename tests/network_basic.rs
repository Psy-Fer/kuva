mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::network::{NetworkLayout, NetworkPlot, NodeShape};
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

#[test]
fn network_basic() {
    let net = NetworkPlot::new()
        .with_edge("A", "B", 1.0)
        .with_edge("A", "C", 1.0)
        .with_edge("B", "C", 1.0)
        .with_edge("C", "D", 1.0)
        .with_labels();
    let plots = vec![Plot::Network(net)];
    let layout = Layout::auto_from_plots(&plots).with_title("Basic Network");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/network_basic.svg", svg).unwrap();
}

#[test]
fn network_directed() {
    let net = NetworkPlot::new()
        .with_edge("A", "B", 1.0)
        .with_edge("A", "C", 2.0)
        .with_edge("B", "C", 1.5)
        .with_edge("C", "D", 3.0)
        .with_edge("D", "A", 0.5)
        .with_directed()
        .with_labels();
    let plots = vec![Plot::Network(net)];
    let layout = Layout::auto_from_plots(&plots).with_title("Directed Network");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/network_directed.svg", svg).unwrap();
}

#[test]
fn network_circle_layout() {
    let net = NetworkPlot::new()
        .with_edges([
            ("A", "B", 1.0),
            ("B", "C", 1.0),
            ("C", "D", 1.0),
            ("D", "E", 1.0),
            ("E", "F", 1.0),
            ("F", "A", 1.0),
            ("A", "D", 0.5),
            ("B", "E", 0.5),
            ("C", "F", 0.5),
        ])
        .with_layout(NetworkLayout::Circle)
        .with_labels();
    let plots = vec![Plot::Network(net)];
    let layout = Layout::auto_from_plots(&plots).with_title("Circle Layout");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/network_circle.svg", svg).unwrap();
}

#[test]
fn network_self_loop() {
    let net = NetworkPlot::new()
        .with_edge("A", "B", 1.0)
        .with_edge("B", "C", 1.0)
        .with_edge("C", "C", 1.0) // self-loop
        .with_edge("C", "A", 1.0)
        .with_directed()
        .with_labels();
    let plots = vec![Plot::Network(net)];
    let layout = Layout::auto_from_plots(&plots).with_title("Self-Loop");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/network_self_loop.svg", svg).unwrap();
}

#[test]
fn network_matrix() {
    let matrix = vec![
        vec![0.0, 1.0, 1.0, 0.0],
        vec![1.0, 0.0, 1.0, 1.0],
        vec![1.0, 1.0, 0.0, 1.0],
        vec![0.0, 1.0, 1.0, 0.0],
    ];
    let net = NetworkPlot::new()
        .with_matrix(matrix, ["Alpha", "Beta", "Gamma", "Delta"])
        .with_labels();
    let plots = vec![Plot::Network(net)];
    let layout = Layout::auto_from_plots(&plots).with_title("From Adjacency Matrix");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/network_matrix.svg", svg).unwrap();
}

#[test]
fn network_groups_legend() {
    let net = NetworkPlot::new()
        .with_edge("A", "B", 1.0)
        .with_edge("A", "C", 1.0)
        .with_edge("B", "D", 1.0)
        .with_edge("C", "D", 1.0)
        .with_node_group("A", "Input")
        .with_node_group("B", "Hidden")
        .with_node_group("C", "Hidden")
        .with_node_group("D", "Output")
        .with_labels()
        .with_legend("Layer");
    let plots = vec![Plot::Network(net)];
    let layout = Layout::auto_from_plots(&plots).with_title("Grouped Network");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/network_groups_legend.svg", svg).unwrap();
}

#[test]
fn network_weighted() {
    let net = NetworkPlot::new()
        .with_edge("A", "B", 1.0)
        .with_edge("A", "C", 5.0)
        .with_edge("B", "C", 2.0)
        .with_edge("C", "D", 10.0)
        .with_edge("D", "E", 0.5)
        .with_labels();
    let plots = vec![Plot::Network(net)];
    let layout = Layout::auto_from_plots(&plots).with_title("Weighted Edges");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/network_weighted.svg", svg).unwrap();
}

#[test]
fn network_node_sizes() {
    let net = NetworkPlot::new()
        .with_edge("Hub", "A", 1.0)
        .with_edge("Hub", "B", 1.0)
        .with_edge("Hub", "C", 1.0)
        .with_edge("Hub", "D", 1.0)
        .with_edge("A", "B", 1.0)
        .with_node_size("Hub", 20.0)
        .with_node_size("A", 12.0)
        .with_node_size("B", 8.0)
        .with_node_size("C", 5.0)
        .with_node_size("D", 3.0)
        .with_labels();
    let plots = vec![Plot::Network(net)];
    let layout = Layout::auto_from_plots(&plots).with_title("Variable Node Sizes");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/network_node_sizes.svg", svg).unwrap();
}

#[test]
fn network_disconnected() {
    // Three separate connected components with no edges between them.
    let net = NetworkPlot::new()
        // Component 1: triangle
        .with_edge("A1", "A2", 1.0)
        .with_edge("A2", "A3", 1.0)
        .with_edge("A3", "A1", 1.0)
        // Component 2: pair
        .with_edge("B1", "B2", 1.0)
        // Component 3: star
        .with_edge("C1", "C2", 1.0)
        .with_edge("C1", "C3", 1.0)
        .with_edge("C1", "C4", 1.0)
        .with_edge("C1", "C5", 1.0)
        .with_node_group("A1", "Alpha")
        .with_node_group("A2", "Alpha")
        .with_node_group("A3", "Alpha")
        .with_node_group("B1", "Beta")
        .with_node_group("B2", "Beta")
        .with_node_group("C1", "Gamma")
        .with_node_group("C2", "Gamma")
        .with_node_group("C3", "Gamma")
        .with_node_group("C4", "Gamma")
        .with_node_group("C5", "Gamma")
        .with_labels()
        .with_legend("Component");
    let plots = vec![Plot::Network(net)];
    let layout = Layout::auto_from_plots(&plots).with_title("Disconnected Components");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/network_disconnected.svg", svg).unwrap();
}

#[test]
fn network_pinned_positions() {
    let net = NetworkPlot::new()
        .with_edge("A", "B", 1.0)
        .with_edge("B", "C", 1.0)
        .with_edge("C", "A", 1.0)
        .with_node_position("A", 0.0, 0.0)
        .with_node_position("C", 1.0, 1.0)
        .with_labels();
    let positions = net.compute_positions();
    // A and C should remain at their pinned positions.
    assert!(
        (positions[0].0 - 0.0).abs() < 1e-6,
        "pinned node A x should be 0.0"
    );
    assert!(
        (positions[0].1 - 0.0).abs() < 1e-6,
        "pinned node A y should be 0.0"
    );
    assert!(
        (positions[2].0 - 1.0).abs() < 1e-6,
        "pinned node C x should be 1.0"
    );
    assert!(
        (positions[2].1 - 1.0).abs() < 1e-6,
        "pinned node C y should be 1.0"
    );
    let plots = vec![Plot::Network(net)];
    let layout = Layout::auto_from_plots(&plots).with_title("Pinned Positions");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/network_pinned.svg", svg).unwrap();
}

#[test]
fn network_explicit_node_colors() {
    let net = NetworkPlot::new()
        .with_edge("A", "B", 1.0)
        .with_edge("B", "C", 1.0)
        .with_node_color("A", "#e41a1c")
        .with_node_color("B", "#377eb8")
        .with_node_color("C", "#4daf4a")
        .with_node_group("A", "Group1")
        .with_node_group("B", "Group1")
        .with_node_group("C", "Group2")
        .with_labels()
        .with_legend("Groups");
    let plots = vec![Plot::Network(net)];
    let layout = Layout::auto_from_plots(&plots).with_title("Explicit Colors Override Group");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    // Verify the explicit colors appear in the SVG, not palette defaults.
    assert!(svg.contains("#e41a1c"), "node A should use explicit red");
    assert!(svg.contains("#377eb8"), "node B should use explicit blue");
    assert!(svg.contains("#4daf4a"), "node C should use explicit green");
    common::write_test_output("test_outputs/network_explicit_colors.svg", svg).unwrap();
}

#[test]
fn network_single_node_self_loop() {
    let net = NetworkPlot::new()
        .with_edge("X", "X", 1.0)
        .with_directed()
        .with_labels();
    let plots = vec![Plot::Network(net)];
    let layout = Layout::auto_from_plots(&plots).with_title("Single Node Self-Loop");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    // Should not panic and should contain a bezier path for the loop.
    assert!(
        svg.contains("<path"),
        "single-node self-loop should produce a path"
    );
    common::write_test_output("test_outputs/network_single_self_loop.svg", svg).unwrap();
}

#[test]
fn network_matrix_directed_order_independent() {
    // with_directed() called AFTER with_matrix() should still produce
    // directed edges (both triangles of the matrix).
    let matrix = vec![
        vec![0.0, 1.0, 0.0],
        vec![0.0, 0.0, 1.0],
        vec![1.0, 0.0, 0.0],
    ];
    let net = NetworkPlot::new()
        .with_matrix(matrix, ["A", "B", "C"])
        .with_directed();
    let plots = vec![Plot::Network(net)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    // Directed graph from this matrix has 3 edges: A→B, B→C, C→A.
    // Each directed edge emits a triangle arrowhead path.
    let arrow_count = svg.matches("<path").count();
    assert!(
        arrow_count >= 3,
        "directed matrix should produce at least 3 arrowhead paths, got {arrow_count}"
    );
    common::write_test_output("test_outputs/network_matrix_directed.svg", svg).unwrap();
}

#[test]
fn network_kamada_kawai() {
    let net = NetworkPlot::new()
        .with_edge("A", "B", 1.0)
        .with_edge("B", "C", 1.0)
        .with_edge("C", "D", 1.0)
        .with_edge("D", "E", 1.0)
        .with_edge("E", "A", 1.0)
        .with_edge("A", "C", 0.5)
        .with_layout(NetworkLayout::KamadaKawai)
        .with_labels();
    let plots = vec![Plot::Network(net)];
    let layout = Layout::auto_from_plots(&plots).with_title("Kamada-Kawai Layout");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert!(svg.contains("<circle"), "KK layout should produce nodes");
    common::write_test_output("test_outputs/network_kamada_kawai.svg", svg).unwrap();
}

#[test]
fn network_edge_labels() {
    let net = NetworkPlot::new()
        .with_edge_label("A", "B", 0.95, "0.95")
        .with_edge_label("B", "C", 0.72, "0.72")
        .with_edge_label("C", "A", 0.45, "0.45")
        .with_labels();
    let plots = vec![Plot::Network(net)];
    let layout = Layout::auto_from_plots(&plots).with_title("Edge Labels");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert!(svg.contains("0.95"), "edge label should appear in SVG");
    assert!(svg.contains("0.72"), "edge label should appear in SVG");
    common::write_test_output("test_outputs/network_edge_labels.svg", svg).unwrap();
}

#[test]
fn network_node_shapes() {
    let net = NetworkPlot::new()
        .with_edge("Circle", "Square", 1.0)
        .with_edge("Square", "Diamond", 1.0)
        .with_edge("Diamond", "Triangle", 1.0)
        .with_edge("Triangle", "Circle", 1.0)
        .with_node_shape("Circle", NodeShape::Circle)
        .with_node_shape("Square", NodeShape::Square)
        .with_node_shape("Diamond", NodeShape::Diamond)
        .with_node_shape("Triangle", NodeShape::Triangle)
        .with_layout(NetworkLayout::Circle)
        .with_labels();
    let plots = vec![Plot::Network(net)];
    let layout = Layout::auto_from_plots(&plots).with_title("Node Shapes");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert!(svg.contains("<circle"), "should have circle nodes");
    assert!(svg.contains("<rect"), "should have square nodes");
    // Diamond and triangle are rendered as <path>
    common::write_test_output("test_outputs/network_node_shapes.svg", svg).unwrap();
}

#[test]
fn network_antiparallel_curved() {
    let net = NetworkPlot::new()
        .with_edge_label("A", "B", 2.0, "strong")
        .with_edge_label("B", "A", 1.0, "weak")
        .with_edge("B", "C", 1.5)
        .with_directed()
        .with_labels();
    let plots = vec![Plot::Network(net)];
    let layout = Layout::auto_from_plots(&plots).with_title("Antiparallel Curved Edges");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert!(
        svg.contains(" Q "),
        "antiparallel edges should use quadratic bezier curves"
    );
    assert!(svg.contains("strong"), "A→B edge label should appear");
    assert!(svg.contains("weak"), "B→A edge label should appear");
    common::write_test_output("test_outputs/network_antiparallel.svg", svg).unwrap();
}

#[test]
fn network_repel_labels() {
    // Many nodes close together to trigger label overlap
    let net = NetworkPlot::new()
        .with_edge("Alpha", "Beta", 1.0)
        .with_edge("Beta", "Gamma", 1.0)
        .with_edge("Gamma", "Delta", 1.0)
        .with_edge("Delta", "Epsilon", 1.0)
        .with_edge("Epsilon", "Alpha", 1.0)
        .with_labels()
        .with_repel_labels();
    let plots = vec![Plot::Network(net)];
    let layout = Layout::auto_from_plots(&plots).with_title("Label Repulsion");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert!(svg.contains("Alpha"), "labels should still be present");
    common::write_test_output("test_outputs/network_repel_labels.svg", svg).unwrap();
}

#[test]
fn network_dense_clusters() {
    // Three densely connected clusters with sparse bridges between them.
    // FR should place each cluster tightly and separate them spatially.
    let mut net = NetworkPlot::new();

    // Cluster A: 8 nodes, heavily interconnected
    let a: Vec<&str> = vec!["A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8"];
    for i in 0..a.len() {
        for j in (i + 1)..a.len() {
            net = net.with_edge(a[i], a[j], 1.0);
        }
    }
    // Cluster B: 6 nodes, heavily interconnected
    let b: Vec<&str> = vec!["B1", "B2", "B3", "B4", "B5", "B6"];
    for i in 0..b.len() {
        for j in (i + 1)..b.len() {
            net = net.with_edge(b[i], b[j], 1.0);
        }
    }
    // Cluster C: 5 nodes, heavily interconnected
    let c: Vec<&str> = vec!["C1", "C2", "C3", "C4", "C5"];
    for i in 0..c.len() {
        for j in (i + 1)..c.len() {
            net = net.with_edge(c[i], c[j], 1.0);
        }
    }
    // Sparse bridges between clusters
    net = net
        .with_edge("A1", "B1", 0.3)
        .with_edge("B3", "C1", 0.3)
        .with_edge("A5", "C3", 0.2);

    // Assign groups
    for &label in &a {
        net = net.with_node_group(label, "Cluster A");
    }
    for &label in &b {
        net = net.with_node_group(label, "Cluster B");
    }
    for &label in &c {
        net = net.with_node_group(label, "Cluster C");
    }

    net = net.with_labels().with_legend("Cluster");
    let plots = vec![Plot::Network(net)];
    let layout = Layout::auto_from_plots(&plots).with_title("Dense Clusters with Bridges");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));

    // All three cluster groups should be present
    assert!(svg.contains("A1"), "cluster A nodes present");
    assert!(svg.contains("B1"), "cluster B nodes present");
    assert!(svg.contains("C1"), "cluster C nodes present");
    // Should have many edges (28+15+10+3 = 56 edges → 56 groups with opacity)
    let group_count = svg.matches("opacity=").count();
    assert!(
        group_count >= 50,
        "dense graph should have many edge groups, got {group_count}"
    );
    common::write_test_output("test_outputs/network_dense_clusters.svg", svg).unwrap();
}

#[test]
fn network_matrix_self_loop_directed() {
    // Diagonal entries produce self-loops when directed=true.
    // A=0 has a self-loop (diagonal 2.0); B=1 does not (diagonal 0.0).
    let matrix = vec![vec![2.0, 1.0], vec![1.0, 0.0]];
    let mut net = NetworkPlot::new()
        .with_matrix(matrix, ["A", "B"])
        .with_directed();
    net.resolve_matrix();
    let self_loops: Vec<_> = net.edges.iter().filter(|e| e.source == e.target).collect();
    assert_eq!(
        self_loops.len(),
        1,
        "directed matrix with one nonzero diagonal entry should produce exactly one self-loop"
    );
    assert_eq!(
        self_loops[0].source, 0,
        "self-loop should be on node A (index 0)"
    );
    assert!(
        (self_loops[0].weight - 2.0).abs() < 1e-9,
        "self-loop weight should equal diagonal value"
    );
}

#[test]
fn network_matrix_self_loop_undirected() {
    // Diagonal entries are intentionally ignored for undirected graphs
    // (symmetric self-loops have no physical meaning).
    let matrix = vec![
        vec![5.0, 1.0, 1.0],
        vec![1.0, 3.0, 1.0],
        vec![1.0, 1.0, 7.0],
    ];
    let mut net = NetworkPlot::new().with_matrix(matrix, ["A", "B", "C"]);
    net.resolve_matrix();
    let self_loops: Vec<_> = net.edges.iter().filter(|e| e.source == e.target).collect();
    assert_eq!(
        self_loops.len(),
        0,
        "undirected matrix should produce no self-loops from diagonal"
    );
    // Should still have 3 off-diagonal edges: A-B, A-C, B-C
    assert_eq!(
        net.edges.len(),
        3,
        "undirected 3-node fully-connected matrix should have 3 edges"
    );
}
