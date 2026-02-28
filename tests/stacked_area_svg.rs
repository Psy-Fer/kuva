use kuva::plot::StackedAreaPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::palette::Palette;

#[test]
fn stacked_area_basic() {
    let sa = StackedAreaPlot::new()
        .with_x([0.0, 1.0, 2.0, 3.0, 4.0])
        .with_series([10.0, 20.0, 15.0, 25.0, 18.0])
        .with_color("steelblue")
        .with_legend("Series A")
        .with_series([5.0, 10.0, 8.0, 12.0, 9.0])
        .with_color("orange")
        .with_legend("Series B")
        .with_series([3.0, 6.0, 4.0, 7.0, 5.0])
        .with_color("green")
        .with_legend("Series C");

    let plots = vec![Plot::StackedArea(sa)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Stacked Area — Basic")
        .with_x_label("Time")
        .with_y_label("Value");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/stacked_area_basic.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"));
    assert!(svg.contains("steelblue"));
    assert!(svg.contains("orange"));
    assert!(svg.contains("green"));
    assert!(svg.contains("Series A"));
    assert!(svg.contains("Series B"));
    assert!(svg.contains("Series C"));
}

#[test]
fn stacked_area_normalized() {
    let sa = StackedAreaPlot::new()
        .with_x([0.0, 1.0, 2.0, 3.0, 4.0])
        .with_series([10.0, 20.0, 15.0, 25.0, 18.0])
        .with_color("steelblue")
        .with_legend("Series A")
        .with_series([5.0, 10.0, 8.0, 12.0, 9.0])
        .with_color("orange")
        .with_legend("Series B")
        .with_series([3.0, 6.0, 4.0, 7.0, 5.0])
        .with_color("green")
        .with_legend("Series C")
        .with_normalized();

    let plots = vec![Plot::StackedArea(sa)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Stacked Area — Normalized (100%)")
        .with_x_label("Time")
        .with_y_label("Percent (%)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/stacked_area_normalized.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"));
    // y-axis should show 100
    assert!(svg.contains("100"));
}

#[test]
fn stacked_area_palette() {
    // No explicit colors; rely on default fallback colors in StackedAreaPlot
    let sa = StackedAreaPlot::new()
        .with_x([2020.0, 2021.0, 2022.0, 2023.0, 2024.0])
        .with_series([30.0, 32.0, 35.0, 38.0, 40.0])
        .with_legend("Alpha")
        .with_series([20.0, 22.0, 21.0, 25.0, 27.0])
        .with_legend("Beta")
        .with_series([10.0, 12.0, 14.0, 13.0, 15.0])
        .with_legend("Gamma");

    let plots = vec![Plot::StackedArea(sa)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Stacked Area — Default Colors")
        .with_x_label("Year")
        .with_y_label("Count")
        .with_palette(Palette::wong());

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/stacked_area_palette.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"));
    assert!(svg.contains("Alpha"));
    assert!(svg.contains("Beta"));
    assert!(svg.contains("Gamma"));
}
