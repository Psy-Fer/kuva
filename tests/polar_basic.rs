use kuva::plot::polar::{PolarMode, PolarPlot};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::backend::svg::SvgBackend;

fn render(plot: PolarPlot) -> String {
    let plots = vec![Plot::Polar(plot)];
    let layout = Layout::auto_from_plots(&plots);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

fn write(name: &str, svg: &str) {
    std::fs::create_dir_all("test_outputs").ok();
    std::fs::write(format!("test_outputs/{name}.svg"), svg).unwrap();
}

#[test]
fn test_polar_basic() {
    let theta: Vec<f64> = (0..36).map(|i| i as f64 * 10.0).collect();
    let r: Vec<f64> = theta.iter().map(|&t| 1.0 + t.to_radians().cos()).collect();

    let plot = PolarPlot::new().with_series(r, theta);
    let svg = render(plot);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle") || svg.contains("<path"));
    write("polar_basic", &svg);
}

#[test]
fn test_polar_line() {
    let theta: Vec<f64> = (0..36).map(|i| i as f64 * 10.0).collect();
    let r: Vec<f64> = vec![1.5; 36];

    let plot = PolarPlot::new().with_series_line(r, theta);
    let svg = render(plot);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"));
    write("polar_line", &svg);
}

#[test]
fn test_polar_grid() {
    let theta: Vec<f64> = (0..12).map(|i| i as f64 * 30.0).collect();
    let r: Vec<f64> = vec![1.0; 12];

    let plot = PolarPlot::new()
        .with_series(r, theta)
        .with_grid(true)
        .with_r_grid_lines(4);
    let svg = render(plot);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"));
    write("polar_grid", &svg);
}

#[test]
fn test_polar_clockwise() {
    let theta: Vec<f64> = (0..36).map(|i| i as f64 * 10.0).collect();
    let r: Vec<f64> = theta.iter().map(|&t| 1.0 + 0.5 * t.to_radians().cos()).collect();

    let plot = PolarPlot::new()
        .with_series(r, theta)
        .with_clockwise(true)
        .with_theta_start(0.0);
    let svg = render(plot);
    assert!(svg.contains("<svg"));
    write("polar_clockwise", &svg);
}

#[test]
fn test_polar_r_max_override() {
    let theta: Vec<f64> = (0..36).map(|i| i as f64 * 10.0).collect();
    let r: Vec<f64> = vec![0.5; 36];

    let plot = PolarPlot::new()
        .with_series(r, theta)
        .with_r_max(2.0);
    let svg = render(plot);
    assert!(svg.contains("<svg"));
    write("polar_r_max", &svg);
}

#[test]
fn test_polar_multiple_series() {
    let theta: Vec<f64> = (0..36).map(|i| i as f64 * 10.0).collect();
    let r1: Vec<f64> = vec![1.0; 36];
    let r2: Vec<f64> = vec![2.0; 36];

    let plot = PolarPlot::new()
        .with_series_labeled(r1, theta.clone(), "Series A", PolarMode::Scatter)
        .with_series_labeled(r2, theta, "Series B", PolarMode::Scatter);
    let svg = render(plot);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle") || svg.contains("<path"));
    write("polar_multiple_series", &svg);
}

#[test]
fn test_polar_legend() {
    let theta: Vec<f64> = (0..36).map(|i| i as f64 * 10.0).collect();
    let r: Vec<f64> = vec![1.0; 36];

    let plot = PolarPlot::new()
        .with_series_labeled(r, theta, "Wind speed", PolarMode::Scatter)
        .with_legend(true);
    let plots = vec![Plot::Polar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Polar Legend Test");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Wind speed"));
    write("polar_legend", &svg);
}
