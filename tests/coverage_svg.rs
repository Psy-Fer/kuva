//! SVG-output tests for the `CoveragePlot` genomics preset (issue #2), the library path. Writes
//! figures to `test_outputs/` for local visual inspection (skipped under CI). Mirrors the CLI
//! `kuva coverage` smoke outputs so the library and CLI paths can be compared side by side.
mod common;

use kuva::backend::svg::SvgBackend;
use kuva::render::coverage::CoveragePlot;
use kuva::render::track_stack::{Interval, Strand};

fn sample(seed: f64, scale: f64) -> Vec<(f64, f64)> {
    (0..=200)
        .map(|i| {
            let x = 1_000_000.0 + i as f64 / 200.0 * 60_000.0;
            let d = (((x / 2500.0) + seed).sin().abs() + 0.3 * (((x / 900.0) + seed).cos().abs()))
                * scale
                + 15.0;
            (x, d)
        })
        .collect()
}

/// The full figure: two depth tracks + variant lane + amplicon bands + genomic axis.
#[test]
fn coverage_full() {
    let scene = CoveragePlot::new()
        .with_title("Tumour / normal coverage — chr7")
        .with_locus(1_000_000.0, 1_060_000.0)
        .with_sample("tumour", sample(0.0, 420.0))
        .with_sample("normal", sample(1.7, 260.0))
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
        .render_sized(960.0, 400.0);

    let svg = SvgBackend::new().render_scene(&scene);
    common::write_test_output("test_outputs/coverage_full.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Tumour / normal coverage"));
    assert!(svg.contains("tumour") && svg.contains("normal"));
    assert!(svg.contains("SNV") && svg.contains("amp1"));
}

/// Depth tracks only (no variants/features) — the minimal multi-sample case.
#[test]
fn coverage_multi_sample() {
    let scene = CoveragePlot::new()
        .with_locus(1_000_000.0, 1_060_000.0)
        .with_sample("tumour", sample(0.0, 420.0))
        .with_sample("normal", sample(1.7, 260.0))
        .with_x_label("chr7 position")
        .render_sized(960.0, 300.0);

    let svg = SvgBackend::new().render_scene(&scene);
    common::write_test_output("test_outputs/coverage_multi_sample.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("tumour") && svg.contains("normal"));
}

/// Build a `CoveragePlot` from the real CoVarPlot example data (ARTIC nCoV-2019 scheme): two
/// per-pool depths, real SNV/InDel calls, the 14 real amplicons, and the SARS-CoV-2 genes. Shared
/// by the stacked and overlaid figures below. Data in `examples/data/covar_*.tsv` (depth
/// downsampled ×10; derived from Psy-Fer/CoVarPlot `example/`).
fn covarplot_from_real_data() -> CoveragePlot {
    let dir = format!("{}/examples/data", env!("CARGO_MANIFEST_DIR"));
    let read = |name: &str| std::fs::read_to_string(format!("{dir}/{name}")).unwrap();

    // Depth: pos, pool1, pool2 (skip header).
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

    // Variants: pos, type -> grouped by type.
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

    // Amplicons: start, end, label -> real ARTIC scheme (alternating pools => 2 tiled rows).
    for line in read("covar_amplicons.tsv").lines().skip(1) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 3 {
            continue;
        }
        cov = cov.with_feature(f[0].parse().unwrap(), f[1].parse().unwrap(), f[2]);
    }

    // Genes/ORFs: the real SARS-CoV-2 genome annotation (big genes labelled; tiny 3' ORFs stay
    // bars). Every SARS-CoV-2 gene is on the + strand, so all draw as right-pointing arrows.
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

    cov
}

