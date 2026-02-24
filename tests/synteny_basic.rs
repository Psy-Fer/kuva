use visus::plot::synteny::SyntenyPlot;
use visus::render::{plots::Plot, layout::Layout};
use visus::backend::svg::SvgBackend;
use visus::render_synteny;
use std::fs;

fn save(name: &str, svg: &str) {
    fs::create_dir_all("test_outputs").unwrap();
    fs::write(format!("test_outputs/{}.svg", name), svg).unwrap();
}

fn render(plot: &SyntenyPlot, layout: Layout) -> String {
    SvgBackend.render_scene(&render_synteny(plot, &layout))
}

/// Two sequences, three forward blocks — basic smoke test.
#[test]
fn synteny_pairwise_forward() {
    let plot = SyntenyPlot::new()
        .with_sequences([("Human chr1", 248_956_422.0), ("Mouse chr1", 195_471_971.0)])
        .with_block(0, 0.0, 50_000_000.0, 1, 0.0, 45_000_000.0)
        .with_block(0, 60_000_000.0, 120_000_000.0, 1, 55_000_000.0, 100_000_000.0)
        .with_block(0, 130_000_000.0, 200_000_000.0, 1, 110_000_000.0, 170_000_000.0);

    let layout = Layout::auto_from_plots(&[Plot::Synteny(plot.clone())]);
    let svg = render(&plot, layout);
    save("synteny_pairwise_forward", &svg);

    assert!(svg.contains("<svg"), "Missing SVG root");
    assert!(svg.contains("Human chr1"), "Missing sequence label");
    assert!(svg.contains("Mouse chr1"), "Missing sequence label");
    let path_count = svg.matches("<path").count();
    assert!(path_count >= 3, "Expected ≥3 paths, got {path_count}");
}

/// Mixed forward and inverted blocks — check crossed ribbon (bowtie) paths.
#[test]
fn synteny_with_inversions() {
    let plot = SyntenyPlot::new()
        .with_sequences([("Seq A", 1_000_000.0), ("Seq B", 1_000_000.0)])
        .with_block(0, 0.0, 200_000.0, 1, 0.0, 200_000.0)
        .with_inv_block(0, 250_000.0, 500_000.0, 1, 250_000.0, 500_000.0)
        .with_block(0, 600_000.0, 900_000.0, 1, 600_000.0, 900_000.0);

    let layout = Layout::auto_from_plots(&[Plot::Synteny(plot.clone())]);
    let svg = render(&plot, layout);
    save("synteny_with_inversions", &svg);

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Seq A"));
    assert!(svg.contains("Seq B"));
    let path_count = svg.matches("<path").count();
    assert!(path_count >= 3, "Expected ≥3 paths, got {path_count}");
}

/// Three sequences with blocks between adjacent pairs.
#[test]
fn synteny_three_sequences() {
    let plot = SyntenyPlot::new()
        .with_sequences([
            ("Genome A", 500_000.0),
            ("Genome B", 480_000.0),
            ("Genome C", 450_000.0),
        ])
        .with_block(0, 0.0, 100_000.0, 1, 0.0, 95_000.0)
        .with_block(0, 150_000.0, 300_000.0, 1, 140_000.0, 280_000.0)
        .with_block(1, 0.0, 100_000.0, 2, 5_000.0, 105_000.0)
        .with_inv_block(1, 200_000.0, 350_000.0, 2, 190_000.0, 340_000.0);

    let layout = Layout::auto_from_plots(&[Plot::Synteny(plot.clone())]);
    let svg = render(&plot, layout);
    save("synteny_three_sequences", &svg);

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Genome A"));
    assert!(svg.contains("Genome B"));
    assert!(svg.contains("Genome C"));
    let path_count = svg.matches("<path").count();
    assert!(path_count >= 4, "Expected ≥4 paths, got {path_count}");
}

/// Shared scale mode — shorter sequence draws narrower bar.
#[test]
fn synteny_shared_scale() {
    let plot = SyntenyPlot::new()
        .with_sequences([("Long", 1_000_000.0), ("Short", 400_000.0)])
        .with_shared_scale()
        .with_block(0, 0.0, 300_000.0, 1, 0.0, 300_000.0)
        .with_block(0, 350_000.0, 700_000.0, 1, 50_000.0, 380_000.0);

    let layout = Layout::auto_from_plots(&[Plot::Synteny(plot.clone())]);
    let svg = render(&plot, layout);
    save("synteny_shared_scale", &svg);

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Long"));
    assert!(svg.contains("Short"));
}

/// Custom sequence colors, block colors, and legend.
#[test]
fn synteny_custom_colors() {
    let plot = SyntenyPlot::new()
        .with_sequences([("Seq1", 500_000.0), ("Seq2", 500_000.0)])
        .with_sequence_colors(["#4393c3", "#d6604d"])
        .with_colored_block(0, 0.0, 150_000.0, 1, 0.0, 150_000.0, "#2ca02c")
        .with_colored_inv_block(0, 200_000.0, 400_000.0, 1, 200_000.0, 400_000.0, "#ff7f0e")
        .with_legend("Synteny blocks");

    let layout = Layout::auto_from_plots(&[Plot::Synteny(plot.clone())]);
    let svg = render(&plot, layout);
    save("synteny_custom_colors", &svg);

    assert!(svg.contains("<svg"));
    assert!(svg.contains("#4393c3"), "Missing bar color for Seq1");
    assert!(svg.contains("#d6604d"), "Missing bar color for Seq2");
    assert!(svg.contains("#2ca02c"), "Missing block color");
    assert!(svg.contains("#ff7f0e"), "Missing inverted block color");
}

/// Larger stress test: 6 sequences, ~22 blocks, with title.
/// All block coordinates are within the shortest sequence (Chr 6 = 170M).
/// Forward blocks are separated by gaps; inversions sit in those gaps.
#[test]
fn synteny_large() {
    let mut plot = SyntenyPlot::new()
        .with_sequences([
            ("Chr 1", 248_956_422.0),
            ("Chr 2", 242_193_529.0),
            ("Chr 3", 198_295_559.0),
            ("Chr 4", 190_214_555.0),
            ("Chr 5", 181_538_259.0),
            ("Chr 6", 170_805_979.0),
        ]);

    let pairs: [(usize, usize); 5] = [(0,1),(1,2),(2,3),(3,4),(4,5)];
    // Four forward blocks — all endpoints fit within Chr 6 (170M)
    // Gaps: 35–45M, 85–95M, 130–140M
    let fwd = [
        (0.0_f64, 35e6_f64, 0.0_f64, 33e6_f64),
        (45e6, 83e6, 43e6, 80e6),
        (95e6, 128e6, 93e6, 125e6),
        (140e6, 165e6, 138e6, 163e6),
    ];
    for (s1, s2) in pairs {
        for (a, b, c, d) in fwd {
            plot = plot.with_block(s1, a, b, s2, c, d);
        }
    }
    // Inversions placed in the gaps between forward blocks
    plot = plot
        .with_inv_block(0, 37e6, 43e6, 1, 35e6, 41e6)
        .with_inv_block(2, 87e6, 93e6, 3, 85e6, 91e6);

    let layout = Layout::auto_from_plots(&[Plot::Synteny(plot.clone())])
        .with_title("Synteny — 6 chromosomes");
    let svg = render(&plot, layout);
    save("synteny_large", &svg);

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Chr 1"));
    assert!(svg.contains("Chr 6"));
    let path_count = svg.matches("<path").count();
    assert!(path_count >= 20, "Expected ≥20 paths, got {path_count}");
}
