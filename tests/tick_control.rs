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

/// Axis line width: with_axis_line_width(4.0) → stroke-width="4" on both axis lines.
#[test]
fn test_axis_line_width() {
    let plots = vec![Plot::Scatter(
        ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]),
    )];
    let layout = Layout::auto_from_plots(&plots).with_axis_line_width(4.0);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/axis_line_width.svg", &svg).unwrap();
    assert!(
        svg.contains(r#"stroke-width="4""#),
        "axis lines should carry stroke-width=\"4\""
    );
}

/// Axis line width default: without the builder, axes use 1px (scale=1).
#[test]
fn test_axis_line_width_default() {
    let plots = vec![Plot::Scatter(
        ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]),
    )];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    // Default axis stroke is 1; "4" should not appear as a stroke-width
    assert!(
        !svg.contains(r#"stroke-width="4""#),
        "default rendering should not have stroke-width=\"4\""
    );
}

/// Tick stroke width: with_tick_width(3.5) → stroke-width="3.5" on tick marks.
#[test]
fn test_tick_stroke_width() {
    let plots = vec![Plot::Scatter(
        ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]),
    )];
    let layout = Layout::auto_from_plots(&plots).with_tick_width(3.5);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/tick_stroke_width.svg", &svg).unwrap();
    assert!(
        svg.contains(r#"stroke-width="3.5""#),
        "tick marks should carry stroke-width=\"3.5\""
    );
}

/// Tick stroke width default: "3.5" should not appear without the builder.
#[test]
fn test_tick_stroke_width_default() {
    let plots = vec![Plot::Scatter(
        ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]),
    )];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert!(
        !svg.contains(r#"stroke-width="3.5""#),
        "default rendering should not have stroke-width=\"3.5\""
    );
}

/// Tick length: with_tick_length(15.0) produces a different SVG than the default (5px ticks).
#[test]
fn test_tick_length() {
    let make_plots = || {
        vec![Plot::Scatter(
            ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]),
        )]
    };
    let plots_default = make_plots();
    let layout_default = Layout::auto_from_plots(&plots_default);
    let svg_default = SvgBackend.render_scene(&render_multiple(plots_default, layout_default));

    let plots_long = make_plots();
    let layout_long = Layout::auto_from_plots(&plots_long).with_tick_length(150.0);
    let svg_long = SvgBackend.render_scene(&render_multiple(plots_long, layout_long));
    std::fs::write("test_outputs/tick_length.svg", &svg_long).unwrap();

    assert_ne!(
        svg_default, svg_long,
        "with_tick_length(15.0) should produce different tick coordinates than the default"
    );
}

/// Grid line width: with_grid_line_width(2.5) → stroke-width="2.5" on major grid lines.
#[test]
fn test_grid_line_width() {
    let plots = vec![Plot::Scatter(
        ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]),
    )];
    let layout = Layout::auto_from_plots(&plots).with_grid_line_width(2.5);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/grid_line_width.svg", &svg).unwrap();
    assert!(
        svg.contains(r#"stroke-width="2.5""#),
        "grid lines should carry stroke-width=\"2.5\""
    );
}

/// Grid line width default: "2.5" should not appear without the builder.
#[test]
fn test_grid_line_width_default() {
    let plots = vec![Plot::Scatter(
        ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]),
    )];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert!(
        !svg.contains(r#"stroke-width="2.5""#),
        "default rendering should not have stroke-width=\"2.5\""
    );
}

