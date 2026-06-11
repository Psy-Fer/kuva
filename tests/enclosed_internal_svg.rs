mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::scatter::ScatterPlot;
use kuva::render::layout::{AxisLine, Layout, TickAlign};
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

fn make_scatter() -> Vec<(f64, f64)> {
    vec![(1.0, 2.0), (2.0, 5.0), (3.0, 3.0)]
}

#[test]
fn test_enclosed_internal_svg() {
    let plot = ScatterPlot::new()
        .with_data(make_scatter())
        .with_color("steelblue");
    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Enclosed Internal")
        .with_axis_line(AxisLine::Box)
        .with_tick_align(TickAlign::Inside);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/enclosed_internal.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));

    // Box mode adds a top and right border. Count <line> elements in box vs
    // the identical plot rendered with the default Open axis line.
    let plot_open = ScatterPlot::new()
        .with_data(make_scatter())
        .with_color("steelblue");
    let plots_open = vec![Plot::Scatter(plot_open)];
    let layout_open = Layout::auto_from_plots(&plots_open).with_title("Enclosed Internal");
    let svg_open = SvgBackend.render_scene(&render_multiple(plots_open, layout_open));

    let box_lines = svg.matches("<line").count();
    let open_lines = svg_open.matches("<line").count();
    assert!(
        box_lines > open_lines,
        "box mode should emit more <line> elements than open mode ({box_lines} vs {open_lines})"
    );
}

#[test]
fn test_axis_line_open_is_default() {
    let layout = Layout::new((0.0, 1.0), (0.0, 1.0));
    assert_eq!(layout.axis_line, AxisLine::Open);
}
