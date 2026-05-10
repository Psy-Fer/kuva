use kuva::backend::svg::SvgBackend;
use kuva::plot::scatter::ScatterPlot;
use kuva::render::layout::Layout;
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
        .with_internal_ticks(true)
        .with_mirror_ticks(true)
        .with_y2_range(0.0, 500.0)
        .with_y2_label("Secondary Axis");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/y2_override_internal.svg", svg.clone()).unwrap();

    assert!(svg.contains("Secondary Axis"));
    // Top axis line (even with Y2, top should be enclosed if mirror_ticks is true)
    assert!(svg.contains("y1=\"44\" x2=\"659\" y2=\"44\""));
    // Right axis line (drawn by add_y2_axis)
    assert!(svg.contains("x1=\"659\" y1=\"44\" x2=\"659\" y2=\"494\""));

    // Ensure y2 labels are present
    assert!(svg.contains(">500<") || svg.contains(">400<") || svg.contains(">250<"));
}
