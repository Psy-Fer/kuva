//! Verify math rendering survives the PDF backend (SvgBackend → svg2pdf/usvg).
//!
//! The typst SVG fragment uses glyph references; the math module rewrites
//! `xlink:href` → `href` so usvg (which rejects the undeclared `xlink`
//! namespace prefix once the outer `<svg>` is stripped) can parse it.

#![cfg(all(feature = "math", feature = "pdf"))]

use kuva::backend::pdf::PdfBackend;
use kuva::plot::scatter::ScatterPlot;
use kuva::render::layout::Layout;
use kuva::render::render::render_scatter;

#[test]
fn math_label_renders_to_pdf() {
    let plot = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 1.0), (2.0, 4.0)])
        .with_color("steelblue");
    let layout = Layout::new((0.0, 3.0), (0.0, 10.0))
        .with_x_label("Variance, $\\sigma^2$ (units)")
        .with_y_label("Y");
    let scene = render_scatter(&plot, layout).with_background(Some("white"));

    // The key assertion is simply that this does NOT error — pre-fix, usvg
    // rejected the fragment with "unknown namespace prefix 'xlink'".
    let bytes = PdfBackend::default()
        .render_scene(&scene)
        .expect("pdf render succeeded (xlink rewrite lets usvg parse the math)");

    std::fs::create_dir_all("test_outputs").ok();
    std::fs::write("test_outputs/math_label.pdf", &bytes).unwrap();

    assert_eq!(&bytes[..5], b"%PDF-");
    assert!(bytes.len() > 2000, "non-trivial PDF output");
}
