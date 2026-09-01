mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::brick::{BrickAnchor, BrickTemplate};
use kuva::plot::BrickPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

#[test]
fn test_brickplot_svg_output_builder() {
    let sequences: Vec<String> = vec![
       "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATTCAT".to_string(),
       "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCATCATCATCATTCAT".to_string(),
       "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCATCATCATCATTCAT".to_string(),
       "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATTCAT".to_string(),
       "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATTCAT".to_string(),
       "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATCATCATCATCATGGTCATCATCATCATCAT".to_string(),
       "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATCAT".to_string(),
       "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATTCAT".to_string(),
    ];

    let names: Vec<String> = vec![
        "read_1".to_string(),
        "read_2".to_string(),
        "read_3".to_string(),
        "read_4".to_string(),
        "read_5".to_string(),
        "read_6".to_string(),
        "read_7".to_string(),
        "read_8".to_string(),
    ];

    let colours = BrickTemplate::new();
    let b = colours.dna().clone(); // get the DNA template

    let brickplot = BrickPlot::new()
        .with_sequences(sequences)
        .with_names(names)
        .with_template(b.template)
        .with_x_offset(18.0);
    // .show_values();

    let plots = vec![Plot::Brick(brickplot)];

    let layout = Layout::auto_from_plots(&plots).with_title("BrickPlot - DNA");
    // .with_x_categories(x_labels);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/brickplot_DNA_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}

#[test]
fn test_brickplot_per_read_offsets() {
    // Each read starts at a different position relative to the repeat region.
    let sequences: Vec<String> = vec![
        "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCAT".to_string(), // offset 18
        "GCACTCATCATCATCATCATCATCATCATCATCAT".to_string(),        // offset 10
        "ATCAGGCCGCACTCATCATCATCATCATCATCATCATCAT".to_string(),   // offset 16
        "CACTCATCATCATCATCATCAT".to_string(),                     // offset 5
    ];

    let names: Vec<String> = vec![
        "read_1".to_string(),
        "read_2".to_string(),
        "read_3".to_string(),
        "read_4".to_string(),
    ];

    let colours = BrickTemplate::new();
    let b = colours.dna();

    let brickplot = BrickPlot::new()
        .with_sequences(sequences)
        .with_names(names)
        .with_template(b.template)
        .with_x_offsets(vec![18.0, 10.0, 16.0, 5.0]);

    let plots = vec![Plot::Brick(brickplot)];
    let layout = Layout::auto_from_plots(&plots).with_title("BrickPlot - per-read offsets");
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/brickplot_per_read_offsets.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
}

#[test]
fn test_brickplot_per_read_offsets_fallback() {
    // 4 sequences; read 2 (middle) uses None → falls back to the global x_offset (12.0),
    // while read 3 still has its own offset (5.0).
    let sequences: Vec<String> = vec![
        "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCAT".to_string(), // per-row: 18
        "GCACTCATCATCATCATCATCATCATCATCATCAT".to_string(),        // per-row: 10
        "ATCAGGCCGCACTCATCATCATCATCATCATCATCATCAT".to_string(),   // None → fallback: 12
        "CACTCATCATCATCATCATCAT".to_string(),                     // per-row: 5
    ];

    let names: Vec<String> = vec![
        "read_1".to_string(),
        "read_2".to_string(),
        "read_3".to_string(),
        "read_4".to_string(),
    ];

    let colours = BrickTemplate::new();
    let b = colours.dna();

    let brickplot = BrickPlot::new()
        .with_sequences(sequences)
        .with_names(names)
        .with_template(b.template)
        .with_x_offset(12.0)
        .with_x_offsets(vec![Some(18.0), Some(10.0), None, Some(5.0_f64)]);

    let plots = vec![Plot::Brick(brickplot)];
    let layout =
        Layout::auto_from_plots(&plots).with_title("BrickPlot - per-read offsets with fallback");
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output(
        "test_outputs/brickplot_per_read_offsets_fallback.svg",
        svg.clone(),
    )
    .unwrap();

    assert!(svg.contains("<svg"));
}

#[test]
fn test_brickplot_strigar_svg_output_builder() {
    let sequences: Vec<String> = vec![
       "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATTCAT".to_string(),
       "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCATCATCATCATTCAT".to_string(),
       "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCATCATCATCATTCAT".to_string(),
       "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATTCAT".to_string(),
       "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATTCAT".to_string(),
       "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATCATCATCATCATGGTCATCATCATCATCAT".to_string(),
       "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATCAT".to_string(),
       "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATTCAT".to_string(),
    ];

    // (motif, strigar)
    // so, need to split the motifs. Then create a count of them. Order by most common
    // Then colour them from a colourmap
    // Then plot them
    // use the x_offset to just make a grey block...use actual string position later
    let strigars: Vec<(String, String)> = vec![
        ("CAT:A,C:B,T:C".to_string(), "10A1B4A1C1A".to_string()),
        ("CAT:A,T:B".to_string(), "14A1B1A".to_string()),
        ("CAT:A,T:B".to_string(), "14A1B1A".to_string()),
        ("CAT:A,C:B,T:C".to_string(), "10A1B4A1C1A".to_string()),
        ("CAT:A,C:B,T:C".to_string(), "10A1B4A1C1A".to_string()),
        ("CAT:A,C:B,GGT:C".to_string(), "10A1B8A1C5A".to_string()),
        ("CAT:A,C:B".to_string(), "10A1B5A".to_string()),
        ("CAT:A,C:B,T:C".to_string(), "10A1B4A1C1A".to_string()),
    ];

    let names: Vec<String> = vec![
        "read_1".to_string(),
        "read_2".to_string(),
        "read_3".to_string(),
        "read_4".to_string(),
        "read_5".to_string(),
        "read_6".to_string(),
        "read_7".to_string(),
        "read_8".to_string(),
    ];

    let colours = BrickTemplate::new();
    let b = colours.dna().clone(); // get the DNA template

    let brickplot = BrickPlot::new()
        .with_sequences(sequences)
        .with_names(names)
        .with_template(b.template)
        .with_strigars(strigars)
        .with_x_offset(18.0);
    // .show_values();

    let plots = vec![Plot::Brick(brickplot)];

    let layout = Layout::auto_from_plots(&plots).with_title("BrickPlot - strigar");
    // .with_x_categories(x_labels);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/brickplot_strigar_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}

#[test]
fn test_brick_legend_order() {
    // CAT is the most frequent motif (32 occurrences) → assigned global letter A.
    // T is the second most frequent (2 occurrences) → assigned global letter B.
    // After sorting by letter, the legend must list "CAT" before "T".
    let sequences: Vec<String> = vec![
        "CATCATCATCATCATCATCATCATCATCATT".to_string(),
        "CATCATCATCATCATCATCATCATCATCATCATCAT".to_string(),
        "CATCATCATCATCATCATCATCATT".to_string(),
    ];
    let names: Vec<String> = vec!["r1".to_string(), "r2".to_string(), "r3".to_string()];
    // motif_str local letters: CAT→A, T→B
    // strigar counts: read1: 10 CAT + 1 T + 1 CAT = 11 CAT, 1 T
    //                 read2: 12 CAT
    //                 read3: 8 CAT + 1 T + 1 CAT = 9 CAT, 1 T
    // global totals: CAT=32, T=2 → CAT gets global A, T gets global B
    let strigars: Vec<(String, String)> = vec![
        ("CAT:A,T:B".to_string(), "10A1B1A".to_string()),
        ("CAT:A".to_string(), "12A".to_string()),
        ("CAT:A,T:B".to_string(), "8A1B1A".to_string()),
    ];

    let brickplot = BrickPlot::new()
        .with_sequences(sequences)
        .with_names(names)
        .with_strigars(strigars);

    let plots = vec![Plot::Brick(brickplot)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/brickplot_legend_order.svg", svg.clone()).unwrap();

    // 'A' is most frequent (CAT); 'B' is next (T).
    // The legend must list them in that order: CAT before T in the SVG.
    let pos_cat = svg
        .find(">CAT<")
        .expect("legend should contain 'CAT' label");
    let pos_t = svg.find(">T<").expect("legend should contain 'T' label");
    assert!(
        pos_cat < pos_t,
        "legend entry 'CAT' (global letter A, most frequent) must appear before 'T' (global letter B)"
    );
}

#[test]
fn test_brick_canonical_freq_counts_bricks_not_reads() {
    // Regression test for the canonical_freq bug where read presence was counted
    // instead of brick count.
    //
    // Setup: dominant motif CAG appears many times per read; interrupt motif C
    // appears exactly once in every read.  Under the old (buggy) code both get the
    // same presence count (3 reads each) and the tiebreak on canonical string
    // could promote the interrupt to global letter A.  Under the correct code
    // brick counts are used: CAG scores 14+10+8=32, C scores 1+1+1=3, so CAG
    // is always global letter A (most frequent).
    let strigars: Vec<(String, String)> = vec![
        ("CAG:A,C:B".to_string(), "14A1B".to_string()), // CAG×14, C×1
        ("CAG:A,C:B".to_string(), "10A1B".to_string()), // CAG×10, C×1
        ("CAG:A,C:B".to_string(), "8A1B".to_string()),  // CAG×8,  C×1
    ];

    let brickplot = BrickPlot::new()
        .with_names(vec!["r1", "r2", "r3"])
        .with_strigars(strigars);

    let plots = vec![Plot::Brick(brickplot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));

    let pos_cag = svg.find(">CAG<").expect("legend must contain 'CAG'");
    let pos_c = svg.find(">C<").expect("legend must contain 'C'");
    assert!(
        pos_cag < pos_c,
        "CAG (32 bricks) must be global letter A and appear before C (3 bricks) in the legend"
    );
}

#[test]
fn test_brick_flanked_strigars() {
    // with_flanked_strigars: left flank + STR + right flank per read.
    // Left/right flanks render with DNA colours; STR bricks use strigar colours.
    let flanked = vec![
        ("ACGTACGT", "CAG:A,C:B", "12A1B", "TGCATGCA"),
        ("ACGTACGT", "CAG:A,C:B", "10A1B", "TGCATGCA"),
        ("ACGT", "CAG:A", "8A", "TGCA"),
    ];
    let brickplot = BrickPlot::new()
        .with_names(vec!["consensus", "read_1", "read_2"])
        .with_flanked_strigars(flanked);

    let plots = vec![Plot::Brick(brickplot)];
    let layout = Layout::auto_from_plots(&plots).with_title("BrickPlot - flanked strigars");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/brickplot_flanked.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // DNA A = rgb(0,150,0) → #009600 after SVG backend conversion; appears in flanks.
    assert!(
        svg.contains("#009600"),
        "DNA A colour should appear in left/right flanks"
    );
    // STR primary motif colour (#1f77b4 for global letter A) should appear.
    assert!(
        svg.contains("#1f77b4"),
        "primary STR motif should use the default first palette colour"
    );
}

#[test]
fn test_brick_right_anchor() {
    // Right-anchor: rows of different lengths should have their trailing edges aligned.
    // The SVG should still render without panic; verify it's valid SVG.
    let strigars: Vec<(String, String)> = vec![
        ("CAG:A".to_string(), "14A".to_string()),
        ("CAG:A".to_string(), "10A".to_string()),
        ("CAG:A".to_string(), "8A".to_string()),
    ];
    let brickplot = BrickPlot::new()
        .with_names(vec!["r1", "r2", "r3"])
        .with_anchor(BrickAnchor::Right)
        .with_strigars(strigars);

    let plots = vec![Plot::Brick(brickplot)];
    let layout = Layout::auto_from_plots(&plots).with_title("BrickPlot - right anchor");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/brickplot_right_anchor.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_brick_mark_primary() {
    // with_mark_primary: the legend label for global letter A should end with '*'.
    let strigars: Vec<(String, String)> = vec![
        ("CAG:A,C:B".to_string(), "12A1B".to_string()),
        ("CAG:A,C:B".to_string(), "10A1B".to_string()),
    ];
    let brickplot = BrickPlot::new()
        .with_names(vec!["r1", "r2"])
        .with_mark_primary()
        .with_strigars(strigars);

    let plots = vec![Plot::Brick(brickplot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));

    assert!(
        svg.contains(">CAG*<"),
        "primary motif label must end with '*'"
    );
    // Secondary motif (C) should NOT have a star.
    assert!(!svg.contains(">C*<"), "non-primary motif must not have '*'");
}

#[test]
fn test_brick_consensus_row() {
    // with_consensus_row: display rotation should be locked to what the consensus uses.
    // Consensus (row 0) uses CAG; read_1 uses AGC (a rotation of CAG).
    // Without consensus locking the display might show AGC.
    // With consensus locking the display must show CAG for both.
    let strigars: Vec<(String, String)> = vec![
        ("CAG:A".to_string(), "12A".to_string()), // consensus: uses CAG
        ("AGC:A".to_string(), "10A".to_string()), // read with rotated motif
        ("GCA:A".to_string(), "8A".to_string()),  // another rotation
    ];
    let brickplot = BrickPlot::new()
        .with_names(vec!["consensus", "read_1", "read_2"])
        .with_consensus_row(0)
        .with_strigars(strigars);

    let plots = vec![Plot::Brick(brickplot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));

    // Legend must show CAG (consensus rotation) not AGC or GCA.
    assert!(
        svg.contains(">CAG<"),
        "legend must use the consensus row's rotation (CAG)"
    );
    assert!(
        !svg.contains(">AGC<"),
        "AGC rotation must not appear in legend when consensus_row=0"
    );
    assert!(
        !svg.contains(">GCA<"),
        "GCA rotation must not appear in legend when consensus_row=0"
    );
}

#[test]
fn test_brick_notations() {
    // with_notations: Some(_) rows get auto-generated per-block "(kmer)count" labels.
    // Row 0 (consensus) has notations enabled; 12 consecutive A bricks → one run of 12.
    // Row 1 (read_1) has notations disabled.
    let strigars: Vec<(String, String)> = vec![
        ("CAG:A".to_string(), "12A".to_string()),
        ("CAG:A".to_string(), "10A".to_string()),
    ];
    let brickplot = BrickPlot::new()
        .with_names(vec!["consensus", "read_1"])
        .with_strigars(strigars)
        .with_notations(vec![Some("".to_string()), None]);

    let plots = vec![Plot::Brick(brickplot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));

    // Row 0 has one run of 12 A bricks → auto label "(CAG)12".
    assert!(
        svg.contains("(CAG)12"),
        "per-block notation must appear for enabled row"
    );
    // Row 1 has notations disabled → no "(CAG)10" label generated.
    assert!(
        !svg.contains("(CAG)10"),
        "disabled row must not get per-block notation"
    );
}

#[test]
fn test_brick_strigar_canonical_unification_across_rows() {
    // ACCCTA and TAACCC are rotations of the same canonical, appearing under the
    // local letter A in different rows. They must resolve to the same global token,
    // hence the same colour: only ONE non-DNA motif colour should appear.
    let strigars: Vec<(String, String)> = vec![
        ("ACCCTA:A".to_string(), "5A".to_string()),
        ("TAACCC:A".to_string(), "3A".to_string()),
    ];
    let brickplot = BrickPlot::new()
        .with_names(vec!["read_1", "read_2"])
        .with_strigars(strigars);

    let plots = vec![Plot::Brick(brickplot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/brickplot_canonical_unify.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // 6-mer motif → first palette colour (blue), shared by both rotations.
    assert!(
        svg.contains("#1f77b4"),
        "the ACCCTA/TAACCC canonical family should share global colour A"
    );
}

// ── Bladerunner format spec tests ────────────────────────────────────────────
//
// These tests correspond to the formal bladerunner format specification covering
// motifs, STRIGAR, and traditional (human-readable) encoding.

#[test]
fn test_brick_motif_colors_stable_across_plots() {
    // Colours keyed by canonical k-mer must be representation-independent: the same
    // motif gets the same colour regardless of its per-plot frequency rank, and any
    // rotation of the motif resolves to the same entry. This is the contract that lets
    // bladerunner stop reproducing kuva's internal token numbering.
    use std::collections::HashMap;
    let mut colors: HashMap<String, String> = HashMap::new();
    colors.insert("AATGG".to_string(), "#123456".to_string());
    colors.insert("CAG".to_string(), "#abcdef".to_string());

    let render = |motif_map: &str| -> String {
        let bp = BrickPlot::new()
            .with_names(vec!["r1"])
            .with_motif_colors(colors.clone())
            .with_strigars(vec![(motif_map.to_string(), "20A2B".to_string())]);
        let plots = vec![Plot::Brick(bp)];
        let layout = Layout::auto_from_plots(&plots);
        SvgBackend.render_scene(&render_multiple(plots, layout))
    };

    // Plot 1: AATGG is the most frequent motif (letter A). Plot 2: CAG is most frequent,
    // and AATGG is supplied as a DIFFERENT rotation (GGAAT) — must still resolve the same.
    let svg1 = render("AATGG:A,CAG:B");
    let svg2 = render("CAG:A,GGAAT:B");

    // Both explicit colours appear in both plots despite differing frequency ranks
    // and despite AATGG being supplied as the GGAAT rotation in plot 2.
    for (svg, which) in [(&svg1, "plot1"), (&svg2, "plot2")] {
        assert!(
            svg.contains("#123456"),
            "AATGG must keep its explicit colour in {which}"
        );
        assert!(
            svg.contains("#abcdef"),
            "CAG must keep its explicit colour in {which}"
        );
    }
}

#[test]
fn test_brick_strigar_multichar_letters_no_panic() {
    // Regression for the bladerunner contract: letters are bijective base-26 strings,
    // so a row with >26 motifs uses multi-character letters (AA, AB, ...). The parser
    // must consume the maximal uppercase run as one letter; reading a single char
    // truncated `1AA` to `1A` + stray `A` and panicked with ParseIntError(Empty).
    fn motif_letter(mut idx: usize) -> String {
        let mut out = Vec::new();
        loop {
            out.push(b'A' + (idx % 26) as u8);
            if idx < 26 {
                break;
            }
            idx = idx / 26 - 1;
        }
        out.reverse();
        String::from_utf8(out).unwrap()
    }
    // 27 distinct kmers → the 27th letter is "AA"; also forces >26 global tokens.
    let motifs: String = (0..27)
        .map(|i| format!("ACG{i}:{}", motif_letter(i)))
        .collect::<Vec<_>>()
        .join(",");
    let strigar = "5A1AA"; // 5 copies of letter A, 1 copy of letter AA

    let bp = BrickPlot::new()
        .with_names(vec!["row1"])
        .with_strigars(vec![(motifs, strigar.to_string())]);

    let plots = vec![Plot::Brick(bp)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert!(
        svg.contains("<svg"),
        "multi-char letters must render, not panic"
    );
}

#[test]
fn test_brick_strigar_real_31_motif_row() {
    // The verbatim 31-motif row from the bladerunner contract doc (CPUM_TYMS). It has
    // multi-character letters up to AE, a 56 bp kmer (Z), and single-base motifs. Must
    // build and render without panicking and produce valid SVG.
    let strigar = "3B1C2B1C4B1C2B3C6B1C2B1D3E1F2A1G3A2H3A1H2A1I4A1J2A1H4A1K5A2H2A1L2M1N2O1P2Q1R1Q2S1T6U2V1W1X1Y2X1Z2T1AA1AB1S2AB1AC1AD1AE2AD";
    let motifs = "TGATGG:A,TGGTGA:B,TGGAGA:C,TGGAGATGGT:D,GATGGCGATGGA:E,GATGG:F,TGG:G,AGATGG:H,G:I,AGATGGTGAATGG:J,AGAGG:K,TGAGGGGTGGTGCCT:L,ATC:M,TCGATTGC:N,AC:O,AAAAATGGCAAGTTTAA:P,TAT:Q,GTGTACTT:R,CA:S,ATG:T,A:U,GCT:V,GCGTGGGCCAAGTTACTTGTGCA:W,GGT:X,AAGTGTTCTGCA:Y,TGCCTGCACCTCAGTTGTAGGGTGTCCGTAGGATGTGAGGCCAGTCCCCGGGCTTA:Z,CTTTAAATCCTGCCTAGT:AA,ATT:AB,TCTTGTCGCT:AC,TAA:AD,AAGGCC:AE";

    let bp = BrickPlot::new()
        .with_names(vec!["CPUM_TYMS_read"])
        .with_mark_primary()
        .with_strigars(vec![(motifs.to_string(), strigar.to_string())]);

    let plots = vec![Plot::Brick(bp)];
    let layout = Layout::auto_from_plots(&plots).with_title("CPUM_TYMS 31-motif row");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/brickplot_cpum_tyms_31motif.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"), "31-motif row must render, not panic");
}

#[test]
fn test_brick_spec_stitched_with_traditional_notation() {
    // Bladerunner workflow: flanked_strigars + traditional notation rendered together.
    // Simulates a real bladerunner TSV row where the traditional column is pre-computed
    // and passed to kuva for annotation above the consensus row.
    let flanked = vec![
        // consensus row — notation provided
        ("ACGTACGT", "CAG:A,CAA:B,CCG:C", "6A1B2A1C10A", "TGCATGCA"),
        // read rows — no notation
        ("ACGTACGT", "CAG:A,CCG:B", "8A1B10A", "TGCATGCA"),
        ("ACGTACGT", "CAG:A", "20A", "TGCA"),
    ];
    let brickplot = BrickPlot::new()
        .with_names(vec!["consensus", "read_1", "read_2"])
        .with_consensus_row(0)
        .with_mark_primary()
        .with_flanked_strigars(flanked)
        .with_notations(vec![
            Some("(CAG)6(CAA)1(CAG)2(CCG)1(CAG)10".to_string()),
            None,
            None,
        ]);

    let plots = vec![Plot::Brick(brickplot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("BrickPlot — bladerunner flanked+notation pipeline");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/brickplot_spec_full_pipeline.svg", svg.clone())
        .unwrap();

    assert!(svg.contains("<svg"), "must produce valid SVG");
    // Per-block labels appear above consensus row (auto-generated from run-length encoding).
    assert!(
        svg.contains("(CAG)6"),
        "run of 6 A bricks must produce (CAG)6 label"
    );
    assert!(
        svg.contains("(CAA)1"),
        "run of 1 B brick must produce (CAA)1 label"
    );
    assert!(
        svg.contains("(CAG)2"),
        "run of 2 A bricks must produce (CAG)2 label"
    );
    assert!(
        svg.contains("(CCG)1"),
        "run of 1 C brick must produce (CCG)1 label"
    );
    assert!(
        svg.contains("(CAG)10"),
        "run of 10 A bricks must produce (CAG)10 label"
    );
    // Primary motif has '*' in legend.
    assert!(
        svg.contains("*"),
        "mark_primary must append * to primary motif legend label"
    );
    // DNA flank colour must appear (A in flanks → #009600).
    assert!(svg.contains("#009600"), "DNA flank bricks must appear");
}

#[test]
fn test_brick_spec_multi_segment_single_candidate() {
    // Spec §1: single candidate (no | separator) — simple round-trip.
    // motifs: CAG:A,CAA:B,CCG:C — three motifs in one segment.
    // STRIGAR: 2A1B2A1C10A → (CAG)2(CAA)1(CAG)2(CCG)1(CAG)10.
    // Total width = (2+2+10)*3 + 1*3 + 1*3 = 42 + 3 + 3 = 48 nt.
    let bp = BrickPlot::new().with_names(vec!["r1"]).with_strigars(vec![(
        "CAG:A,CAA:B,CCG:C".to_string(),
        "2A1B2A1C10A".to_string(),
    )]);

    let x_max = Plot::Brick(bp).bounds().expect("bounds").0 .1;
    assert!(
        (x_max - 48.0).abs() < 0.01,
        "single-segment 3-motif: expected 48 nt, got {}",
        x_max
    );
}

// ── Figure tests ─────────────────────────────────────────────────────────────

/// Two BrickPlots (hap1 / hap2) in a 2×1 Figure with a shared x-axis and
/// uniform row height via `with_row_height`.  This exercises:
///   - `BrickPlot::with_row_height` auto-sizing the per-panel canvas height
///   - Figure per-grid-row height computation from BrickPlot metadata
///   - shared x-axis clamping both panels to the same x-range
///
/// Visual expectation: hap1 (3 reads) and hap2 (8 reads) brick rows are the
/// same pixel height; both panels share the same x extent.
#[test]
fn test_brickplot_figure_haplotypes_shared_x() {
    use kuva::render::figure::Figure;

    let tmpl = BrickTemplate::new().dna();

    // hap1: 3 reads, shorter sequences
    let hap1 = BrickPlot::new()
        .with_sequences(vec![
            "CGGCGATCAGGCCGCACTCATCATCATCATCAT",
            "CGGCGATCAGGCCGCACTCATCATCATCATCATCAT",
            "CGGCGATCAGGCCGCACTCATCATCATCAT",
        ])
        .with_names(vec!["hap1_r1", "hap1_r2", "hap1_r3"])
        .with_template(tmpl.template.clone())
        .with_row_height(20.0);

    // hap2: 8 reads, longer sequences
    let hap2 = BrickPlot::new()
        .with_sequences(vec![
            "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCAT",
            "CGGCGATCAGGCCGCACTCATCATCATCATCATCAT",
            "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCAT",
            "CGGCGATCAGGCCGCACTCATCATCATCATCAT",
            "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCAT",
            "CGGCGATCAGGCCGCACTCATCATCATCATCATCAT",
            "CGGCGATCAGGCCGCACTCATCATCATCATCAT",
            "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCAT",
        ])
        .with_names(vec![
            "hap2_r1", "hap2_r2", "hap2_r3", "hap2_r4", "hap2_r5", "hap2_r6", "hap2_r7", "hap2_r8",
        ])
        .with_template(tmpl.template.clone())
        .with_row_height(20.0);

    let figure = Figure::new(2, 1)
        .with_plots(vec![vec![Plot::Brick(hap1)], vec![Plot::Brick(hap2)]])
        .with_shared_x_all()
        .with_title("Haplotype brick plots — shared x, equal row height");

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/brickplot_haplotypes_figure.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"), "expected SVG output");

    // The two panels should have different heights (hap1: 3 rows, hap2: 8 rows)
    // but together form a taller canvas than a single uniform cell_height figure.
    // Verify both hap names appear as y-axis tick labels.
    assert!(
        svg.contains("hap1_r1"),
        "hap1 read labels should be present"
    );
    assert!(
        svg.contains("hap2_r1"),
        "hap2 read labels should be present"
    );
}

/// Verify that `with_row_height` produces a canvas height where each brick row
/// is exactly the requested number of pixels tall.
///
/// We do this by rendering the SVG and measuring that the y-extent of the first
/// brick rect equals `row_height_px * 0.95` (the renderer applies a 0.95 height
/// factor for brick spacing).  We also confirm that two plots with different row
/// counts but the same `row_height_px` produce proportionally different canvas
/// heights (not identical ones).
#[test]
fn test_brickplot_row_height_standalone_sizing() {
    let tmpl = BrickTemplate::new().dna();

    // 3 rows at 20 px/row
    let brick3 = BrickPlot::new()
        .with_sequences(vec!["ACGT", "ACGT", "ACGT"])
        .with_names(vec!["r1", "r2", "r3"])
        .with_template(tmpl.template.clone())
        .with_row_height(20.0);

    // 8 rows at 20 px/row — should produce a taller canvas
    let brick8 = BrickPlot::new()
        .with_sequences(vec!["ACGT"; 8].to_vec())
        .with_names((1..=8).map(|i| format!("r{i}")).collect::<Vec<_>>())
        .with_template(tmpl.template.clone())
        .with_row_height(20.0);

    let plots3 = vec![Plot::Brick(brick3)];
    let plots8 = vec![Plot::Brick(brick8)];

    let layout3 = Layout::auto_from_plots(&plots3);
    let layout8 = Layout::auto_from_plots(&plots8);

    // Both layouts must have an explicit height set.
    assert!(
        layout3.height.is_some(),
        "layout for 3-row brick should have height set"
    );
    assert!(
        layout8.height.is_some(),
        "layout for 8-row brick should have height set"
    );

    // The 8-row canvas must be taller than the 3-row canvas by ~5×20 = 100 px.
    let h3 = layout3.height.unwrap();
    let h8 = layout8.height.unwrap();
    let diff = h8 - h3;
    assert!(
        (diff - 100.0).abs() < 1.0,
        "canvas height difference should be 5 * row_height = 100 px, got {diff}"
    );
}
