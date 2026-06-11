//! End-to-end smoke test for the Typst output backend.
//!
//! Renders a small scatter plot to Typst markup and verifies the output is
//! a syntactically plausible Typst document with the expected drawing
//! primitives. We don't invoke `typst compile` here (it would require Typst
//! on the test runner); the structural checks below are sufficient to catch
//! regressions in the emitter.

#![cfg(feature = "typst")]

use kuva::backend::typst::TypstBackend;
use kuva::plot::scatter::ScatterPlot;
use kuva::render::layout::Layout;
use kuva::render::render::render_scatter;

#[test]
fn scatter_plot_emits_valid_typst_document() {
    let plot = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 1.0), (2.0, 4.0), (3.0, 9.0)])
        .with_color("steelblue");

    let layout = Layout::new((0.0, 4.0), (0.0, 10.0))
        .with_title("Test plot")
        .with_x_label("X axis")
        .with_y_label("Y axis");

    let scene = render_scatter(&plot, layout).with_background(Some("white"));
    let typst_src = TypstBackend::default().render_scene(&scene);

    std::fs::create_dir_all("test_outputs").ok();
    std::fs::write("test_outputs/typst_basic.typ", &typst_src).unwrap();

    // Preamble checks.
    assert!(typst_src.starts_with("#set page("));
    assert!(typst_src.contains("#import \"@preview/cetz:"));
    assert!(typst_src.contains("#cetz.canvas("));
    assert!(typst_src.contains("import cetz.draw: *"));

    // The title and labels should appear as text in content() calls.
    assert!(typst_src.contains("Test plot"));
    assert!(typst_src.contains("X axis"));
    assert!(typst_src.contains("Y axis"));

    // Scatter points should produce circle() calls.
    assert!(typst_src.contains("circle(("));

    // Closing brace of the canvas block.
    assert!(typst_src.trim_end().ends_with("})"));
}

#[test]
fn typst_color_emits_hex() {
    // Verify that an RGB color emitted by a primitive ends up as a
    // rgb("#...") literal in the output.
    let plot = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0)])
        .with_color("#ff8800");
    let layout = Layout::new((0.0, 2.0), (0.0, 5.0))
        .with_x_label("X")
        .with_y_label("Y");
    let scene = render_scatter(&plot, layout).with_background(Some("white"));
    let typst_src = TypstBackend::default().render_scene(&scene);
    assert!(typst_src.contains("rgb(\"#ff8800\")"));
}

#[test]
fn typst_passes_math_through_natively() {
    // Verify the Typst backend emits `$...$` math syntax for label content
    // containing math regions. This is the whole point of the Typst
    // backend: its native math typesetter handles the expression with
    // proper fonts. Non-Typst backends render the label literally.
    let plot = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 1.0)])
        .with_color("steelblue");
    let layout = Layout::new((0.0, 2.0), (0.0, 5.0))
        .with_x_label("Variance, $\\sigma^2$ (units)")
        .with_y_label("Y");

    let scene = render_scatter(&plot, layout).with_background(Some("white"));
    let typst_src = TypstBackend::default().render_scene(&scene);

    std::fs::create_dir_all("test_outputs").ok();
    std::fs::write("test_outputs/typst_math.typ", &typst_src).unwrap();

    // The math region becomes native Typst math: `\sigma`→σ (Unicode, which
    // Typst renders), `^2`→`^(2)`, wrapped in `$...$` for Typst's typesetter.
    assert!(
        typst_src.contains("$σ^(2)$"),
        "expected `$σ^(2)$` in Typst output (math translation); got: {typst_src}"
    );

    // The surrounding text should still appear as regular content.
    assert!(typst_src.contains("Variance,"));
    assert!(typst_src.contains("(units)"));
}

#[test]
fn typst_escapes_markup_specials_in_labels() {
    // Regression: the markup backend's escaper must cover `_` and `*` (Typst
    // subscript/emphasis), not just `# [ ] $ @` — otherwise `a_b` / `*x*`
    // would be mis-typeset. Shared with the math tier's escaper.
    let plot = ScatterPlot::new().with_data(vec![(1.0_f64, 1.0)]);
    let layout = Layout::new((0.0, 2.0), (0.0, 5.0)).with_x_label("rate a_b and *x* < 5");
    let scene = render_scatter(&plot, layout).with_background(Some("white"));
    let typst_src = TypstBackend::default().render_scene(&scene);
    assert!(typst_src.contains("a\\_b"), "underscore must be escaped");
    assert!(typst_src.contains("\\*x\\*"), "asterisks must be escaped");
    assert!(typst_src.contains("\\< 5"), "less-than must be escaped");
}

#[test]
fn typst_y_axis_is_flipped() {
    // Pick a point with known coordinates and verify its y is flipped
    // relative to the scene height.
    let plot = ScatterPlot::new().with_data(vec![(1.0_f64, 1.0)]);
    let layout = Layout::new((0.0, 2.0), (0.0, 5.0));
    let scene = render_scatter(&plot, layout).with_background(Some("white"));
    let typst_src = TypstBackend::default().render_scene(&scene);

    let scene_h = scene.height;

    // Find the first `circle((x, y)` and ensure its y is < scene_h (i.e.
    // flipped from SVG's "y grows down" to CETZ's "y grows up").
    let needle = "circle((";
    let idx = typst_src
        .find(needle)
        .expect("expected at least one circle");
    let after = &typst_src[idx + needle.len()..];
    // After "((", we have "<x>, <y>)" — bare numbers (CETZ uses canvas `length:`).
    let comma = after.find(',').unwrap();
    let after_y = &after[comma + 1..];
    let close = after_y.find(')').unwrap();
    let y_str = after_y[..close].trim();
    let y: f64 = y_str.parse().unwrap();

    // Flipped y: must be between 0 and scene_h. Original y was somewhere
    // inside the plot area, so flipped y is on the opposite side.
    assert!(y > 0.0 && y < scene_h, "y={y} out of expected range");
}
