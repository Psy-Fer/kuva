use kuva::plot::{BoxPlot, ViolinPlot};
use kuva::plot::StripPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::Palette;

#[test]
fn test_strip_basic() {
    let strip = StripPlot::new()
        .with_group("A", vec![1.0, 2.0, 2.5, 3.1, 4.0, 3.5, 2.8])
        .with_group("B", vec![2.0, 2.1, 3.5, 3.8, 4.0, 4.2, 5.0])
        .with_group("C", vec![0.5, 1.5, 1.8, 2.2, 3.0, 3.3, 4.5])
        .with_color("steelblue");

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Strip Plot Basic")
        .with_y_label("Values");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/strip_basic.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_strip_swarm() {
    let strip = StripPlot::new()
        .with_group("Control", vec![1.0, 1.2, 1.5, 1.8, 2.0, 2.1, 2.3, 2.5, 2.7, 3.0])
        .with_group("Treatment", vec![2.5, 2.7, 3.0, 3.2, 3.5, 3.8, 4.0, 4.2, 4.5, 5.0])
        .with_color("coral")
        .with_swarm()
        .with_point_size(5.0);

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Strip Plot - Swarm Layout")
        .with_y_label("Measurement");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/strip_swarm.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_strip_center() {
    let strip = StripPlot::new()
        .with_group("Group1", vec![1.0, 2.0, 3.0, 4.0, 5.0])
        .with_group("Group2", vec![1.5, 2.5, 3.5, 4.5])
        .with_color("purple")
        .with_center()
        .with_point_size(3.0);

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Strip Plot - Center Layout")
        .with_y_label("Values");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/strip_center.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_strip_legend_palette() {
    let strip_a = StripPlot::new()
        .with_group("WT", vec![1.0, 1.5, 2.0, 2.5, 3.0])
        .with_legend("Wild Type");

    let strip_b = StripPlot::new()
        .with_group("KO", vec![2.0, 2.5, 3.0, 3.5, 4.0])
        .with_legend("Knockout");

    let plots = vec![Plot::Strip(strip_a), Plot::Strip(strip_b)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Strip Plot - Palette + Legend")
        .with_y_label("Expression")
        .with_palette(Palette::wong());

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/strip_legend_palette.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_box_with_strip_overlay() {
    let boxplot = BoxPlot::new()
        .with_group("A", vec![1.0, 2.0, 2.5, 3.0, 4.0, 5.0, 2.2, 3.3])
        .with_group("B", vec![2.0, 2.1, 3.5, 3.8, 4.0, 4.2, 5.5, 3.0])
        .with_group("C", vec![0.5, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0])
        .with_color("steelblue")
        .with_strip(0.25);

    let plots = vec![Plot::Box(boxplot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Box Plot with Strip Overlay")
        .with_y_label("Values");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/box_with_strip_overlay.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_box_with_swarm_overlay() {
    let boxplot = BoxPlot::new()
        .with_group("Control", vec![1.0, 1.2, 1.5, 1.8, 2.0, 2.1, 2.3, 2.5, 2.7, 3.0])
        .with_group("Treated", vec![2.5, 2.7, 3.0, 3.2, 3.5, 3.8, 4.0, 4.2, 4.5, 5.0])
        .with_color("lightblue")
        .with_swarm_overlay()
        .with_overlay_color("rgba(30,100,200,0.7)")
        .with_overlay_size(4.0);

    let plots = vec![Plot::Box(boxplot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Box Plot with Swarm Overlay")
        .with_y_label("Measurement");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/box_with_swarm_overlay.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_violin_with_strip_overlay() {
    let violin = ViolinPlot::new()
        .with_group("Alpha", vec![1.0, 1.5, 2.0, 2.2, 2.8, 3.0, 3.5, 4.0])
        .with_group("Beta", vec![2.0, 2.5, 3.0, 3.1, 3.5, 4.0, 4.2, 5.0])
        .with_color("mediumpurple")
        .with_strip(0.2)
        .with_overlay_color("rgba(0,0,0,0.5)")
        .with_overlay_size(3.0);

    let plots = vec![Plot::Violin(violin)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Violin Plot with Strip Overlay")
        .with_y_label("Values");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/violin_with_strip_overlay.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_strip_and_box_composed() {
    // Box and Strip sharing the same categorical x-axis
    let boxplot = BoxPlot::new()
        .with_group("A", vec![1.0, 2.0, 2.5, 3.0, 4.0, 5.0])
        .with_group("B", vec![2.0, 2.5, 3.5, 4.0, 4.5, 5.0])
        .with_color("lightblue")
        .with_legend("Boxes");

    let strip = StripPlot::new()
        .with_group("A", vec![1.0, 2.0, 2.5, 3.0, 4.0, 5.0])
        .with_group("B", vec![2.0, 2.5, 3.5, 4.0, 4.5, 5.0])
        .with_color("rgba(200,50,50,0.7)")
        .with_jitter(0.15)
        .with_point_size(3.5)
        .with_legend("Points");

    let plots = vec![Plot::Box(boxplot), Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Box + Strip Composed")
        .with_y_label("Values");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/strip_and_box_composed.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}
