

use kuva::plot::{ScatterPlot, LinePlot};
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

#[test]
fn test_line_svg_output_builder() {
    let sine = LinePlot::new()
    .with_data((0..100)
    .map(|x| (x as f64 / 10.0, (x as f64 / 10.0).sin()))
    .collect::<Vec<_>>())
    .with_color("blue")
    .with_legend("sine");


    let markers = ScatterPlot::new()
        .with_data(vec![(0.0, 0.0),
                        (1.57, 1.0),
                        (3.14, 0.0),
        ])
        .with_color("red")
        .with_legend("Markers");

    let scatter: ScatterPlot = ScatterPlot::new()
        .with_data(vec![(0.8, -0.5),
                        (2.0, 1.2),
                        (4.0, 0.4),
        ])
        .with_color("purple")
        .with_size(6.0)
        .with_legend("Scatter");

    let plots = vec![
        Plot::Line(sine),
        Plot::Scatter(markers),
        Plot::Scatter(scatter),
    ];

    let layout = Layout::auto_from_plots(&plots)
                .with_title("Sine Wave with Markers")
                .with_x_label("Rads")
                .with_y_label("Amp")
                .with_ticks(10);

    // let layout = Layout::new((0.0, 10.0), (-1.5, 1.5))
    //     .with_title("Sine Wave with Markers")
    //     .with_x_label("Rads")
    //     .with_y_label("Amp")
    //     .with_ticks(6);

    let scene = render_multiple(plots, layout)
    .with_background(Some("white"));


    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/multi_plot.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}
