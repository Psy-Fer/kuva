//! DicePlot examples — ports of the ggdiceplot R package demo plots.
//!
//! Run with:
//!
//! ```bash
//! cargo run --example diceplot
//! ```
//!
//! SVGs are written to `docs/src/assets/diceplot/`.

use kuva::backend::svg::SvgBackend;
use kuva::plot::diceplot::DicePlot;
use kuva::plot::heatmap::ColorMap;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use std::sync::Arc;

const OUT: &str = "docs/src/assets/diceplot";

/// ggdiceplot's diverging colour scale: #40004B (purple) → white → #00441B (green).
/// Matches `scale_fill_gradient2(low="#40004B", high="#00441B", mid="white")`.
/// `(taxon, group, cell_index, log2 fold change, -log10 p)` for one dice cell.
type DiceRow = (&'static str, &'static str, usize, Option<f64>, Option<f64>);

fn ggdiceplot_diverging() -> ColorMap {
    ColorMap::Custom(Arc::new(|t: f64| {
        // t in [0,1]: 0 = low (#40004B), 0.5 = mid (white), 1 = high (#00441B)
        let (r, g, b) = if t < 0.5 {
            let s = t * 2.0; // 0→1 within low half
                             // #40004B → #FFFFFF
            let r = 0x40 as f64 + s * (255.0 - 0x40 as f64);
            let g = 0x00 as f64 + s * (255.0 - 0x00 as f64);
            let b = 0x4B as f64 + s * (255.0 - 0x4B as f64);
            (r, g, b)
        } else {
            let s = (t - 0.5) * 2.0; // 0→1 within high half
                                     // #FFFFFF → #00441B
            let r = 255.0 + s * (0x00 as f64 - 255.0);
            let g = 255.0 + s * (0x44 as f64 - 255.0);
            let b = 255.0 + s * (0x1B as f64 - 255.0);
            (r, g, b)
        };
        format!("rgb({},{},{})", r as u8, g as u8, b as u8)
    }))
}

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/diceplot");

    mirna_compound();
    oral_microbiome();
    zebra_domino();
}

