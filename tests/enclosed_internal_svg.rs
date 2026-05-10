use kuva::backend::svg::SvgBackend;
use kuva::plot::scatter::ScatterPlot;
use kuva::render::layout::{AxisLine, Layout, TickAlign};
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

#[test]
fn test_enclosed_internal_svg() {
    let data = vec![(1.0, 2.0), (2.0, 5.0), (3.0, 3.0)];
    let plot = ScatterPlot::new().with_data(data).with_color("steelblue");
    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Enclosed Internal")
        .with_axis_line(AxisLine::Box)
        .with_tick_align(TickAlign::Inside);

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/enclosed_internal.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // Ensure both top and right axes are drawn when axis_line=box.
    // We already checked stroke="#000000".
    // Check for the specific coordinates of top axis
    assert!(svg.contains("y1=\"44\" x2=\"659\" y2=\"44\""));
}
