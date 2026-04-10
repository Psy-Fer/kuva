use kuva::plot::scatter3d::Scatter3DPlot;
use kuva::plot::heatmap::ColorMap;
use kuva::plot::scatter::MarkerShape;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

#[test]
fn test_scatter3d_basic() {
    let data = vec![
        (1.0, 2.0, 3.0),
        (4.0, 5.0, 6.0),
        (7.0, 8.0, 9.0),
        (2.0, 6.0, 1.0),
        (5.0, 3.0, 7.0),
    ];

    let plot = Scatter3DPlot::new()
        .with_data(data)
        .with_color("steelblue");

    let plots = vec![Plot::Scatter3D(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("3D Scatter Basic");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter3d_basic.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // Should contain circle markers
    assert!(svg.contains("<circle"), "SVG should contain circle markers");
}

#[test]
fn test_scatter3d_wireframe() {
    let data = vec![(0.0, 0.0, 0.0), (1.0, 1.0, 1.0)];
    let plot = Scatter3DPlot::new()
        .with_data(data);

    let plots = vec![Plot::Scatter3D(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter3d_wireframe.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // Wireframe box produces line elements
    assert!(svg.contains("<line"), "SVG should contain line elements for wireframe");
}

#[test]
fn test_scatter3d_custom_view() {
    let data = vec![(1.0, 2.0, 3.0), (4.0, 5.0, 6.0)];

    let plot = Scatter3DPlot::new()
        .with_data(data)
        .with_azimuth(-30.0)
        .with_elevation(45.0);

    let plots = vec![Plot::Scatter3D(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter3d_custom_view.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
}

#[test]
fn test_scatter3d_z_colormap() {
    let data: Vec<(f64, f64, f64)> = (0..20)
        .map(|i| {
            let t = i as f64 / 19.0;
            (t * 10.0, t.sin() * 5.0, t * 8.0)
        })
        .collect();

    let plot = Scatter3DPlot::new()
        .with_data(data)
        .with_z_colormap(ColorMap::Viridis);

    let plots = vec![Plot::Scatter3D(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter3d_colormap.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // Z-colormap produces varied fill colors from viridis (rendered as hex)
    // With 20 points at different z values, there should be multiple distinct fill colors
    let circle_count = svg.matches("<circle").count();
    assert!(circle_count >= 15, "SVG should contain many circle markers, got {circle_count}");
}

#[test]
fn test_scatter3d_depth_shade() {
    let data = vec![
        (0.0, 0.0, 0.0),
        (5.0, 5.0, 5.0),
        (10.0, 10.0, 10.0),
    ];

    let plot = Scatter3DPlot::new()
        .with_data(data)
        .with_depth_shade();

    let plots = vec![Plot::Scatter3D(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter3d_depth_shade.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // Depth shading produces opacity attributes
    assert!(svg.contains("fill-opacity"), "SVG should contain fill-opacity for depth shading");
}

#[test]
fn test_scatter3d_legend() {
    let plot = Scatter3DPlot::new()
        .with_data(vec![(1.0, 2.0, 3.0), (4.0, 5.0, 6.0)])
        .with_legend("Group A");

    let plots = vec![Plot::Scatter3D(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter3d_legend.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Group A"), "SVG should contain legend label");
}

#[test]
fn test_scatter3d_empty() {
    let plot = Scatter3DPlot::new();

    let plots = vec![Plot::Scatter3D(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter3d_empty.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // Should not crash, just produce an empty-ish SVG
}

#[test]
fn test_scatter3d_axis_labels() {
    let data = vec![(1.0, 2.0, 3.0), (4.0, 5.0, 6.0)];
    let plot = Scatter3DPlot::new()
        .with_data(data)
        .with_x_label("X Axis")
        .with_y_label("Y Axis")
        .with_z_label("Z Axis");

    let plots = vec![Plot::Scatter3D(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter3d_labels.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("X Axis"), "SVG should contain X axis label");
    assert!(svg.contains("Y Axis"), "SVG should contain Y axis label");
    assert!(svg.contains("Z Axis"), "SVG should contain Z axis label");
}

#[test]
fn test_scatter3d_marker_shapes() {
    let data = vec![(1.0, 2.0, 3.0), (4.0, 5.0, 6.0)];
    let plot = Scatter3DPlot::new()
        .with_data(data)
        .with_marker(MarkerShape::Square);

    let plots = vec![Plot::Scatter3D(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter3d_squares.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // Square markers produce rect elements
    assert!(svg.contains("<rect"), "SVG should contain rect elements for square markers");
}

#[test]
fn test_scatter3d_no_grid_no_box() {
    let data = vec![(1.0, 2.0, 3.0), (4.0, 5.0, 6.0)];
    let plot = Scatter3DPlot::new()
        .with_data(data)
        .with_no_grid()
        .with_no_box();

    let plots = vec![Plot::Scatter3D(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter3d_no_grid_box.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"), "should still have data points");
}

#[test]
fn test_scatter3d_auto_z_axis() {
    let data = vec![(0.0, 0.0, 0.0), (10.0, 10.0, 10.0)];

    // Default view (azimuth=-60°): auto should place Z on the right.
    // Mirrored view (azimuth=+60°): auto should flip Z to the left.
    // Both should render without panicking and produce valid SVG.
    let default_plot = Scatter3DPlot::new()
        .with_data(data.clone())
        .with_x_label("X").with_y_label("Y").with_z_label("Z");

    let plots = vec![Plot::Scatter3D(default_plot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg_default = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/scatter3d_auto_z_default.svg", svg_default.clone()).unwrap();
    assert!(svg_default.contains("Z"), "Z axis label should be present");

    let mirrored_plot = Scatter3DPlot::new()
        .with_data(data.clone())
        .with_azimuth(60.0)
        .with_x_label("X").with_y_label("Y").with_z_label("Z");

    let plots = vec![Plot::Scatter3D(mirrored_plot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg_mirrored = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/scatter3d_auto_z_mirrored.svg", svg_mirrored.clone()).unwrap();
    assert!(svg_mirrored.contains("Z"), "Z axis label should be present");

    // Explicit override: force left even at default azimuth
    let forced_left = Scatter3DPlot::new()
        .with_data(data)
        .with_z_axis_right(false)
        .with_z_label("Z");

    let plots = vec![Plot::Scatter3D(forced_left)];
    let layout = Layout::auto_from_plots(&plots);
    let svg_left = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/scatter3d_z_axis_left.svg", svg_left.clone()).unwrap();
    assert!(svg_left.contains("Z"));
}
