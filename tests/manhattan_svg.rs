use visus::plot::{ManhattanPlot, GenomeBuild, LabelStyle};
use visus::backend::svg::SvgBackend;
use visus::render::render::{render_multiple, render_manhattan};
use visus::render::layout::Layout;
use visus::render::plots::Plot;
use visus::Palette;

// ── Deterministic pseudo-random helpers ──────────────────────────────────────

fn lcg_next(seed: &mut u64) -> f64 {
    *seed = seed.wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
    (*seed >> 33) as f64 / 4_294_967_296.0 // [0, 1)
}

// ── Shared genome layout (hg38-approximate sizes in bp) ─────────────────────

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

/// Association signals: (chrom, center_bp, lead_pvalue).
/// Peaks decay geometrically toward 1e-3 at the edge of a ±5 Mb window.
const SIGNALS: &[(&str, u64, f64)] = &[
    // Genome-wide significant
    ("1",   100_000_000, 3e-10),
    ("3",    50_000_000, 8e-9),
    ("6",   150_000_000, 2e-9),
    ("11",   70_000_000, 5e-10),
    ("15",   40_000_000, 1e-9),
    // Suggestive (between 1e-5 and 5e-8)
    ("2",    80_000_000, 2e-6),
    ("7",   100_000_000, 5e-7),
    ("12",   60_000_000, 1e-6),
    ("18",   30_000_000, 8e-6),
];

fn signal_pvalue(chrom: &str, bp: u64, base_p: f64) -> f64 {
    let window = 5_000_000u64;
    let mut p = base_p;
    for &(sig_chrom, sig_bp, lead_p) in SIGNALS {
        if chrom == sig_chrom {
            let dist = (bp as i64 - sig_bp as i64).unsigned_abs();
            if dist < window {
                let t = dist as f64 / window as f64;
                // geometric interpolation: lead_p at center → 1e-3 at edge
                let signal = lead_p.powf(1.0 - t) * 1e-3_f64.powf(t);
                p = p.min(signal);
            }
        }
    }
    p
}

/// Realistic bp-coordinate GWAS data covering all chr1-22 + X (~100 SNPs each).
fn make_gwas_bp_data() -> Vec<(String, f64, f64)> {
    let n: u64 = 100;
    let mut data = Vec::new();
    let mut seed = 77777u64;
    for &(chrom, size) in CHROMS {
        let step = size / n;
        for i in 0..n {
            let bp = step * i + step / 2;
            let r = lcg_next(&mut seed);
            let base_p = 0.05 + r * 0.85;
            let p = signal_pvalue(chrom, bp, base_p);
            data.push((chrom.to_string(), bp as f64, p));
        }
    }
    data
}

/// Sequential-index GWAS data covering all chr1-22 + X (~80 points each).
fn make_gwas_seq_data() -> Vec<(String, f64)> {
    let n: usize = 80;
    let mut data = Vec::new();
    let mut seed = 11111u64;
    for &(chrom, size) in CHROMS {
        let step = size / n as u64;
        for i in 0..n {
            let bp = step * i as u64 + step / 2;
            let r = lcg_next(&mut seed);
            let base_p = 0.05 + r * 0.85;
            let p = signal_pvalue(chrom, bp, base_p);
            data.push((chrom.to_string(), p));
        }
    }
    data
}

