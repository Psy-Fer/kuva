use std::vec;

use visus::plot::SeriesPlot;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

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
    std::fs::write("test_outputs/series_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}