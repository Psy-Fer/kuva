use kuva::backend::svg::SvgBackend;
use kuva::plot::scatter::ScatterPlot;
use kuva::render::layout::{AxisLine, Layout, TickAlign, TickPos};
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

#[test]
fn test_enclosed_mirrored_svg() {
    let data = vec![(1.0, 2.0), (2.0, 5.0), (3.0, 3.0)];
    let plot = ScatterPlot::new().with_data(data).with_color("seagreen");
    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Enclosed Mirrored")
        .with_tick_align(TickAlign::Inside)
        .with_tick_pos(TickPos::Both); // Should automatically enable axis_line=box

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/enclosed_mirrored.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // Top axis
    assert!(svg.contains("y1=\"44\" x2=\"659\" y2=\"44\""));
    // Right axis
    assert!(svg.contains("x1=\"659\" y1=\"44\" x2=\"659\" y2=\"494\""));
    // Mirrored ticks should also be present
    assert!(svg.matches("stroke=\"#000000\"").count() > 10);
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
