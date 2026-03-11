use kuva::plot::polar::{PolarMode, PolarPlot};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::backend::svg::SvgBackend;

fn main() {
    // Cardioid: r = 1 + cos(theta)
    let n = 72;
    let theta_cardioid: Vec<f64> = (0..n).map(|i| i as f64 * 360.0 / n as f64).collect();
    let r_cardioid: Vec<f64> = theta_cardioid
        .iter()
        .map(|&t| {
            let rad = t.to_radians();
            1.0 + rad.cos()
        })
        .collect();

    // Reference circle: r = 1.0
    let theta_circle: Vec<f64> = (0..=n).map(|i| i as f64 * 360.0 / n as f64).collect();
    let r_circle: Vec<f64> = vec![1.0; theta_circle.len()];

    let plot = PolarPlot::new()
        .with_series_labeled(r_cardioid, theta_cardioid, "Cardioid", PolarMode::Line)
        .with_series_labeled(r_circle, theta_circle, "Unit circle", PolarMode::Line)
        .with_r_max(2.1)
        .with_r_grid_lines(4)
        .with_theta_divisions(12)
        .with_legend(true);

    let plots = vec![Plot::Polar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Polar Plot");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::create_dir_all("docs/src/assets/polar").unwrap();
    std::fs::write("docs/src/assets/polar/basic.svg", svg).unwrap();
    println!("Written docs/src/assets/polar/basic.svg");
}
