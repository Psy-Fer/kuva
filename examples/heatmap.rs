//! Heatmap documentation examples.
//!
//! Generates canonical SVG outputs used in the visus documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example heatmap
//! ```
//!
//! SVGs are written to `docs/src/assets/heatmap/`.

use visus::plot::{Heatmap, ColorMap};
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

const OUT: &str = "docs/src/assets/heatmap";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/heatmap");

    basic();
    labeled();
    values();
    colormaps();

    println!("Heatmap SVGs written to {OUT}/");
}

/// Minimal heatmap — data only, Viridis colormap, no axis labels.
fn basic() {
    let data = vec![
        vec![0.8, 0.3, 0.9, 0.2, 0.6],
        vec![0.4, 0.7, 0.1, 0.8, 0.3],
        vec![0.5, 0.9, 0.4, 0.6, 0.1],
        vec![0.2, 0.5, 0.8, 0.3, 0.7],
    ];

    let heatmap = Heatmap::new().with_data(data);

    let plots = vec![Plot::Heatmap(heatmap)];
    let layout = Layout::auto_from_plots(&plots).with_title("Heatmap");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Heatmap with row and column axis labels via Layout::with_x/y_categories.
fn labeled() {
    let data = vec![
        vec![2.1, 0.4, 3.2, 1.1, 2.8],
        vec![0.9, 3.5, 0.3, 2.7, 1.2],
        vec![1.8, 2.9, 1.5, 0.6, 3.1],
        vec![3.3, 1.1, 2.0, 3.8, 0.5],
    ];

    let row_labels = ["GeneA", "GeneB", "GeneC", "GeneD"]
        .map(String::from).to_vec();
    let col_labels = ["Ctrl", "T1", "T2", "T3", "T4"]
        .map(String::from).to_vec();

    let heatmap = Heatmap::new().with_data(data);
    let plots = vec![Plot::Heatmap(heatmap)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Gene Expression Heatmap")
        .with_x_categories(col_labels)
        .with_y_categories(row_labels);

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/labeled.svg"), svg).unwrap();
}

/// Value overlay — numeric values printed inside each cell.
fn values() {
    let data = vec![
        vec![10.0, 20.0, 30.0, 15.0],
        vec![45.0, 55.0, 25.0, 60.0],
        vec![70.0, 35.0, 80.0, 40.0],
        vec![50.0, 90.0, 65.0, 20.0],
    ];

    let heatmap = Heatmap::new()
        .with_data(data)
        .with_values();

    let plots = vec![Plot::Heatmap(heatmap)];
    let layout = Layout::auto_from_plots(&plots).with_title("Value Overlay");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/values.svg"), svg).unwrap();
}

/// Three color maps applied to the same data.
fn colormaps() {
    let data = vec![
        vec![10.0, 20.0, 30.0, 40.0],
        vec![50.0, 60.0, 70.0, 80.0],
        vec![90.0, 75.0, 55.0, 35.0],
        vec![15.0, 40.0, 65.0, 85.0],
    ];

    for (cmap, name, title) in [
        (ColorMap::Viridis,   "viridis",   "Viridis"),
        (ColorMap::Inferno,   "inferno",   "Inferno"),
        (ColorMap::Grayscale, "greyscale", "Greyscale"),
    ] {
        let heatmap = Heatmap::new()
            .with_data(data.clone())
            .with_color_map(cmap);

        let plots = vec![Plot::Heatmap(heatmap)];
        let layout = Layout::auto_from_plots(&plots).with_title(title);

        let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
        std::fs::write(format!("{OUT}/colormap_{name}.svg"), svg).unwrap();
    }
}
