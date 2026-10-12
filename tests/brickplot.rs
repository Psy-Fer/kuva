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
fn test_brick_consensus_label_deterministic() {
    // A consensus row can carry two rotations of the same canonical under different
    // letters (CAGA and ACAG both canonicalise to ACAG). The display label must lock to
    // the higher-copy-count rotation (CAGA: 10 vs ACAG: 2), never flipping with HashMap
    // iteration order. Build many times — each parse builds a freshly-seeded HashMap, so
    // any residual order dependence would surface as an intermittent wrong winner.
    for _ in 0..64 {
        let bp = BrickPlot::new()
            .with_names(vec!["consensus"])
            .with_consensus_row(0)
            .with_strigars(vec![("CAGA:A,ACAG:B".to_string(), "10A2B".to_string())]);
        let motifs = bp.motifs.as_ref().expect("strigar mode sets motifs");
        let displays: Vec<&str> = motifs.values().map(String::as_str).collect();
        assert!(
            displays.contains(&"CAGA"),
            "consensus label must lock to higher-count rotation CAGA, got {displays:?}"
        );
        assert!(
            !displays.contains(&"ACAG"),
            "lower-count rotation ACAG must never win, got {displays:?}"
        );
    }
}

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
        .with_sequences(["ACGT"; 8].to_vec())
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

