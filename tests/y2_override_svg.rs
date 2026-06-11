mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::scatter::ScatterPlot;
use kuva::render::layout::{Layout, TickAlign, TickPos};
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

#[test]
fn test_y2_override_svg() {
    let data1 = vec![(1.0, 2.0), (2.0, 4.0), (3.0, 6.0)];
    let data2 = vec![(1.0, 100.0), (2.0, 250.0), (3.0, 150.0)];

    let plot1 = ScatterPlot::new()
        .with_data(data1)
        .with_color("steelblue")
        .with_legend("Primary");
    let plot2 = ScatterPlot::new()
        .with_data(data2)
        .with_color("crimson")
        .with_legend("Secondary");
    let plots = vec![Plot::Scatter(plot1), Plot::Scatter(plot2)];

    let layout = Layout::new((0.5, 3.5), (0.0, 10.0))
        .with_title("Y2 Override")
        .with_tick_align(TickAlign::Inside)
        .with_tick_pos(TickPos::Both)
        .with_y2_range(0.0, 500.0)
        .with_y2_label("Secondary Axis");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/y2_override_internal.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Secondary Axis"), "y2 axis label should appear");
    // y2 tick labels — at least one of the expected values must appear.
    assert!(
        svg.contains(">500<") || svg.contains(">400<") || svg.contains(">250<"),
        "y2 tick labels should appear in SVG"
    );
    // With tick_pos=Both the plot is promoted to box mode; the top border
    // line must be present. The right axis is owned by add_y2_axis.
    // Both are checked structurally: the enclosed+y2 render should emit more
    // <line> elements than the same two-series plot with default (Open/Primary) axes.
    let plot1b = ScatterPlot::new()
        .with_data(vec![(1.0, 2.0), (2.0, 4.0), (3.0, 6.0)])
        .with_color("steelblue")
        .with_legend("Primary");
    let plot2b = ScatterPlot::new()
        .with_data(vec![(1.0, 100.0), (2.0, 250.0), (3.0, 150.0)])
        .with_color("crimson")
        .with_legend("Secondary");
    let plots_plain = vec![Plot::Scatter(plot1b), Plot::Scatter(plot2b)];
    let layout_plain = Layout::new((0.5, 3.5), (0.0, 10.0))
        .with_title("Y2 Override")
        .with_y2_range(0.0, 500.0)
        .with_y2_label("Secondary Axis");
    let svg_plain = SvgBackend.render_scene(&render_multiple(plots_plain, layout_plain));

    assert!(
        svg.matches("<line").count() > svg_plain.matches("<line").count(),
        "enclosed+inside ticks should add more <line> elements than default axes"
    );
}
