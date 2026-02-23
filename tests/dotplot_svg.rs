use visus::plot::{DotPlot, ColorMap};
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

#[test]
fn test_dot_basic() {
    // 3 genes x 3 cell types, default Viridis colormap
    let data = vec![
        ("CellTypeA", "GeneX", 80.0_f64, 2.5_f64),
        ("CellTypeA", "GeneY", 60.0, 1.8),
        ("CellTypeA", "GeneZ", 40.0, 1.2),
        ("CellTypeB", "GeneX", 50.0, 3.1),
        ("CellTypeB", "GeneY", 90.0, 2.9),
        ("CellTypeB", "GeneZ", 70.0, 2.0),
        ("CellTypeC", "GeneX", 30.0, 0.9),
        ("CellTypeC", "GeneY", 45.0, 1.4),
        ("CellTypeC", "GeneZ", 55.0, 1.7),
    ];

    let dot = DotPlot::new().with_data(data);
    let plots = vec![Plot::DotPlot(dot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Dot Plot Basic")
        .with_x_label("Cell Type")
        .with_y_label("Gene");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/dot_basic.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
}

#[test]
fn test_dot_matrix() {
    // Dense matrix input
    let x_cats = vec!["TypeA", "TypeB", "TypeC"];
    let y_cats = vec!["Gene1", "Gene2", "Gene3"];
    let sizes = vec![
        vec![80.0_f64, 50.0, 30.0],
        vec![60.0, 90.0, 45.0],
        vec![40.0, 70.0, 55.0],
    ];
    let colors = vec![
        vec![2.5_f64, 3.1, 0.9],
        vec![1.8, 2.9, 1.4],
        vec![1.2, 2.0, 1.7],
    ];

    let dot = DotPlot::new().with_matrix(x_cats, y_cats, sizes, colors);
    let plots = vec![Plot::DotPlot(dot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Dot Plot Matrix Input");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/dot_matrix.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // 9 circles for a 3x3 matrix
    assert_eq!(svg.matches("<circle").count(), 9);
}

#[test]
fn test_dot_custom_colormap() {
    let data = vec![
        ("A", "G1", 70.0_f64, 3.0_f64),
        ("A", "G2", 50.0, 1.5),
        ("B", "G1", 40.0, 2.0),
        ("B", "G2", 80.0, 4.0),
    ];

    let dot = DotPlot::new()
        .with_data(data)
        .with_color_map(ColorMap::Inferno);

    let plots = vec![Plot::DotPlot(dot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Dot Plot Inferno");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/dot_custom_colormap.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
}

#[test]
fn test_dot_size_range() {
    // Explicit size range clamp
    let data = vec![
        ("CT1", "G1", 20.0_f64, 1.0_f64),
        ("CT1", "G2", 50.0, 2.0),
        ("CT2", "G1", 80.0, 3.0),
        ("CT2", "G2", 110.0, 4.0),  // will be clamped to 100
    ];

    let dot = DotPlot::new()
        .with_data(data)
        .with_size_range(0.0, 100.0);

    let plots = vec![Plot::DotPlot(dot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Dot Plot Size Range");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/dot_size_range.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
}

#[test]
fn test_dot_color_range() {
    // Explicit color range clamp
    let data = vec![
        ("CT1", "G1", 50.0_f64, 0.5_f64),
        ("CT1", "G2", 70.0, 3.0),
        ("CT2", "G1", 40.0, 5.5),  // will be clamped to 5
        ("CT2", "G2", 90.0, 2.0),
    ];

    let dot = DotPlot::new()
        .with_data(data)
        .with_color_range(0.0, 5.0);

    let plots = vec![Plot::DotPlot(dot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Dot Plot Color Range");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/dot_color_range.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
}

#[test]
fn test_dot_size_legend() {
    let data = vec![
        ("TypeA", "GeneX", 25.0_f64, 1.0_f64),
        ("TypeA", "GeneY", 75.0, 2.0),
        ("TypeB", "GeneX", 50.0, 3.0),
        ("TypeB", "GeneY", 100.0, 4.0),
    ];

    let dot = DotPlot::new()
        .with_data(data)
        .with_size_legend("% Expressing");

    let plots = vec![Plot::DotPlot(dot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Dot Plot Size Legend");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/dot_size_legend.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // Should have legend entries
    assert!(svg.contains("25.0") || svg.contains("100.0"));
}

#[test]
fn test_dot_colorbar() {
    let data = vec![
        ("TypeA", "GeneX", 80.0_f64, 2.5_f64),
        ("TypeA", "GeneY", 60.0, 1.8),
        ("TypeB", "GeneX", 50.0, 3.1),
        ("TypeB", "GeneY", 90.0, 2.9),
    ];

    let dot = DotPlot::new()
        .with_data(data)
        .with_colorbar("Mean expression");

    let plots = vec![Plot::DotPlot(dot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Dot Plot Colorbar");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/dot_colorbar.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // Colorbar draws many rects
    assert!(svg.contains("<rect"));
}

#[test]
fn test_dot_both_legends() {
    let data = vec![
        ("TypeA", "GeneX", 80.0_f64, 2.5_f64),
        ("TypeA", "GeneY", 60.0, 1.8),
        ("TypeA", "GeneZ", 40.0, 1.2),
        ("TypeB", "GeneX", 50.0, 3.1),
        ("TypeB", "GeneY", 90.0, 2.9),
        ("TypeB", "GeneZ", 70.0, 2.0),
    ];

    let dot = DotPlot::new()
        .with_data(data)
        .with_size_legend("% Expressing")
        .with_colorbar("Mean expression");

    let plots = vec![Plot::DotPlot(dot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Dot Plot: Size Legend + Colorbar");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/dot_both_legends.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
    assert!(svg.contains("<rect"));
}

#[test]
fn test_dot_sparse() {
    // Some (x, y) pairs are absent — fewer circles than grid cells
    let data = vec![
        ("TypeA", "GeneX", 80.0_f64, 2.5_f64),
        // ("TypeA", "GeneY", ...) missing
        ("TypeA", "GeneZ", 40.0, 1.2),
        // ("TypeB", "GeneX", ...) missing
        ("TypeB", "GeneY", 90.0, 2.9),
        ("TypeB", "GeneZ", 70.0, 2.0),
        ("TypeC", "GeneX", 30.0, 0.9),
        // ("TypeC", "GeneY", ...) missing
        // ("TypeC", "GeneZ", ...) missing
    ];

    let dot = DotPlot::new().with_data(data);
    let plots = vec![Plot::DotPlot(dot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Dot Plot Sparse");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/dot_sparse.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // Only 5 circles (not 9 for a 3x3 grid)
    assert_eq!(svg.matches("<circle").count(), 5);
}

#[test]
fn test_dot_large() {
    // 50 genes x 10 cell types
    let cell_types: Vec<String> = (0..10).map(|i| format!("CellType{}", i)).collect();
    let genes: Vec<String> = (0..50).map(|i| format!("Gene{:02}", i)).collect();

    let mut data = Vec::new();
    for (gi, gene) in genes.iter().enumerate() {
        for (ci, cell) in cell_types.iter().enumerate() {
            let size = ((gi * 2 + ci * 3) % 100) as f64;
            let color = ((gi + ci * 5) % 50) as f64 / 10.0;
            data.push((cell.as_str(), gene.as_str(), size, color));
        }
    }

    let dot = DotPlot::new()
        .with_data(data)
        .with_size_legend("% Expressing")
        .with_colorbar("Mean expression");

    let plots = vec![Plot::DotPlot(dot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Dot Plot Large (50 genes × 10 cell types)")
        .with_x_label("Cell Type")
        .with_y_label("Gene");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/dot_large.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // 500 circles for 50x10
    assert_eq!(svg.matches("<circle").count(), 504); // 500 data + 4 legend
}
