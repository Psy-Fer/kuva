use kuva::{Palette, Theme};
use kuva::plot::scatter::ScatterPlot;
use kuva::plot::line::LinePlot;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::backend::svg::SvgBackend;

#[test]
fn test_palette_dark_theme_with_colorblind() {
    let pal = Palette::wong();

    let s1 = ScatterPlot::new()
        .with_data(vec![(1.0, 2.0), (2.0, 3.0), (3.0, 5.0)])
        .with_color(&pal[0])
        .with_legend("Group A");

    let s2 = ScatterPlot::new()
        .with_data(vec![(1.0, 4.0), (2.0, 1.0), (3.0, 3.0)])
        .with_color(&pal[1])
        .with_legend("Group B");

    let plots = vec![Plot::Scatter(s1), Plot::Scatter(s2)];
    let layout = Layout::auto_from_plots(&plots)
        .with_theme(Theme::dark())
        .with_title("Wong Palette + Dark Theme");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/palette_dark_colorblind.svg", &svg).unwrap();

    // Wong palette colors appear in the SVG
    assert!(svg.contains("#E69F00"), "expected wong color 0");
    assert!(svg.contains("#56B4E9"), "expected wong color 1");
    // Dark background
    assert!(svg.contains(r##"fill="#1e1e1e""##), "expected dark background");
}

#[test]
fn test_palette_auto_cycle() {
    let s1 = ScatterPlot::new()
        .with_data(vec![(0.0, 1.0), (1.0, 2.0)])
        .with_legend("A");

    let s2 = ScatterPlot::new()
        .with_data(vec![(0.0, 3.0), (1.0, 4.0)])
        .with_legend("B");

    let s3 = ScatterPlot::new()
        .with_data(vec![(0.0, 5.0), (1.0, 6.0)])
        .with_legend("C");

    let plots = vec![Plot::Scatter(s1), Plot::Scatter(s2), Plot::Scatter(s3)];
    let layout = Layout::auto_from_plots(&plots)
        .with_palette(Palette::wong())
        .with_title("Auto-Cycle Wong");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/palette_auto_cycle.svg", &svg).unwrap();

    // First three Wong colors assigned automatically
    assert!(svg.contains("#E69F00"), "expected wong[0]");
    assert!(svg.contains("#56B4E9"), "expected wong[1]");
    assert!(svg.contains("#009E73"), "expected wong[2]");
}

#[test]
fn test_palette_indexing() {
    let pal = Palette::wong();

    assert_eq!(pal.len(), 8);
    assert_eq!(&pal[0], "#E69F00");
    assert_eq!(&pal[7], "#000000");
    // Wraps on overflow
    assert_eq!(&pal[8], "#E69F00");
    assert_eq!(&pal[10], "#009E73");
}

#[test]
fn test_palette_custom() {
    let pal = Palette::custom("mine", vec![
        "red".into(), "green".into(), "blue".into(),
    ]);

    assert_eq!(pal.name, "mine");
    assert_eq!(pal.len(), 3);
    assert_eq!(&pal[0], "red");
    assert_eq!(&pal[1], "green");
    assert_eq!(&pal[2], "blue");
    // Wraps
    assert_eq!(&pal[3], "red");
}

#[test]
fn test_palette_tritanopia() {
    let pal = Palette::tritanopia();

    // Tritanopia maps to Tol Bright
    assert_eq!(pal.len(), 7);
    assert_eq!(&pal[0], "#4477AA");
    assert_eq!(&pal[1], "#EE6677");

    // Use it with auto-cycle
    let l1 = LinePlot::new()
        .with_data(vec![(0.0, 0.0), (1.0, 1.0), (2.0, 0.5)])
        .with_legend("X");

    let l2 = LinePlot::new()
        .with_data(vec![(0.0, 1.0), (1.0, 0.5), (2.0, 1.5)])
        .with_legend("Y");

    let plots = vec![Plot::Line(l1), Plot::Line(l2)];
    let layout = Layout::auto_from_plots(&plots)
        .with_palette(Palette::tritanopia())
        .with_title("Tritanopia Safe");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/palette_tritanopia.svg", &svg).unwrap();

    assert!(svg.contains("#4477AA"), "expected tol_bright[0]");
    assert!(svg.contains("#EE6677"), "expected tol_bright[1]");
}
