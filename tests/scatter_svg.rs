use rand::Rng;
use visus::plot::scatter::{ScatterPlot, TrendLine};
use visus::backend::svg::SvgBackend;
use visus::render::render::{render_scatter, render_multiple};
use visus::render::layout::Layout;
use visus::render::plots::Plot;

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
        y_categories: None,
        show_legend: false,
    };

    let scene = render_scatter(&plot, layout).with_background(Some("white"));
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter_layout.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
}

#[test]
fn test_scatter_trend_svg() {

    // Generate some noisy linear data: y = 2x + 1 + noise
    let mut rng = rand::rng();
    let data: Vec<(f64, f64)> = (1..49)
        .map(|i| {
            let x = i as f64 * 0.2;
            let noise: f64 = rng.random_range(-1.0..1.0);
            let y = 0.5 * x + 1.0 + noise;
            (x, y)
        })
        .collect();

    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("blue")
        .with_trend(TrendLine::Linear)
        .show_equation()
        .show_correlation();
        

    let plot = vec![Plot::Scatter(scatter)];


    let layout = Layout::auto_from_plots(&plot)
                        .with_title("Scatter with trend")
                        .with_x_label("The X axis")
                        .with_y_label("The Y axis");

    let scene = render_multiple(plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter_trend_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));

}


#[test]
fn test_scatter_trend_error_svg() {

    let data = vec![(1.5, 2), (2.0, 3), (3.0, 5), (4.0, 6)];
    let x_err = vec![0.1, 0.05, 0.2, 0.3];
    let y_err = vec![(1, 1), (1, 1), (1, 1), (1, 1)];

    let scatter = ScatterPlot::new()
        .with_data(data) // i32 -> f64 input test 
        .with_x_err(x_err)
        .with_y_err_asymmetric(y_err)
        .with_color("red")
        .with_trend(TrendLine::Linear)
        .show_equation()
        .show_correlation();

    let plot = vec![Plot::Scatter(scatter)];


    let layout = Layout::auto_from_plots(&plot)
                        .with_title("Scatter with trend + error")
                        .with_x_label("The X axis")
                        .with_y_label("The Y axis");

    let scene = render_multiple(plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter_trend_error_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));

}