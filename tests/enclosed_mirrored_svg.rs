mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::scatter::ScatterPlot;
use kuva::render::layout::{AxisLine, Layout, TickAlign, TickPos};
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

fn make_scatter() -> Vec<(f64, f64)> {
    vec![(1.0, 2.0), (2.0, 5.0), (3.0, 3.0)]
}

#[test]
fn test_enclosed_mirrored_svg() {
    let plot = ScatterPlot::new().with_data(make_scatter()).with_color("seagreen");
    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Enclosed Mirrored")
        .with_tick_align(TickAlign::Inside)
        .with_tick_pos(TickPos::Both);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/enclosed_mirrored.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));

    // with_tick_pos(Both) should auto-promote to AxisLine::Box — verify by
    // comparing line count against the same plot with Primary ticks only.
    let plot_primary = ScatterPlot::new().with_data(make_scatter()).with_color("seagreen");
    let plots_primary = vec![Plot::Scatter(plot_primary)];
    let layout_primary = Layout::auto_from_plots(&plots_primary)
        .with_title("Enclosed Mirrored")
        .with_tick_align(TickAlign::Inside)
        .with_tick_pos(TickPos::Primary);
    let svg_primary = SvgBackend.render_scene(&render_multiple(plots_primary, layout_primary));

    let mirrored_lines = svg.matches("<line").count();
    let primary_lines = svg_primary.matches("<line").count();
    assert!(
        mirrored_lines > primary_lines,
        "mirrored ticks should emit more <line> elements than primary-only \
         ({mirrored_lines} vs {primary_lines})"
    );
}

#[test]
fn test_tick_pos_both_auto_sets_box() {
    let layout = Layout::new((0.0, 1.0), (0.0, 1.0)).with_tick_pos(TickPos::Both);
    assert_eq!(
        layout.axis_line,
        AxisLine::Box,
        "with_tick_pos(Both) must promote axis_line to Box"
    );
}

#[test]
fn test_axis_line_tick_align_tick_pos_builders() {
    let layout = Layout::new((0.0, 1.0), (0.0, 1.0))
        .with_axis_line("box")
        .with_tick_align("center")
        .with_tick_pos("both");

    assert_eq!(layout.axis_line, AxisLine::Box);
    assert_eq!(layout.tick_align, TickAlign::Center);
    assert_eq!(layout.tick_pos, TickPos::Both);
}
