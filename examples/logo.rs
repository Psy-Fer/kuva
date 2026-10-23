//! Sequence logo documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example logo --features full
//! ```
//!
//! SVGs are written to `docs/src/assets/logo/`.

use kuva::backend::svg::SvgBackend;
use kuva::plot::logo::{Alphabet, LogoUnits, PalettePreset, SequenceLogo};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use std::fs;

const OUT: &str = "docs/src/assets/logo";

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all(OUT).unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

fn main() {
    // ── Kozak motif (default: bits, classic palette) ──────────────────────
    let kozak = vec![
        "GCCACCATGG",
        "CCCACCATGG",
        "GCCGCCATGG",
        "GCCACCATGA",
        "GCCACCATGG",
        "GCTGCCATGA",
        "GCAACCATGG",
        "GCCTCCATGG",
        "GCCACCATGG",
        "GACGCCATGA",
    ];
    let logo = SequenceLogo::from_sequences(kozak.clone(), Alphabet::Dna).unwrap();
    let plots: Vec<Plot> = vec![logo.into()];
    let mut layout = Layout::auto_from_plots(&plots);
    layout.title = Some("Kozak motif (bits)".to_string());
    layout.x_label = Some("Position".to_string());
    layout.y_label = Some("Information (bits)".to_string());
    layout.width = Some(560.0);
    layout.height = Some(280.0);
    write("kozak_bits", plots, layout);

    // ── Kozak motif in probability space ──────────────────────────────────
    let logo = SequenceLogo::from_sequences(kozak.clone(), Alphabet::Dna)
        .unwrap()
        .with_units(LogoUnits::Probability);
    let plots: Vec<Plot> = vec![logo.into()];
    let mut layout = Layout::auto_from_plots(&plots);
    layout.title = Some("Kozak motif (probability)".to_string());
    layout.x_label = Some("Position".to_string());
    layout.y_label = Some("Probability".to_string());
    layout.width = Some(560.0);
    layout.height = Some(280.0);
    write("kozak_probability", plots, layout);

    // ── Genomic coordinates via --start ───────────────────────────────────
    let logo = SequenceLogo::from_sequences(kozak.clone(), Alphabet::Dna)
        .unwrap()
        .with_start(1_234_567)
        .with_consensus()
        .with_consensus_threshold(0.6);
    let plots: Vec<Plot> = vec![logo.into()];
    let mut layout = Layout::auto_from_plots(&plots);
    layout.title = Some("Kozak motif at chr7:1234567".to_string());
    layout.x_label = Some("Genomic position".to_string());
    layout.y_label = Some("Information (bits)".to_string());
    layout.width = Some(680.0);
    layout.height = Some(300.0);
    write("kozak_genomic", plots, layout);

    // ── Protein motif (default per-residue palette, auto-widened canvas) ──
    let logo =
        SequenceLogo::from_fasta_str(include_str!("data/logo_protein.fasta"), Alphabet::Protein)
            .unwrap();
    let plots: Vec<Plot> = vec![logo.into()];
    let mut layout = Layout::auto_from_plots(&plots);
    layout.title = Some("C2H2 zinc finger (protein, default palette)".to_string());
    layout.x_label = Some("Position".to_string());
    layout.y_label = Some("Information (bits)".to_string());
    // No explicit width: 25 positions trigger the ~48 px/position auto-width.
    layout.height = Some(340.0);
    write("zinc_finger_protein", plots, layout);

    // ── Reverse-complement view ───────────────────────────────────────────
    let logo = SequenceLogo::from_sequences(kozak.clone(), Alphabet::Dna)
        .unwrap()
        .with_reverse_complement()
        .unwrap();
    let plots: Vec<Plot> = vec![logo.into()];
    let mut layout = Layout::auto_from_plots(&plots);
    layout.title = Some("Kozak motif — reverse complement".to_string());
    layout.x_label = Some("Position (RC)".to_string());
    layout.y_label = Some("Information (bits)".to_string());
    layout.width = Some(560.0);
    layout.height = Some(280.0);
    write("kozak_reverse_complement", plots, layout);

    // ── Palette override (mono) ───────────────────────────────────────────
    let logo = SequenceLogo::from_sequences(kozak, Alphabet::Dna)
        .unwrap()
        .with_preset(PalettePreset::Mono);
    let plots: Vec<Plot> = vec![logo.into()];
    let mut layout = Layout::auto_from_plots(&plots);
    layout.title = Some("Kozak motif — mono palette".to_string());
    layout.x_label = Some("Position".to_string());
    layout.y_label = Some("Information (bits)".to_string());
    layout.width = Some(560.0);
    layout.height = Some(280.0);
    write("kozak_mono", plots, layout);

    println!("Logo SVGs written to {OUT}/");
}
