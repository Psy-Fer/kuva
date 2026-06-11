mod common;
use std::vec;

use kuva::backend::svg::SvgBackend;
use kuva::plot::SeriesPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

#[test]
fn test_line_svg_output_builder() {
    let data = (0..100)
        .map(|x| (x as f64 / 10.0).sin())
        .collect::<Vec<_>>();
    let series = SeriesPlot::new()
        .with_data(data)
        .with_color("green")
        .with_line_point_style()
        .with_legend("sine");

    let plots = vec![Plot::Series(series)];

    let layout = Layout::auto_from_plots(&plots)
        .with_x_label("Time (s)")
        .with_y_label("Amplitude")
        .with_title("Sine Wave");
    // .with_ticks(6);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/series_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}
