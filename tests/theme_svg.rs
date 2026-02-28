use kuva::Theme;
use kuva::plot::scatter::ScatterPlot;
use kuva::plot::line::LinePlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_scatter;
use kuva::render::render::render_line;
use kuva::render::layout::Layout;

#[test]
fn test_theme_dark() {
    let plot = ScatterPlot::new()
        .with_data(vec![(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)])
        .with_color("cyan")
        .with_size(4.0);

    let layout = Layout::new((0.0, 6.0), (0.0, 7.0))
        .with_theme(Theme::dark())
        .with_title("Dark Theme Scatter");

    let scene = render_scatter(&plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/theme_dark.svg", &svg).unwrap();

    // Dark background
    assert!(svg.contains(r##"fill="#1e1e1e""##), "expected dark background");
    // Text color on root svg element
    assert!(svg.contains(r##"fill="#e0e0e0""##), "expected light text fill");
    // Axis color (not red/green)
    assert!(svg.contains(r##"stroke="#cccccc""##), "expected themed axis color");
    // No red/green debug axes
    assert!(!svg.contains(r#"stroke="red""#), "should not have red axis");
    assert!(!svg.contains(r#"stroke="green""#), "should not have green axis");
}

#[test]
fn test_theme_minimal() {
    let plot = LinePlot::new()
        .with_data(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 2.0), (3.0, 5.0)])
        .with_color("steelblue");

    let layout = Layout::new((0.0, 4.0), (0.0, 6.0))
        .with_theme(Theme::minimal())
        .with_title("Minimal Theme Line");

    let scene = render_line(&plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/theme_minimal.svg", &svg).unwrap();

    // Minimal has show_grid=false, so no grid lines (grid color should not appear)
    assert!(!svg.contains(r##"stroke="#e0e0e0""##), "should have no grid lines");
    // Serif font from theme
    assert!(svg.contains(r#"font-family="serif""#), "expected serif font");
}

#[test]
fn test_theme_solarized() {
    let plot = ScatterPlot::new()
        .with_data(vec![(1.0, 1.0), (2.0, 4.0), (3.0, 2.0)])
        .with_color("#268bd2")
        .with_size(5.0);

    let layout = Layout::new((0.0, 4.0), (0.0, 5.0))
        .with_theme(Theme::solarized())
        .with_title("Solar Scatter");

    let scene = render_scatter(&plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/theme_solarized.svg", &svg).unwrap();

    // Solarized background
    assert!(svg.contains(r##"fill="#fdf6e3""##), "expected solarized background");
    // Solarized text color
    assert!(svg.contains(r##"fill="#657b83""##), "expected solarized text color");
}

#[test]
fn test_theme_override() {
    // Theme sets serif, but explicit with_font_family overrides
    let plot = ScatterPlot::new()
        .with_data(vec![(1.0, 2.0), (3.0, 4.0)])
        .with_color("red")
        .with_size(4.0);

    let layout = Layout::new((0.0, 4.0), (0.0, 5.0))
        .with_theme(Theme::minimal())   // sets serif
        .with_font_family("monospace"); // override to monospace

    let scene = render_scatter(&plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/theme_override.svg", &svg).unwrap();

    // Override wins
    assert!(svg.contains(r#"font-family="monospace""#), "expected monospace override");
    assert!(!svg.contains(r#"font-family="serif""#), "serif should be overridden");
}
