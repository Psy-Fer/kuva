//! Brick plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example brick
//! ```
//!
//! SVGs are written to `docs/src/assets/brick/`.

use std::collections::HashMap;
use kuva::plot::BrickPlot;
use kuva::plot::brick::BrickTemplate;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

const OUT: &str = "docs/src/assets/brick";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/brick");

    dna();
    per_row_offsets();
    custom_template();
    strigar();

    println!("Brick SVGs written to {OUT}/");
}

/// DNA sequences with a global x-offset to align the repeat region.
///
/// `BrickTemplate::dna()` provides a standard A/C/G/T color scheme.
/// `with_x_offset(n)` shifts all rows left by `n` characters so the
/// region of interest starts at x = 0.
fn dna() {
    let sequences = vec![
        "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCATCATCATCATTCAT",
        "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCATCATCATTCAT",
        "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCATCATCATCATTCAT",
        "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATTCAT",
        "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATTCAT",
        "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATCATCATCAT",
    ];

    let names = vec![
        "read_1", "read_2", "read_3", "read_4", "read_5", "read_6",
    ];

    let tmpl = BrickTemplate::new().dna();

    let plot = BrickPlot::new()
        .with_sequences(sequences)
        .with_names(names)
        .with_template(tmpl.template)
        .with_x_offset(18.0);  // skip the 18-base flanking region

    let plots = vec![Plot::Brick(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Brick Plot — DNA Sequences");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/dna.svg"), svg).unwrap();
}

/// Per-row x-offsets — each read starts at a different position.
///
/// `with_x_offsets` accepts one offset per row. Pass `None` for any row
/// that should fall back to the global `x_offset`.
fn per_row_offsets() {
    let sequences = vec![
        "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCAT",  // offset 18
        "GCACTCATCATCATCATCATCATCATCATCATCAT",          // offset 10
        "ATCAGGCCGCACTCATCATCATCATCATCATCATCATCAT",    // offset 16
        "CACTCATCATCATCATCATCAT",                        // offset  5
        "AGGCCGCACTCATCATCATCATCATCATCATCATCATCAT",    // None → global 12
    ];

    let names = vec!["read_1", "read_2", "read_3", "read_4", "read_5"];

    let tmpl = BrickTemplate::new().dna();

    let plot = BrickPlot::new()
        .with_sequences(sequences)
        .with_names(names)
        .with_template(tmpl.template)
        .with_x_offset(12.0)                            // global fallback
        .with_x_offsets(vec![
            Some(18.0_f64), Some(10.0), Some(16.0), Some(5.0), None,
        ]);

    let plots = vec![Plot::Brick(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Brick Plot — Per-row Offsets");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/per_row_offsets.svg"), svg).unwrap();
}

/// Custom character-to-color template.
///
/// `with_template` accepts any `HashMap<char, String>`. Here a protein
/// secondary-structure alphabet (H=helix, E=strand, C=coil, T=turn) is
/// given custom colors. `with_values()` overlays the character label inside
/// each brick.
fn custom_template() {
    let mut tmpl: HashMap<char, String> = HashMap::new();
    tmpl.insert('H', "steelblue".into());    // α-helix
    tmpl.insert('E', "firebrick".into());    // β-strand
    tmpl.insert('C', "#aaaaaa".into());      // coil
    tmpl.insert('T', "seagreen".into());     // turn

    let sequences = vec![
        "CCCCCHHHHHHHHHHCCCCEEEEEECCCTTCCCEEEEECCC",
        "CCCHHHHHHHHHHHHHCCCCEEEEEEECCTCCCEEEEECCC",
        "CCCCHHHHHHHHHCCCCCEEEEEECCCTTCCCEEEEEECCC",
        "CCCHHHHHHHHHHHHCCCCCEEEEECCCTTCCCEEEEEECCC",
        "CCCCHHHHHHHHHCCCCEEEEEECCCTTCCCCEEEEECCCCC",
    ];

    let names = vec!["prot_1", "prot_2", "prot_3", "prot_4", "prot_5"];

    let plot = BrickPlot::new()
        .with_sequences(sequences)
        .with_names(names)
        .with_template(tmpl)
        .with_values();

    let plots = vec![Plot::Brick(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Brick Plot — Protein Secondary Structure");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/custom.svg"), svg).unwrap();
}

/// Strigar mode — structured tandem-repeat motif representations.
///
/// `with_strigars` takes `(motif_string, strigar_string)` pairs, where the
/// motif string maps local letters to k-mers (`CAT:A,C:B`) and the strigar
/// string is a run-length encoding of those letters (`10A1B4A`). The plot
/// normalises k-mers across reads by canonical rotation, assigns global
/// letters, auto-generates colors, and renders variable-width bricks
/// proportional to each motif's nucleotide length.
fn strigar() {
    // Simulated CAT-repeat region with occasional single-nucleotide interruptions.
    let strigars: Vec<(String, String)> = vec![
        ("CAT:A,C:B,T:C".to_string(),   "10A1B4A1C1A".to_string()),
        ("CAT:A,T:B".to_string(),        "14A1B1A".to_string()),
        ("CAT:A,T:B".to_string(),        "14A1B1A".to_string()),
        ("CAT:A,C:B,T:C".to_string(),   "10A1B4A1C1A".to_string()),
        ("CAT:A,C:B,T:C".to_string(),   "10A1B4A1C1A".to_string()),
        ("CAT:A,C:B,GGT:C".to_string(), "10A1B8A1C5A".to_string()),
        ("CAT:A,C:B".to_string(),        "10A1B5A".to_string()),
        ("CAT:A,C:B,T:C".to_string(),   "10A1B4A1C1A".to_string()),
    ];

    let names = vec![
        "read_1", "read_2", "read_3", "read_4",
        "read_5", "read_6", "read_7", "read_8",
    ];

    let plot = BrickPlot::new()
        .with_names(names)
        .with_strigars(strigars);

    let plots = vec![Plot::Brick(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Brick Plot — Strigar Mode (CAT repeats)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/strigar.svg"), svg).unwrap();
}
