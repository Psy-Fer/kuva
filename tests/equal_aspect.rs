/// Tests for `Layout::with_equal_aspect()` — equal x/y axis scaling.
///
/// The dramatic demo: render the same unit circle both ways.
///   - Without equal_aspect → x_scale ≠ y_scale → the "circle" is an ellipse.
///   - With    equal_aspect → x_scale == y_scale → the circle is actually circular.
use kuva::prelude::*;
use std::f64::consts::PI;

const CIRCLE_PTS: usize = 64;

/// Generate points on the unit circle.
fn circle_points() -> (Vec<f64>, Vec<f64>) {
    let xs: Vec<f64> = (0..CIRCLE_PTS)
        .map(|i| (2.0 * PI * i as f64 / CIRCLE_PTS as f64).cos())
        .collect();
    let ys: Vec<f64> = (0..CIRCLE_PTS)
        .map(|i| (2.0 * PI * i as f64 / CIRCLE_PTS as f64).sin())
        .collect();
    (xs, ys)
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn render(layout: Layout) -> String {
    let (xs, ys) = circle_points();
    let scatter = ScatterPlot::new()
        .with_data(xs.iter().copied().zip(ys.iter().copied()))
        .with_color("#2196f3");
    render_to_svg(vec![Plot::Scatter(scatter)], layout)
}

// ─── Core correctness ────────────────────────────────────────────────────────

/// Without equal_aspect, a 500×300 canvas with a symmetric data range
/// produces different x_scale and y_scale, so the unit circle becomes an ellipse.
/// We verify this by checking that the circle points span a different pixel
/// width vs pixel height in the SVG — i.e. `cx=` values have a wider spread
/// than `cy=` values.
#[test]
fn test_without_equal_aspect_circle_is_ellipse() {
    // Wide canvas, square data range → x axis has more pixels per span → ellipse
    let layout = Layout::new((-1.5, 1.5), (-1.5, 1.5))
        .with_width(600.0)
        .with_height(300.0)
        .with_title("Circle WITHOUT equal_aspect — should look like an ellipse");
    let svg = render(layout);

    let cx_vals = extract_attr_f64_values(&svg, "cx=");
    let cy_vals = extract_attr_f64_values(&svg, "cy=");

    let cx_span = range_span(&cx_vals);
    let cy_span = range_span(&cy_vals);

    // On a 600×300 canvas the x pixels per unit >> y pixels per unit.
    // The circle's rendered x-span should be noticeably wider than y-span.
    assert!(
        cx_span > cy_span * 1.5,
        "Expected cx_span ({cx_span:.1}) >> cy_span ({cy_span:.1}) without equal_aspect"
    );

    write_test_output("equal_aspect_off_ellipse.svg", &svg);
}

/// With equal_aspect the circle renders with equal pixel spans in x and y.
#[test]
fn test_with_equal_aspect_circle_is_circular() {
    let layout = Layout::new((-1.5, 1.5), (-1.5, 1.5))
        .with_width(600.0)
        .with_height(300.0)
        .with_equal_aspect()
        .with_title("Circle WITH equal_aspect — should look perfectly circular");
    let svg = render(layout);

    let cx_vals = extract_attr_f64_values(&svg, "cx=");
    let cy_vals = extract_attr_f64_values(&svg, "cy=");

    let cx_span = range_span(&cx_vals);
    let cy_span = range_span(&cy_vals);

    // With equal aspect both spans must be within 1 px of each other.
    let diff = (cx_span - cy_span).abs();
    assert!(
        diff < 2.0,
        "With equal_aspect cx_span ({cx_span:.1}) should ≈ cy_span ({cy_span:.1}), diff={diff:.2}"
    );

    write_test_output("equal_aspect_on_circle.svg", &svg);
}

/// Equal aspect always produces equal cx_span and cy_span regardless of canvas shape.
/// Even a nominally "square" canvas has asymmetric margins (title, axes), so
/// equal_aspect still adjusts — and the result is a true circle in pixel space.
#[test]
fn test_equal_aspect_always_equalises_spans() {
    // Use several canvas aspect ratios and confirm cx_span ≈ cy_span each time.
    let cases: &[(f64, f64)] = &[
        (400.0, 400.0),
        (600.0, 400.0),
        (400.0, 600.0),
        (800.0, 300.0),
    ];
    for &(w, h) in cases {
        let layout = Layout::new((-1.5, 1.5), (-1.5, 1.5))
            .with_width(w)
            .with_height(h)
            .with_equal_aspect();
        let svg = render(layout);
        let cx_span = range_span(&extract_attr_f64_values(&svg, "cx="));
        let cy_span = range_span(&extract_attr_f64_values(&svg, "cy="));
        let diff = (cx_span - cy_span).abs();
        assert!(
            diff < 2.0,
            "Canvas {w}×{h}: cx_span ({cx_span:.1}) ≈ cy_span ({cy_span:.1}), diff={diff:.2}"
        );
    }
}

/// Equal aspect with a tall canvas (height > width) — now y is the wider axis
/// and x must be expanded to match.
#[test]
fn test_equal_aspect_tall_canvas() {
    let layout = Layout::new((-1.5, 1.5), (-1.5, 1.5))
        .with_width(300.0)
        .with_height(600.0)
        .with_equal_aspect()
        .with_title("Circle on tall canvas — still circular");
    let svg = render(layout);

    let cx_vals = extract_attr_f64_values(&svg, "cx=");
    let cy_vals = extract_attr_f64_values(&svg, "cy=");
    let cx_span = range_span(&cx_vals);
    let cy_span = range_span(&cy_vals);

    let diff = (cx_span - cy_span).abs();
    assert!(
        diff < 2.0,
        "Tall canvas equal_aspect: cx_span ({cx_span:.1}) ≈ cy_span ({cy_span:.1}), diff={diff:.2}"
    );

    write_test_output("equal_aspect_tall_circle.svg", &svg);
}

/// API-level: `with_equal_aspect()` must be chainable and not consume Layout.
#[test]
fn test_equal_aspect_builder_is_chainable() {
    let layout = Layout::new((-1.0, 1.0), (-1.0, 1.0))
        .with_equal_aspect()
        .with_title("chained")
        .with_width(400.0)
        .with_height(400.0);
    // If this compiles and renders without panic, the builder chain is fine.
    let svg = render(layout);
    assert!(svg.contains("<svg"));
}

// ─── Utilities ───────────────────────────────────────────────────────────────

/// Scan `haystack` for all occurrences of `attr` (e.g. `"cx="`) and parse the
/// quoted f64 that follows.  Works for SVG attributes like `cx="123.45"`.
fn extract_attr_f64_values(haystack: &str, attr: &str) -> Vec<f64> {
    let mut vals = Vec::new();
    let mut rest = haystack;
    while let Some(pos) = rest.find(attr) {
        rest = &rest[pos + attr.len()..];
        // skip optional quote
        let rest2 = rest.trim_start_matches('"');
        let end = rest2
            .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
            .unwrap_or(rest2.len());
        if let Ok(v) = rest2[..end].parse::<f64>() {
            vals.push(v);
        }
    }
    vals
}

fn range_span(vals: &[f64]) -> f64 {
    if vals.is_empty() {
        return 0.0;
    }
    let min = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    max - min
}

fn write_test_output(name: &str, content: &str) {
    let dir = std::path::Path::new("test_outputs");
    let _ = std::fs::create_dir_all(dir);
    let _ = std::fs::write(dir.join(name), content);
}
