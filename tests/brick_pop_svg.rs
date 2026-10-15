mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::BrickPlot;
use kuva::plot::ColorMap;
use kuva::render::brick_pop::{BrickPopPlot, MetricColumn};
use std::collections::HashMap;

/// A small RFC1-like locus: five frequency-sorted alleles of differing expansion sizes
/// (one interrupted), a frequency bar, and two metric heatbox columns (one with a
/// missing value). Exercises the whole composite: name gutter, freq bar, heatboxes,
/// and the embedded run-length-merged brick column, all row-aligned.
#[test]
fn test_brick_pop_basic() {
    let strigars: Vec<(String, String)> = vec![
        ("AATGG:A".to_string(), "40A".to_string()),
        ("AATGG:A".to_string(), "22A".to_string()),
        ("AATGG:A,AAGGG:B".to_string(), "20A5B15A".to_string()),
        ("AATGG:A".to_string(), "12A".to_string()),
        ("AATGG:A".to_string(), "8A".to_string()),
    ];
    let names = vec!["allele_1", "allele_2", "allele_3", "allele_4", "allele_5"];

    let mut motif_colors: HashMap<String, String> = HashMap::new();
    motif_colors.insert("AATGG".to_string(), "#4c78a8".to_string());
    motif_colors.insert("AAGGG".to_string(), "#e45756".to_string());

    let brick = BrickPlot::new()
        .with_names(names)
        .with_motif_colors(motif_colors)
        .with_strigars(strigars);

    let plot = BrickPopPlot::new(brick)
        .with_title("RFC1-like locus - population alleles")
        .with_frequencies(vec![0.55, 0.20, 0.12, 0.08, 0.05])
        .with_metric(
            "methylation",
            vec![Some(0.9), Some(0.6), None, Some(0.3), Some(0.1)],
            ColorMap::Viridis,
        )
        .with_metric_column(
            MetricColumn::new(
                "motif entropy",
                vec![Some(0.10), Some(0.15), Some(0.80), Some(0.05), Some(0.0)],
                ColorMap::Inferno,
            )
            .with_range(0.0, 1.0),
        );

    let scene = plot.render(900.0);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/brick_pop_basic.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"), "must produce valid SVG");
    // Frequency bar colour, pinned motif colour, and NA cell colour should all appear.
    assert!(svg.contains("#4c78a8"), "freq bars / AATGG motif colour");
    assert!(
        svg.contains("#eeeeee"),
        "NA metric cell colour (allele_3 methylation)"
    );
}

/// Scale test: a 120-allele cohort at an RFC1-like locus with long pure AATGG expansions
/// of widely varying size (frequency-sorted, ragged right edge), run-length merging on,
/// a frequency bar, and three metric columns (one with periodic missing values). Exercises
/// many short rows, the leftward freq bar + axis, and per-column colour scales at scale.
#[test]
fn test_brick_pop_large_cohort() {
    let n = 120usize;
    let mut strigars: Vec<(String, String)> = Vec::with_capacity(n);
    let mut names: Vec<String> = Vec::with_capacity(n);
    let mut freqs: Vec<f64> = Vec::with_capacity(n);
    let mut methyl: Vec<Option<f64>> = Vec::with_capacity(n);
    let mut entropy: Vec<Option<f64>> = Vec::with_capacity(n);
    let mut longest: Vec<Option<f64>> = Vec::with_capacity(n);

    for i in 0..n {
        // Expansion shrinks down the cohort; every 9th allele is interrupted with AAGGG.
        let units = 560usize.saturating_sub(i * 4).max(6);
        if i % 9 == 4 {
            let a = units / 2;
            let b = 6usize;
            let c = units.saturating_sub(a + b);
            strigars.push(("AATGG:A,AAGGG:B".to_string(), format!("{a}A{b}B{c}A")));
        } else {
            strigars.push(("AATGG:A".to_string(), format!("{units}A")));
        }
        names.push(format!("a{:03}", i + 1));
        freqs.push((n - i) as f64); // strictly descending sample counts
        methyl.push(if i % 7 == 0 {
            None
        } else {
            Some((i % 11) as f64 / 10.0)
        });
        entropy.push(Some(((i * 13) % 100) as f64 / 100.0));
        longest.push(Some(units as f64));
    }

    let mut motif_colors: HashMap<String, String> = HashMap::new();
    motif_colors.insert("AATGG".to_string(), "#4c78a8".to_string());
    motif_colors.insert("AAGGG".to_string(), "#e45756".to_string());

    let brick = BrickPlot::new()
        .with_names(names)
        .with_motif_colors(motif_colors)
        .with_merge_runs(true)
        .with_strigars(strigars);

    let plot = BrickPopPlot::new(brick)
        .with_title("RFC1 cohort - 120 alleles")
        .with_row_height(6.0)
        .with_frequency_label("Samples")
        .with_frequencies(freqs)
        .with_metric("methylation", methyl, ColorMap::Viridis)
        .with_metric_column(
            MetricColumn::new("motif entropy", entropy, ColorMap::Inferno).with_range(0.0, 1.0),
        )
        .with_metric("longest pure run", longest, ColorMap::Turbo);

    let scene = plot.render(1000.0);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/brick_pop_large_cohort.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"), "must render a 120-row cohort");
}