/// REAL DATA, one depth track per pool (stacked).
#[test]
fn coverage_covarplot_real() {
    let scene = covarplot_from_real_data()
        .with_title("SARS-CoV-2 amplicon coverage (CoVarPlot example)")
        .render_sized(1100.0, 460.0);
    let svg = SvgBackend::new().render_scene(&scene);
    common::write_test_output("test_outputs/coverage_covarplot_real.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("pool 1") && svg.contains("pool 2"));
    assert!(svg.contains("SNV") && svg.contains("InDel"));
    assert!(svg.contains("genes") && svg.contains("ORF1ab")); // gene track + a fitting label
    assert!(svg.contains(" kb")); // ~30 kb genome -> kb ticks
}

/// REAL DATA, both pools OVERLAID in one shared depth track (common y-axis) — for directly
/// comparing pool 1 vs pool 2 coverage. Same data, `.with_overlaid_samples()`.
#[test]
fn coverage_covarplot_shared_pools() {
    let scene = covarplot_from_real_data()
        .with_overlaid_samples()
        .with_title("SARS-CoV-2 coverage — pools overlaid")
        .render_sized(1100.0, 420.0);
    let svg = SvgBackend::new().render_scene(&scene);
    common::write_test_output(
        "test_outputs/coverage_covarplot_shared_pools.svg",
        svg.clone(),
    )
    .unwrap();
    assert!(svg.contains("<svg"));
    // Both pools present (as legend entries on the one shared track).
    assert!(svg.contains("pool 1") && svg.contains("pool 2"));
    assert!(svg.contains("genes") && svg.contains("ORF1ab"));
}

/// Coverage threshold lines draw one dashed reference line per depth track.
#[test]
fn coverage_threshold_lines() {
    let scene = CoveragePlot::new()
        .with_title("Coverage with min-depth threshold")
        .with_locus(1_000_000.0, 1_060_000.0)
        .with_sample("pool 1", sample(0.0, 420.0))
        .with_sample("pool 2", sample(1.7, 260.0))
        .with_coverage_threshold_labeled(50.0, "min 50x")
        .with_x_label("position")
        .render_sized(960.0, 400.0);

    let svg = SvgBackend::new().render_scene(&scene);
    common::write_test_output("test_outputs/coverage_threshold.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // One dashed threshold line per stacked depth track (2 samples).
    assert_eq!(
        svg.matches("stroke-dasharray=\"6 4\"").count(),
        2,
        "expected one threshold line per depth track"
    );
    assert!(svg.contains(">min 50x<"), "threshold label missing");
}

/// Gene direction (#126): stranded regions draw as arrows pointing the way each gene is
/// transcribed. Covers forward, reverse, unstranded, a gene too narrow for its label and one
/// narrower than the arrowhead (drawn as a bare triangle), plus overlapping genes that tile
/// onto two rows.
#[test]
fn coverage_gene_strands() {
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
        Interval::new(1_033_000.0, 1_033_600.0)
            .with_label("small")
            .with_strand(Strand::Reverse),
        Interval::new(1_034_500.0, 1_039_000.0).with_label("lncRNA"),
    ];
    let scene = CoveragePlot::new()
        .with_title("Gene direction")
        .with_locus(1_000_000.0, 1_040_000.0)
        .with_sample("tumour", sample(0.0, 300.0))
        .with_regions(genes)
        .with_x_label("chr7 position")
        .render_sized(960.0, 300.0);
    let svg = SvgBackend::new().render_scene(&scene);
    common::write_test_output("test_outputs/coverage_gene_strands.svg", svg.clone()).unwrap();

    // Five stranded genes are arrow paths; the unstranded lncRNA stays a plain box.
    let arrows: Vec<&str> = svg
        .split("<path d=\"M ")
        .skip(1)
        .filter(|p| p.split('"').next().unwrap().matches(" L ").count() == 4)
        .collect();
    assert_eq!(arrows.len(), 5, "expected 5 arrow-shaped genes");
    for label in ["GENE1", "GENE2", "GENE3", "GENE4", "lncRNA"] {
        assert!(svg.contains(&format!(">{label}<")), "missing label {label}");
    }
}
