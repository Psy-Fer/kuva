//! Math-in-labels documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example math
//! ```
//!
//! SVGs are written to `docs/src/assets/math/`.

use kuva::backend::svg::SvgBackend;
use kuva::plot::line::LinePlot;
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

    let (p, l) = scatter(
        "Quadratic formula",
        "$x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}$",
        "roots",
    );
    write("quadratic", p, l);

    // ── Rotated y-axis title + mixed text/math ────────────────────────────
    let (p, l) = scatter("Mass–energy equivalence", "time", "Energy $E = mc^2$");
    write("rotated_ylabel", p, l);

    let (p, l) = scatter(
        "Mixed text and math",
        "Variance, $\\sigma^2$ (units)",
        "$\\nabla \\cdot F$",
    );
    write("mixed", p, l);

    // ── Operator names (\log, \sin, etc.) — realistic scientific use cases ─

    // Volcano-plot-style axes: very common in RNA-seq / GWAS
    {
        let pts: Vec<(f64, f64)> = vec![
            (-4.1, 8.2),
            (-2.3, 5.1),
            (-1.2, 1.8),
            (0.1, 0.4),
            (0.9, 1.2),
            (1.8, 3.7),
            (2.5, 4.9),
            (3.6, 7.8),
            (4.4, 11.1),
        ];
        let plot = ScatterPlot::new().with_data(pts).with_color("steelblue");
        let layout = Layout::new((-5.0, 5.0), (0.0, 12.0))
            .with_title("Differential expression")
            .with_x_label("$\\log_2$ fold change")
            .with_y_label("$-\\log_{10}(p)$");
        write("log_axes", vec![Plot::Scatter(plot)], layout);
    }

    // Sinusoidal line: shows \sin, \theta, \pi in axis labels
    {
        use std::f64::consts::PI;
        let data: Vec<(f64, f64)> = (0..=60)
            .map(|i| {
                let x = i as f64 * PI / 30.0;
                (x, x.sin())
            })
            .collect();
        let plot = LinePlot::new().with_data(data).with_color("steelblue");
        let layout = Layout::new((0.0, 2.0 * PI), (-1.1, 1.1))
            .with_title("$\\sin(\\theta)$")
            .with_x_label("$\\theta$ (radians)")
            .with_y_label("$\\sin(\\theta)$");
        write("trig", vec![Plot::Line(plot)], layout);
    }

    // Exponential decay: shows \exp in a label
    {
        let data: Vec<(f64, f64)> = (0..=40)
            .map(|i| {
                let x = i as f64 * 0.1;
                (x, (-x).exp())
            })
            .collect();
        let plot = LinePlot::new().with_data(data).with_color("steelblue");
        let layout = Layout::new((0.0, 4.0), (0.0, 1.05))
            .with_title("Exponential decay")
            .with_x_label("time (s)")
            .with_y_label("$\\exp(-t)$");
        write("exp_decay", vec![Plot::Line(plot)], layout);
    }

    println!("Math SVGs written to {OUT}/");
}
