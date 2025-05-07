use visus::plot::Histogram;
use visus::backend::svg::SvgBackend;
use visus::render::{render_histogram, Layout};

#[test]
fn test_bar_svg_output_builder() {
    let hist = Histogram::new()
        .with_data(vec![1.1, 2.3, 2.7, 3.2, 3.8, 3.9, 4.0])
        .with_bins(20)
        .with_color("navy");

    let layout = Layout::auto_from_data(&hist.data, 0.0..5.0)
        .with_title("Histogram")
        .with_x_label("Value")
        .with_y_label("Frequency")
        .with_ticks(10);

    let scene = render_histogram(&hist, &layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/hist_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}