/// Stress test for the bottom motif legend: a complex locus with ~31 distinct motifs
/// (the CPUM_TYMS row from the bladerunner contract) plus a few shorter alleles. The
/// embedded `LegendPlot` must wrap the large motif list into several rows without
/// overflowing the canvas.
#[test]
fn test_brick_pop_complex_locus_many_motifs() {
    // 31-motif consensus-like allele (verbatim from the STRIGAR contract doc).
    let big_strigar = "3B1C2B1C4B1C2B3C6B1C2B1D3E1F2A1G3A2H3A1H2A1I4A1J2A1H4A1K5A2H2A1L2M1N2O1P2Q1R1Q2S1T6U2V1W1X1Y2X1Z2T1AA1AB1S2AB1AC1AD1AE2AD";
    let big_motifs = "TGATGG:A,TGGTGA:B,TGGAGA:C,TGGAGATGGT:D,GATGGCGATGGA:E,GATGG:F,TGG:G,AGATGG:H,G:I,AGATGGTGAATGG:J,AGAGG:K,TGAGGGGTGGTGCCT:L,ATC:M,TCGATTGC:N,AC:O,AAAAATGGCAAGTTTAA:P,TAT:Q,GTGTACTT:R,CA:S,ATG:T,A:U,GCT:V,GCGTGGGCCAAGTTACTTGTGCA:W,GGT:X,AAGTGTTCTGCA:Y,TGCCTGCACCTCAGTTGTAGGGTGTCCGTAGGATGTGAGGCCAGTCCCCGGGCTTA:Z,CTTTAAATCCTGCCTAGT:AA,ATT:AB,TCTTGTCGCT:AC,TAA:AD,AAGGCC:AE";

    let strigars: Vec<(String, String)> = vec![
        (big_motifs.to_string(), big_strigar.to_string()),
        ("TGATGG:A,AGATGG:B".to_string(), "20A3B10A".to_string()),
        ("TGATGG:A,TGGTGA:B".to_string(), "14A2B8A".to_string()),
        ("TGATGG:A".to_string(), "9A".to_string()),
    ];
    let names = vec!["consensus", "allele_2", "allele_3", "allele_4"];

    let brick = BrickPlot::new()
        .with_names(names)
        .with_consensus_row(0)
        .with_mark_primary()
        .with_strigars(strigars);

    let plot = BrickPopPlot::new(brick)
        .with_title("CPUM_TYMS - complex locus (many motifs)")
        .with_row_height(22.0)
        .with_frequencies(vec![0.40, 0.30, 0.20, 0.10])
        .with_metric(
            "methylation",
            vec![Some(0.8), Some(0.5), None, Some(0.2)],
            ColorMap::Viridis,
        );

    let scene = plot.render(1000.0);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/brick_pop_complex_locus.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"), "complex locus must render");
    // The long Z motif (56 bp) should appear in the bottom legend.
    assert!(
        svg.contains("TGCCTGCACCTCAGTTGTAGGGTGTCCGTAGGATGTGAGGCCAGTCCCCGGGCTTA"),
        "56 bp Z motif should be listed in the bottom legend"
    );
}

