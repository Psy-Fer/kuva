//! Integration tests for the always-on **lookup tier**: `$...$` math in labels
//! is lowered to inline Unicode by every backend, with no `math` feature.
//!
//! This is the path `cargo test --features cli,full` exercises (math is
//! excluded from `full`), so these guard the default rendering behaviour.

#[cfg(not(feature = "math"))]
use kuva::backend::svg::SvgBackend;
use kuva::backend::terminal::TerminalBackend;
use kuva::plot::scatter::ScatterPlot;
use kuva::render::layout::Layout;
use kuva::render::render::render_scatter;

fn scatter_with_labels(title: &str, x: &str, y: &str) -> kuva::render::render::Scene {
    let plot = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 1.0), (2.0, 4.0)])
        .with_color("steelblue");
    let layout = Layout::new((0.0, 3.0), (0.0, 10.0))
        .with_title(title)
        .with_x_label(x)
        .with_y_label(y);
    render_scatter(&plot, layout).with_background(Some("white"))
}

// The terminal backend can only ever use the lookup tier — assert it lowers
// math regardless of features.
#[test]
fn terminal_lowers_math_to_unicode() {
    let scene = scatter_with_labels("$\\sigma^2$ over $\\mu$", "x", "y");
    let out = TerminalBackend::new(120, 40).render_scene(&scene);
    assert!(out.contains('σ'), "expected σ in terminal output");
    assert!(out.contains('μ'), "expected μ in terminal output");
    assert!(
        out.contains('²'),
        "expected superscript ² in terminal output"
    );
    // No raw math source survives.
    assert!(!out.contains('$'), "no `$` markers should remain");
    assert!(!out.contains("\\sigma"), "no LaTeX command should remain");
}

// Without the `math` feature the SVG backend uses the lookup tier too: math
// becomes ordinary Unicode `<text>`, never a typst fragment or raw source.
#[cfg(not(feature = "math"))]
#[test]
fn svg_lookup_tier_emits_unicode_text() {
    let scene = scatter_with_labels("Title", "Variance, $\\sigma^2$ (units)", "y");
    let svg = SvgBackend::default().render_scene(&scene);

    assert!(svg.contains("σ²"), "expected lowered σ² in SVG text");
    assert!(
        !svg.contains("$\\sigma"),
        "raw LaTeX source must not appear"
    );
    assert!(
        !svg.contains("$\\sigma^2$"),
        "raw math region must not appear"
    );
    // No typst fragment markers (that's the math-feature path, not this one).
    assert!(!svg.contains("typst-text"));
}

#[cfg(not(feature = "math"))]
#[test]
fn svg_lookup_tier_fractions_and_sqrt() {
    let scene = scatter_with_labels("$\\frac{a}{b}$", "$\\sqrt{x}$", "y");
    let svg = SvgBackend::default().render_scene(&scene);
    assert!(svg.contains("a/b"), "fraction lowered inline");
    assert!(svg.contains("√x"), "sqrt lowered inline");
    assert!(!svg.contains('$'), "no `$` markers should remain");
}

// `contains_math` requires two unescaped `$`; an escaped `\$` is a literal
// dollar and must pass through untouched in every backend.
#[cfg(not(feature = "math"))]
#[test]
fn svg_escaped_dollar_is_literal() {
    let scene = scatter_with_labels("Price \\$5", "x", "y");
    let svg = SvgBackend::default().render_scene(&scene);
    assert!(svg.contains("$5") || svg.contains("\\$5"));
}

// Math also works inside markdown TextPlot bodies (rich text). It's lowered to
// inline Unicode after markdown markers are parsed.
#[test]
fn markdown_textplot_lowers_math() {
    use kuva::backend::svg::SvgBackend;
    use kuva::plot::text::TextPlot;
    use kuva::render::plots::Plot;
    use kuva::render::render::render_multiple;

    let tp = TextPlot::new()
        .with_title("Result")
        .with_body("The **variance** is $\\sigma^2$ and the mean is $\\mu$.");
    let layout = Layout::new((0.0, 1.0), (0.0, 1.0));
    let svg = SvgBackend::default().render_scene(&render_multiple(vec![Plot::Text(tp)], layout));

    assert!(svg.contains('σ'), "expected lowered σ in markdown body");
    assert!(svg.contains('μ'), "expected lowered μ in markdown body");
    assert!(svg.contains('²'), "expected superscript ²");
    assert!(!svg.contains('$'), "no raw `$` markers should remain");
}
