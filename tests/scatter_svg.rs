use rand::Rng;
use visus::plot::scatter::{ScatterPlot, TrendLine, MarkerShape};
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

    let layout = Layout::new((0.0, 11.0), (0.0, 10.0))
        .with_title("Scatter Layout Plot")
        .with_x_label("The X axis")
        .with_y_label("The Y axis");

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
        .with_equation()
        .with_correlation();


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
        .with_equation()
        .with_correlation();

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


#[test]
fn test_scatter_log_scale() {
    // Data spanning several orders of magnitude
    let data: Vec<(f64, f64)> = vec![
        (1.0, 0.001),
        (5.0, 0.01),
        (10.0, 0.1),
        (50.0, 1.0),
        (100.0, 10.0),
        (500.0, 100.0),
        (1000.0, 1000.0),
        (5000.0, 5000.0),
        (10000.0, 10000.0),
    ];

    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("teal")
        .with_size(5.0);

    let plot = vec![Plot::Scatter(scatter)];

    let layout = Layout::auto_from_plots(&plot)
        .with_log_scale()
        .with_title("Log-Scale Scatter")
        .with_x_label("X (log)")
        .with_y_label("Y (log)");

    let scene = render_multiple(plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter_log.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
}

#[test]
fn test_scatter_markers() {
    let circle = ScatterPlot::new()
        .with_data(vec![(1.0, 1.0), (2.0, 2.0), (3.0, 1.5)])
        .with_color("blue")
        .with_size(5.0)
        .with_marker(MarkerShape::Circle)
        .with_legend("Circle");

    let square = ScatterPlot::new()
        .with_data(vec![(1.0, 3.0), (2.0, 4.0), (3.0, 3.5)])
        .with_color("red")
        .with_size(5.0)
        .with_marker(MarkerShape::Square)
        .with_legend("Square");

    let triangle = ScatterPlot::new()
        .with_data(vec![(1.0, 5.0), (2.0, 6.0), (3.0, 5.5)])
        .with_color("green")
        .with_size(5.0)
        .with_marker(MarkerShape::Triangle)
        .with_legend("Triangle");

    let diamond = ScatterPlot::new()
        .with_data(vec![(1.0, 7.0), (2.0, 8.0), (3.0, 7.5)])
        .with_color("purple")
        .with_size(5.0)
        .with_marker(MarkerShape::Diamond)
        .with_legend("Diamond");

    let plots = vec![
        Plot::Scatter(circle),
        Plot::Scatter(square),
        Plot::Scatter(triangle),
        Plot::Scatter(diamond),
    ];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Marker Shapes")
        .with_x_label("X")
        .with_y_label("Y");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter_markers.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));  // Circle markers
    assert!(svg.contains("<rect"));    // Square markers
}

#[test]
fn test_bubble_plot() {
    let data: Vec<(f64, f64)> = vec![
        (1.0, 2.0), (2.0, 3.0), (3.0, 5.0),
        (4.0, 4.0), (5.0, 6.0), (6.0, 3.0),
    ];
    let sizes = vec![3.0, 6.0, 10.0, 4.0, 8.0, 12.0];

    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_sizes(sizes);

    let plot = vec![Plot::Scatter(scatter)];

    let layout = Layout::auto_from_plots(&plot)
        .with_title("Bubble Plot")
        .with_x_label("X")
        .with_y_label("Y");

    let scene = render_multiple(plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/bubble_plot.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
    // Verify varying radii: the smallest (r=3) and largest (r=12) should both appear
    assert!(svg.contains(r#"r="3""#));
    assert!(svg.contains(r#"r="12""#));
}

#[test]
fn test_scatter_log_x_only() {
    // Log X with linear Y — e.g. dose-response
    let data: Vec<(f64, f64)> = vec![
        (0.01, 5.0), (0.1, 12.0), (1.0, 45.0),
        (10.0, 78.0), (100.0, 95.0), (1000.0, 99.0),
    ];

    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("darkred")
        .with_size(5.0);

    let plot = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plot)
        .with_log_x()
        .with_title("Log X / Linear Y")
        .with_x_label("Concentration")
        .with_y_label("Response (%)");

    let scene = render_multiple(plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter_log_x.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
    // X ticks should be log formatted (powers of 10)
    assert!(svg.contains(">1<") || svg.contains(">1</text>"));
    assert!(svg.contains(">100<") || svg.contains(">100</text>"));
    // Y ticks should be linear (decimal)
    assert!(svg.contains(".0</text>"));
}

#[test]
fn test_scatter_log_y_only() {
    // Linear X with log Y — e.g. exponential growth
    let data: Vec<(f64, f64)> = vec![
        (1.0, 2.0), (2.0, 8.0), (3.0, 30.0),
        (4.0, 120.0), (5.0, 500.0), (6.0, 2000.0),
        (7.0, 8000.0),
    ];

    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("navy")
        .with_size(5.0);

    let plot = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plot)
        .with_log_y()
        .with_title("Linear X / Log Y")
        .with_x_label("Time")
        .with_y_label("Population");

    let scene = render_multiple(plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter_log_y.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
    // Y ticks should be log formatted
    assert!(svg.contains(">1</text>"));
    assert!(svg.contains(">1000</text>"));
}

#[test]
fn test_scatter_log_small_values() {
    // Very small values (sub-unity range)
    let data: Vec<(f64, f64)> = vec![
        (0.001, 0.0001), (0.005, 0.0008),
        (0.02, 0.003), (0.1, 0.015),
        (0.5, 0.08), (1.0, 0.5),
    ];

    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("purple")
        .with_size(5.0);

    let plot = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plot)
        .with_log_scale()
        .with_title("Log Scale — Small Values")
        .with_x_label("X")
        .with_y_label("Y");

    let scene = render_multiple(plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter_log_small.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
    // Should have sub-unity tick labels
    assert!(svg.contains("1e-"));
}

#[test]
fn test_scatter_log_narrow_range() {
    // Narrow range (< 3 decades) should show 2x and 5x sub-ticks
    let data: Vec<(f64, f64)> = vec![
        (5.0, 10.0), (10.0, 25.0), (20.0, 50.0),
        (50.0, 80.0), (100.0, 200.0),
    ];

    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("teal")
        .with_size(6.0);

    let plot = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plot)
        .with_log_scale()
        .with_title("Log Scale — Narrow Range (sub-ticks)")
        .with_x_label("X")
        .with_y_label("Y");

    let scene = render_multiple(plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter_log_narrow.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // Narrow range should have 2x/5x sub-ticks (e.g. "20", "50")
    assert!(svg.contains(">20</text>") || svg.contains(">50</text>"));
}