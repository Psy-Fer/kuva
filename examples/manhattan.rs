//! Manhattan plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example manhattan
//! ```
//!
//! SVGs are written to `docs/src/assets/manhattan/`.

use kuva::plot::{ManhattanPlot, GenomeBuild, LabelStyle};
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::Palette;

const OUT: &str = "docs/src/assets/manhattan";

// ── Deterministic pseudo-random helpers ──────────────────────────────────────

fn lcg_next(seed: &mut u64) -> f64 {
    *seed = seed.wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
    (*seed >> 33) as f64 / 4_294_967_296.0
}

// ── Shared genome + signal layout ────────────────────────────────────────────

const CHROMS: &[(&str, u64)] = &[
    ("1", 248_956_422), ("2", 242_193_529), ("3", 198_295_559),
    ("4", 190_214_555), ("5", 181_538_259), ("6", 170_805_979),
    ("7", 159_345_973), ("8", 145_138_636), ("9", 138_394_717),
    ("10", 133_797_422), ("11", 135_086_622), ("12", 133_275_309),
    ("13", 114_364_328), ("14", 107_043_718), ("15", 101_991_189),
    ("16", 90_338_345),  ("17", 83_257_441),  ("18", 80_373_285),
    ("19", 58_617_616),  ("20", 64_444_167),  ("21", 46_709_983),
    ("22", 50_818_468),  ("X", 156_040_895),
];

/// Simulated association signals: (chrom, center_bp, lead_pvalue).
const SIGNALS: &[(&str, u64, f64)] = &[
    ("1",   100_000_000, 3e-10),
    ("3",    50_000_000, 8e-9),
    ("6",   150_000_000, 2e-9),
    ("11",   70_000_000, 5e-10),
    ("15",   40_000_000, 1e-9),
    // Suggestive
    ("2",    80_000_000, 2e-6),
    ("7",   100_000_000, 5e-7),
    ("12",   60_000_000, 1e-6),
    ("18",   30_000_000, 8e-6),
];

fn signal_pvalue(chrom: &str, bp: u64, base_p: f64) -> f64 {
    let window = 5_000_000u64;
    let mut p = base_p;
    for &(sc, sb, lp) in SIGNALS {
        if chrom == sc {
            let dist = (bp as i64 - sb as i64).unsigned_abs();
            if dist < window {
                let t = dist as f64 / window as f64;
                let signal = lp.powf(1.0 - t) * 1e-3_f64.powf(t);
                p = p.min(signal);
            }
        }
    }
    p
}

fn gwas_seq_data() -> Vec<(String, f64)> {
    let n: usize = 80;
    let mut data = Vec::new();
    let mut seed = 11111u64;
    for &(chrom, size) in CHROMS {
        let step = size / n as u64;
        for i in 0..n {
            let bp = step * i as u64 + step / 2;
            let r = lcg_next(&mut seed);
            let p = signal_pvalue(chrom, bp, 0.05 + r * 0.85);
            data.push((chrom.to_string(), p));
        }
    }
    data
}

fn gwas_bp_data() -> Vec<(String, f64, f64)> {
    let n: u64 = 100;
    let mut data = Vec::new();
    let mut seed = 77777u64;
    for &(chrom, size) in CHROMS {
        let step = size / n;
        for i in 0..n {
            let bp = step * i + step / 2;
            let r = lcg_next(&mut seed);
            let p = signal_pvalue(chrom, bp, 0.05 + r * 0.85);
            data.push((chrom.to_string(), bp as f64, p));
        }
    }
    data
}

// ── Examples ─────────────────────────────────────────────────────────────────

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/manhattan");

    basic();
    bp_hg38();
    gene_labels();
    custom_build();

    println!("Manhattan SVGs written to {OUT}/");
}

