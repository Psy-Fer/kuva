use visus::plot::{ScatterPlot, types::Point};
use visus::backend::svg::SvgBackend;
use visus::render::{render_scatter, Layout};

#[test]
fn test_scatter_svg_output() {
    let data = vec![
        Point { x: 1.0, y: 2.0 },
        Point { x: 2.0, y: 3.5 },
        Point { x: 3.0, y: 1.0 },
    ];

    let plot = ScatterPlot::new(data);
    let layout = Layout {
        width: None,
        height: None,
        x_range: (0.0, 5.0),
        y_range: (0.0, 5.0),
        ticks: 5,
        show_grid: true,
        title: Some("Some silly dots".into()),
        x_label: Some("The X axis".into()),
        y_label: Some("The Y axis".into()),
    };

    // let layout = Layout::new((0.0, 10.0), (0.0, 100.0))
    // .with_title("Auto-sized Plot")
    // .with_x_label("Time (s)")
    // .with_y_label("Amplitude");

    let scene = render_scatter(&plot, layout).with_background(Some("white"));
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter_with_labels.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
}