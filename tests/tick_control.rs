mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::{LinePlot, ScatterPlot};
use kuva::render::{
    layout::{ComputedLayout, Layout},
    plots::Plot,
    render::render_multiple,
};

fn scatter_svg(layout: Layout) -> String {
    // Simple scatter: two points giving x in [0,13], y in [0,5]
    let plot = ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (13.0, 5.0)]);
    let plots = vec![Plot::Scatter(plot)];
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

/// Axis range override: x capped at 10 should suppress auto-tick "15".
#[test]
fn test_axis_range_override() {
    // Without override, auto-nice_range on [0, 13.13] produces ticks up to 15.
    let plots = vec![Plot::Scatter(
        ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (13.0, 5.0)]),
    )];
    let layout_auto = Layout::auto_from_plots(&plots);
    let svg_auto = SvgBackend.render_scene(&render_multiple(plots, layout_auto));
    assert!(svg_auto.contains("15"), "auto range should include tick 15");

    // With override, x stops at 10.
    let plots2 = vec![Plot::Scatter(
        ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (13.0, 5.0)]),
    )];
    let layout_override = Layout::auto_from_plots(&plots2)
        .with_x_axis_min(0.0)
        .with_x_axis_max(10.0);
    let svg_override = scatter_svg(layout_override);
    common::write_test_output("test_outputs/tick_control_range.svg", &svg_override).unwrap();
    assert!(
        svg_override.contains("10"),
        "overridden range should include tick 10"
    );
    assert!(
        !svg_override.contains(">15<"),
        "overridden range should not show tick 15"
    );
}

/// Explicit tick step: with_x_tick_step(2.5) on [0,10] produces 0, 2.5, 5, 7.5, 10.
#[test]
fn test_explicit_tick_step() {
    let plots = vec![Plot::Scatter(
        ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]),
    )];
    let layout = Layout::auto_from_plots(&plots)
        .with_x_axis_min(0.0)
        .with_x_axis_max(10.0)
        .with_x_tick_step(2.5);
    let svg = scatter_svg(layout);
    common::write_test_output("test_outputs/tick_control_step.svg", &svg).unwrap();
    assert!(
        svg.contains(">0<") || svg.contains(">0.0<") || svg.contains("\"0\"") || svg.contains(">0"),
        "tick 0 should appear"
    );
    assert!(svg.contains("2.5"), "tick 2.5 should appear");
    assert!(
        svg.contains(">5<") || svg.contains("5.0") || svg.contains(">5"),
        "tick 5 should appear"
    );
    assert!(svg.contains("7.5"), "tick 7.5 should appear");
    assert!(
        svg.contains(">10<") || svg.contains(">10"),
        "tick 10 should appear"
    );
}

/// Minor ticks: enabling minor_ticks=5 adds more line elements to the SVG.
#[test]
fn test_minor_ticks() {
    let plots = vec![Plot::Scatter(
        ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]),
    )];
    let layout_no_minor = Layout::auto_from_plots(&plots);
    let svg_no_minor = scatter_svg(layout_no_minor);

    let plots2 = vec![Plot::Scatter(
        ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]),
    )];
    let layout_minor = Layout::auto_from_plots(&plots2).with_minor_ticks(5);
    let svg_minor = scatter_svg(layout_minor);
    common::write_test_output("test_outputs/tick_control_minor.svg", &svg_minor).unwrap();

    let lines_without = svg_no_minor.matches("<line").count();
    let lines_with = svg_minor.matches("<line").count();
    assert!(
        lines_with > lines_without,
        "minor ticks should add more line elements ({} vs {})",
        lines_with,
        lines_without
    );
}

/// Minor grid: enabling show_minor_grid adds even more line elements.
#[test]
fn test_minor_grid() {
    let plots = vec![Plot::Scatter(
        ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]),
    )];
    let layout_minor = Layout::auto_from_plots(&plots).with_minor_ticks(5);
    let svg_minor = scatter_svg(layout_minor);

    let plots2 = vec![Plot::Scatter(
        ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]),
    )];
    let layout_grid = Layout::auto_from_plots(&plots2)
        .with_minor_ticks(5)
        .with_show_minor_grid(true);
    let svg_grid = scatter_svg(layout_grid);
    common::write_test_output("test_outputs/tick_control_minor_grid.svg", &svg_grid).unwrap();

    let lines_minor = svg_minor.matches("<line").count();
    let lines_grid = svg_grid.matches("<line").count();
    assert!(
        lines_grid > lines_minor,
        "minor grid should add even more line elements ({} vs {})",
        lines_grid,
        lines_minor
    );
}

