/// Tests for SVG plot-area clipping (#53).
///
/// When `with_y_axis_min` / `with_x_axis_min` (or max) clips data, points
/// that fall outside the axis limits must not render beyond the plot area.
/// The fix adds a `<clipPath>` rect and wraps all data elements in a
/// `<g clip-path="url(...)">` group.
///
/// Each test below saves its SVG to `test_outputs/` for visual inspection,
/// then asserts that a `<clipPath` element is present in the SVG.
/// Before the fix these assertions will FAIL; after the fix they will PASS.

use kuva::plot::LinePlot;
use kuva::plot::ScatterPlot;
use kuva::plot::bar::BarPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

// ── helpers ──────────────────────────────────────────────────────────────────

fn render(plots: Vec<Plot>, layout: Layout) -> String {
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

// ── issue #53 exact reproduction ─────────────────────────────────────────────

/// Exact code from issue #53: a line from (0,0)→(1,1) with y_axis_min=0.3.
/// The point at y=0.0 is below the axis floor; without clipping it renders
/// outside the plot area.
#[test]
fn test_clip_y_axis_min_line() {
    let plots = vec![Plot::Line(
        LinePlot::new().with_data([(0.0_f64, 0.0_f64), (1.0, 1.0)]),
    )];
    let layout = Layout::auto_from_plots(&plots)
        .with_x_label("x")
        .with_y_label("y")
        .with_y_axis_min(0.3);

    let svg = render(plots, layout);
    std::fs::write("test_outputs/clip_y_axis_min_line.svg", &svg).unwrap();

    assert!(svg.contains("<clipPath"), "SVG must contain a <clipPath> element");
    assert!(svg.contains("clip-path=\"url(#"), "SVG must have a clip-path reference");
}

/// Same setup but clamping the x axis: a line from (0,0)→(1,1) with
/// x_axis_min=0.5 clips the left portion of the line.
#[test]
fn test_clip_x_axis_min_line() {
    let plots = vec![Plot::Line(
        LinePlot::new().with_data([(0.0_f64, 0.0_f64), (1.0, 1.0)]),
    )];
    let layout = Layout::auto_from_plots(&plots)
        .with_x_label("x")
        .with_y_label("y")
        .with_x_axis_min(0.5);

    let svg = render(plots, layout);
    std::fs::write("test_outputs/clip_x_axis_min_line.svg", &svg).unwrap();

    assert!(svg.contains("<clipPath"), "SVG must contain a <clipPath> element");
    assert!(svg.contains("clip-path=\"url(#"), "SVG must have a clip-path reference");
}

/// y_axis_max clips the top: scatter points above the ceiling must not
/// render beyond the plot area.
#[test]
fn test_clip_y_axis_max_scatter() {
    // Points at y = 0, 5, 10, 15 — top two are above the axis ceiling of 8.
    let data: Vec<(f64, f64)> = (0..4).map(|i| (i as f64, i as f64 * 5.0)).collect();
    let plots = vec![Plot::Scatter(
        ScatterPlot::new().with_data(data),
    )];
    let layout = Layout::auto_from_plots(&plots)
        .with_y_axis_max(8.0);

    let svg = render(plots, layout);
    std::fs::write("test_outputs/clip_y_axis_max_scatter.svg", &svg).unwrap();

    assert!(svg.contains("<clipPath"), "SVG must contain a <clipPath> element");
    assert!(svg.contains("clip-path=\"url(#"), "SVG must have a clip-path reference");
}

/// Both axes clipped simultaneously.
#[test]
fn test_clip_both_axes() {
    let data: Vec<(f64, f64)> = vec![
        (-1.0, -1.0), (0.5, 0.5), (2.0, 2.0), (3.5, 3.5),
    ];
    let plots = vec![Plot::Scatter(
        ScatterPlot::new().with_data(data),
    )];
    let layout = Layout::auto_from_plots(&plots)
        .with_x_axis_min(0.0)
        .with_x_axis_max(3.0)
        .with_y_axis_min(0.0)
        .with_y_axis_max(3.0);

    let svg = render(plots, layout);
    std::fs::write("test_outputs/clip_both_axes.svg", &svg).unwrap();

    assert!(svg.contains("<clipPath"), "SVG must contain a <clipPath> element");
    assert!(svg.contains("clip-path=\"url(#"), "SVG must have a clip-path reference");
}

/// Clipping is applied even when no data actually overflows — the clip path
/// should always be present for axis-bearing plots so the SVG is consistent.
#[test]
fn test_clip_present_even_without_overflow() {
    let plots = vec![Plot::Line(
        LinePlot::new().with_data([(0.0_f64, 0.5_f64), (1.0, 0.8)]),
    )];
    // y_axis_min=0.0 — all data is above it, nothing actually overflows.
    let layout = Layout::auto_from_plots(&plots)
        .with_y_axis_min(0.0);

    let svg = render(plots, layout);
    std::fs::write("test_outputs/clip_no_overflow.svg", &svg).unwrap();

    assert!(svg.contains("<clipPath"), "SVG must contain a <clipPath> element");
}

/// Bar plot with a manual y_axis_max that clips the tallest bar.
#[test]
fn test_clip_bar_y_axis_max() {
    let plots = vec![Plot::Bar(
        BarPlot::new().with_bars(vec![
            ("A".to_string(), 2.0_f64),
            ("B".to_string(), 8.0),
            ("C".to_string(), 5.0),
        ]),
    )];
    // Cap at 6 — bar B extends to 8 and should be clipped.
    let layout = Layout::auto_from_plots(&plots)
        .with_y_axis_max(6.0);

    let svg = render(plots, layout);
    std::fs::write("test_outputs/clip_bar_y_axis_max.svg", &svg).unwrap();

    assert!(svg.contains("<clipPath"), "SVG must contain a <clipPath> element");
    assert!(svg.contains("clip-path=\"url(#"), "SVG must have a clip-path reference");
}

/// Pixel-space plots (Pie) must NOT get a clip path — they don't use axis
/// coordinate mapping and a clip rect would incorrectly hide their content.
#[test]
fn test_no_clip_for_pixel_space_plots() {
    use kuva::plot::PiePlot;
    let plot = PiePlot::new()
        .with_slice("A", 40.0, "steelblue")
        .with_slice("B", 60.0, "tomato");
    let plots = vec![Plot::Pie(plot)];
    let layout = Layout::auto_from_plots(&plots);

    let svg = render(plots, layout);

    // Pie is pixel-space — no clip path should be injected.
    assert!(!svg.contains("<clipPath"), "Pie SVG must NOT contain a <clipPath> element");
}
