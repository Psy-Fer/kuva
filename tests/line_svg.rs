use visus::plot::LinePlot;
use visus::backend::svg::SvgBackend;
use visus::render::{render_line, Layout};

#[test]
fn test_line_svg_output_builder() {
    let plot = LinePlot::new()
                        .with_data((0..100)
                        .map(|x| (x as f64 / 10.0, (x as f64 / 10.0).sin()))
                        .collect())
                        .with_color("green");

    let layout = Layout::new((0.0, 10.0), (-1.5, 1.5))
        .with_x_label("Time (s)")
        .with_y_label("Amplitude")
        .with_title("Sine Wave")
        .with_ticks(6);

    let scene = render_line(&plot, layout).with_background(Some("white"));
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/line_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}