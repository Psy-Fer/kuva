

use visus::plot::{ScatterPlot, LinePlot, types::Point};
use visus::backend::svg::SvgBackend;
use visus::render::{render_multiple, Layout, Plot};

#[test]
fn test_line_svg_output_builder() {
    let sine = LinePlot::new()
    .with_data((0..100)
    .map(|x| Point{x: x as f64 / 10.0, y: (x as f64 / 10.0).sin()})
    .collect())
    .with_color("blue");

    let markers = ScatterPlot::new()
        .with_data(vec![Point {x: 0.0, y: 0.0},
                        Point {x: 1.57, y: 1.0},
                        Point {x: 3.14, y: 0.0},
        ])
        .with_color("red");

    let scatter: ScatterPlot = ScatterPlot::new()
        .with_data(vec![Point {x: 0.8, y: -0.5},
                        Point {x: 2.0, y: 1.2},
                        Point {x: 4.0, y: 0.4},
        ])
        .with_color("purple")
        .with_size(6.0);

    let layout = Layout::new((0.0, 10.0), (-1.5, 1.5))
        .with_title("Sine Wave with Markers")
        .with_x_label("Rads")
        .with_y_label("Amp")
        .with_ticks(6);

    let scene = render_multiple(vec![
        Plot::Line(sine),
        Plot::Scatter(markers),
        Plot::Scatter(scatter),
    ], layout)
    .with_background(Some("white"));

    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/multi_plot.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}
