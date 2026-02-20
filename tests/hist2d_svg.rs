use visus::plot::Histogram2D;
use visus::plot::histogram2d::ColorMap;
use visus::backend::svg::SvgBackend;
// use visus::render::render::render_histogram;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

use rand_distr::{Normal, Distribution};
// use rand::prelude::*;


#[test]
fn test_histogram2d_svg_output_builder() {

    // Generate 1000 random points from a 2D Gaussian
    let normal_x = Normal::new(10.0, 2.0).unwrap();
    let normal_y = Normal::new(12.0, 3.0).unwrap();
    let mut rng = rand::rng();
    let data: Vec<(f64, f64)> = (0..10000)
        .map(|_| (normal_x.sample(&mut rng), normal_y.sample(&mut rng)))
        .collect();

    let hist2d = Histogram2D::new()
        .with_data(data, (0.0, 30.0), (0.0, 30.0), 30, 30)
        .with_color_map(ColorMap::Inferno)
        .with_correlation();

    let plots = vec![Plot::Histogram2d(hist2d.clone())];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Histogram2D");
        // .with_x_label("Value")
        // .with_y_label("Frequency");
        // .with_ticks(10);

    // let scene = render_histogram(&hist, &layout);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/hist2d_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}














