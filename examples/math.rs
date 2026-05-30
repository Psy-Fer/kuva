//! Math-in-labels documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with (the `math` feature is required for real Typst typesetting;
//! without it the always-on lookup tier renders inline Unicode):
//!
//! ```bash
//! cargo run --example math --features math
//! ```
//!
//! SVGs are written to `docs/src/assets/math/`.

use kuva::backend::svg::SvgBackend;
use kuva::plot::scatter::ScatterPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use std::fs;

const OUT: &str = "docs/src/assets/math";

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all(OUT).unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

/// A scatter with a fixed dataset; only the labels vary between scenarios.
fn scatter(title: &str, x_label: &str, y_label: &str) -> (Vec<Plot>, Layout) {
    let plot = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 1.0), (2.0, 4.0), (3.0, 9.0)])
        .with_color("steelblue");
    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::new((0.0, 4.0), (0.0, 10.0))
        .with_title(title)
        .with_x_label(x_label)
        .with_y_label(y_label);
    (plots, layout)
}

fn main() {
    // ── Greek, super/subscripts ───────────────────────────────────────────
    let (p, l) = scatter("Standard deviation", "$\\mu \\pm \\sigma$", "count");
    write("greek", p, l);

    let (p, l) = scatter("Power law", "$x^2 + y^2 = r^2$", "$f(x)$");
    write("superscript", p, l);

    // ── Fractions & radicals ──────────────────────────────────────────────
    let (p, l) = scatter("Fraction", "$\\frac{a + b}{c}$", "rate");
    write("fraction", p, l);

    let (p, l) = scatter("Square root", "$\\sqrt{x^2 + y^2}$", "distance");
    write("sqrt", p, l);

    // ── Large operators ───────────────────────────────────────────────────
    let (p, l) = scatter("Summation", "$\\sum_{i=1}^{n} x_i$", "total");
    write("sum", p, l);

    // Note: Typst math treats a multi-letter run as one identifier, so factors
    // are spaced (`4 a c`) — otherwise `ac` is an unknown variable.
    let (p, l) = scatter(
        "Quadratic formula",
        "$x = \\frac{-b \\pm \\sqrt{b^2 - 4 a c}}{2 a}$",
        "roots",
    );
    write("quadratic", p, l);

    // ── Rotated y-axis title + mixed text/math ────────────────────────────
    let (p, l) = scatter("Mass–energy equivalence", "time", "Energy $E = m c^2$");
    write("rotated_ylabel", p, l);

    let (p, l) = scatter(
        "Mixed text and math",
        "Variance, $\\sigma^2$ (units)",
        "$\\nabla \\cdot F$",
    );
    write("mixed", p, l);

    println!("Math SVGs written to {OUT}/");
}