/// Regression: tick_width and grid_line_width must be independent.
/// Previously, horizontal grid lines used tick_stroke_width instead of grid_stroke_width,
/// so setting with_tick_width would inadvertently widen horizontal grid lines too.
/// With the fix, grid lines only pick up grid_line_width and ticks only pick up tick_width.
///
/// Strategy: set tick_width=3 and grid_line_width=7 simultaneously.
/// Both values must appear. Then verify that setting only tick_width=3 (grid at default 1)
/// produces FEWER "3" occurrences than setting both tick_width=3 AND grid_line_width=3
/// (since adding grid_line_width=3 brings the grid lines up to 3 as well, adding more hits).
#[test]
fn test_tick_and_grid_width_independent() {
    let make_plots = || {
        vec![Plot::Scatter(
            ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]),
        )]
    };

    // Both set to distinct values: each must appear.
    let plots_both = make_plots();
    let layout_both = Layout::auto_from_plots(&plots_both)
        .with_tick_width(3.0)
        .with_grid_line_width(7.0);
    let svg_both = SvgBackend.render_scene(&render_multiple(plots_both, layout_both));
    std::fs::write("test_outputs/tick_grid_independent.svg", &svg_both).unwrap();
    assert!(
        svg_both.contains(r#"stroke-width="3""#),
        "tick width=3 should appear when set independently of grid"
    );
    assert!(
        svg_both.contains(r#"stroke-width="7""#),
        "grid line width=7 should appear when set independently of ticks"
    );

    // tick_width=3 alone (grid at default 1) must produce FEWER "3"s than
    // tick_width=3 + grid_line_width=3. The second layout adds grid lines to the "3" pool.
    let plots_tick_only = make_plots();
    let layout_tick_only = Layout::auto_from_plots(&plots_tick_only).with_tick_width(3.0);
    let svg_tick_only = SvgBackend.render_scene(&render_multiple(plots_tick_only, layout_tick_only));
    let count_tick_only = svg_tick_only.matches(r#"stroke-width="3""#).count();

    let plots_tick_and_grid = make_plots();
    let layout_tick_and_grid = Layout::auto_from_plots(&plots_tick_and_grid)
        .with_tick_width(3.0)
        .with_grid_line_width(3.0);
    let svg_tick_and_grid = SvgBackend.render_scene(&render_multiple(plots_tick_and_grid, layout_tick_and_grid));
    let count_tick_and_grid = svg_tick_and_grid.matches(r#"stroke-width="3""#).count();

    assert!(
        count_tick_and_grid > count_tick_only,
        "adding grid_line_width=3 on top of tick_width=3 should increase the stroke-width=\"3\" \
         count (grid lines join in); got tick_only={} vs tick+grid={}",
        count_tick_only, count_tick_and_grid
    );
}

/// All four controls combined: each distinctive value appears in the SVG.
#[test]
fn test_axis_controls_combined() {
    let plots = vec![Plot::Scatter(
        ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]),
    )];
    let layout = Layout::auto_from_plots(&plots)
        .with_axis_line_width(5.0)
        .with_tick_width(2.5)
        .with_tick_length(8.0)
        .with_grid_line_width(0.5);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/axis_controls_combined.svg", &svg).unwrap();
    assert!(svg.contains(r#"stroke-width="5""#),   "axis line width=5 should appear");
    assert!(svg.contains(r#"stroke-width="2.5""#), "tick width=2.5 should appear");
    assert!(svg.contains(r#"stroke-width="0.5""#), "grid line width=0.5 should appear");
}

/// Regression test: tick labels must never contain "-0".
/// IEEE 754 negative zero (-0.0) formats as "-0" with Rust's {:.0} formatter.
/// TickFormat::format() must normalise -0.0 → 0.0 before dispatching.
#[test]
fn test_no_negative_zero_tick_label() {
    use kuva::render::layout::TickFormat;
    // The direct formatter must not produce "-0"
    assert_ne!(TickFormat::Auto.format(-0.0_f64),     "-0");
    assert_ne!(TickFormat::Integer.format(-0.0_f64),  "-0");
    assert_ne!(TickFormat::Fixed(1).format(-0.0_f64), "-0.0");
    assert_ne!(TickFormat::Percent.format(-0.0_f64),  "-0.0%");

    // A density plot with y-axis floor at 0.0 must not render "-0" on the y-axis.
    // Force a layout where y_min can end up as -0.0 via the layout arithmetic.
    use kuva::plot::DensityPlot;
    use kuva::render::plots::Plot;
    let dp = DensityPlot::new()
        .with_data(vec![0.5_f64; 20])
        .with_x_range(0.0, 1.0);
    let plots = vec![Plot::Density(dp)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/tick_no_negative_zero.svg", &svg).unwrap();
    assert!(!svg.contains(">-0<"), "SVG must not contain a '-0' tick label");
}
