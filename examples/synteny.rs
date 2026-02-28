//! Synteny plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example synteny
//! ```
//!
//! SVGs are written to `docs/src/assets/synteny/`.

use kuva::plot::synteny::SyntenyPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

const OUT: &str = "docs/src/assets/synteny";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/synteny");

    basic();
    inversions();
    three_seq();
    colors();

    println!("Synteny SVGs written to {OUT}/");
}

/// Pairwise synteny between two sequences, all forward blocks.
fn basic() {
    let plot = SyntenyPlot::new()
        .with_sequences([
            ("Human chr1", 248_956_422.0),
            ("Mouse chr1", 195_471_971.0),
        ])
        .with_block(0,  0.0,       50_000_000.0,  1,  0.0,       45_000_000.0)
        .with_block(0,  60_000_000.0, 120_000_000.0, 1, 55_000_000.0, 100_000_000.0)
        .with_block(0, 130_000_000.0, 200_000_000.0, 1, 110_000_000.0, 170_000_000.0);

    let plots = vec![Plot::Synteny(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Human chr1 vs Mouse chr1");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Forward and inverted blocks — inverted blocks draw crossed (bowtie) ribbons.
fn inversions() {
    let plot = SyntenyPlot::new()
        .with_sequences([("Seq A", 1_000_000.0), ("Seq B", 1_000_000.0)])
        .with_block(    0,       0.0, 200_000.0, 1,       0.0, 200_000.0)
        .with_inv_block(0, 250_000.0, 500_000.0, 1, 250_000.0, 500_000.0)
        .with_block(    0, 600_000.0, 900_000.0, 1, 600_000.0, 900_000.0);

    let plots = vec![Plot::Synteny(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Forward and Inverted Blocks");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/inversions.svg"), svg).unwrap();
}

/// Three sequences with synteny blocks between adjacent pairs.
fn three_seq() {
    let plot = SyntenyPlot::new()
        .with_sequences([
            ("Genome A", 500_000.0),
            ("Genome B", 480_000.0),
            ("Genome C", 450_000.0),
        ])
        .with_block(    0,       0.0, 100_000.0, 1,       0.0,  95_000.0)
        .with_block(    0, 150_000.0, 300_000.0, 1, 140_000.0, 280_000.0)
        .with_block(    1,       0.0, 100_000.0, 2,   5_000.0, 105_000.0)
        .with_inv_block(1, 200_000.0, 350_000.0, 2, 190_000.0, 340_000.0);

    let plots = vec![Plot::Synteny(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Three-way Synteny");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/three_seq.svg"), svg).unwrap();
}

/// Custom sequence bar colors, per-block colors, and a legend.
fn colors() {
    let plot = SyntenyPlot::new()
        .with_sequences([("Seq 1", 500_000.0), ("Seq 2", 500_000.0)])
        .with_sequence_colors(["#4393c3", "#d6604d"])
        .with_colored_block(    0,       0.0, 150_000.0, 1,       0.0, 150_000.0, "#2ca02c")
        .with_colored_block(    0, 200_000.0, 350_000.0, 1, 200_000.0, 340_000.0, "#9467bd")
        .with_colored_inv_block(0, 380_000.0, 480_000.0, 1, 370_000.0, 470_000.0, "#ff7f0e")
        .with_legend("Blocks");

    let plots = vec![Plot::Synteny(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Custom Colors & Legend");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/colors.svg"), svg).unwrap();
}
