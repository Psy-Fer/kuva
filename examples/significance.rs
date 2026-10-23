//! Significance bracket documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example significance
//! ```
//!
//! SVGs are written to `docs/src/assets/significance/`.

use kuva::backend::svg::SvgBackend;
use kuva::plot::annotations::SignificanceBracket;
use kuva::plot::{BarPlot, BoxPlot, ViolinPlot};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use std::fs;

const OUT: &str = "docs/src/assets/significance";

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all(OUT).unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

fn samples(mean: f64, std: f64, n: usize, seed: u64) -> Vec<f64> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    Normal::new(mean, std)
        .unwrap()
        .sample_iter(&mut rng)
        .take(n)
        .collect()
}

fn four_groups() -> BoxPlot {
    BoxPlot::new()
        .with_group("WT", samples(5.0, 1.0, 60, 1))
        .with_group("KO", samples(7.2, 1.2, 60, 2))
        .with_group("Rescue", samples(5.4, 0.9, 60, 3))
        .with_group("KO + drug", samples(6.1, 1.1, 60, 4))
        .with_group_colors(["#4C72B0", "#DD8452", "#55A868", "#C44E52"])
}

fn main() {
    // ── Stars, auto-stacked ─────────────────────────────────────────────────
    // Three comparisons by category label. Adjacent pairs share the lowest
    // level; the WT–KO + drug bracket spans both and is lifted above them.
    let plots = vec![Plot::Box(four_groups())];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Box plot with significance brackets")
        .with_y_label("Expression (a.u.)")
        .with_significance("WT", "KO", 0.00002)
        .with_significance("KO", "Rescue", 0.0008)
        .with_significance("WT", "KO + drug", 0.03);
    write("boxplot_stars", plots, layout);

    // ── Numeric p-values and custom text on bars ────────────────────────────
    let bar = BarPlot::new()
        .with_bar("Vehicle", 12.4)
        .with_bar("10 µM", 18.9)
        .with_bar("100 µM", 27.3)
        .with_error(vec![1.1, 1.6, 2.2])
        .with_color("#4C72B0");
    let plots = vec![Plot::Bar(bar)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Dose response")
        .with_y_label("Colonies")
        .with_bracket(SignificanceBracket::new("Vehicle", "10 µM", 0.041).with_p_value_label())
        .with_bracket(SignificanceBracket::new("Vehicle", "100 µM", 0.00031).with_p_value_label())
        .with_bracket(SignificanceBracket::text("10 µM", "100 µM", "n.s."));
    write("bar_pvalues", plots, layout);

    // ── Stars plus value, styled ────────────────────────────────────────────
    let violin = ViolinPlot::new()
        .with_group("Control", samples(50.0, 8.0, 80, 7))
        .with_group("Treated", samples(61.0, 9.0, 80, 8))
        .with_group_colors(["#8172B3", "#937860"]);
    let plots = vec![Plot::Violin(violin)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Violin plot, stars with the p-value")
        .with_y_label("Score")
        .with_bracket(
            SignificanceBracket::new(0usize, 1usize, 0.0031)
                .with_both_labels()
                .with_color("#C44E52")
                .with_stroke_width(1.5)
                .with_font_size(14),
        );
    write("violin_both", plots, layout);

    // ── Horizontal plot: brackets run along y ───────────────────────────────
    // Orientation is inferred from the layout; no flag on the brackets.
    let plots = vec![Plot::Box(four_groups().with_horizontal(true))];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Horizontal box plot")
        .with_x_label("Expression (a.u.)")
        .with_significance("WT", "KO", 0.00002)
        .with_significance("Rescue", "KO + drug", 0.2)
        .with_bracket(SignificanceBracket::new("WT", "KO + drug", 0.03).with_p_value_label());
    write("horizontal", plots, layout);

    // ── Log value axis ──────────────────────────────────────────────────────
    let boxes = BoxPlot::new()
        .with_group("Input", samples(120.0, 20.0, 40, 11))
        .with_group("IP", samples(2400.0, 500.0, 40, 12))
        .with_group("IgG", samples(90.0, 25.0, 40, 13))
        .with_group_colors(["#4C72B0", "#DD8452", "#8C8C8C"]);
    let plots = vec![Plot::Box(boxes)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Log axis")
        .with_y_label("Reads")
        .with_log_y()
        .with_significance("Input", "IP", 0.000001)
        .with_significance("IP", "IgG", 0.000004)
        .with_significance("Input", "IgG", 0.4);
    write("log_axis", plots, layout);

    // ── Pinned position ─────────────────────────────────────────────────────
    // An explicit position opts out of stacking and axis growth.
    let plots = vec![Plot::Box(four_groups())];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Pinned bracket")
        .with_y_label("Expression (a.u.)")
        .with_bracket(SignificanceBracket::new("WT", "KO", 0.00002).with_position(3.0));
    write("pinned", plots, layout);

    println!("Significance SVGs written to {OUT}/");
}