fn write_svg(filename: &str, svg: &str) {
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write(format!("test_outputs/{}", filename), svg).unwrap();
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[test]
fn test_manhattan_sequential() {
    let mp = ManhattanPlot::new()
        .with_data(make_gwas_seq_data());

    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Manhattan — Sequential (all chromosomes)")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    write_svg("manhattan_sequential.svg", &svg);

    assert!(svg.contains("<svg"));
    assert!(svg.contains("4 4")); // threshold lines
}

#[test]
fn test_manhattan_bp_hg38() {
    let mp = ManhattanPlot::new()
        .with_data_bp(make_gwas_bp_data(), GenomeBuild::Hg38);

    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Manhattan — Base-pair coordinates, hg38")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    write_svg("manhattan_bp_hg38.svg", &svg);

    assert!(svg.contains("<svg"));
    assert!(svg.contains("4 4"));
    // All 23 chromosome labels should be present (we put data on all of them)
    assert!(svg.contains(">1<") || svg.contains(">1 "));
    assert!(svg.contains(">22<") || svg.contains(">22 "));
}

#[test]
fn test_manhattan_bp_hg19() {
    let mp = ManhattanPlot::new()
        .with_data_bp(make_gwas_bp_data(), GenomeBuild::Hg19);

    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Manhattan — Base-pair coordinates, hg19")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    write_svg("manhattan_bp_hg19.svg", &svg);

    assert!(svg.contains("<svg"));
}

#[test]
fn test_manhattan_bp_t2t() {
    let mp = ManhattanPlot::new()
        .with_data_bp(make_gwas_bp_data(), GenomeBuild::T2T);

    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Manhattan — Base-pair coordinates, T2T")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    write_svg("manhattan_bp_t2t.svg", &svg);

    assert!(svg.contains("<svg"));
}

#[test]
fn test_manhattan_bp_custom() {
    let custom_build = GenomeBuild::Custom(vec![
        ("chr1".to_string(), 100_000_000),
        ("chr2".to_string(), 80_000_000),
        ("chr3".to_string(), 60_000_000),
        ("chrX".to_string(), 50_000_000),
    ]);

    // 40 SNPs per chromosome
    let mut data = Vec::new();
    let mut seed = 42424u64;
    for (chrom, size) in [("chr1", 100_000_000u64), ("chr2", 80_000_000), ("chr3", 60_000_000), ("chrX", 50_000_000)] {
        let step = size / 40;
        for i in 0u64..40 {
            let bp = step * i + step / 2;
            let r = lcg_next(&mut seed);
            let p = if chrom == "chr2" && bp > 30_000_000 && bp < 50_000_000 {
                r * 1e-7 // signal on chr2
            } else {
                0.05 + r * 0.8
            };
            data.push((chrom.to_string(), bp as f64, p));
        }
    }

    let mp = ManhattanPlot::new().with_data_bp(data, custom_build);

    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Manhattan — Custom build (4 chromosomes)")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    write_svg("manhattan_bp_custom.svg", &svg);

    assert!(svg.contains("<svg"));
}

#[test]
fn test_manhattan_bp_chr_prefix() {
    // Data with "chr" prefix — should be normalised transparently
    let data: Vec<(String, f64, f64)> = make_gwas_bp_data()
        .into_iter()
        .map(|(c, bp, p)| (format!("chr{}", c), bp, p))
        .collect();

    let mp = ManhattanPlot::new().with_data_bp(data, GenomeBuild::Hg38);

    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Manhattan — hg38, chr-prefix input")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    write_svg("manhattan_bp_chr_prefix.svg", &svg);

    assert!(svg.contains("<svg"));
}

#[test]
fn test_manhattan_custom_x() {
    // Pre-computed cumulative x coordinates mimicking 3 chromosomes
    let mut data = Vec::new();
    let mut seed = 55555u64;
    // chr1: x 0..100
    for i in 0..80u64 {
        let r = lcg_next(&mut seed);
        let p = if i > 35 && i < 45 { r * 5e-9 } else { 0.05 + r * 0.8 };
        data.push(("1".to_string(), i as f64, p));
    }
    // chr2: x 110..200
    for i in 0..70u64 {
        let r = lcg_next(&mut seed);
        data.push(("2".to_string(), (110 + i) as f64, 0.05 + r * 0.8));
    }
    // chrX: x 220..300
    for i in 0..60u64 {
        let r = lcg_next(&mut seed);
        let p = if i > 25 && i < 35 { r * 1e-8 } else { 0.05 + r * 0.8 };
        data.push(("X".to_string(), (220 + i) as f64, p));
    }

    let mp = ManhattanPlot::new().with_data_x(data);

    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Manhattan — Pre-computed x")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    write_svg("manhattan_custom_x.svg", &svg);

    assert!(svg.contains("<svg"));
}

#[test]
fn test_manhattan_labels_nudge() {
    let mp = ManhattanPlot::new()
        .with_data(make_gwas_seq_data())
        .with_label_top(10);

    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Manhattan — Top-10 labels, Nudge style")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    write_svg("manhattan_labels_nudge.svg", &svg);

    assert!(svg.contains("<svg"));
    assert!(svg.contains("4 4"));
}

#[test]
fn test_manhattan_labels_arrow() {
    let mp = ManhattanPlot::new()
        .with_data(make_gwas_seq_data())
        .with_label_top(8)
        .with_label_style(LabelStyle::Arrow { offset_x: 10.0, offset_y: 15.0 });

    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Manhattan — Top-8 labels, Arrow style")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    write_svg("manhattan_labels_arrow.svg", &svg);

    assert!(svg.contains("<svg"));
    assert!(svg.contains("#666666")); // leader line colour
}

#[test]
fn test_manhattan_gene_labels() {
    // Use with_data_x so x positions are exact and easy to match against with_point_labels.
    let data: Vec<(&str, f64, f64)> = vec![
        ("1",   50.0, 0.3), ("1",  100.0, 2e-10), ("1",  150.0, 0.6),
        ("2",  200.0, 0.4), ("2",  250.0, 5e-9),  ("2",  300.0, 0.2),
        ("3",  350.0, 0.7), ("3",  400.0, 3e-8),  ("3",  450.0, 0.5),
    ];
    let mp = ManhattanPlot::new()
        .with_data_x(data)
        .with_label_top(3)
        // Attach gene names to the three significant points by (chrom, x).
        .with_point_labels(vec![
            ("1",  100.0, "BRCA1"),
            ("2",  250.0, "TP53"),
            ("3",  400.0, "EGFR"),
        ]);

    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Manhattan — Gene name labels")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    write_svg("manhattan_gene_labels.svg", &svg);

    assert!(svg.contains("BRCA1"), "gene label BRCA1 missing");
    assert!(svg.contains("TP53"),  "gene label TP53 missing");
    assert!(svg.contains("EGFR"),  "gene label EGFR missing");
}

#[test]
fn test_manhattan_labels_exact() {
    let mp = ManhattanPlot::new()
        .with_data(make_gwas_seq_data())
        .with_label_top(5)
        .with_label_style(LabelStyle::Exact);

    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Manhattan — Top-5 labels, Exact style")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    write_svg("manhattan_labels_exact.svg", &svg);

    assert!(svg.contains("<svg"));
}

#[test]
fn test_manhattan_custom_colors() {
    let mp = ManhattanPlot::new()
        .with_data(make_gwas_seq_data())
        .with_color_a("navy")
        .with_color_b("cornflowerblue");

    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Manhattan — Custom alternating colors")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    write_svg("manhattan_custom_colors.svg", &svg);

    assert!(svg.contains("<svg"));
    assert!(svg.contains("navy") || svg.contains("cornflowerblue"));
}

#[test]
fn test_manhattan_palette() {
    let mp = ManhattanPlot::new()
        .with_data(make_gwas_seq_data())
        .with_palette(Palette::tol_bright());

    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Manhattan — Palette (Tol Bright)")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    write_svg("manhattan_palette.svg", &svg);

    assert!(svg.contains("<svg"));
}

#[test]
fn test_manhattan_thresholds() {
    let mp = ManhattanPlot::new()
        .with_data(make_gwas_seq_data())
        .with_genome_wide(8.0)
        .with_suggestive(6.0);

    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Manhattan — Custom thresholds (gw=8, sg=6)")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    write_svg("manhattan_thresholds.svg", &svg);

    assert!(svg.contains("<svg"));
    assert!(svg.contains("#cc3333")); // genome-wide line
    assert!(svg.contains("#888888")); // suggestive line
}

#[test]
fn test_manhattan_legend() {
    let mp = ManhattanPlot::new()
        .with_data(make_gwas_seq_data())
        .with_legend("GWAS Study");

    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Manhattan — Legend")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    write_svg("manhattan_legend.svg", &svg);

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Genome-wide") || svg.contains("Suggestive"));
}

#[test]
fn test_manhattan_render_fn() {
    let mp = ManhattanPlot::new()
        .with_data(make_gwas_seq_data())
        .with_label_top(5);

    // Build layout from a clone for auto-ranging
    let layout = Layout::auto_from_plots(&[Plot::Manhattan(
        ManhattanPlot::new().with_data(make_gwas_seq_data()),
    )])
    .with_title("Manhattan — render_manhattan()")
    .with_y_label("-log10(p-value)");

    let scene = render_manhattan(&mp, &layout);
    let svg = SvgBackend.render_scene(&scene);
    write_svg("manhattan_render_fn.svg", &svg);

    assert!(svg.contains("<svg"));
}

#[test]
fn test_manhattan_pvalue_floor() {
    let mp = ManhattanPlot::new()
        .with_data(make_gwas_seq_data())
        .with_pvalue_floor(1e-12);

    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Manhattan — p-value floor 1e-12")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    write_svg("manhattan_pvalue_floor.svg", &svg);

    assert!(svg.contains("<svg"));
}
