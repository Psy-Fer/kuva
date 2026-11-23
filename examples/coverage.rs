//! Coverage plot documentation examples.
//!
//! Generates the canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example coverage
//! ```
//!
//! SVGs are written to `docs/src/assets/coverage/`. The real-data figures read the ARTIC
//! SARS-CoV-2 example data in `examples/data/covar_*.tsv` (derived from Psy-Fer/CoVarPlot).

use kuva::backend::svg::SvgBackend;
use kuva::render::coverage::CoveragePlot;
use kuva::render::track_stack::{Interval, Strand};

const OUT: &str = "docs/src/assets/coverage";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/coverage");
    basic();
    strands();
    covarplot(false);
    covarplot(true);
}

/// A small, self-contained two-sample coverage figure (matches the "Basic usage" doc snippet).
fn basic() {
    let depth = |seed: f64, scale: f64| -> Vec<(f64, f64)> {
        (0..=200)
            .map(|i| {
                let x = 1_000_000.0 + i as f64 / 200.0 * 60_000.0;
                let d = (((x / 2500.0) + seed).sin().abs()
                    + 0.3 * (((x / 900.0) + seed).cos().abs()))
                    * scale
                    + 15.0;
                (x, d)
            })
            .collect()
    };

    let scene = CoveragePlot::new()
        .with_title("Tumour / normal coverage")
        .with_locus(1_000_000.0, 1_060_000.0)
        .with_sample("tumour", depth(0.0, 420.0))
        .with_sample("normal", depth(1.7, 260.0))
        .with_variants(
            "SNV",
            "#d1495b",
            vec![1_012_000.0, 1_028_500.0, 1_041_000.0],
        )
        .with_variants("InDel", "#e9c46a", vec![1_033_000.0])
        .with_feature(1_005_000.0, 1_022_000.0, "amp1")
        .with_feature(1_020_000.0, 1_038_000.0, "amp2")
        .with_feature(1_036_000.0, 1_055_000.0, "amp3")
        .with_x_label("chr7 position")
        .render_sized(820.0, 380.0);

    let svg = SvgBackend::new().render_scene(&scene);
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Gene direction (#126): stranded regions draw as arrows (matches the "Gene direction" doc snippet).
fn strands() {
    let depth: Vec<(f64, f64)> = (0..=200)
        .map(|i| {
            let x = 1_000_000.0 + i as f64 / 200.0 * 40_000.0;
            let d = ((x / 2500.0).sin().abs() + 0.3 * (x / 900.0).cos().abs()) * 300.0 + 15.0;
            (x, d)
        })
        .collect();

    let genes = vec![
        Interval::new(1_001_500.0, 1_009_000.0)
            .with_label("GENE1")
            .with_strand(Strand::Forward),
        Interval::new(1_010_500.0, 1_018_500.0)
            .with_label("GENE2")
            .with_strand(Strand::Reverse),
        Interval::new(1_017_000.0, 1_021_000.0)
            .with_label("GENE3")
            .with_strand(Strand::Forward),
        Interval::new(1_023_000.0, 1_031_500.0)
            .with_label("GENE4")
            .with_strand(Strand::Reverse),
        Interval::new(1_034_500.0, 1_039_000.0).with_label("lncRNA"),
    ];

    let scene = CoveragePlot::new()
        .with_title("Gene direction")
        .with_locus(1_000_000.0, 1_040_000.0)
        .with_sample("tumour", depth)
        .with_regions(genes)
        .with_x_label("chr7 position")
        .render_sized(820.0, 280.0);

    let svg = SvgBackend::new().render_scene(&scene);
    std::fs::write(format!("{OUT}/strands.svg"), svg).unwrap();
}

/// The real ARTIC SARS-CoV-2 figure, stacked or overlaid, from the CoVarPlot example data.
fn covarplot(overlaid: bool) {
    let read = |name: &str| {
        std::fs::read_to_string(format!("examples/data/{name}"))
            .unwrap_or_else(|_| panic!("run from the repo root: examples/data/{name} not found"))
    };

    let (mut pool1, mut pool2) = (Vec::new(), Vec::new());
    for line in read("covar_depth.tsv").lines().skip(1) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 3 {
            continue;
        }
        let pos: f64 = f[0].parse().unwrap();
        pool1.push((pos, f[1].parse().unwrap()));
        pool2.push((pos, f[2].parse().unwrap()));
    }

    let (mut snv, mut indel) = (Vec::new(), Vec::new());
    for line in read("covar_variants.tsv").lines().skip(1) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 2 {
            continue;
        }
        let pos: f64 = f[0].parse().unwrap();
        match f[1] {
            "InDel" => indel.push(pos),
            _ => snv.push(pos),
        }
    }

    let mut cov = CoveragePlot::new()
        .with_sample("pool 1", pool1)
        .with_sample("pool 2", pool2)
        .with_variants("SNV", "#d1495b", snv)
        .with_variants("InDel", "#e9c46a", indel)
        .with_feature_track_name("amplicons")
        .with_region_track_name("genes")
        .with_x_label("MN908947.3");

    for name in ["covar_amplicons.tsv"] {
        for line in read(name).lines().skip(1) {
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() < 3 {
                continue;
            }
            cov = cov.with_feature(f[0].parse().unwrap(), f[1].parse().unwrap(), f[2]);
        }
    }

    let mut genes = Vec::new();
    for line in read("covar_genes.tsv").lines().skip(1) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 3 {
            continue;
        }
        let mut gene = Interval::new(f[0].parse().unwrap(), f[1].parse().unwrap()).with_label(f[2]);
        if let Some(Some(strand)) = f.get(3).map(|t| Strand::parse(t).unwrap()) {
            gene = gene.with_strand(strand);
        }
        genes.push(gene);
    }
    cov = cov.with_regions(genes);

    let (title, file, height) = if overlaid {
        cov = cov.with_overlaid_samples();
        (
            "SARS-CoV-2 coverage — pools overlaid",
            "overlaid.svg",
            420.0,
        )
    } else {
        (
            "SARS-CoV-2 amplicon coverage (ARTIC)",
            "covarplot.svg",
            460.0,
        )
    };

    let scene = cov.with_title(title).render_sized(1100.0, height);
    let svg = SvgBackend::new().render_scene(&scene);
    std::fs::write(format!("{OUT}/{file}"), svg).unwrap();
}
