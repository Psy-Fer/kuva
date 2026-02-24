//! Chord diagram documentation examples.
//!
//! Generates canonical SVG outputs used in the visus documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example chord
//! ```
//!
//! SVGs are written to `docs/src/assets/chord/`.

use visus::plot::ChordPlot;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

const OUT: &str = "docs/src/assets/chord";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/chord");

    basic();
    asymmetric();
    styled();

    println!("Chord diagram SVGs written to {OUT}/");
}

/// Symmetric co-occurrence matrix — immune cell type co-clustering rates.
///
/// `matrix[i][i] = 0`; off-diagonal values are symmetric, so each ribbon
/// has equal width at both ends. Default colors from the category10 palette.
fn basic() {
    // Co-clustering proximity scores between major PBMC cell types
    let matrix = vec![
        //         CD4T   CD8T    NK   Bcell   Mono
        vec![   0.0, 120.0,  70.0,  40.0,  25.0],  // CD4 T
        vec![ 120.0,   0.0,  88.0,  32.0,  18.0],  // CD8 T
        vec![  70.0,  88.0,   0.0,  15.0,  35.0],  // NK
        vec![  40.0,  32.0,  15.0,   0.0,  10.0],  // B cell
        vec![  25.0,  18.0,  35.0,  10.0,   0.0],  // Monocyte
    ];

    let chord = ChordPlot::new()
        .with_matrix(matrix)
        .with_labels(["CD4 T", "CD8 T", "NK", "B cell", "Monocyte"]);

    let plots = vec![Plot::Chord(chord)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("PBMC Cell Type Co-clustering");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Asymmetric directed flows — gene regulatory network.
///
/// `matrix[i][j]` is the regulatory influence of node i on node j.
/// Because i→j ≠ j→i, each ribbon is thicker at the source end than the
/// target end. Custom colors and a node legend are enabled.
fn asymmetric() {
    // Directed regulatory influence strengths between five transcription factors
    let matrix = vec![
        //        TF1    TF2    TF3    TF4    TF5
        vec![   0.0,  85.0,  20.0,  45.0,  10.0],  // TF1
        vec![  15.0,   0.0,  65.0,  30.0,   8.0],  // TF2
        vec![  30.0,  12.0,   0.0,  75.0,  25.0],  // TF3
        vec![   5.0,  40.0,  18.0,   0.0,  90.0],  // TF4
        vec![  50.0,   8.0,  35.0,  12.0,   0.0],  // TF5
    ];

    let chord = ChordPlot::new()
        .with_matrix(matrix)
        .with_labels(["TF1", "TF2", "TF3", "TF4", "TF5"])
        .with_colors(["#e6194b", "#3cb44b", "#4363d8", "#f58231", "#911eb4"])
        .with_gap(3.0)
        .with_legend("Transcription factors");

    let plots = vec![Plot::Chord(chord)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Gene Regulatory Network");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/asymmetric.svg"), svg).unwrap();
}

/// Styling — wider gaps and reduced opacity.
///
/// `.with_gap(degrees)` increases the white space between arc segments,
/// making individual nodes easier to distinguish. `.with_opacity(f)` reduces
/// ribbon transparency so overlapping ribbons remain readable.
fn styled() {
    // Same co-clustering data but with wider gaps and lower opacity
    let matrix = vec![
        vec![  0.0, 120.0,  70.0,  40.0,  25.0],
        vec![120.0,   0.0,  88.0,  32.0,  18.0],
        vec![ 70.0,  88.0,   0.0,  15.0,  35.0],
        vec![ 40.0,  32.0,  15.0,   0.0,  10.0],
        vec![ 25.0,  18.0,  35.0,  10.0,   0.0],
    ];

    let chord = ChordPlot::new()
        .with_matrix(matrix)
        .with_labels(["CD4 T", "CD8 T", "NK", "B cell", "Monocyte"])
        .with_gap(6.0)       // default 2.0
        .with_opacity(0.45); // default 0.7

    let plots = vec![Plot::Chord(chord)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Wider Gaps, Reduced Opacity");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/styled.svg"), svg).unwrap();
}