/// Sequential mode — all chromosomes, default thresholds, legend.
///
/// `with_data` accepts `(chrom, pvalue)` pairs. Chromosomes are sorted in
/// standard genomic order (1–22, X, Y, MT). Points within each chromosome
/// receive consecutive integer x positions.
fn basic() {
    let mp = ManhattanPlot::new()
        .with_data(gwas_seq_data())
        .with_legend("GWAS thresholds");

    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("GWAS — Sequential x-coordinates")
        .with_y_label("−log₁₀(p-value)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Base-pair mode with GRCh38 — signals labeled at the top hits.
///
/// `with_data_bp` maps each `(chrom, bp, pvalue)` triplet onto a
/// cumulative genomic x-axis using the chosen `GenomeBuild`. All 23
/// chromosomes appear as labeled spans regardless of whether they contain
/// data (empty chromosomes are still ticked).
fn bp_hg38() {
    let mp = ManhattanPlot::new()
        .with_data_bp(gwas_bp_data(), GenomeBuild::Hg38)
        .with_label_top(10)
        .with_legend("GWAS thresholds");

    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("GWAS — Base-pair coordinates (GRCh38)")
        .with_y_label("−log₁₀(p-value)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/bp_hg38.svg"), svg).unwrap();
}

/// Gene-name labels on specific significant SNPs via `with_point_labels`.
///
/// `with_data_x` accepts `(chrom, x, pvalue)` triplets with pre-computed
/// cumulative x values. `with_point_labels` then attaches a gene name to
/// each significant point by matching (chrom, x) exactly.
fn gene_labels() {
    // Three chromosomes with exact x positions — easy to match.
    let data: Vec<(&str, f64, f64)> = vec![
        // chr1 — BRCA2 locus
        ("1",  10.0, 0.42), ("1",  20.0, 0.18), ("1",  30.0, 0.61),
        ("1",  40.0, 2e-10),("1",  50.0, 3e-8), ("1",  60.0, 0.09),
        ("1",  70.0, 0.55), ("1",  80.0, 0.33),
        // chr2 — TP53 locus
        ("2", 120.0, 0.71), ("2", 130.0, 0.25), ("2", 140.0, 5e-9),
        ("2", 150.0, 4e-8), ("2", 160.0, 0.13), ("2", 170.0, 0.48),
        // chr3 — EGFR locus
        ("3", 220.0, 0.62), ("3", 230.0, 0.38), ("3", 240.0, 3e-8),
        ("3", 250.0, 1e-9), ("3", 260.0, 0.07), ("3", 270.0, 0.51),
    ];

    let mp = ManhattanPlot::new()
        .with_data_x(data)
        .with_label_top(5)
        .with_label_style(LabelStyle::Arrow { offset_x: 10.0, offset_y: 14.0 })
        .with_point_labels(vec![
            ("1",  40.0, "BRCA2"),
            ("2", 140.0, "TP53"),
            ("3", 250.0, "EGFR"),
        ]);

    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Manhattan — Gene-name Labels")
        .with_y_label("−log₁₀(p-value)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/gene_labels.svg"), svg).unwrap();
}

/// Custom genome build — four chromosomes from a non-human organism.
///
/// `GenomeBuild::Custom` accepts a `Vec<(chrom_name, size_in_bp)>` list.
/// Chromosome names are accepted with or without the "chr" prefix.
/// A named palette replaces the default alternating blue scheme.
fn custom_build() {
    let build = GenomeBuild::Custom(vec![
        ("chr1".to_string(), 120_000_000),
        ("chr2".to_string(),  95_000_000),
        ("chr3".to_string(),  80_000_000),
        ("chrX".to_string(),  55_000_000),
    ]);

    let mut seed = 42424u64;
    let mut data = Vec::new();
    for (chrom, size) in [("chr1", 120_000_000u64), ("chr2", 95_000_000),
                           ("chr3", 80_000_000), ("chrX", 55_000_000)] {
        let step = size / 60;
        for i in 0u64..60 {
            let bp = step * i + step / 2;
            let r = lcg_next(&mut seed);
            // Plant a signal on chr2 around 40–50 Mb
            let p = if chrom == "chr2" && bp > 38_000_000 && bp < 52_000_000 {
                r * 5e-9
            } else {
                0.05 + r * 0.80
            };
            data.push((chrom.to_string(), bp as f64, p));
        }
    }

    let mp = ManhattanPlot::new()
        .with_data_bp(data, build)
        .with_palette(Palette::tol_bright())
        .with_label_top(6)
        .with_legend("GWAS thresholds");

    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Manhattan — Custom Genome Build")
        .with_y_label("−log₁₀(p-value)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/custom_build.svg"), svg).unwrap();
}
