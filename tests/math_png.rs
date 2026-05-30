//! Structural tests for the typst math tier in the raster (PNG) backend,
//! including rotated labels (y-axis titles).

#![cfg(all(feature = "math", feature = "png"))]

use kuva::backend::png::PngBackend;
use kuva::plot::scatter::ScatterPlot;
use kuva::render::layout::Layout;
use kuva::render::math::render_label_pixmap;
use kuva::render::render::render_scatter;

#[test]
fn pixmap_has_visible_content() {
    let pm = render_label_pixmap("$\\sigma^2$", 14.0, None, 2.0).expect("pixmap");
    assert!(pm.width_px > 0 && pm.height_px > 0);
    assert!(pm.baseline_offset_px > 0.0);
    let opaque = pm.rgba.chunks_exact(4).filter(|p| p[3] > 0).count();
    assert!(
        opaque > 20,
        "expected many non-transparent pixels, got {opaque}"
    );
}

fn render_png(x_label: &str, y_label: &str) -> Vec<u8> {
    let plot = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 1.0), (2.0, 4.0)])
        .with_color("steelblue");
    let layout = Layout::new((0.0, 3.0), (0.0, 10.0))
        .with_x_label(x_label)
        .with_y_label(y_label);
    let scene = render_scatter(&plot, layout).with_background(Some("white"));
    PngBackend::default()
        .render_scene(&scene)
        .expect("png render")
}

#[test]
fn math_label_composites_into_png() {
    let with_math = render_png("Variance, $\\sigma^2$ (units)", "Y");
    std::fs::create_dir_all("test_outputs").ok();
    std::fs::write("test_outputs/math_xlabel.png", &with_math).unwrap();

    assert_eq!(&with_math[..8], b"\x89PNG\r\n\x1a\n");
    // Compositing the typst pixmap must change pixels vs a math-free label.
    let plain = render_png("Variance, sigma2 (units)", "Y");
    assert_ne!(
        with_math, plain,
        "math PNG should differ from math-free render"
    );
}

#[test]
fn rotated_math_label_composites_into_png() {
    // Y-axis titles are rotated -90°; math must still be typeset (offscreen
    // rotate-blit), not dropped to literal text.
    let with_math = render_png("x", "Energy $E = m c^2$");
    std::fs::create_dir_all("test_outputs").ok();
    std::fs::write("test_outputs/math_ylabel_rotated.png", &with_math).unwrap();

    assert_eq!(&with_math[..8], b"\x89PNG\r\n\x1a\n");
    let plain = render_png("x", "Energy E = m c2");
    assert_ne!(
        with_math, plain,
        "rotated math PNG should differ from math-free"
    );
}

#[test]
fn box_not_undersized_so_descenders_are_captured() {
    // Regression for the rotated-y-label clip: `height: auto` + `margin: 0pt`
    // collapsed the page to ~0.75em and chopped descenders. The page margin
    // keeps the box ≥ 1em tall (here ~1.3em), comfortably containing g/y/j/p/q.
    let size = 14.0_f64;
    let ppp = 2.0_f32;
    let pm = render_label_pixmap("gyjpq $\\sigma$", size, None, ppp).expect("pixmap");
    assert!(
        (pm.height_px as f64) > size * ppp as f64,
        "pixmap height {}px should exceed one em ({}px) — box was undersized",
        pm.height_px,
        size * ppp as f64
    );
}

#[test]
fn multiple_math_regions_composite() {
    let with_math = render_png("$\\alpha$ and $\\beta$", "Y");
    std::fs::create_dir_all("test_outputs").ok();
    std::fs::write("test_outputs/math_multiple_regions.png", &with_math).unwrap();
    assert_eq!(&with_math[..8], b"\x89PNG\r\n\x1a\n");
    let plain = render_png("alpha and beta", "Y");
    assert_ne!(with_math, plain);
}

#[test]
fn colored_math_label_renders() {
    // Math fill follows the label color — a colored label differs from the
    // default-color render at the same size.
    use kuva::render::color::Color;
    let pm_red = render_label_pixmap("$x^2$", 14.0, Some(&Color::Rgb(255, 0, 0)), 2.0).unwrap();
    let pm_black = render_label_pixmap("$x^2$", 14.0, None, 2.0).unwrap();
    assert_eq!(pm_red.width_px, pm_black.width_px, "same glyphs, same size");
    assert_ne!(pm_red.rgba, pm_black.rgba, "red vs black pixels differ");
}