/// Categorical mode — miRNA × Compound with organ direction.
///
/// Port of ggdiceplot `sample_dice_miRNA`: 5 miRNAs, 5 compounds, 4 organs.
/// Each dot position = organ, colour = direction (Down / Unchanged / Up).
fn mirna_compound() {
    let organs = vec![
        "Lung".into(),
        "Liver".into(),
        "Brain".into(),
        "Kidney".into(),
    ];

    let records: Vec<(&str, &str, &str, &str)> = vec![
        // miR-1
        ("miR-1", "Control", "Lung", "#2166ac"),
        ("miR-1", "Control", "Liver", "#2166ac"),
        ("miR-1", "Control", "Brain", "#cccccc"),
        ("miR-1", "Control", "Kidney", "#2166ac"),
        ("miR-1", "Compound_1", "Lung", "#2166ac"),
        ("miR-1", "Compound_1", "Liver", "#cccccc"),
        ("miR-1", "Compound_1", "Brain", "#2166ac"),
        ("miR-1", "Compound_2", "Lung", "#cccccc"),
        ("miR-1", "Compound_2", "Kidney", "#b2182b"),
        ("miR-1", "Compound_3", "Lung", "#b2182b"),
        ("miR-1", "Compound_3", "Liver", "#cccccc"),
        ("miR-1", "Compound_3", "Brain", "#b2182b"),
        ("miR-1", "Compound_3", "Kidney", "#b2182b"),
        ("miR-1", "Compound_4", "Lung", "#b2182b"),
        ("miR-1", "Compound_4", "Brain", "#b2182b"),
        ("miR-1", "Compound_4", "Kidney", "#cccccc"),
        // miR-2
        ("miR-2", "Control", "Lung", "#2166ac"),
        ("miR-2", "Control", "Brain", "#2166ac"),
        ("miR-2", "Control", "Kidney", "#cccccc"),
        ("miR-2", "Compound_1", "Lung", "#2166ac"),
        ("miR-2", "Compound_1", "Liver", "#cccccc"),
        ("miR-2", "Compound_1", "Brain", "#2166ac"),
        ("miR-2", "Compound_1", "Kidney", "#2166ac"),
        ("miR-2", "Compound_2", "Lung", "#cccccc"),
        ("miR-2", "Compound_2", "Liver", "#b2182b"),
        ("miR-2", "Compound_2", "Brain", "#cccccc"),
        ("miR-2", "Compound_3", "Lung", "#b2182b"),
        ("miR-2", "Compound_3", "Liver", "#cccccc"),
        ("miR-2", "Compound_3", "Brain", "#b2182b"),
        ("miR-2", "Compound_3", "Kidney", "#b2182b"),
        ("miR-2", "Compound_4", "Liver", "#b2182b"),
        ("miR-2", "Compound_4", "Kidney", "#b2182b"),
        // miR-3
        ("miR-3", "Control", "Lung", "#2166ac"),
        ("miR-3", "Control", "Liver", "#2166ac"),
        ("miR-3", "Control", "Brain", "#cccccc"),
        ("miR-3", "Control", "Kidney", "#2166ac"),
        ("miR-3", "Compound_1", "Lung", "#cccccc"),
        ("miR-3", "Compound_1", "Liver", "#2166ac"),
        ("miR-3", "Compound_1", "Kidney", "#cccccc"),
        ("miR-3", "Compound_2", "Lung", "#b2182b"),
        ("miR-3", "Compound_2", "Liver", "#cccccc"),
        ("miR-3", "Compound_2", "Brain", "#2166ac"),
        ("miR-3", "Compound_2", "Kidney", "#cccccc"),
        ("miR-3", "Compound_3", "Lung", "#b2182b"),
        ("miR-3", "Compound_3", "Liver", "#b2182b"),
        ("miR-3", "Compound_3", "Brain", "#cccccc"),
        ("miR-3", "Compound_4", "Lung", "#b2182b"),
        ("miR-3", "Compound_4", "Liver", "#b2182b"),
        ("miR-3", "Compound_4", "Brain", "#b2182b"),
        ("miR-3", "Compound_4", "Kidney", "#cccccc"),
        // miR-4
        ("miR-4", "Control", "Liver", "#2166ac"),
        ("miR-4", "Control", "Brain", "#2166ac"),
        ("miR-4", "Control", "Kidney", "#2166ac"),
        ("miR-4", "Compound_1", "Lung", "#2166ac"),
        ("miR-4", "Compound_1", "Liver", "#cccccc"),
        ("miR-4", "Compound_1", "Brain", "#2166ac"),
        ("miR-4", "Compound_1", "Kidney", "#cccccc"),
        ("miR-4", "Compound_2", "Lung", "#cccccc"),
        ("miR-4", "Compound_2", "Liver", "#cccccc"),
        ("miR-4", "Compound_2", "Brain", "#b2182b"),
        ("miR-4", "Compound_3", "Lung", "#cccccc"),
        ("miR-4", "Compound_3", "Liver", "#b2182b"),
        ("miR-4", "Compound_3", "Kidney", "#b2182b"),
        ("miR-4", "Compound_4", "Lung", "#b2182b"),
        ("miR-4", "Compound_4", "Liver", "#b2182b"),
        ("miR-4", "Compound_4", "Brain", "#b2182b"),
        ("miR-4", "Compound_4", "Kidney", "#b2182b"),
        // miR-5
        ("miR-5", "Control", "Lung", "#2166ac"),
        ("miR-5", "Control", "Liver", "#cccccc"),
        ("miR-5", "Control", "Kidney", "#2166ac"),
        ("miR-5", "Compound_1", "Lung", "#cccccc"),
        ("miR-5", "Compound_1", "Brain", "#2166ac"),
        ("miR-5", "Compound_1", "Kidney", "#2166ac"),
        ("miR-5", "Compound_2", "Lung", "#cccccc"),
        ("miR-5", "Compound_2", "Liver", "#b2182b"),
        ("miR-5", "Compound_2", "Brain", "#cccccc"),
        ("miR-5", "Compound_2", "Kidney", "#cccccc"),
        ("miR-5", "Compound_3", "Lung", "#b2182b"),
        ("miR-5", "Compound_3", "Liver", "#b2182b"),
        ("miR-5", "Compound_3", "Brain", "#cccccc"),
        ("miR-5", "Compound_3", "Kidney", "#b2182b"),
        ("miR-5", "Compound_4", "Liver", "#b2182b"),
        ("miR-5", "Compound_4", "Brain", "#b2182b"),
        ("miR-5", "Compound_4", "Kidney", "#b2182b"),
    ];

    let dice = DicePlot::new(4)
        .with_category_labels(organs)
        .with_records(records)
        .with_dot_legend(vec![
            ("Down", "#2166ac"),
            ("Unchanged", "#cccccc"),
            ("Up", "#b2182b"),
        ])
        .with_position_legend("Organ");

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("miRNA Compound Screening")
        .with_x_label("miRNA")
        .with_y_label("Compound")
        // Square cells: 5 cols × 5 rows, margins ~107L + ~134R, ~44T + ~51B
        .with_width(741.0)
        .with_height(595.0);

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/mirna_compound.svg"), svg).unwrap();
    println!("Wrote {OUT}/mirna_compound.svg");
}

