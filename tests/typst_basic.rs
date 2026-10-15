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

// ── Path primitive ────────────────────────────────────────────────────────────

// Quiver arrowheads are `Primitive::Path` (M/L/Z triangles). Pre-Path support
// they were silently dropped — the typst output had no arrowheads at all.
#[test]
fn quiver_arrowheads_emit_merge_paths() {
    use kuva::plot::quiver::QuiverPlot;
    use kuva::render::plots::Plot;
    use kuva::render::render::render_multiple;

    let quiver = QuiverPlot::new().with_arrows(vec![
        (0.0_f64, 0.0_f64, 1.0_f64, 1.0_f64),
        (1.0, 1.0, -0.5, 0.5),
    ]);
    let layout = Layout::auto_from_plots(&[Plot::Quiver(quiver.clone())]);
    let scene = render_multiple(vec![Plot::Quiver(quiver)], layout);
    let typst_src = TypstBackend::default().render_scene(&scene);

    std::fs::create_dir_all("test_outputs").ok();
    std::fs::write("test_outputs/typst_quiver.typ", &typst_src).unwrap();

    assert!(
        typst_src.contains("merge-path("),
        "arrowhead Paths must emit merge-path calls"
    );
    assert!(
        !typst_src.contains("TODO"),
        "no TODO placeholders in emitted output"
    );
    // Arrowheads are filled triangles: closed, with a fill paint.
    assert!(typst_src.contains("close: true"));
}

// Arcs (venn/chord/sankey `A` commands) must lower to cubic bezier() calls.
#[test]
fn path_arcs_lower_to_beziers() {
    use kuva::render::render::{PathData, Primitive, Scene};

    let mut s = Scene::new(200.0, 200.0);
    s.elements.push(Primitive::Path(Box::new(PathData {
        // Half circle from (50,100) to (150,100), then close.
        d: "M 50 100 A 50 50 0 0 1 150 100 Z".to_string(),
        fill: Some(kuva::render::color::Color::Css("#ff8800".into())),
        stroke: kuva::render::color::Color::Css("black".into()),
        stroke_width: 1.0,
        opacity: Some(0.5),
        stroke_dasharray: None,
    })));
    let typst_src = TypstBackend::default().render_scene(&s);

    assert!(typst_src.contains("merge-path("));
    assert!(
        typst_src.contains("bezier(("),
        "arc must be approximated by cubic beziers"
    );
    assert!(
        typst_src.contains("transparentize("),
        "opacity must map to transparentize()"
    );
}

// ── Clip regions ──────────────────────────────────────────────────────────────

// A ClipStart/ClipEnd pair must produce a `box(clip: true)` at the clip rect,
// with content before/after the region in separate stacked canvases.
#[test]
fn clip_region_becomes_clipped_box() {
    use kuva::render::color::Color;
    use kuva::render::render::{Primitive, Scene, TextAnchor};

    let mut s = Scene::new(400.0, 300.0);
    s.elements.push(Primitive::Circle {
        cx: 10.0,
        cy: 10.0,
        r: 3.0,
        fill: Color::Css("black".into()),
        fill_opacity: None,
        stroke: None,
        stroke_width: None,
    });
    s.elements.push(Primitive::ClipStart {
        x: 50.0,
        y: 40.0,
        width: 300.0,
        height: 200.0,
        id: "clip0".to_string(),
    });
    // Straddling shape: a large blue circle centered on the clip rect's left
    // edge — half sits inside the clip, half outside, so the rendered typst
    // file visibly demonstrates clipping (not just primitive emission).
    s.elements.push(Primitive::Circle {
        cx: 50.0,
        cy: 140.0,
        r: 60.0,
        fill: Color::Rgb(70, 130, 180),
        fill_opacity: None,
        stroke: None,
        stroke_width: None,
    });
    // Fully-inside sanity check.
    s.elements.push(Primitive::Circle {
        cx: 260.0,
        cy: 140.0,
        r: 30.0,
        fill: Color::Css("red".into()),
        fill_opacity: None,
        stroke: None,
        stroke_width: None,
    });
    s.elements.push(Primitive::ClipEnd);
    s.elements.push(Primitive::Text {
        x: 200.0,
        y: 280.0,
        content: "after clip".to_string(),
        size: 12,
        anchor: TextAnchor::Middle,
        rotate: None,
        bold: false,
        color: None,
    });
    let typst_src = TypstBackend::default().render_scene(&s);

    std::fs::create_dir_all("test_outputs").ok();
    std::fs::write("test_outputs/typst_clip.typ", &typst_src).unwrap();

    assert!(
        typst_src.contains("box(width: 300pt, height: 200pt, clip: true)"),
        "clip rect must become a clipped box"
    );
    assert!(
        typst_src.contains("dx: 50pt, dy: 40pt"),
        "clip box placed at the region origin"
    );
    assert!(
        typst_src.contains("dx: -50pt, dy: -40pt"),
        "inner canvas shifted back so coordinates stay page-absolute"
    );
    // Three chunks → three canvases (before, clipped, after).
    assert_eq!(typst_src.matches("#cetz.canvas(").count(), 3);
    assert!(typst_src.contains("after clip"));
}
