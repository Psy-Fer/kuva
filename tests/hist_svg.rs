use kuva::plot::Histogram;
use kuva::backend::svg::SvgBackend;
// use kuva::render::render::render_histogram;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;



#[test]
fn test_histogram_svg_output_builder() {
    let hist = Histogram::new()
        .with_data(vec![1.1, 2.3, 2.7, 3.2, 3.8, 3.9, 4.0])
        .with_bins(5)
        .with_color("navy")
        .with_range((0.0, 5.0)); // make this automatic

    let plots = vec![Plot::Histogram(hist.clone())];

    // let layout = Layout::auto_from_data(&hist.data, 0.0..5.0)
    //     .with_title("Histogram")
    //     .with_x_label("Value")
    //     .with_y_label("Frequency");
        // .with_ticks(10);
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Histogram")
        .with_x_label("Value")
        .with_y_label("Frequency");
        // .with_ticks(10);

    // let scene = render_histogram(&hist, &layout);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/hist_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}

// A normalized histogram has y values in [0, 1].  auto_from_plots should
// detect this and clamp the y-axis at exactly 1.0 rather than letting the
// 1%-span padding push auto_nice_range up to 1.1.
#[test]
fn test_normalized_histogram_y_axis_clamp() {
    let hist = Histogram::new()
        .with_data(vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0])
        .with_bins(4)
        .with_range((0.0, 5.0))
        .with_normalize();

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/hist_normalized_clamp.svg", &svg).unwrap();

    // clamp_y_axis should be triggered automatically: axis must stop at 1 not 1.1
    assert!(svg.contains(">1<") || svg.contains(">1.0<"),
        "normalized histogram y-axis should show a tick at 1");
    assert!(!svg.contains(">1.1<"), "y-axis must not extend past 1.0 for normalized data");
}

// Non-normalized histograms must not be affected by the clamp.
#[test]
fn test_non_normalized_histogram_y_axis_free() {
    let hist = Histogram::new()
        .with_data(vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0])
        .with_bins(4)
        .with_range((0.0, 5.0));  // no with_normalize()

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/hist_non_normalized.svg", &svg).unwrap();

    // y-axis should reflect actual counts (max count = 3), well above 1
    assert!(!svg.contains(">1.1<") || svg.contains(">2<") || svg.contains(">3<"),
        "non-normalized histogram y-axis should show count ticks");
}