/// Per-dot continuous — oral microbiome taxa × specimen.
///
/// Port of ggdiceplot `sample_dice_data1`: 6 taxa, 2 specimens, 4 diseases.
/// Each dot position = disease, fill = log2FC, size = -log10(q-value).
fn oral_microbiome() {
    let diseases = vec![
        "Caries".into(),
        "Periodontitis".into(),
        "Healthy".into(),
        "Gingivitis".into(),
    ];

    let data: Vec<DiceRow> = vec![
        // Campylobacter_showae
        ("C. showae", "Saliva", 0, Some(2.55), Some(4.82)),
        ("C. showae", "Saliva", 1, Some(-0.67), Some(1.30)),
        ("C. showae", "Saliva", 2, Some(0.63), Some(2.10)),
        ("C. showae", "Saliva", 3, Some(-1.69), Some(3.52)),
        ("C. showae", "Plaque", 0, Some(1.32), Some(1.89)),
        ("C. showae", "Plaque", 1, Some(-2.14), Some(5.00)),
        ("C. showae", "Plaque", 2, Some(-0.39), Some(0.70)),
        ("C. showae", "Plaque", 3, Some(0.51), Some(2.30)),
        // Porphyromonas_gingivalis
        ("P. gingivalis", "Saliva", 0, Some(-1.44), Some(3.15)),
        ("P. gingivalis", "Saliva", 1, Some(3.01), Some(5.70)),
        ("P. gingivalis", "Saliva", 2, Some(-0.24), Some(0.52)),
        ("P. gingivalis", "Saliva", 3, Some(1.87), Some(2.89)),
        ("P. gingivalis", "Plaque", 0, Some(0.75), Some(1.60)),
        ("P. gingivalis", "Plaque", 1, Some(2.46), Some(4.22)),
        ("P. gingivalis", "Plaque", 2, Some(-1.93), Some(3.70)),
        ("P. gingivalis", "Plaque", 3, Some(0.08), Some(0.40)),
        // Rothia_mucilaginosa
        ("R. mucilaginosa", "Saliva", 0, Some(0.91), Some(2.05)),
        ("R. mucilaginosa", "Saliva", 1, Some(-2.33), Some(4.10)),
        ("R. mucilaginosa", "Saliva", 2, Some(1.44), Some(1.74)),
        ("R. mucilaginosa", "Saliva", 3, Some(-0.82), Some(3.00)),
        ("R. mucilaginosa", "Plaque", 0, Some(-0.15), Some(0.60)),
        ("R. mucilaginosa", "Plaque", 1, Some(1.67), Some(2.55)),
        ("R. mucilaginosa", "Plaque", 2, Some(0.33), Some(1.20)),
        ("R. mucilaginosa", "Plaque", 3, Some(-1.21), Some(4.50)),
        // Fusobacterium_nucleatum
        ("F. nucleatum", "Saliva", 0, Some(1.78), Some(3.80)),
        ("F. nucleatum", "Saliva", 1, Some(-0.52), Some(1.00)),
        ("F. nucleatum", "Saliva", 2, Some(-2.01), Some(5.20)),
        ("F. nucleatum", "Saliva", 3, Some(0.29), Some(0.85)),
        ("F. nucleatum", "Plaque", 0, Some(2.10), Some(4.60)),
        ("F. nucleatum", "Plaque", 1, Some(0.44), Some(1.45)),
        ("F. nucleatum", "Plaque", 2, Some(-1.56), Some(3.30)),
        ("F. nucleatum", "Plaque", 3, Some(1.03), Some(2.70)),
        // Streptococcus_mutans
        ("S. mutans", "Saliva", 0, Some(-0.88), Some(2.40)),
        ("S. mutans", "Saliva", 1, Some(1.95), Some(4.90)),
        ("S. mutans", "Saliva", 2, Some(0.17), Some(0.55)),
        ("S. mutans", "Saliva", 3, Some(-1.34), Some(3.60)),
        ("S. mutans", "Plaque", 0, Some(0.62), Some(1.15)),
        ("S. mutans", "Plaque", 1, Some(-2.47), Some(5.50)),
        ("S. mutans", "Plaque", 2, Some(1.21), Some(2.20)),
        ("S. mutans", "Plaque", 3, Some(-0.05), Some(0.35)),
        // Prevotella_intermedia
        ("P. intermedia", "Saliva", 0, Some(1.12), Some(3.25)),
        ("P. intermedia", "Saliva", 1, Some(-1.78), Some(4.40)),
        ("P. intermedia", "Saliva", 2, Some(0.56), Some(1.50)),
        ("P. intermedia", "Saliva", 3, Some(2.34), Some(5.80)),
        ("P. intermedia", "Plaque", 0, Some(-0.71), Some(2.00)),
        ("P. intermedia", "Plaque", 1, Some(0.89), Some(1.70)),
        ("P. intermedia", "Plaque", 2, Some(-1.45), Some(3.95)),
        ("P. intermedia", "Plaque", 3, Some(1.56), Some(4.15)),
    ];

    let dice = DicePlot::new(4)
        .with_category_labels(diseases)
        .with_dot_data(data)
        .with_color_map(ggdiceplot_diverging())
        .with_fill_legend("Log2FC")
        .with_size_legend("q-value")
        .with_position_legend("Disease");

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Oral Microbiome: Taxa vs Specimen")
        .with_x_label("Taxon")
        .with_y_label("Specimen")
        // Square cells: 6 cols × 2 rows, margins ~78L + ~244R, ~44T + ~51B
        .with_width(1042.0)
        .with_height(335.0);

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/oral_microbiome.svg"), svg).unwrap();
    println!("Wrote {OUT}/oral_microbiome.svg");
}

