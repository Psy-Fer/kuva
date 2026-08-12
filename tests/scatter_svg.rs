mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::scatter::{MarkerShape, ScatterPlot, TrendLine};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::{render_multiple, render_scatter};
use rand::Rng;

#[test]
fn test_scatter_svg_output_builder() {
    let data = vec![(1.0, 5.0), (4.5, 3.5), (5.0, 8.7)];

    let plot = ScatterPlot::new()
        .with_data(data)
        .with_color("blue")
        .with_size(5.0);

    let layout = Layout::new((0.0, 10.0), (0.0, 40.0))
        .with_title("Scatter Builder Plot")
        .with_x_label("The X axis")
        .with_y_label("The Y axis");

    let scene = render_scatter(&plot, layout).with_background(Some("white"));
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/scatter_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
}

#[test]
fn test_scatter_svg_output_layout() {
    let data = vec![(1.0, 5.0), (4.5, 3.5), (5.0, 8.7)];

    let plot = ScatterPlot::new()
        .with_data(data)
        .with_color("purple")
        .with_size(3.0);

    let layout = Layout::new((0.0, 11.0), (0.0, 10.0))
        .with_title("Scatter Layout Plot")
        .with_x_label("The X axis")
        .with_y_label("The Y axis");

    let scene = render_scatter(&plot, layout).with_background(Some("white"));
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/scatter_layout.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
}