/// Minor grid lines (half the grid width) as (vertical, horizontal) counts.
fn minor_grid_lines(svg: &str) -> (usize, usize) {
    let attr = |tag: &str, name: &str| -> f64 {
        let key = format!("{name}=\"");
        let start = tag.find(&key).unwrap() + key.len();
        tag[start..].split('"').next().unwrap().parse().unwrap()
    };
    let mut counts = (0, 0);
    for tag in svg.split("<line").skip(1) {
        let tag = tag.split('>').next().unwrap();
        if !tag.contains("stroke-width=\"0.5\"") {
            continue;
        }
        if attr(tag, "x1") == attr(tag, "x2") {
            counts.0 += 1;
        } else if attr(tag, "y1") == attr(tag, "y2") {
            counts.1 += 1;
        }
    }
    counts
}

/// Major grid lines (default `#cccccc` at `stroke-width="1"`) as (vertical, horizontal) counts.
fn major_grid_lines(svg: &str) -> (usize, usize) {
    let attr = |tag: &str, name: &str| -> f64 {
        let key = format!("{name}=\"");
        let start = tag.find(&key).unwrap() + key.len();
        tag[start..].split('"').next().unwrap().parse().unwrap()
    };
    let mut counts = (0, 0);
    for tag in svg.split("<line").skip(1) {
        let tag = tag.split('>').next().unwrap();
        if !tag.contains("stroke=\"#cccccc\"") || !tag.contains("stroke-width=\"1\"") {
            continue;
        }
        if attr(tag, "x1") == attr(tag, "x2") {
            counts.0 += 1;
        } else if attr(tag, "y1") == attr(tag, "y2") {
            counts.1 += 1;
        }
    }
    counts
}

/// #142: minor ticks (and so minor grid lines) on one axis only, and an axis's
/// own count overriding the shared one.
#[test]
fn test_minor_ticks_per_axis() {
    let base = || {
        let plots = vec![Plot::Scatter(
            ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]),
        )];
        Layout::auto_from_plots(&plots).with_show_minor_grid(true)
    };
    let x_only = scatter_svg(base().with_x_minor_ticks(5));
    common::write_test_output("test_outputs/tick_control_minor_x_only.svg", &x_only).unwrap();
    let (v, h) = minor_grid_lines(&x_only);
    assert!(v > 0, "x minor grid lines expected");
    assert_eq!(h, 0, "no y minor grid lines expected");

    let (v, h) = minor_grid_lines(&scatter_svg(base().with_y_minor_ticks(5)));
    assert!(h > 0);
    assert_eq!(v, 0);

    // Per-axis count wins over the shared one.
    let (v_shared, h_shared) = minor_grid_lines(&scatter_svg(base().with_minor_ticks(5)));
    let (v_over, h_over) = minor_grid_lines(&scatter_svg(
        base().with_minor_ticks(5).with_x_minor_ticks(2),
    ));
    assert_eq!(h_over, h_shared, "y keeps the shared count");
    assert!(v_over < v_shared, "x uses its own, smaller count");
}

/// Text of every `<text>` element, in order.
fn texts(svg: &str) -> Vec<String> {
    svg.split("<text")
        .skip(1)
        .filter_map(|t| {
            t.split_once('>')
                .map(|(_, rest)| rest.split('<').next().unwrap().to_string())
        })
        .collect()
}

/// #137: explicit major tick positions, here an audio spectrum on a log axis
/// past three decades (which otherwise ticks at powers of ten only).
#[test]
fn test_explicit_tick_values_on_a_log_axis() {
    let data: Vec<(f64, f64)> = (0..200)
        .map(|i| {
            let f = 20.0 * 1000f64.powf(i as f64 / 199.0);
            (f, -60.0 - 10.0 * (f / 1000.0).log10())
        })
        .collect();
    let plots = vec![Plot::Line(LinePlot::new().with_data(data))];
    let ticks = [
        20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0, 20000.0,
    ];
    let layout = Layout::auto_from_plots(&plots)
        .with_log_x()
        .with_x_axis_min(20.0)
        .with_x_axis_max(20000.0)
        // Outside the range, and not positive on a log axis: skipped.
        .with_x_tick_values(ticks.iter().copied().chain([0.0, -5.0, 1e6]));
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/tick_control_explicit_log.svg", &svg).unwrap();
    let t = texts(&svg);
    for v in ticks {
        assert!(t.contains(&format!("{v}")), "tick label {v} missing: {t:?}");
    }
    for skipped in ["0", "-5", "1000000"] {
        assert!(
            !t.contains(&skipped.to_string()),
            "{skipped} should be skipped"
        );
    }
}