/// Per-dot continuous — ZEBRA sex DEGs domino plot.
///
/// Port of the ZEBRA domino example: gene × cell type, 5 disease contrasts.
/// Each dot position = contrast, fill = logFC, size = -log10(FDR).
fn zebra_domino() {
    let contrasts = vec![
        "MS-CT".into(),
        "AD-CT".into(),
        "ASD-CT".into(),
        "FTD-CT".into(),
        "HD-CT".into(),
    ];

    let genes = [
        "SPP1", "APOE", "SERPINA1", "PINK1", "ANGPT1", "ANGPT2", "APP", "CLU", "ABCA7",
    ];
    let cell_types = [
        "Astrocyte",
        "Endothelial",
        "Microglia",
        "Neuron",
        "Oligodendrocyte",
    ];

    let mut data: Vec<DiceRow> = Vec::new();

    for (gi, gene) in genes.iter().enumerate() {
        for (ci, cell_type) in cell_types.iter().enumerate() {
            for k in 0..5 {
                let present = ((gi + ci + k) * 7 + 3) % 11 > 2;
                if !present {
                    continue;
                }

                let base = (gi as f64 - 4.0) * 0.4 + (k as f64 - 2.0) * 0.3;
                let cell_effect = (ci as f64 - 2.0) * 0.2;
                let logfc = base + cell_effect;
                let fdr_raw =
                    0.0001 + (gi as f64 * 0.003) + (ci as f64 * 0.002) + (k as f64 * 0.008);
                let sig = -fdr_raw.log10();

                data.push((*gene, *cell_type, k, Some(logfc), Some(sig)));
            }
        }
    }

    let dice = DicePlot::new(5)
        .with_category_labels(contrasts)
        .with_dot_data(data)
        .with_color_map(ggdiceplot_diverging())
        .with_fill_legend("logFC")
        .with_size_legend("-log10(FDR)")
        .with_position_legend("Contrast");

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("ZEBRA Sex DEGs Domino Plot")
        .with_x_label("Gene")
        .with_y_label("Cell Type")
        // Square cells: 9 cols × 5 rows, margins ~143L + ~262R, ~44T + ~51B
        .with_width(1325.0)
        .with_height(595.0);

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/zebra_domino.svg"), svg).unwrap();
    println!("Wrote {OUT}/zebra_domino.svg");
}
