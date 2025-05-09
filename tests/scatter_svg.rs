use visus::plot::ScatterPlot;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_scatter;
use visus::render::layout::Layout;

#[test]
fn test_scatter_svg_output_builder() {
    let data = vec![
        (1.0, 5.0),
        (4.5, 3.5),
        (5.0, 8.7),
    ];

    let plot = ScatterPlot::new()
        .with_data(data)
        .with_color("blue")
        .with_size(5.0);
        
    let layout = Layout::new((0.0, 10.0), (0.0, 40.0))
    .with_title("Scatter Builder Plot")
    .with_x_label("The X axis")
    .with_y_label("The Y axis");

    let scene = render_scatter(&plot, layout).with_background(Some("white"));
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
}

#[test]
fn test_scatter_svg_output_layout() {
    let data = vec![
        (1.0, 5.0),
        (4.5, 3.5),
        (5.0, 8.7),
    ];

    let plot = ScatterPlot::new()
        .with_data(data)
        .with_color("purple")
        .with_size(3.0);

    let layout = Layout {
        width: None,
        height: None,
        x_range: (0.0, 11.0),
        y_range: (0.0, 10.0),
        ticks: 5,
        show_grid: true,
        title: Some("Scatter Layout Plot".into()),
        x_label: Some("The X axis".into()),
        y_label: Some("The Y axis".into()),
        x_categories: None,
        show_legend: false,
    };

    let scene = render_scatter(&plot, layout).with_background(Some("white"));
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter_layout.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
}