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