/// With hand-picked log ticks, minor ticks still follow the decades (2 to 9
/// times each power of ten), not linear steps between the given ticks.
#[test]
fn test_explicit_log_ticks_keep_decade_minors() {
    let plots = vec![Plot::Line(
        LinePlot::new().with_data(vec![(20.0, 0.0), (20000.0, 1.0)]),
    )];
    let layout = Layout::auto_from_plots(&plots)
        .with_log_x()
        .with_x_axis_min(20.0)
        .with_x_axis_max(20000.0)
        .with_x_tick_values([20.0, 50.0, 100.0, 1000.0, 2000.0, 20000.0])
        .with_x_minor_ticks(9)
        .with_show_minor_grid(true);
    let computed = ComputedLayout::from_layout(&layout);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    // Vertical minor grid lines (half-width strokes), as x positions.
    let minor_xs: Vec<f64> = svg
        .split("<line")
        .skip(1)
        .map(|t| t.split('>').next().unwrap())
        .filter(|t| t.contains("stroke-width=\"0.5\""))
        .filter_map(|t| {
            let x1 = t
                .split(" x1=\"")
                .nth(1)?
                .split('"')
                .next()?
                .parse::<f64>()
                .ok()?;
            let x2 = t
                .split(" x2=\"")
                .nth(1)?
                .split('"')
                .next()?
                .parse::<f64>()
                .ok()?;
            (x1 == x2).then_some(x1)
        })
        .collect();
    let has_minor_at = |v: f64| {
        minor_xs
            .iter()
            .any(|x| (x - computed.map_x(v)).abs() < 0.05)
    };
    for v in [30.0, 40.0, 300.0, 3000.0, 9000.0] {
        assert!(has_minor_at(v), "decade minor at {v} expected");
    }
    assert!(
        !has_minor_at(1000.0 + 1000.0 / 9.0),
        "no linear step between 1000 and 2000"
    );
}

