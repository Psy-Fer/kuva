use kuva::backend::svg::SvgBackend;
use kuva::plot::scatter::ScatterPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

#[test]
fn test_enclosed_mirrored_svg() {
    let data = vec![(1.0, 2.0), (2.0, 5.0), (3.0, 3.0)];
    let plot = ScatterPlot::new().with_data(data).with_color("seagreen");
    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Enclosed Mirrored")
        .with_internal_ticks(true)
        .with_mirror_ticks(true); // Should automatically enable enclosed_axes

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
