use visus::plot::{LinePlot, ScatterPlot, BandPlot};
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

#[test]
fn test_standalone_band() {
    let x: Vec<f64> = (0..50).map(|i| i as f64 * 0.2).collect();
    let y: Vec<f64> = x.iter().map(|&v| v.sin()).collect();
    let y_lower: Vec<f64> = y.iter().map(|&v| v - 0.3).collect();
    let y_upper: Vec<f64> = y.iter().map(|&v| v + 0.3).collect();

    let band = BandPlot::new(x.clone(), y_lower, y_upper)
        .with_color("steelblue")
        .with_opacity(0.25);

    let line = LinePlot::new()
        .with_data(x.iter().zip(y.iter()).map(|(&a, &b)| (a, b)).collect::<Vec<_>>())
        .with_color("steelblue");

    let plots = vec![
        Plot::Band(band),
        Plot::Line(line),
    ];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Standalone Band")
        .with_x_label("x")
        .with_y_label("y");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/band_standalone.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("fill-opacity"));
    assert!(svg.contains("<path"));
}

#[test]
fn test_line_with_band() {
    let x: Vec<f64> = (0..50).map(|i| i as f64 * 0.2).collect();
    let y: Vec<f64> = x.iter().map(|&v| v.sin()).collect();
    let y_lower: Vec<f64> = y.iter().map(|&v| v - 0.4).collect();
    let y_upper: Vec<f64> = y.iter().map(|&v| v + 0.4).collect();

    let line = LinePlot::new()
        .with_data(x.iter().zip(y.iter()).map(|(&a, &b)| (a, b)).collect::<Vec<_>>())
        .with_color("crimson")
        .with_band(y_lower, y_upper);

    let plots = vec![Plot::Line(line)];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Line with Confidence Band")
        .with_x_label("x")
        .with_y_label("sin(x)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/line_with_band.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("fill-opacity"));
}

#[test]
fn test_scatter_with_band() {
    let x: Vec<f64> = (0..30).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&v| v * 0.5 + 3.0).collect();
    let y_lower: Vec<f64> = y.iter().map(|&v| v - 2.0).collect();
    let y_upper: Vec<f64> = y.iter().map(|&v| v + 2.0).collect();

    let scatter = ScatterPlot::new()
        .with_data(x.iter().zip(y.iter()).map(|(&a, &b)| (a, b)).collect::<Vec<_>>())
        .with_color("darkorange")
        .with_band(y_lower, y_upper);

    let plots = vec![Plot::Scatter(scatter)];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Scatter with Confidence Band")
        .with_x_label("x")
        .with_y_label("y");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/scatter_with_band.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("fill-opacity"));
    assert!(svg.contains("<circle"));
}
