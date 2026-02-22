use visus::plot::LinePlot;
use visus::backend::svg::SvgBackend;
use visus::render::render::{render_line, render_multiple};
use visus::render::layout::Layout;
use visus::render::plots::Plot;

#[test]
fn test_line_svg_output_builder() {
    let plot = LinePlot::new()
                        .with_data((0..100)
                        .map(|x| (x as f64 / 10.0, (x as f64 / 10.0).sin()))
                        .collect::<Vec<_>>())
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

#[test]
fn test_line_styles() {
    let xs: Vec<f64> = (0..100).map(|x| x as f64 / 10.0).collect();

    let solid = LinePlot::new()
        .with_data(xs.iter().map(|&x| (x, x.sin())))
        .with_color("blue")
        .with_legend("Solid");

    let dashed = LinePlot::new()
        .with_data(xs.iter().map(|&x| (x, x.cos())))
        .with_color("red")
        .with_dashed()
        .with_legend("Dashed");

    let dotted = LinePlot::new()
        .with_data(xs.iter().map(|&x| (x, (x * 0.5).sin())))
        .with_color("green")
        .with_dotted()
        .with_legend("Dotted");

    let dashdot = LinePlot::new()
        .with_data(xs.iter().map(|&x| (x, (x * 0.5).cos())))
        .with_color("purple")
        .with_dashdot()
        .with_legend("Dash-Dot");

    let plots = vec![
        Plot::Line(solid),
        Plot::Line(dashed),
        Plot::Line(dotted),
        Plot::Line(dashdot),
    ];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Line Styles")
        .with_x_label("X")
        .with_y_label("Y")
;

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/line_styles.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains(r#"stroke-dasharray="8 4""#)); // dashed
    assert!(svg.contains(r#"stroke-dasharray="2 4""#)); // dotted
    assert!(svg.contains(r#"stroke-dasharray="8 4 2 4""#)); // dashdot
}

#[test]
fn test_line_step() {
    let data: Vec<(f64, f64)> = (0..10).map(|x| (x as f64, (x as f64).sin())).collect();
    let n = data.len();

    let plot = LinePlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_step();

    let plots = vec![Plot::Line(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Step Plot");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/line_step.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // Step path should have more L segments than data points (2 per step after the first)
    let l_count = svg.matches(" L ").count();
    assert!(l_count > n, "step path should have more L segments than data points");
}

#[test]
fn test_line_area() {
    let plot = LinePlot::new()
        .with_data((0..100).map(|x| (x as f64 / 10.0, (x as f64 / 10.0).sin())))
        .with_color("coral")
        .with_fill();

    let plots = vec![Plot::Line(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Area Plot");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/line_area.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("fill-opacity"));
    assert!(svg.contains(" Z"));
}

#[test]
fn test_line_step_area() {
    let plot = LinePlot::new()
        .with_data((0..20).map(|x| (x as f64, (x as f64 * 0.5).sin())))
        .with_color("seagreen")
        .with_step()
        .with_fill()
        .with_fill_opacity(0.4);

    let plots = vec![Plot::Line(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Step Area Plot");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/line_step_area.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("fill-opacity"));
    assert!(svg.contains(" Z"));
}

#[test]
fn test_font_family() {
    let plot = LinePlot::new()
        .with_data(vec![(0.0, 0.0), (1.0, 1.0), (2.0, 0.5)])
        .with_color("black");

    let plots = vec![Plot::Line(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Font Test")
        .with_x_label("X")
        .with_y_label("Y")
        .with_font_family("Arial, sans-serif");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/line_font_family.svg", svg.clone()).unwrap();

    assert!(svg.contains(r#"font-family="Arial, sans-serif""#));
}

#[test]
fn test_custom_font_sizes() {
    let plot = LinePlot::new()
        .with_data(vec![(0.0, 0.0), (1.0, 1.0), (2.0, 0.5)])
        .with_color("black");

    let plots = vec![Plot::Line(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Big Title")
        .with_x_label("X axis")
        .with_y_label("Y axis")
        .with_title_size(24)
        .with_label_size(18)
        .with_tick_size(14)
        .with_font_family("Courier, monospace");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/line_custom_fonts.svg", svg.clone()).unwrap();

    assert!(svg.contains(r#"font-size="24""#)); // title
    assert!(svg.contains(r#"font-size="18""#)); // axis labels
    assert!(svg.contains(r#"font-size="14""#)); // tick labels
    assert!(svg.contains(r#"font-family="Courier, monospace""#));
}

#[test]
fn test_line_log_y() {
    // Exponential growth on log Y
    let data: Vec<(f64, f64)> = (0..10)
        .map(|i| (i as f64, 10f64.powf(i as f64 * 0.5)))
        .collect();

    let line = LinePlot::new()
        .with_data(data)
        .with_color("steelblue");

    let plots = vec![Plot::Line(line)];
    let layout = Layout::auto_from_plots(&plots)
        .with_log_y()
        .with_title("Exponential Growth (log Y)")
        .with_x_label("Time")
        .with_y_label("Value");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/line_log_y.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"));
    // Y axis should have log ticks
    assert!(svg.contains(">1</text>"));
    assert!(svg.contains(">100</text>") || svg.contains(">1000</text>"));
    // X axis should have linear ticks
    assert!(svg.contains(".0</text>"));
}

#[test]
fn test_line_log_xy_wide_range() {
    // Data spanning many decades on both axes
    let data: Vec<(f64, f64)> = vec![
        (0.001, 0.01), (0.01, 0.1), (0.1, 1.0),
        (1.0, 10.0), (10.0, 100.0), (100.0, 1000.0),
        (1000.0, 10000.0), (10000.0, 100000.0),
    ];

    let line = LinePlot::new()
        .with_data(data)
        .with_color("darkgreen")
        .with_stroke_width(2.0);

    let plots = vec![Plot::Line(line)];
    let layout = Layout::auto_from_plots(&plots)
        .with_log_scale()
        .with_title("Log-Log — Wide Range (8 decades)")
        .with_x_label("X")
        .with_y_label("Y");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/line_log_wide.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"));
    // Should have ticks spanning many decades
    assert!(svg.contains("1e-"));
    assert!(svg.contains(">10000</text>") || svg.contains(">100000</text>"));
}