#[test]
fn test_scatter_trend_svg() {
    // Generate some noisy linear data: y = 2x + 1 + noise
    let mut rng = rand::rng();
    let data: Vec<(f64, f64)> = (1..49)
        .map(|i| {
            let x = i as f64 * 0.2;
            let noise: f64 = rng.random_range(-1.0..1.0);
            let y = 0.5 * x + 1.0 + noise;
            (x, y)
        })
        .collect();

    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("blue")
        .with_trend(TrendLine::Linear)
        .with_equation()
        .with_correlation();

    let plot = vec![Plot::Scatter(scatter)];

    let layout = Layout::auto_from_plots(&plot)
        .with_title("Scatter with trend")
        .with_x_label("The X axis")
        .with_y_label("The Y axis");

    let scene = render_multiple(plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/scatter_trend_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
}

#[test]
fn test_scatter_trend_error_svg() {
    let data = vec![(1.5, 2), (2.0, 3), (3.0, 5), (4.0, 6)];
    let x_err = vec![0.1, 0.05, 0.2, 0.3];
    let y_err = vec![(1, 1), (1, 1), (1, 1), (1, 1)];

    let scatter = ScatterPlot::new()
        .with_data(data) // i32 -> f64 input test
        .with_x_err(x_err)
        .with_y_err_asymmetric(y_err)
        .with_color("red")
        .with_trend(TrendLine::Linear)
        .with_equation()
        .with_correlation();

    let plot = vec![Plot::Scatter(scatter)];

    let layout = Layout::auto_from_plots(&plot)
        .with_title("Scatter with trend + error")
        .with_x_label("The X axis")
        .with_y_label("The Y axis");

    let scene = render_multiple(plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/scatter_trend_error_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
}

#[test]
fn test_scatter_log_scale() {
    // Data spanning several orders of magnitude
    let data: Vec<(f64, f64)> = vec![
        (1.0, 0.001),
        (5.0, 0.01),
        (10.0, 0.1),
        (50.0, 1.0),
        (100.0, 10.0),
        (500.0, 100.0),
        (1000.0, 1000.0),
        (5000.0, 5000.0),
        (10000.0, 10000.0),
    ];

    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("teal")
        .with_size(5.0);

    let plot = vec![Plot::Scatter(scatter)];

    let layout = Layout::auto_from_plots(&plot)
        .with_log_scale()
        .with_title("Log-Scale Scatter")
        .with_x_label("X (log)")
        .with_y_label("Y (log)");

    let scene = render_multiple(plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/scatter_log.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
}

#[test]
fn test_scatter_markers() {
    let circle = ScatterPlot::new()
        .with_data(vec![(1.0, 1.0), (2.0, 2.0), (3.0, 1.5)])
        .with_color("blue")
        .with_size(5.0)
        .with_marker(MarkerShape::Circle)
        .with_legend("Circle");

    let square = ScatterPlot::new()
        .with_data(vec![(1.0, 3.0), (2.0, 4.0), (3.0, 3.5)])
        .with_color("red")
        .with_size(5.0)
        .with_marker(MarkerShape::Square)
        .with_legend("Square");

    let triangle = ScatterPlot::new()
        .with_data(vec![(1.0, 5.0), (2.0, 6.0), (3.0, 5.5)])
        .with_color("green")
        .with_size(5.0)
        .with_marker(MarkerShape::Triangle)
        .with_legend("Triangle");

    let diamond = ScatterPlot::new()
        .with_data(vec![(1.0, 7.0), (2.0, 8.0), (3.0, 7.5)])
        .with_color("purple")
        .with_size(5.0)
        .with_marker(MarkerShape::Diamond)
        .with_legend("Diamond");

    let plots = vec![
        Plot::Scatter(circle),
        Plot::Scatter(square),
        Plot::Scatter(triangle),
        Plot::Scatter(diamond),
    ];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Marker Shapes")
        .with_x_label("X")
        .with_y_label("Y");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/scatter_markers.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle")); // Circle markers
    assert!(svg.contains("<rect")); // Square markers
}

#[test]
fn test_bubble_plot() {
    let data: Vec<(f64, f64)> = vec![
        (1.0, 2.0),
        (2.0, 3.0),
        (3.0, 5.0),
        (4.0, 4.0),
        (5.0, 6.0),
        (6.0, 3.0),
    ];
    let sizes = vec![3.0, 6.0, 10.0, 4.0, 8.0, 12.0];

    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_sizes(sizes);

    let plot = vec![Plot::Scatter(scatter)];

    let layout = Layout::auto_from_plots(&plot)
        .with_title("Bubble Plot")
        .with_x_label("X")
        .with_y_label("Y");

    let scene = render_multiple(plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/bubble_plot.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
    // Verify varying radii: the smallest (r=3) and largest (r=12) should both appear
    assert!(svg.contains(r#"r="3""#));
    assert!(svg.contains(r#"r="12""#));
}

#[test]
fn test_scatter_log_x_only() {
    // Log X with linear Y — e.g. dose-response
    let data: Vec<(f64, f64)> = vec![
        (0.01, 5.0),
        (0.1, 12.0),
        (1.0, 45.0),
        (10.0, 78.0),
        (100.0, 95.0),
        (1000.0, 99.0),
    ];

    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("darkred")
        .with_size(5.0);

    let plot = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plot)
        .with_log_x()
        .with_title("Log X / Linear Y")
        .with_x_label("Concentration")
        .with_y_label("Response (%)");

    let scene = render_multiple(plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/scatter_log_x.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
    // X ticks should be log formatted (powers of 10)
    assert!(svg.contains(">1<") || svg.contains(">1</text>"));
    assert!(svg.contains(">100<") || svg.contains(">100</text>"));
    // Y ticks should be linear (integer values in this range)
    assert!(svg.contains(">0</text>") || svg.contains(">20</text>") || svg.contains(">40</text>"));
}

#[test]
fn test_scatter_log_y_only() {
    // Linear X with log Y — e.g. exponential growth
    let data: Vec<(f64, f64)> = vec![
        (1.0, 2.0),
        (2.0, 8.0),
        (3.0, 30.0),
        (4.0, 120.0),
        (5.0, 500.0),
        (6.0, 2000.0),
        (7.0, 8000.0),
    ];

    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("navy")
        .with_size(5.0);

    let plot = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plot)
        .with_log_y()
        .with_title("Linear X / Log Y")
        .with_x_label("Time")
        .with_y_label("Population");

    let scene = render_multiple(plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/scatter_log_y.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
    // Y ticks should be log formatted
    assert!(svg.contains(">1</text>"));
    assert!(svg.contains(">1000</text>"));
}

#[test]
fn test_scatter_log_small_values() {
    // Very small values (sub-unity range)
    let data: Vec<(f64, f64)> = vec![
        (0.001, 0.0001),
        (0.005, 0.0008),
        (0.02, 0.003),
        (0.1, 0.015),
        (0.5, 0.08),
        (1.0, 0.5),
    ];

    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("purple")
        .with_size(5.0);

    let plot = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plot)
        .with_log_scale()
        .with_title("Log Scale — Small Values")
        .with_x_label("X")
        .with_y_label("Y");

    let scene = render_multiple(plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/scatter_log_small.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
    // Should have sub-unity tick labels
    assert!(svg.contains("1e-"));
}

#[test]
fn test_scatter_per_point_colors() {
    let data: Vec<(f64, f64)> = vec![
        (1.0, 1.0),
        (2.0, 2.0),
        (3.0, 3.0),
        (4.0, 4.0),
        (5.0, 5.0),
        (6.0, 6.0),
    ];
    let colors = vec!["red", "green", "blue", "red", "green", "blue"];

    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("black")
        .with_colors(colors)
        .with_size(6.0);

    let plot = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plot)
        .with_title("Per-Point Colors")
        .with_x_label("X")
        .with_y_label("Y");

    let scene = render_multiple(plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/scatter_per_point_colors.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains(r##"fill="#ff0000""##));
    assert!(svg.contains(r##"fill="#008000""##));
    assert!(svg.contains(r##"fill="#0000ff""##));
}

#[test]
fn test_scatter_log_narrow_range() {
    // Narrow range (< 3 decades) should show 2x and 5x sub-ticks
    let data: Vec<(f64, f64)> = vec![
        (5.0, 10.0),
        (10.0, 25.0),
        (20.0, 50.0),
        (50.0, 80.0),
        (100.0, 200.0),
    ];

    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("teal")
        .with_size(6.0);

    let plot = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plot)
        .with_log_scale()
        .with_title("Log Scale — Narrow Range (sub-ticks)")
        .with_x_label("X")
        .with_y_label("Y");

    let scene = render_multiple(plot, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/scatter_log_narrow.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // Narrow range should have 2x/5x sub-ticks (e.g. "20", "50")
    assert!(svg.contains(">20</text>") || svg.contains(">50</text>"));
}

/// Empty ScatterPlot should not panic and should produce valid SVG,
/// whether used alone or mixed with a populated series.
#[test]
fn test_scatter_empty_data() {
    let empty: Vec<(f64, f64)> = vec![];

    // Alone: auto_from_plots skips empty plot (bounds returns None),
    // render_multiple should still produce a valid SVG frame.
    let plots = vec![Plot::Scatter(ScatterPlot::new().with_data(empty.clone()))];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/scatter_empty.svg", &svg).unwrap();
    assert!(
        svg.contains("<svg"),
        "empty scatter should still produce SVG"
    );

    // Mixed: one empty series + one with data — only the populated series
    // should determine the axis range, and both should render without panic.
    let populated = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
        .with_color("steelblue")
        .with_legend("data");
    let plots2 = vec![
        Plot::Scatter(ScatterPlot::new().with_data(empty)),
        Plot::Scatter(populated),
    ];
    let layout2 = Layout::auto_from_plots(&plots2);
    let svg2 = SvgBackend.render_scene(&render_multiple(plots2, layout2));
    common::write_test_output("test_outputs/scatter_empty_mixed.svg", &svg2).unwrap();
    assert!(
        svg2.contains("<svg"),
        "mixed empty+populated should produce SVG"
    );
    assert!(
        svg2.contains("data"),
        "populated series legend should appear"
    );
}

// ── LOESS smoother ───────────────────────────────────────────────────────────

/// LOESS smoother draws a multi-segment stroked path (not the single straight
/// line a linear trend produces). Written to test_outputs/ for visual inspection.
#[test]
fn test_scatter_loess_smoother() {
    // Noisy sine so the smoother has a curve to follow.
    let data: Vec<(f64, f64)> = (0..120)
        .map(|i| {
            let x = i as f64 / 120.0 * 10.0;
            let noise = ((i as f64 * 12.9898).sin() * 43758.5453).fract() - 0.5;
            (x, x.sin() + 0.6 * noise)
        })
        .collect();
    let plot = ScatterPlot::new()
        .with_data(data)
        .with_size(3.0)
        .with_color("#4e79a7")
        .with_loess()
        .with_trend_color("#e15759");

    let layout = Layout::auto_from_plots(&[Plot::Scatter(plot.clone())]).with_title("LOESS");
    let svg = SvgBackend.render_scene(&render_multiple(vec![Plot::Scatter(plot)], layout));
    common::write_test_output("test_outputs/scatter_loess.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // The smoother is a stroked, unfilled polyline in the trend colour.
    assert!(
        svg.contains("#e15759") && svg.contains("fill=\"none\""),
        "loess should draw a stroked path in the trend colour"
    );
    // A LOESS path has many segments; a linear trend would have a single <line>.
    let loess_path = svg
        .match_indices("<path")
        .map(|(i, _)| {
            let end = svg[i..].find('>').unwrap() + i;
            &svg[i..=end]
        })
        .find(|t| t.contains("#e15759"))
        .expect("loess path present");
    assert!(
        loess_path.matches(" L").count() > 20,
        "loess path should be a many-segment polyline"
    );
}

/// A smaller span produces a wigglier curve (more total vertical variation) than
/// a larger span on the same data.
#[test]
fn test_scatter_loess_span_controls_smoothness() {
    use kuva::render::render_utils::loess;
    let data: Vec<(f64, f64)> = (0..120)
        .map(|i| {
            let x = i as f64 / 120.0 * 10.0;
            let noise = ((i as f64 * 78.233).sin() * 43758.5453).fract() - 0.5;
            (x, x.sin() + 0.7 * noise)
        })
        .collect();
    let variation = |span: f64| -> f64 {
        loess(data.iter().copied(), span, 100)
            .windows(2)
            .map(|w| (w[1].1 - w[0].1).abs())
            .sum::<f64>()
    };
    assert!(
        variation(0.1) > variation(0.6),
        "smaller span should wiggle more than a larger span"
    );
}

// ── Point labels + repel ─────────────────────────────────────────────────────

/// Per-point labels with force-directed (Repel) placement draw the label text plus
/// thin leader lines back to clustered points. Written to test_outputs/ for review.
#[test]
fn test_scatter_repel_labels() {
    use kuva::plot::LabelStyle;
    // Two tight clusters so repulsion has to move labels off each other.
    let pts = [
        (1.0, 2.0, "Alpha"),
        (1.1, 2.1, "Beta"),
        (1.05, 1.95, "Gamma"),
        (4.0, 5.0, "Delta"),
        (4.1, 4.9, "Epsilon"),
        (3.95, 5.05, "Zeta"),
    ];
    let plot = ScatterPlot::new()
        .with_data(pts.iter().map(|(x, y, _)| (*x, *y)).collect::<Vec<_>>())
        .with_size(5.0)
        .with_color("#4e79a7")
        .with_labels(
            pts.iter()
                .map(|(_, _, l)| l.to_string())
                .collect::<Vec<_>>(),
        )
        .with_label_style(LabelStyle::Repel);

    let layout = Layout::auto_from_plots(&[Plot::Scatter(plot.clone())]).with_title("Repel labels");
    let svg = SvgBackend.render_scene(&render_multiple(vec![Plot::Scatter(plot)], layout));
    common::write_test_output("test_outputs/scatter_repel_labels.svg", &svg).unwrap();

    for name in ["Alpha", "Delta", "Zeta"] {
        assert!(svg.contains(&format!(">{name}<")), "label {name} missing");
    }
    // Leader lines are thin gray strokes.
    assert!(
        svg.contains("stroke=\"#888888\""),
        "repel should draw leader lines"
    );
}

/// Empty label strings leave their points unlabelled.
#[test]
fn test_scatter_labels_skip_empty() {
    let plot = ScatterPlot::new()
        .with_data(vec![(1.0, 1.0), (2.0, 2.0), (3.0, 3.0)])
        .with_labels(vec!["keep", "", "alsokeep"]);
    let layout = Layout::auto_from_plots(&[Plot::Scatter(plot.clone())]);
    let svg = SvgBackend.render_scene(&render_multiple(vec![Plot::Scatter(plot)], layout));
    common::write_test_output("test_outputs/scatter_labels_partial.svg", &svg).unwrap();
    assert!(svg.contains(">keep<") && svg.contains(">alsokeep<"));
}
