use kuva::plot::ScatterPlot;
use kuva::render::{plots::Plot, layout::Layout, render::render_multiple};
use kuva::backend::svg::SvgBackend;

fn scatter_svg(layout: Layout) -> String {
    // Simple scatter: two points giving x in [0,13], y in [0,5]
    let plot = ScatterPlot::new()
        .with_data(vec![(0.0f64, 0.0f64), (13.0, 5.0)]);
    let plots = vec![Plot::Scatter(plot)];
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

/// Axis range override: x capped at 10 should suppress auto-tick "15".
#[test]
fn test_axis_range_override() {
    // Without override, auto-nice_range on [0, 13.13] produces ticks up to 15.
    let plots = vec![Plot::Scatter(ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (13.0, 5.0)]))];
    let layout_auto = Layout::auto_from_plots(&plots);
    let svg_auto = SvgBackend.render_scene(&render_multiple(plots, layout_auto));
    assert!(svg_auto.contains("15"), "auto range should include tick 15");

    // With override, x stops at 10.
    let plots2 = vec![Plot::Scatter(ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (13.0, 5.0)]))];
    let layout_override = Layout::auto_from_plots(&plots2)
        .with_x_axis_min(0.0)
        .with_x_axis_max(10.0);
    let svg_override = scatter_svg(layout_override);
    std::fs::write("test_outputs/tick_control_range.svg", &svg_override).unwrap();
    assert!(svg_override.contains("10"), "overridden range should include tick 10");
    assert!(!svg_override.contains(">15<"), "overridden range should not show tick 15");
}

/// Explicit tick step: with_x_tick_step(2.5) on [0,10] produces 0, 2.5, 5, 7.5, 10.
#[test]
fn test_explicit_tick_step() {
    let plots = vec![Plot::Scatter(ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]))];
    let layout = Layout::auto_from_plots(&plots)
        .with_x_axis_min(0.0)
        .with_x_axis_max(10.0)
        .with_x_tick_step(2.5);
    let svg = scatter_svg(layout);
    std::fs::write("test_outputs/tick_control_step.svg", &svg).unwrap();
    assert!(svg.contains(">0<") || svg.contains(">0.0<") || svg.contains("\"0\"") || svg.contains(">0"),
        "tick 0 should appear");
    assert!(svg.contains("2.5"), "tick 2.5 should appear");
    assert!(svg.contains(">5<") || svg.contains("5.0") || svg.contains(">5"),
        "tick 5 should appear");
    assert!(svg.contains("7.5"), "tick 7.5 should appear");
    assert!(svg.contains(">10<") || svg.contains(">10"),
        "tick 10 should appear");
}

/// Minor ticks: enabling minor_ticks=5 adds more line elements to the SVG.
#[test]
fn test_minor_ticks() {
    let plots = vec![Plot::Scatter(ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]))];
    let layout_no_minor = Layout::auto_from_plots(&plots);
    let svg_no_minor = scatter_svg(layout_no_minor);

    let plots2 = vec![Plot::Scatter(ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]))];
    let layout_minor = Layout::auto_from_plots(&plots2).with_minor_ticks(5);
    let svg_minor = scatter_svg(layout_minor);
    std::fs::write("test_outputs/tick_control_minor.svg", &svg_minor).unwrap();

    let lines_without = svg_no_minor.matches("<line").count();
    let lines_with    = svg_minor.matches("<line").count();
    assert!(lines_with > lines_without,
        "minor ticks should add more line elements ({} vs {})", lines_with, lines_without);
}

/// Minor grid: enabling show_minor_grid adds even more line elements.
#[test]
fn test_minor_grid() {
    let plots = vec![Plot::Scatter(ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]))];
    let layout_minor = Layout::auto_from_plots(&plots).with_minor_ticks(5);
    let svg_minor = scatter_svg(layout_minor);

    let plots2 = vec![Plot::Scatter(ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]))];
    let layout_grid = Layout::auto_from_plots(&plots2)
        .with_minor_ticks(5)
        .with_show_minor_grid(true);
    let svg_grid = scatter_svg(layout_grid);
    std::fs::write("test_outputs/tick_control_minor_grid.svg", &svg_grid).unwrap();

    let lines_minor = svg_minor.matches("<line").count();
    let lines_grid  = svg_grid.matches("<line").count();
    assert!(lines_grid > lines_minor,
        "minor grid should add even more line elements ({} vs {})", lines_grid, lines_minor);
}
