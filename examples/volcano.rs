//! Volcano plot documentation examples.
//!
//! Generates canonical SVG outputs used in the visus documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example volcano
//! ```
//!
//! SVGs are written to `docs/src/assets/volcano/`.

use visus::plot::{VolcanoPlot, LabelStyle};
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

const OUT: &str = "docs/src/assets/volcano";

// DESeq2-style results: (gene, log2FoldChange, pvalue)
// Represents a tumour vs. normal comparison in an RNA-seq experiment.
const DATA: &[(&str, f64, f64)] = &[
    // Up-regulated
    ("AKT1",   3.5, 0.000050),
    ("EGFR",   3.2, 0.000100),
    ("PTEN",   2.8, 0.002000),
    ("BRCA1",  2.5, 0.001000),
    ("BCL2",   2.7, 0.001000), // wait — BCL2 is down in the test, but let's make it up here for balance
    ("VEGFA",  2.3, 0.003000),
    ("KRAS",   2.1, 0.005000),
    ("TP53",   1.8, 0.010000),
    ("RB1",    1.9, 0.008000),
    ("SMAD2",  1.6, 0.025000),
    ("MYC",    1.5, 0.030000),
    ("CDK4",   1.2, 0.040000),
    ("CCND1",  2.4, 0.002000),
    ("PIK3CA", 1.7, 0.015000),
    ("FGFR1",  3.0, 0.000200),
    ("ERBB2",  2.6, 0.001500),
    ("MET",    1.4, 0.035000),
    // Down-regulated
    ("P21",   -3.2, 0.000200),
    ("VHL",   -3.0, 0.000500),
    ("MDM2",  -2.5, 0.003000),
    ("BCL2L", -2.7, 0.001000),
    ("CDKN2A",-2.3, 0.002000),
    ("PUMA",  -2.0, 0.007000),
    ("SMAD4", -1.9, 0.008000),
    ("BAX",   -1.7, 0.015000),
    ("CASP3", -1.6, 0.040000),
    ("FAS",   -1.4, 0.035000),
    ("CDKN1B",-2.1, 0.004000),
    ("RUNX3", -1.5, 0.028000),
    ("MLH1",  -2.8, 0.000800),
    ("RASSF1",-1.8, 0.012000),
    // Not significant — low fold change
    ("GAPDH",  0.3, 0.500),
    ("ACTB",  -0.5, 0.300),
    ("TUBA1",  0.8, 0.100),
    ("HIST1", -0.2, 0.700),
    ("RPL5",   0.6, 0.200),
    ("RPS6",  -0.9, 0.150),
    ("EEF1A",  0.1, 0.800),
    ("HNRNPA",-0.7, 0.400),
    // NS — large FC but p not significant
    ("GeneA",  1.5, 0.200),
    ("GeneB", -1.1, 0.070),
    ("GeneC",  2.0, 0.120),
    ("GeneD", -1.8, 0.080),
    ("GeneE",  1.3, 0.180),
];

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/volcano");

    basic();
    labeled();
    arrow_labels();
    custom_thresholds();

    println!("Volcano SVGs written to {OUT}/");
}

/// Basic volcano plot — default thresholds, no gene labels.
///
/// fc_cutoff = 1.0 (|log2FC| > 1), p_cutoff = 0.05. Dashed threshold
/// lines are drawn automatically. Points are colored by category:
/// up-regulated (red), down-regulated (blue), not significant (gray).
fn basic() {
    let vp = VolcanoPlot::new()
        .with_points(DATA.iter().copied())
        .with_legend("DEG status");

    let plots = vec![Plot::Volcano(vp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Tumour vs. Normal — Volcano Plot")
        .with_x_label("log\u{2082} fold change")
        .with_y_label("\u{2212}log\u{2081}\u{2080}(p-value)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Nudge-style gene labels (default) — top 12 most significant points.
///
/// `.with_label_top(n)` labels the `n` points with the lowest p-values.
/// `LabelStyle::Nudge` (default) sorts labels by x position and nudges
/// them vertically to reduce stacking.
fn labeled() {
    let vp = VolcanoPlot::new()
        .with_points(DATA.iter().copied())
        .with_label_top(12);

    let plots = vec![Plot::Volcano(vp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Volcano Plot — Nudge Labels (top 12)")
        .with_x_label("log\u{2082} fold change")
        .with_y_label("\u{2212}log\u{2081}\u{2080}(p-value)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/labeled.svg"), svg).unwrap();
}

/// Arrow-style labels — offset text with a leader line back to the point.
///
/// `LabelStyle::Arrow { offset_x, offset_y }` places the label at
/// (point_x + offset_x, point_y + offset_y) px and draws a short gray
/// leader line connecting the label to its point.
fn arrow_labels() {
    let vp = VolcanoPlot::new()
        .with_points(DATA.iter().copied())
        .with_label_top(10)
        .with_label_style(LabelStyle::Arrow { offset_x: 14.0, offset_y: 16.0 });

    let plots = vec![Plot::Volcano(vp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Volcano Plot — Arrow Labels (top 10)")
        .with_x_label("log\u{2082} fold change")
        .with_y_label("\u{2212}log\u{2081}\u{2080}(p-value)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/arrow_labels.svg"), svg).unwrap();
}

/// Stricter thresholds — |log2FC| > 2 and p < 0.01.
///
/// Fewer genes pass both filters; the significant set is tighter.
/// Custom colors distinguish this plot from the default palette.
fn custom_thresholds() {
    let vp = VolcanoPlot::new()
        .with_points(DATA.iter().copied())
        .with_fc_cutoff(2.0)
        .with_p_cutoff(0.01)
        .with_label_top(8)
        .with_color_up("darkorange")
        .with_color_down("mediumpurple")
        .with_legend("DEG status");

    let plots = vec![Plot::Volcano(vp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Stricter Thresholds — |log\u{2082}FC| > 2, p < 0.01")
        .with_x_label("log\u{2082} fold change")
        .with_y_label("\u{2212}log\u{2081}\u{2080}(p-value)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/custom_thresholds.svg"), svg).unwrap();
}
