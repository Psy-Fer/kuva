//! Venn diagram documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example venn
//! ```
//!
//! SVGs are written to `docs/src/assets/venn/`.

use kuva::backend::svg::SvgBackend;
use kuva::plot::venn::VennPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use std::fs;

const OUT: &str = "docs/src/assets/venn";

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all(OUT).unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

fn main() {
    // ── 1. Basic 2-set: two gene lists ───────────────────────────────────────
    let deseq2_genes = vec![
        "BRCA1", "TP53", "MYC", "EGFR", "VEGFA", "CDKN2A", "KRAS", "PTEN", "MDM2", "RB1", "CCND1",
        "CDK4",
    ];
    let edger_genes = vec![
        "TP53", "MYC", "KRAS", "PIK3CA", "PTEN", "RB1", "AKT1", "MTOR", "CDK4", "CCND1", "ERBB2",
    ];

    let venn = VennPlot::new()
        .with_set(
            "DESeq2",
            deseq2_genes.iter().map(|s| s.to_string()).collect(),
        )
        .with_set("edgeR", edger_genes.iter().map(|s| s.to_string()).collect())
        .with_percentages(true);
    let plots = vec![Plot::Venn(venn)];
    let layout = Layout::auto_from_plots(&plots).with_title("DE Gene Overlap: DESeq2 vs edgeR");
    write("basic_2set", plots, layout);
    println!("  basic_2set");

    // ── 2. Basic 3-set: three experimental gene lists ─────────────────────────
    let deseq2 = ["BRCA1", "TP53", "MYC", "EGFR", "VEGFA", "CDKN2A", "KRAS"];
    let edger = ["TP53", "MYC", "KRAS", "PIK3CA", "PTEN", "RB1"];
    let limma = ["BRCA1", "MYC", "EGFR", "PIK3CA", "CDKN2A", "MDM2"];

    let venn = VennPlot::new()
        .with_set("DESeq2", deseq2.iter().map(|s| s.to_string()).collect())
        .with_set("edgeR", edger.iter().map(|s| s.to_string()).collect())
        .with_set("limma", limma.iter().map(|s| s.to_string()).collect())
        .with_counts(true)
        .with_percentages(true);
    let plots = vec![Plot::Venn(venn)];
    let layout = Layout::auto_from_plots(&plots).with_title("DE Gene Overlap Across Methods");
    write("basic_3set", plots, layout);
    println!("  basic_3set");

    // ── 3. Proportional mode with loss display ────────────────────────────────
    let venn = VennPlot::new()
        .with_set_size("Proteomics", 850)
        .with_set_size("Transcriptomics", 1200)
        .with_set_size("Metabolomics", 600)
        .with_overlap(["Proteomics", "Transcriptomics"], 320)
        .with_overlap(["Proteomics", "Metabolomics"], 180)
        .with_overlap(["Transcriptomics", "Metabolomics"], 250)
        .with_overlap(["Proteomics", "Transcriptomics", "Metabolomics"], 90)
        .with_proportional(true)
        .with_loss(true)
        .with_counts(true);
    let plots = vec![Plot::Venn(venn)];
    let layout = Layout::auto_from_plots(&plots).with_title("Multi-omics Overlap (Proportional)");
    write("proportional", plots, layout);
    println!("  proportional");

    // ── 4. 4-set with leader lines ────────────────────────────────────────────
    // A 4-set diagram has 15 regions; the small central intersections are too
    // crowded to label inline, so leader_lines=true gives them external labels.
    let venn = VennPlot::new()
        .with_set_size("Condition A", 400)
        .with_set_size("Condition B", 350)
        .with_set_size("Condition C", 300)
        .with_set_size("Condition D", 250)
        .with_overlap(["Condition A", "Condition B"], 120)
        .with_overlap(["Condition A", "Condition C"], 90)
        .with_overlap(["Condition A", "Condition D"], 70)
        .with_overlap(["Condition B", "Condition C"], 100)
        .with_overlap(["Condition B", "Condition D"], 80)
        .with_overlap(["Condition C", "Condition D"], 60)
        .with_overlap(["Condition A", "Condition B", "Condition C"], 35)
        .with_overlap(["Condition A", "Condition B", "Condition D"], 25)
        .with_overlap(["Condition A", "Condition C", "Condition D"], 20)
        .with_overlap(["Condition B", "Condition C", "Condition D"], 30)
        .with_overlap(
            ["Condition A", "Condition B", "Condition C", "Condition D"],
            10,
        )
        .with_counts(true)
        .with_legend("Conditions")
        .with_leader_lines(true);
    let plots = vec![Plot::Venn(venn)];
    let layout = Layout::auto_from_plots(&plots).with_title("4-Set Venn with Leader Lines");
    write("leader_lines", plots, layout);
    println!("  leader_lines");

    // ── 5. 4-set with pre-computed sizes ─────────────────────────────────────
    let venn = VennPlot::new()
        .with_set_size("Condition A", 400)
        .with_set_size("Condition B", 350)
        .with_set_size("Condition C", 300)
        .with_set_size("Condition D", 250)
        .with_overlap(["Condition A", "Condition B"], 120)
        .with_overlap(["Condition A", "Condition C"], 90)
        .with_overlap(["Condition A", "Condition D"], 70)
        .with_overlap(["Condition B", "Condition C"], 100)
        .with_overlap(["Condition B", "Condition D"], 80)
        .with_overlap(["Condition C", "Condition D"], 60)
        .with_overlap(["Condition A", "Condition B", "Condition C"], 35)
        .with_overlap(["Condition A", "Condition B", "Condition D"], 25)
        .with_overlap(["Condition A", "Condition C", "Condition D"], 20)
        .with_overlap(["Condition B", "Condition C", "Condition D"], 30)
        .with_overlap(
            ["Condition A", "Condition B", "Condition C", "Condition D"],
            10,
        )
        .with_counts(true)
        .with_legend("Conditions");
    let plots = vec![Plot::Venn(venn)];
    let layout = Layout::auto_from_plots(&plots).with_title("4-Set Venn Diagram");
    write("four_set", plots, layout);
    println!("  four_set");
}