/// Explicit y ticks on a linear axis replace the automatic ones.
#[test]
fn test_explicit_tick_values_linear_y() {
    let layout = Layout::auto_from_plots(&[Plot::Scatter(
        ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (13.0, 5.0)]),
    )])
    .with_y_tick_values([0.5, 2.5, 4.5]);
    let svg = scatter_svg(layout);
    // y tick labels are the right-anchored texts.
    let y_labels: Vec<String> = svg
        .split("<text")
        .skip(1)
        .filter(|t| t.split('>').next().unwrap().contains("text-anchor=\"end\""))
        .map(|t| {
            t.split_once('>')
                .unwrap()
                .1
                .split('<')
                .next()
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(y_labels, ["0.5", "2.5", "4.5"]);
}

/// Axis line width: with_axis_line_width(4.0) → stroke-width="4" on both axis lines.
#[test]
fn test_axis_line_width() {
    let plots = vec![Plot::Scatter(
        ScatterPlot::new().with_data(vec![(0.0f64, 0.0f64), (10.0, 5.0)]),
    )];
    let layout = Layout::auto_from_plots(&plots).with_axis_line_width(4.0);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/axis_line_width.svg", &svg).unwrap();
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
    common::write_test_output("test_outputs/tick_stroke_width.svg", &svg).unwrap();
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
    common::write_test_output("test_outputs/tick_length.svg", &svg_long).unwrap();

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
    common::write_test_output("test_outputs/grid_line_width.svg", &svg).unwrap();
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
    common::write_test_output("test_outputs/tick_grid_independent.svg", &svg_both).unwrap();
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
    let svg_tick_only =
        SvgBackend.render_scene(&render_multiple(plots_tick_only, layout_tick_only));
    let count_tick_only = svg_tick_only.matches(r#"stroke-width="3""#).count();

    let plots_tick_and_grid = make_plots();
    let layout_tick_and_grid = Layout::auto_from_plots(&plots_tick_and_grid)
        .with_tick_width(3.0)
        .with_grid_line_width(3.0);
    let svg_tick_and_grid =
        SvgBackend.render_scene(&render_multiple(plots_tick_and_grid, layout_tick_and_grid));
    let count_tick_and_grid = svg_tick_and_grid.matches(r#"stroke-width="3""#).count();

    assert!(
        count_tick_and_grid > count_tick_only,
        "adding grid_line_width=3 on top of tick_width=3 should increase the stroke-width=\"3\" \
         count (grid lines join in); got tick_only={} vs tick+grid={}",
        count_tick_only,
        count_tick_and_grid
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
    common::write_test_output("test_outputs/axis_controls_combined.svg", &svg).unwrap();
    assert!(
        svg.contains(r#"stroke-width="5""#),
        "axis line width=5 should appear"
    );
    assert!(
        svg.contains(r#"stroke-width="2.5""#),
        "tick width=2.5 should appear"
    );
    assert!(
        svg.contains(r#"stroke-width="0.5""#),
        "grid line width=0.5 should appear"
    );
}

/// Regression test: tick labels must never contain "-0".
/// IEEE 754 negative zero (-0.0) formats as "-0" with Rust's {:.0} formatter.
/// TickFormat::format() must normalise -0.0 → 0.0 before dispatching.
#[test]
fn test_no_negative_zero_tick_label() {
    use kuva::render::layout::TickFormat;
    // The direct formatter must not produce "-0"
    assert_ne!(TickFormat::Auto.format(-0.0_f64), "-0");
    assert_ne!(TickFormat::Integer.format(-0.0_f64), "-0");
    assert_ne!(TickFormat::Fixed(1).format(-0.0_f64), "-0.0");
    assert_ne!(TickFormat::Percent.format(-0.0_f64), "-0.0%");

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
    common::write_test_output("test_outputs/tick_no_negative_zero.svg", &svg).unwrap();
    assert!(
        !svg.contains(">-0<"),
        "SVG must not contain a '-0' tick label"
    );
}

/// Regression #80: generate_ticks with tiny values (1e-14 scale) must produce a
/// bounded tick count. The old code used `end + 1e-8` as a loop termination
/// tolerance; when the entire data range is smaller than 1e-8, the loop ran for
/// millions of iterations, producing gigabyte-sized SVG output.
#[test]
fn test_generate_ticks_small_scale_bounded() {
    let ticks = kuva::render::render_utils::generate_ticks(1e-14, 2e-14, 10);
    assert!(
        ticks.len() <= 20,
        "generate_ticks([1e-14, 2e-14], 10) should produce ≤20 ticks, got {}",
        ticks.len()
    );
    assert!(!ticks.is_empty(), "should produce at least one tick");
    // All tick values must be within the axis range
    for &t in &ticks {
        assert!(
            (1e-14 * 0.999..=2e-14 * 1.001).contains(&t),
            "tick {t} is outside expected range [1e-14, 2e-14]"
        );
    }
}

/// Regression #80: A LinePlot with very small y-values must produce a compact SVG,
/// not a ~920 MB file.
#[test]
fn test_line_plot_small_values_svg_size() {
    let plot = LinePlot::new().with_data(vec![(0.0_f64, 1.0e-14), (0.5, 2.0e-14), (1.0, 1.5e-14)]);
    let plots = vec![Plot::Line(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Small Values")
        .with_x_label("X")
        .with_y_label("Y");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/tick_small_scale_line.svg", &svg).unwrap();
    assert!(
        svg.len() < 100_000,
        "LinePlot with 1e-14 y-values should produce a small SVG (<100 KB), got {} bytes",
        svg.len()
    );
}

/// Regression (issue #98): data whose max already lands exactly on a "nice"
/// tick must not have the axis rounded out to a whole extra major tick just
/// because of the small breathing-room pad — that can inflate the range by
/// 15-25%+ for an axis with few, large-value ticks. Data 0..20 (step-5 grid)
/// used to round out to (0, 25); it must now stay within 5% of (0, 20).
#[test]
fn test_axis_range_does_not_over_provision_when_max_lands_on_a_tick() {
    let data: Vec<(f64, f64)> = (0..=20).map(|i| (i as f64, (i as f64).sin())).collect();
    let plot = LinePlot::new().with_data(data);
    let plots = vec![Plot::Line(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let computed = ComputedLayout::from_layout(&layout);

    assert_eq!(computed.x_range.0, 0.0);
    assert!(
        computed.x_range.1 <= 21.0 + 1e-9,
        "x_range.1 should be capped near 21.0 (20 + 5% of the 20-unit span), got {}",
        computed.x_range.1
    );
    assert!(
        computed.x_range.1 < 25.0,
        "x_range.1 must not round out to a whole extra major tick (25.0), got {}",
        computed.x_range.1
    );

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/tick_control_no_overprovision.svg", &svg).unwrap();
    assert!(
        !svg.contains(">25<"),
        "the dead-space tick from the old over-provisioned range must not appear"
    );
}

/// Regression (issue #98): when the data's max does NOT land on a nice tick,
/// ordinary nice-rounding already provides natural headroom and must be
/// completely unaffected by the issue #98 cap.
#[test]
fn test_axis_range_natural_overshoot_is_unaffected() {
    let data: Vec<(f64, f64)> = (0..=17).map(|i| (i as f64, (i as f64).sin())).collect();
    let plot = LinePlot::new().with_data(data);
    let plots = vec![Plot::Line(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let computed = ComputedLayout::from_layout(&layout);

    assert_eq!(computed.x_range, (0.0, 17.5));
}

#[test]
fn test_categorical_minor_ticks_and_grid_vertical_and_horizontal() {
    use kuva::plot::BarPlot;

    // Vertical categorical bar plot: Y minor ticks and Y minor gridlines render;
    // X (categorical) gets no minor gridlines.
    let make_v = || {
        vec![Plot::Bar(BarPlot::new().with_bars(vec![
            ("A", 10.0),
            ("B", 20.0),
            ("C", 15.0),
        ]))]
    };
    let v_base_plots = make_v();
    let v_base_layout = Layout::auto_from_plots(&v_base_plots);
    let v_base = SvgBackend.render_scene(&render_multiple(v_base_plots, v_base_layout));
    let v_minor_plots = make_v();
    let v_minor_layout = Layout::auto_from_plots(&v_minor_plots)
        .with_y_minor_ticks(5)
        .with_show_minor_grid(true);
    let v_minor = SvgBackend.render_scene(&render_multiple(v_minor_plots, v_minor_layout));
    let (v_vert, v_horiz) = minor_grid_lines(&v_minor);
    assert_eq!(v_vert, 0, "vertical bar must not draw X minor gridlines");
    assert!(v_horiz > 0, "vertical bar must draw Y minor gridlines");
    assert!(
        v_minor.matches("<line ").count() > v_base.matches("<line ").count() + v_horiz,
        "vertical bar must also draw Y minor tick marks"
    );

    // Horizontal categorical bar plot: X minor ticks and X minor gridlines render;
    // Y (categorical) must not draw fractional-index minor gridlines even with with_minor_ticks(5).
    let make_h = || {
        vec![Plot::Bar(
            BarPlot::new()
                .with_bars(vec![("A", 10.0), ("B", 20.0), ("C", 15.0)])
                .with_horizontal(true),
        )]
    };
    let h_base_plots = make_h();
    let h_base_layout = Layout::auto_from_plots(&h_base_plots);
    let h_base = SvgBackend.render_scene(&render_multiple(h_base_plots, h_base_layout));
    let h_minor_plots = make_h();
    let h_minor_layout = Layout::auto_from_plots(&h_minor_plots)
        .with_minor_ticks(5)
        .with_show_minor_grid(true);
    let h_minor = SvgBackend.render_scene(&render_multiple(h_minor_plots, h_minor_layout));
    let (h_vert, h_horiz) = minor_grid_lines(&h_minor);
    assert!(h_vert > 0, "horizontal bar must draw X minor gridlines");
    assert_eq!(
        h_horiz, 0,
        "horizontal bar must not draw Y minor gridlines on categorical axis"
    );
    assert!(
        h_minor.matches("<line ").count() > h_base.matches("<line ").count() + h_vert,
        "horizontal bar must also draw X minor tick marks"
    );

    // Horizontal categorical plots must also draw major vertical gridlines
    // (no blank gaps at major ticks when show_grid + show_minor_grid are active).
    let (h_maj_vert, h_maj_horiz) = major_grid_lines(&h_base);
    assert!(
        h_maj_vert > 0,
        "horizontal bar must draw major vertical gridlines on its numeric X axis"
    );
    assert_eq!(
        h_maj_horiz, 0,
        "horizontal bar must not draw major horizontal gridlines on categorical Y axis"
    );

    // Non-bar categorical plot (BoxPlot, vertical + horizontal) gets the same treatment.
    use kuva::plot::BoxPlot;
    let box_plots = vec![Plot::Box(
        BoxPlot::new()
            .with_group("Ctrl", vec![1.0, 2.0, 3.0, 4.0, 5.0])
            .with_group("Treat", vec![2.0, 4.0, 6.0, 8.0, 10.0])
            .with_horizontal(true),
    )];
    let box_layout = Layout::auto_from_plots(&box_plots)
        .with_minor_ticks(5)
        .with_show_minor_grid(true);
    let box_svg = SvgBackend.render_scene(&render_multiple(box_plots, box_layout));
    let (b_min_v, b_min_h) = minor_grid_lines(&box_svg);
    let (b_maj_v, b_maj_h) = major_grid_lines(&box_svg);
    assert!(
        b_maj_v > 0 && b_min_v > 0,
        "horizontal box needs major+minor X grid"
    );
    assert_eq!(
        (b_maj_h, b_min_h),
        (0, 0),
        "horizontal box must not draw Y gridlines"
    );
}
