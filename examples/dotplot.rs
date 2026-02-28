//! Dot plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example dotplot
//! ```
//!
//! SVGs are written to `docs/src/assets/dotplot/`.

use kuva::plot::DotPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

const OUT: &str = "docs/src/assets/dotplot";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/dotplot");

    basic();
    matrix_input();
    size_legend_only();
    colorbar_only();

    println!("Dot plot SVGs written to {OUT}/");
}

/// Full bioinformatics gene-expression dot plot.
///
/// Eight marker genes across six major immune cell types.
/// Size = % of cells expressing the gene; color = mean expression level.
/// Both legends stacked in the right margin.
fn basic() {
    let data: Vec<(&str, &str, f64, f64)> = vec![
        // PTPRC (CD45) — pan-leukocyte; broadly expressed
        ("CD4 T",  "PTPRC",  92.0, 4.1), ("CD8 T",  "PTPRC",  95.0, 4.3),
        ("NK",     "PTPRC",  88.0, 3.9), ("B cell", "PTPRC",  91.0, 4.0),
        ("Mono",   "PTPRC",  85.0, 3.7), ("DC",     "PTPRC",  82.0, 3.5),
        // CD3E — pan-T marker
        ("CD4 T",  "CD3E",   88.0, 3.8), ("CD8 T",  "CD3E",   91.0, 4.0),
        ("NK",     "CD3E",   12.0, 0.5), ("B cell", "CD3E",    5.0, 0.2),
        ("Mono",   "CD3E",    3.0, 0.1), ("DC",     "CD3E",    4.0, 0.2),
        // CD4 — helper T marker; low on monocytes/DC (they express it too)
        ("CD4 T",  "CD4",    85.0, 3.5), ("CD8 T",  "CD4",     8.0, 0.3),
        ("NK",     "CD4",     4.0, 0.2), ("B cell", "CD4",     3.0, 0.1),
        ("Mono",   "CD4",    18.0, 0.8), ("DC",     "CD4",    22.0, 1.0),
        // CD8A — cytotoxic T; also on NK subset
        ("CD4 T",  "CD8A",    5.0, 0.2), ("CD8 T",  "CD8A",   82.0, 3.4),
        ("NK",     "CD8A",   15.0, 0.7), ("B cell", "CD8A",    3.0, 0.1),
        ("Mono",   "CD8A",    2.0, 0.1), ("DC",     "CD8A",    4.0, 0.2),
        // GNLY (granulysin) — NK/cytotoxic signature
        ("CD4 T",  "GNLY",    8.0, 0.4), ("CD8 T",  "GNLY",   42.0, 2.1),
        ("NK",     "GNLY",   88.0, 4.5), ("B cell", "GNLY",    3.0, 0.1),
        ("Mono",   "GNLY",    5.0, 0.2), ("DC",     "GNLY",    6.0, 0.3),
        // MS4A1 (CD20) — B cell marker
        ("CD4 T",  "MS4A1",   2.0, 0.1), ("CD8 T",  "MS4A1",   3.0, 0.1),
        ("NK",     "MS4A1",   2.0, 0.1), ("B cell", "MS4A1",  90.0, 4.2),
        ("Mono",   "MS4A1",   5.0, 0.2), ("DC",     "MS4A1",   6.0, 0.3),
        // CD14 — monocyte marker
        ("CD4 T",  "CD14",    3.0, 0.1), ("CD8 T",  "CD14",    2.0, 0.1),
        ("NK",     "CD14",    4.0, 0.2), ("B cell", "CD14",    5.0, 0.2),
        ("Mono",   "CD14",   85.0, 3.9), ("DC",     "CD14",   25.0, 1.2),
        // LYZ (lysozyme) — myeloid marker
        ("CD4 T",  "LYZ",     5.0, 0.2), ("CD8 T",  "LYZ",     4.0, 0.2),
        ("NK",     "LYZ",     8.0, 0.4), ("B cell", "LYZ",     6.0, 0.3),
        ("Mono",   "LYZ",    92.0, 4.4), ("DC",     "LYZ",    78.0, 3.5),
    ];

    let dot = DotPlot::new()
        .with_data(data)
        .with_size_legend("% Expressing")
        .with_colorbar("Mean expression");

    let plots = vec![Plot::DotPlot(dot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Gene Expression Dot Plot")
        .with_x_label("Cell type")
        .with_y_label("Gene");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Dense matrix input via with_matrix.
///
/// sizes[row_i][col_j] maps to y_cat[row_i], x_cat[col_j].
/// All grid cells are filled — no missing combinations.
fn matrix_input() {
    let x_cats = vec!["TypeA", "TypeB", "TypeC", "TypeD", "TypeE"];
    let y_cats = vec!["Gene1", "Gene2", "Gene3", "Gene4", "Gene5", "Gene6"];

    let sizes = vec![
        vec![80.0, 25.0, 60.0, 45.0, 70.0],
        vec![15.0, 90.0, 35.0, 70.0, 20.0],
        vec![55.0, 40.0, 85.0, 20.0, 65.0],
        vec![30.0, 65.0, 10.0, 95.0, 40.0],
        vec![70.0, 50.0, 75.0, 30.0, 88.0],
        vec![45.0, 78.0, 55.0, 60.0, 35.0],
    ];
    let colors = vec![
        vec![3.5, 1.2, 2.8, 2.0, 3.1],
        vec![0.8, 4.1, 1.5, 3.2, 0.9],
        vec![2.4, 1.8, 3.8, 0.9, 2.7],
        vec![1.3, 2.9, 0.5, 4.3, 1.6],
        vec![3.1, 2.2, 3.3, 1.4, 4.0],
        vec![2.0, 3.6, 2.1, 2.8, 1.8],
    ];

    let dot = DotPlot::new()
        .with_matrix(x_cats, y_cats, sizes, colors)
        .with_size_legend("% Expressing")
        .with_colorbar("Mean expression");

    let plots = vec![Plot::DotPlot(dot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Matrix Input")
        .with_x_label("Cell type")
        .with_y_label("Gene");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/matrix.svg"), svg).unwrap();
}

/// Size legend only — dot radius encodes a single variable, no colorbar.
fn size_legend_only() {
    let data: Vec<(&str, &str, f64, f64)> = vec![
        ("CD4 T", "CD3E", 88.0, 0.0), ("CD8 T", "CD3E", 91.0, 0.0),
        ("NK",    "CD3E", 12.0, 0.0), ("Mono",  "CD3E",  3.0, 0.0),
        ("CD4 T", "CD4",  85.0, 0.0), ("CD8 T", "CD4",   8.0, 0.0),
        ("NK",    "CD4",   4.0, 0.0), ("Mono",  "CD4",  18.0, 0.0),
        ("CD4 T", "CD8A",  5.0, 0.0), ("CD8 T", "CD8A", 82.0, 0.0),
        ("NK",    "CD8A", 15.0, 0.0), ("Mono",  "CD8A",  2.0, 0.0),
        ("CD4 T", "CD14",  3.0, 0.0), ("CD8 T", "CD14",  2.0, 0.0),
        ("NK",    "CD14",  4.0, 0.0), ("Mono",  "CD14", 85.0, 0.0),
    ];

    let dot = DotPlot::new()
        .with_data(data)
        .with_size_legend("% Expressing");

    let plots = vec![Plot::DotPlot(dot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Size Legend Only")
        .with_x_label("Cell type")
        .with_y_label("Gene");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/size_legend.svg"), svg).unwrap();
}

/// Colorbar only — all dots the same size, color encodes expression level.
fn colorbar_only() {
    let data: Vec<(&str, &str, f64, f64)> = vec![
        ("CD4 T", "CD3E", 50.0, 3.8), ("CD8 T", "CD3E", 50.0, 4.0),
        ("NK",    "CD3E", 50.0, 0.5), ("Mono",  "CD3E", 50.0, 0.1),
        ("CD4 T", "CD4",  50.0, 3.5), ("CD8 T", "CD4",  50.0, 0.3),
        ("NK",    "CD4",  50.0, 0.2), ("Mono",  "CD4",  50.0, 0.8),
        ("CD4 T", "CD8A", 50.0, 0.2), ("CD8 T", "CD8A", 50.0, 3.4),
        ("NK",    "CD8A", 50.0, 0.7), ("Mono",  "CD8A", 50.0, 0.1),
        ("CD4 T", "CD14", 50.0, 0.1), ("CD8 T", "CD14", 50.0, 0.1),
        ("NK",    "CD14", 50.0, 0.2), ("Mono",  "CD14", 50.0, 3.9),
    ];

    let dot = DotPlot::new()
        .with_data(data)
        .with_colorbar("Mean expression");

    let plots = vec![Plot::DotPlot(dot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Colorbar Only")
        .with_x_label("Cell type")
        .with_y_label("Gene");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/colorbar.svg"), svg).unwrap();
}