/// Regression for the frequency-bar normalisation bug: when all alleles are singletons
/// (equal proportions, the common small-cohort case), bars must NOT all fill to the max.
/// With the default proportions scale ([0,1]) they should be short and equal, so a
/// polymorphic locus looks obviously different from one dominated by a single allele.
#[test]
fn test_brick_pop_frequency_singletons() {
    let n = 12usize;
    let strigars: Vec<(String, String)> = (0..n)
        .map(|i| ("AATGG:A".to_string(), format!("{}A", 8 + i)))
        .collect();
    let names: Vec<String> = (0..n).map(|i| format!("hap{i}")).collect();
    let freqs: Vec<f64> = vec![1.0 / n as f64; n]; // 12 singletons at ~0.083

    let brick = BrickPlot::new().with_names(names).with_strigars(strigars);
    let plot = BrickPopPlot::new(brick)
        .with_title("CPUM_TYMS - 12 singletons")
        .with_frequencies(freqs);
    let svg = SvgBackend.render_scene(&plot.render(900.0));
    common::write_test_output("test_outputs/brick_pop_singletons.svg", svg.clone()).unwrap();

    // Freq bars use the default colour #4c78a8 (no motif is pinned to it here). Every bar
    // must be short (proportion 0.083 of the ~104 px panel), never near the full width.
    let widths: Vec<f64> = regex_widths(&svg, "#4c78a8");
    assert!(!widths.is_empty(), "expected frequency bars");
    let maxw = widths.iter().cloned().fold(0.0_f64, f64::max);
    assert!(
        maxw < 20.0,
        "singleton bars must be short with the [0,1] default, got max width {maxw}"
    );
}

/// Value-in-box metric mode: the numeric value is written in each cell with the heat on the
/// border. Verifies the values render as text.
#[test]
fn test_brick_pop_metric_values_in_box() {
    let strigars: Vec<(String, String)> = vec![
        ("AATGG:A".to_string(), "20A".to_string()),
        ("AATGG:A".to_string(), "10A".to_string()),
    ];
    let brick = BrickPlot::new()
        .with_names(vec!["a1", "a2"])
        .with_strigars(strigars);
    let plot = BrickPopPlot::new(brick)
        .with_row_height(26.0)
        .with_metric_values(true)
        .with_frequencies(vec![0.7, 0.3])
        .with_metric(
            "methylation",
            vec![Some(0.90), Some(0.20)],
            ColorMap::Viridis,
        );
    let svg = SvgBackend.render_scene(&plot.render(700.0));
    common::write_test_output("test_outputs/brick_pop_values_in_box.svg", svg.clone()).unwrap();
    assert!(
        svg.contains(">0.90<"),
        "cell value 0.90 should be written in the box"
    );
    assert!(
        svg.contains(">0.20<"),
        "cell value 0.20 should be written in the box"
    );
}

/// `sorted_by_frequency` permutes rows, frequencies, and metrics together. Given unsorted
/// input, the highest-frequency allele's name must end up in the top row (smallest y).
#[test]
fn test_brick_pop_sorted_by_frequency() {
    let strigars: Vec<(String, String)> = vec![
        ("AATGG:A".to_string(), "10A".to_string()),
        ("AATGG:A".to_string(), "50A".to_string()),
        ("AATGG:A".to_string(), "30A".to_string()),
    ];
    let brick = BrickPlot::new()
        .with_names(vec!["low", "high", "mid"])
        .with_strigars(strigars);
    let plot = BrickPopPlot::new(brick)
        .with_frequencies(vec![0.1, 0.5, 0.3])
        .sorted_by_frequency();
    let svg = SvgBackend.render_scene(&plot.render(700.0));

    let y = |label: &str| -> f64 {
        let pat = format!(r#">{label}<"#);
        let idx = svg
            .find(&pat)
            .unwrap_or_else(|| panic!("missing label {label}"));
        // walk back to the y=".." of this <text>
        let head = &svg[..idx];
        let ystart = head.rfind(r#" y=""#).unwrap() + 4;
        let yend = head[ystart..].find('"').unwrap() + ystart;
        head[ystart..yend].parse().unwrap()
    };
    assert!(
        y("high") < y("mid") && y("mid") < y("low"),
        "rows must be sorted by descending frequency (high at top)"
    );
}

/// Helper: parse the `width` of every `<rect ... fill="COLOR" ...>` in an SVG string.
fn regex_widths(svg: &str, color: &str) -> Vec<f64> {
    let needle = format!(r#"fill="{color}""#);
    let mut out = Vec::new();
    for chunk in svg.split("<rect ").skip(1) {
        let end = chunk.find('>').unwrap_or(chunk.len());
        let tag = &chunk[..end];
        if tag.contains(&needle) {
            if let Some(ws) = tag.find(r#"width=""#) {
                let s = ws + 7;
                if let Some(e) = tag[s..].find('"') {
                    if let Ok(w) = tag[s..s + e].parse::<f64>() {
                        out.push(w);
                    }
                }
            }
        }
    }
    out
}