// Per-run notation labels are packed into vertical stagger tiers. An
// unplaceable label used to be dropped back onto tier 0 (`unwrap_or(0)`),
// overwriting whatever was already there instead of staggering further up.
//
// Reproducing it needs *clustered* labels rather than merely many of them: the
// density guard trips on the total label width, so evenly spread labels are
// suppressed wholesale before the tiers can fill. A tight run of short blocks
// followed by one very long block keeps the total small while piling a dozen
// labels into a narrow x range.
/// A strigar whose motif runs cluster tightly: short `(CAG)n` blocks packed
/// against a long `(CAACACAACAA)n` one, which is the shape that makes labels
/// collide. At width 2100 the full set fits, but only by using every tier.
fn clustered_notation_plot(width: f64) -> kuva::render::render::Scene {
    let strigar = format!("{}{}A", "1A1B".repeat(6), 200);
    let motifs = "CAG:A,CAACACAACAA:B";
    // Row 0 renders at the top and is the only row with a notation: the
    // consensus-above-the-reads shape the notation track is meant for.
    let plot = BrickPlot::new()
        .with_strigars((0..2).map(|_| (motifs, strigar.as_str())))
        .with_names((0..2).map(|i| format!("read{i}")))
        .with_notations([Some("x".to_string()), None]);
    let plots = vec![Plot::Brick(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("notation stagger")
        .with_width(width)
        .with_height(320.0);
    render_multiple(plots, layout)
}

/// Notation labels grouped by the tier (text baseline) they were placed on.
/// Each entry is `(left_px, right_px, text)`; widths are derived from the real
/// font size rather than guessed from markup.
fn notation_tiers(
    scene: &kuva::render::render::Scene,
) -> std::collections::BTreeMap<String, Vec<(f64, f64, String)>> {
    let mut by_tier: std::collections::BTreeMap<String, Vec<(f64, f64, String)>> =
        std::collections::BTreeMap::new();
    for p in &scene.elements {
        if let kuva::render::render::Primitive::Text {
            x,
            y,
            content,
            size,
            ..
        } = p
        {
            // Notation labels are the only "(motif)count" texts.
            if !content.starts_with('(') {
                continue;
            }
            // DejaVu Sans averages ~0.55 em per glyph; labels are centred.
            let half = content.chars().count() as f64 * (*size as f64) * 0.55 * 0.5;
            by_tier.entry(format!("{y:.1}")).or_default().push((
                x - half,
                x + half,
                content.clone(),
            ));
        }
    }
    by_tier
}

#[test]
fn notation_labels_never_overlap_on_a_tier() {
    let scene = clustered_notation_plot(2100.0);
    let by_tier = notation_tiers(&scene);
    assert!(!by_tier.is_empty(), "expected notation labels to render");

    for (y, mut items) in by_tier {
        items.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("finite"));
        for w in items.windows(2) {
            assert!(
                w[1].0 >= w[0].1,
                "labels overlap on tier y={y}: {} [{:.1}..{:.1}] and {} [{:.1}..{:.1}]",
                w[0].2,
                w[0].0,
                w[0].1,
                w[1].2,
                w[1].0,
                w[1].1
            );
        }
    }
}

#[test]
fn notation_labels_use_every_tier_before_giving_up() {
    let scene = clustered_notation_plot(2100.0);
    let by_tier = notation_tiers(&scene);
    let labels: usize = by_tier.values().map(|v| v.len()).sum();

    // 13 labelable runs on the notated row: the whole set must be drawn.
    assert_eq!(labels, 13, "expected every notation label to be drawn");
    // The row needs the full tier budget. Were the budget still four, this
    // locus would have fallen back to "complex structure" instead.
    assert_eq!(
        by_tier.len(),
        kuva::plot::brick::BRICK_NOTATION_TIERS,
        "expected the row to stagger across every available tier"
    );
}

#[test]
fn y_axis_labels_sit_on_the_rows_they_name() {
    // Rows of different lengths, so a band's brick count identifies which row
    // it is independently of the axis labels.
    let plot = BrickPlot::new()
        .with_strigars([("CAG:A", "20A"), ("CAG:A", "60A")])
        .with_names(["row0_short", "row1_long"]);
    let plots = vec![Plot::Brick(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_width(600.0)
        .with_height(300.0);
    let scene = render_multiple(plots, layout);

    // Band top edge -> number of bricks drawn on it.
    let mut bricks: std::collections::BTreeMap<String, usize> = Default::default();
    let mut label_y: std::collections::HashMap<&str, f64> = Default::default();
    for p in &scene.elements {
        match p {
            // Bricks are tall and narrow: taller than a 12px legend swatch,
            // narrower than the canvas background or the legend box.
            kuva::render::render::Primitive::Rect {
                y, height, width, ..
            } if *height > 20.0 && *width < 60.0 => {
                *bricks.entry(format!("{y:.1}")).or_default() += 1;
            }
            kuva::render::render::Primitive::Text { y, content, .. } => {
                for name in ["row0_short", "row1_long"] {
                    if content == name {
                        label_y.insert(name, *y);
                    }
                }
            }
            _ => {}
        }
    }

    let (short_band, long_band) = {
        let two = bricks
            .iter()
            .find(|(_, &n)| n == 20)
            .expect("a band of 20 bricks");
        let six = bricks
            .iter()
            .find(|(_, &n)| n == 60)
            .expect("a band of 60 bricks");
        (two.0.parse::<f64>().unwrap(), six.0.parse::<f64>().unwrap())
    };

    // Each tick label must be nearer its own band than the other one. Before
    // the display-order fix the two labels were swapped: the axis claimed the
    // top row was `row1_long` when the bricks there were `row0_short`.
    let short_label = label_y["row0_short"];
    let long_label = label_y["row1_long"];
    assert!(
        (short_label - short_band).abs() < (short_label - long_band).abs(),
        "row0_short labelled at {short_label:.1}, but its 20-brick band is at \
         {short_band:.1} and the other at {long_band:.1}"
    );
    assert!(
        (long_label - long_band).abs() < (long_label - short_band).abs(),
        "row1_long labelled at {long_label:.1}, but its 60-brick band is at \
         {long_band:.1} and the other at {short_band:.1}"
    );
}

/// A locus whose few, well-spread runs need only one notation tier.
fn simple_notation_plot(width: f64) -> kuva::render::render::Scene {
    let plot = BrickPlot::new()
        .with_strigars((0..2).map(|_| ("CAG:A,CAACACAACAA:B", "5A3B4A2B6A")))
        .with_names((0..2).map(|i| format!("read{i}")))
        .with_notations([Some("x".to_string()), None]);
    let plots = vec![Plot::Brick(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("notation stagger")
        .with_width(width)
        .with_height(320.0);
    render_multiple(plots, layout)
}

/// `(top of the first brick row, baseline of the highest notation label)`.
fn brick_top_and_top_label(scene: &kuva::render::render::Scene) -> (f64, f64) {
    let mut brick_top = f64::INFINITY;
    let mut label_top = f64::INFINITY;
    for p in &scene.elements {
        match p {
            // Bricks are the small rects; skip the plot frame and legend box.
            kuva::render::render::Primitive::Rect {
                y, height, width, ..
            } if *height < 40.0 && *width < 60.0 => brick_top = brick_top.min(*y),
            kuva::render::render::Primitive::Text { y, content, .. }
                if content.starts_with('(') =>
            {
                label_top = label_top.min(*y)
            }
            _ => {}
        }
    }
    (brick_top, label_top)
}

#[test]
fn non_top_row_notations_stay_inside_their_own_band() {
    // Three rows, notation on the middle one only. Drawn above its top edge it
    // would float over the row before it and read as that row's annotation, so
    // it must land between its own band's edges instead.
    let plot = BrickPlot::new()
        .with_strigars((0..3).map(|_| ("CAG:A", "20A")))
        .with_names(["row0", "row1", "row2"])
        .with_notations([None, Some("x".to_string()), None]);
    let plots = vec![Plot::Brick(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_width(800.0)
        .with_height(360.0);
    let scene = render_multiple(plots, layout);

    // Distinct band top edges, in draw order: display row 0, 1, 2.
    let mut band_tops: Vec<f64> = Vec::new();
    let mut label_y: Option<f64> = None;
    for p in &scene.elements {
        match p {
            kuva::render::render::Primitive::Rect {
                y, height, width, ..
            } if *height > 20.0
                && *width < 60.0
                && !band_tops.iter().any(|t| (t - y).abs() < 0.01) =>
            {
                band_tops.push(*y)
            }
            kuva::render::render::Primitive::Text { y, content, .. }
                if content.starts_with('(') =>
            {
                label_y = Some(*y);
            }
            _ => {}
        }
    }
    band_tops.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    assert_eq!(band_tops.len(), 3, "expected three brick bands");
    let band_h = band_tops[1] - band_tops[0];
    let y = label_y.expect("expected a notation label");

    assert!(
        y > band_tops[1] && y < band_tops[1] + band_h,
        "notation at {y:.1} must sit inside its own band \
         ({:.1}..{:.1}), not over the row above",
        band_tops[1],
        band_tops[1] + band_h
    );

    // And it gets a backing plate, or the text would fight the brick colours.
    let plates = scene
        .elements
        .iter()
        .filter(|p| {
            matches!(
                p,
                kuva::render::render::Primitive::Rect { opacity: Some(o), height, .. }
                    if *o < 1.0 && *height < 20.0
            )
        })
        .count();
    assert_eq!(
        plates, 1,
        "expected one backing plate behind the in-band label"
    );
}

#[test]
fn notation_margin_reserves_only_the_tiers_used() {
    // One tier of labels versus six: the top margin must follow the labels,
    // not reserve the worst case for both.
    let (simple_brick, simple_label) = brick_top_and_top_label(&simple_notation_plot(900.0));
    let (dense_brick, dense_label) = brick_top_and_top_label(&clustered_notation_plot(2100.0));

    assert!(
        simple_brick < dense_brick,
        "a one-tier locus must not reserve as much head room as a six-tier one \
         (brick top {simple_brick:.1} vs {dense_brick:.1})"
    );

    // The reservation is consumed exactly, so the highest label sits in the
    // same place either way: no blank tiers left under the title.
    assert!(
        (simple_label - dense_label).abs() < 0.5,
        "top label should start at the same height regardless of tier count \
         ({simple_label:.1} vs {dense_label:.1})"
    );

    // And it stays on the canvas, above the bricks it annotates.
    assert!(simple_label > 0.0, "top label ran off the canvas");
    assert!(
        simple_label < simple_brick,
        "notation labels must sit above the first brick row"
    );
}

#[test]
fn notation_labels_are_all_or_nothing() {
    // The same locus too narrow to label: no tier arrangement fits, so every
    // row must show the fallback rather than a silently incomplete label set.
    let scene = clustered_notation_plot(900.0);
    assert!(
        notation_tiers(&scene).is_empty(),
        "expected no partial notation labels when the locus cannot be labelled in full"
    );

    let fallbacks = scene
        .elements
        .iter()
        .filter(|p| {
            matches!(
                p,
                kuva::render::render::Primitive::Text { content, .. }
                    if content.contains("complex structure")
            )
        })
        .count();
    assert_eq!(
        fallbacks, 1,
        "expected one fallback label on the notated row"
    );
}
