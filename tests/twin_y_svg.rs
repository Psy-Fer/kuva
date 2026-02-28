use kuva::plot::line::LinePlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_twin_y;
use kuva::backend::svg::SvgBackend;
use kuva::Palette;
use std::fs;

fn save_svg(name: &str, svg: &str) {
    fs::create_dir_all("test_outputs").unwrap();
    fs::write(format!("test_outputs/{}.svg", name), svg).unwrap();
}

fn make_temperature_line() -> Plot {
    let points: Vec<(f64, f64)> = vec![
        (1.0, 5.0), (2.0, 8.0), (3.0, 14.0), (4.0, 20.0), (5.0, 24.0), (6.0, 22.0),
    ];
    Plot::Line(LinePlot::new().with_data(points).with_legend("Temperature (°C)"))
}

fn make_rainfall_line() -> Plot {
    let points: Vec<(f64, f64)> = vec![
        (1.0, 80.0), (2.0, 60.0), (3.0, 45.0), (4.0, 30.0), (5.0, 20.0), (6.0, 35.0),
    ];
    Plot::Line(LinePlot::new().with_data(points).with_legend("Rainfall (mm)"))
}

#[test]
fn test_twin_y_basic() {
    let primary = vec![make_temperature_line()];
    let secondary = vec![make_rainfall_line()];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary);
    let scene = render_twin_y(primary, secondary, layout);
    let svg = SvgBackend.render_scene(&scene);
    save_svg("twin_y_basic", &svg);

    assert!(svg.contains("<svg"), "SVG should start with <svg element");
    // Right-side axis line should exist (add_y2_axis draws a vertical line at the right edge)
    assert!(svg.contains("x1="), "SVG should contain line elements");
}

#[test]
fn test_twin_y_labels() {
    let primary = vec![make_temperature_line()];
    let secondary = vec![make_rainfall_line()];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary)
        .with_y_label("Temp (°C)")
        .with_y2_label("Rain");

    let scene = render_twin_y(primary, secondary, layout);
    let svg = SvgBackend.render_scene(&scene);
    save_svg("twin_y_labels", &svg);

    assert!(svg.contains("Rain"), "SVG should contain the y2 label 'Rain'");
    assert!(svg.contains("Temp"), "SVG should contain the y label 'Temp'");
}

#[test]
fn test_twin_y_auto() {
    // Test that auto_from_twin_y_plots builds and renders without panic
    let primary = vec![make_temperature_line()];
    let secondary = vec![make_rainfall_line()];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary);
    let scene = render_twin_y(primary, secondary, layout);
    let svg = SvgBackend.render_scene(&scene);
    save_svg("twin_y_auto", &svg);

    assert!(svg.contains("<svg"), "SVG output should be valid");
}

#[test]
fn test_twin_y_palette() {
    let primary = vec![make_temperature_line()];
    let secondary = vec![make_rainfall_line()];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary)
        .with_palette(Palette::wong());

    let scene = render_twin_y(primary, secondary, layout);
    let svg = SvgBackend.render_scene(&scene);
    save_svg("twin_y_palette", &svg);

    // Wong palette first two colors: #E69F00, #56B4E9
    assert!(svg.contains("#E69F00"), "SVG should contain wong palette color 1");
    assert!(svg.contains("#56B4E9"), "SVG should contain wong palette color 2");
}

#[test]
fn test_twin_y_log_y2() {
    let primary_points: Vec<(f64, f64)> = vec![
        (1.0, 10.0), (2.0, 20.0), (3.0, 30.0), (4.0, 40.0), (5.0, 50.0),
    ];
    let secondary_points: Vec<(f64, f64)> = vec![
        (1.0, 1.0), (2.0, 10.0), (3.0, 100.0), (4.0, 1000.0), (5.0, 10000.0),
    ];

    let primary = vec![Plot::Line(LinePlot::new().with_data(primary_points))];
    let secondary = vec![Plot::Line(LinePlot::new().with_data(secondary_points))];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary)
        .with_log_y2();

    let scene = render_twin_y(primary, secondary, layout);
    let svg = SvgBackend.render_scene(&scene);
    save_svg("twin_y_log_y2", &svg);

    assert!(svg.contains("<svg"), "SVG should be valid");
    // Log ticks like 1, 100 should appear as text elements in the right-side axis labels
    assert!(svg.contains(">1<"), "SVG should contain log tick '1'");
    assert!(svg.contains(">100<"), "SVG should contain log tick '100'");
}
