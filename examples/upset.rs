//! UpSet plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example upset
//! ```
//!
//! SVGs are written to `docs/src/assets/upset/`.

use kuva::plot::{UpSetPlot, UpSetSort};
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

const OUT: &str = "docs/src/assets/upset";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/upset");

    basic();
    precomputed();
    custom();

    println!("UpSet SVGs written to {OUT}/");
}

/// Four DEG sets from a drug treatment experiment — default frequency sort.
///
/// Each set is a collection of gene IDs. `with_sets` computes all
/// non-empty intersections automatically.
fn basic() {
    // Overlapping differentially expressed genes across four conditions.
    // Constructed so every non-trivial combination of sets is represented.
    let cond_a: Vec<u32> = (1..=40).collect();                                // 40 genes
    let cond_b: Vec<u32> = (21..=55).collect();                               // 35 genes
    let cond_c: Vec<u32> = (31..=58).collect();                               // 28 genes
    let cond_d: Vec<u32> = (1u32..=10).chain(45..=60).collect();              // 26 genes

    let up = UpSetPlot::new()
        .with_sets(vec![
            ("Condition A", cond_a),
            ("Condition B", cond_b),
            ("Condition C", cond_c),
            ("Condition D", cond_d),
        ]);

    let plots = vec![Plot::UpSet(up)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("DEG Overlap Across Treatment Conditions");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Precomputed intersection counts — three variant-calling pipelines.
///
/// `with_data` accepts explicit bitmasks and counts. Bit 0 = GATK,
/// bit 1 = FreeBayes, bit 2 = Strelka. The top 5 intersections are shown
/// using `with_max_visible`; set-size bars are hidden with `without_set_sizes`.
fn precomputed() {
    // Variants detected by each combination of three calling pipelines
    let intersections: Vec<(u64, usize)> = vec![
        (0b001, 45),   // GATK only
        (0b010, 35),   // FreeBayes only
        (0b100, 28),   // Strelka only
        (0b011, 62),   // GATK ∩ FreeBayes
        (0b101, 55),   // GATK ∩ Strelka
        (0b110, 48),   // FreeBayes ∩ Strelka
        (0b111, 118),  // all three (high-confidence)
    ];

    let up = UpSetPlot::new()
        .with_data(
            ["GATK", "FreeBayes", "Strelka"],
            [280usize, 263, 249],
            intersections,
        )
        .with_max_visible(5)
        .without_set_sizes();

    let plots = vec![Plot::UpSet(up)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Variant Calls by Pipeline (top 5 intersections)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/precomputed.svg"), svg).unwrap();
}

/// Degree sort and custom colors — higher-order intersections come first.
///
/// `UpSetSort::ByDegree` prioritises intersections involving more sets,
/// making complex multi-set overlaps immediately visible. Custom bar and
/// dot colors replace the default dark gray.
fn custom() {
    let cond_a: Vec<u32> = (1..=40).collect();
    let cond_b: Vec<u32> = (21..=55).collect();
    let cond_c: Vec<u32> = (31..=58).collect();
    let cond_d: Vec<u32> = (1u32..=10).chain(45..=60).collect();

    let up = UpSetPlot::new()
        .with_sets(vec![
            ("Condition A", cond_a),
            ("Condition B", cond_b),
            ("Condition C", cond_c),
            ("Condition D", cond_d),
        ])
        .with_sort(UpSetSort::ByDegree)
        .with_bar_color("#1d4ed8")
        .with_dot_color("#1e3a8a");

    let plots = vec![Plot::UpSet(up)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("DEG Overlap — Sorted by Degree");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/custom.svg"), svg).unwrap();
}
