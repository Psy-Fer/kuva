use kuva::plot::surface3d::Surface3DPlot;
use kuva::plot::heatmap::ColorMap;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

fn paraboloid_grid(n: usize) -> Vec<Vec<f64>> {
    (0..n).map(|i| {
        (0..n).map(|j| {
            let x = (i as f64 - n as f64 / 2.0) / (n as f64 / 4.0);
            let y = (j as f64 - n as f64 / 2.0) / (n as f64 / 4.0);
            x * x + y * y
        }).collect()
    }).collect()
}

#[test]
fn test_surface3d_basic() {
    let surface = Surface3DPlot::new(paraboloid_grid(10))
        .with_color("steelblue");

    let plots = vec![Plot::Surface3D(surface)];
    let layout = Layout::auto_from_plots(&plots).with_title("Surface3D Basic");
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/surface3d_basic.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"), "should contain path elements for surface faces");
}

#[test]
fn test_surface3d_colormap() {
    let surface = Surface3DPlot::new(paraboloid_grid(15))
        .with_z_colormap(ColorMap::Viridis);

    let plots = vec![Plot::Surface3D(surface)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/surface3d_colormap.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    let path_count = svg.matches("<path").count();
    // Should have many paths: back panes + grid + surface faces
    assert!(path_count > 100, "colormap surface should have many paths, got {path_count}");
}

#[test]
fn test_surface3d_no_wireframe() {
    let surface = Surface3DPlot::new(paraboloid_grid(10))
        .with_show_wireframe(false)
        .with_z_colormap(ColorMap::Inferno);

    let plots = vec![Plot::Surface3D(surface)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/surface3d_no_wireframe.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // Wireframe off: stroke should be "none" on surface faces
    assert!(svg.contains("stroke=\"none\""));
}

#[test]
fn test_surface3d_alpha() {
    let surface = Surface3DPlot::new(paraboloid_grid(8))
        .with_alpha(0.7)
        .with_color("coral");

    let plots = vec![Plot::Surface3D(surface)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/surface3d_alpha.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("opacity"), "should contain opacity attribute for alpha");
}

#[test]
fn test_surface3d_empty() {
    let surface = Surface3DPlot::new(vec![]);
    let plots = vec![Plot::Surface3D(surface)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/surface3d_empty.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_surface3d_axis_labels() {
    let surface = Surface3DPlot::new(paraboloid_grid(8))
        .with_x_label("X Axis")
        .with_y_label("Y Axis")
        .with_z_label("Z Axis");

    let plots = vec![Plot::Surface3D(surface)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/surface3d_labels.svg", svg.clone()).unwrap();
    assert!(svg.contains("X Axis"));
    assert!(svg.contains("Y Axis"));
    assert!(svg.contains("Z Axis"));
}

#[test]
fn test_surface3d_explicit_coords() {
    let xs: Vec<f64> = (-5..=5).map(|i| i as f64).collect();
    let ys: Vec<f64> = (-5..=5).map(|i| i as f64).collect();
    let z_data: Vec<Vec<f64>> = ys.iter().map(|&y| {
        xs.iter().map(|&x| (x * x + y * y).sqrt().sin()).collect()
    }).collect();

    let surface = Surface3DPlot::new(z_data)
        .with_x_coords(xs)
        .with_y_coords(ys)
        .with_z_colormap(ColorMap::Viridis);

    let plots = vec![Plot::Surface3D(surface)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/surface3d_coords.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_surface3d_custom_view() {
    let surface = Surface3DPlot::new(paraboloid_grid(10))
        .with_azimuth(-120.0)
        .with_elevation(20.0)
        .with_z_colormap(ColorMap::Viridis);

    let plots = vec![Plot::Surface3D(surface)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/surface3d_custom_view.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_surface3d_with_data_fn() {
    let surface = Surface3DPlot::new(vec![])
        .with_data_fn(|x, y| (x * x + y * y).sqrt().sin(), -3.0..=3.0, -3.0..=3.0, 30, 30)
        .with_z_colormap(ColorMap::Viridis);

    assert_eq!(surface.nrows(), 30);
    assert_eq!(surface.ncols(), 30);

    let plots = vec![Plot::Surface3D(surface)];
    let layout = Layout::auto_from_plots(&plots).with_title("with_data_fn");
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/surface3d_data_fn.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // 30x30 grid = 29x29 = 841 faces + back panes → lots of paths
    let path_count = svg.matches("<path").count();
    assert!(path_count > 800, "high-res surface should have many paths, got {path_count}");
}

#[test]
fn test_surface3d_with_data_fn_low_vs_high_res() {
    // Low res
    let low = Surface3DPlot::new(vec![])
        .with_data_fn(|x, y| x * x + y * y, -2.0..=2.0, -2.0..=2.0, 5, 5);
    assert_eq!(low.nrows(), 5);
    assert_eq!(low.ncols(), 5);

    let plots = vec![Plot::Surface3D(low)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg_low = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/surface3d_low_res.svg", svg_low.clone()).unwrap();

    // High res
    let high = Surface3DPlot::new(vec![])
        .with_data_fn(|x, y| x * x + y * y, -2.0..=2.0, -2.0..=2.0, 40, 40);
    assert_eq!(high.nrows(), 40);
    assert_eq!(high.ncols(), 40);

    let plots = vec![Plot::Surface3D(high)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg_high = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/surface3d_high_res.svg", svg_high.clone()).unwrap();

    // High res should produce significantly more SVG content
    assert!(svg_high.len() > svg_low.len() * 5,
        "high-res ({} bytes) should be much larger than low-res ({} bytes)",
        svg_high.len(), svg_low.len());
}
