//! Structural tests for the typst math tier (SVG backend).
//!
//! These stay at the "compiles, embeds, no source noise" level on purpose —
//! byte-exact golden SVGs would break on every typst/font bump. The
//! deterministic logic (the lookup tier) is unit-tested in `render::math`.

#![cfg(feature = "math")]

use kuva::backend::svg::SvgBackend;
use kuva::plot::scatter::ScatterPlot;
use kuva::render::layout::Layout;
use kuva::render::math::render_label_svg;
use kuva::render::render::render_scatter;

#[test]
fn renders_label_to_svg_fragment() {
    let out = render_label_svg("$\\sigma^2$", 14.0, None).expect("typst compile");
    assert!(out.width_pt > 0.0 && out.height_pt > 0.0);
    assert!(out.baseline_offset_pt > 0.0);
    assert!(
        out.inner_svg.contains("<g") || out.inner_svg.contains("<path"),
        "fragment has drawing primitives"
    );
    // xlink rewritten so usvg/strict parsers accept the embedded fragment.
    assert!(!out.inner_svg.contains("xlink:href"));
}

#[test]
fn color_injected_into_typst_source() {
    use kuva::render::color::Color;
    let out = render_label_svg("$x$", 14.0, Some(&Color::Rgb(0xff, 0x00, 0x00))).expect("compile");
    assert!(
        out.inner_svg.contains("#ff0000") || out.inner_svg.contains("#FF0000"),
        "expected red fill in fragment"
    );
}

#[test]
fn scatter_label_embeds_typst_math() {
    let plot = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 1.0), (2.0, 4.0)])
        .with_color("steelblue");
    let layout = Layout::new((0.0, 3.0), (0.0, 10.0))
        .with_x_label("Variance, $\\sigma^2$ (units)")
        .with_y_label("Y");
    let scene = render_scatter(&plot, layout).with_background(Some("white"));
    let svg = SvgBackend::default().render_scene(&scene);

    std::fs::create_dir_all("test_outputs").ok();
    std::fs::write("test_outputs/math_label_embed.svg", &svg).unwrap();

    assert!(svg.contains("typst-text") || svg.contains("href=\"#g"));
    // Raw source must not survive — it was typeset, not printed.
    assert!(!svg.contains("$\\sigma^2$"));
    assert!(!svg.contains("xlink:href"));
}

#[test]
fn multiple_math_labels_have_unique_ids() {
    // Each Typst fragment carries id="glyph"/id="g…"; with >1 math label those
    // must be namespaced so the SVG has no duplicate IDs (else invalid SVG).
    let plot = ScatterPlot::new().with_data(vec![(1.0_f64, 1.0)]);
    let layout = Layout::new((0.0, 2.0), (0.0, 2.0))
        .with_title("$\\sigma$")
        .with_x_label("$\\mu^2$")
        .with_y_label("$\\sqrt{\\pi}$");
    let scene = render_scatter(&plot, layout).with_background(Some("white"));
    let svg = SvgBackend::default().render_scene(&scene);

    let mut ids = Vec::new();
    let mut rest = svg.as_str();
    while let Some(i) = rest.find("id=\"") {
        rest = &rest[i + 4..];
        if let Some(j) = rest.find('"') {
            ids.push(&rest[..j]);
            rest = &rest[j + 1..];
        }
    }
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(ids.len(), sorted.len(), "duplicate id= in SVG: {ids:?}");
    assert!(
        ids.iter().any(|id| id.starts_with("m0-")),
        "fragment ids should be namespaced"
    );
}

#[test]
fn multiple_math_regions_in_one_label() {
    let plot = ScatterPlot::new().with_data(vec![(1.0_f64, 1.0)]);
    let layout = Layout::new((0.0, 2.0), (0.0, 2.0)).with_x_label("$\\alpha$ to $\\beta$ range");
    let scene = render_scatter(&plot, layout).with_background(Some("white"));
    let svg = SvgBackend::default().render_scene(&scene);
    std::fs::create_dir_all("test_outputs").ok();
    std::fs::write("test_outputs/math_multiple_regions.svg", &svg).unwrap();
    assert!(svg.contains("typst-text") || svg.contains("href=\"#g"));
    assert!(!svg.contains('$'), "no raw math markers should survive");
}

#[test]
fn invalid_typst_falls_back_to_lookup_tier() {
    // `mc` is one identifier in Typst (not `m*c`), so `$E = mc^2$` fails to
    // compile and the label degrades to the lookup tier: literal Unicode text.
    let plot = ScatterPlot::new().with_data(vec![(1.0_f64, 1.0)]);
    let layout = Layout::new((0.0, 2.0), (0.0, 2.0)).with_x_label("$E = mc^2$");
    let scene = render_scatter(&plot, layout).with_background(Some("white"));
    let svg = SvgBackend::default().render_scene(&scene);
    std::fs::create_dir_all("test_outputs").ok();
    std::fs::write("test_outputs/math_fallback.svg", &svg).unwrap();
    // Lowered to inline Unicode text (a typst fragment would use glyph refs,
    // not the literal string "mc²").
    assert!(svg.contains("mc²"), "expected lookup-tier fallback text");
    assert!(!svg.contains("$E = mc"), "raw source must not appear");
}

#[test]
fn brackets_in_label_render_literally() {
    // `[...]` is a Typst content-block delimiter; if the label text isn't
    // escaped the brackets are swallowed. Escaped, they render as glyphs, so a
    // label with brackets must be wider than the same label without them.
    let with = render_label_svg("a [x] $\\sigma$", 14.0, None).expect("compile");
    let without = render_label_svg("a x $\\sigma$", 14.0, None).expect("compile");
    assert!(
        with.width_pt > without.width_pt,
        "bracketed label ({}) must be wider than unbracketed ({}) — brackets dropped?",
        with.width_pt,
        without.width_pt
    );
}

#[test]
fn box_not_undersized_so_descenders_are_captured() {
    // Regression: `height: auto` + `margin: 0pt` collapsed the page to ~0.75em
    // and clipped descenders. The page margin keeps the box ≥ 1em tall.
    let size = 14.0_f64;
    let m = render_label_svg("gyjpq $\\sigma$", size, None).expect("compile");
    assert!(
        m.height_pt > size,
        "rendered box height {} should exceed one em ({}) — box was undersized",
        m.height_pt,
        size
    );
}
