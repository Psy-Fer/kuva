use kuva::plot::brick::{BrickAnchor, BrickTemplate};
use kuva::plot::BrickPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;


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

    let names:Vec<String> = vec![
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

    let layout = Layout::auto_from_plots(&plots)
        .with_title("BrickPlot - DNA");
        // .with_x_categories(x_labels);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/brickplot_DNA_builder.svg", svg.clone()).unwrap();

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
    std::fs::write("test_outputs/brickplot_per_read_offsets.svg", svg.clone()).unwrap();

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
    let layout = Layout::auto_from_plots(&plots)
        .with_title("BrickPlot - per-read offsets with fallback");
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/brickplot_per_read_offsets_fallback.svg", svg.clone()).unwrap();

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

    let names:Vec<String> = vec![
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

    let layout = Layout::auto_from_plots(&plots)
        .with_title("BrickPlot - strigar");
        // .with_x_categories(x_labels);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/brickplot_strigar_builder.svg", svg.clone()).unwrap();

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
        ("CAT:A".to_string(),     "12A".to_string()),
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
    std::fs::write("test_outputs/brickplot_legend_order.svg", svg.clone()).unwrap();

    // 'A' is most frequent (CAT); 'B' is next (T).
    // The legend must list them in that order: CAT before T in the SVG.
    let pos_cat = svg.find(">CAT<").expect("legend should contain 'CAT' label");
    let pos_t   = svg.find(">T<").expect("legend should contain 'T' label");
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
        ("CAG:A,C:B".to_string(), "14A1B".to_string()),  // CAG×14, C×1
        ("CAG:A,C:B".to_string(), "10A1B".to_string()),  // CAG×10, C×1
        ("CAG:A,C:B".to_string(),  "8A1B".to_string()),  // CAG×8,  C×1
    ];

    let brickplot = BrickPlot::new()
        .with_names(vec!["r1", "r2", "r3"])
        .with_strigars(strigars);

    let plots = vec![Plot::Brick(brickplot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));

    let pos_cag = svg.find(">CAG<").expect("legend must contain 'CAG'");
    let pos_c   = svg.find(">C<").expect("legend must contain 'C'");
    assert!(
        pos_cag < pos_c,
        "CAG (32 bricks) must be global letter A and appear before C (3 bricks) in the legend"
    );
}

#[test]
fn test_brick_stitched_format_with_gaps() {
    // Bladerunner stitched STRIGAR format: | as segment separator, @ as gap code.
    // Read_1: 16×A(1nt) + small gap GAA(3nt) + 9×AGA(3nt)
    //         AGA region starts at nt position 16+3 = 19.
    // Read_2: 12×AGA(3nt) starting at position 0.
    //         with_start_positions([0, 19]) aligns read_2's AGA with read_1's.
    let strigars: Vec<(String, String)> = vec![
        ("A:A | @:GAA | AGA:B".to_string(), "16A | 1@ | 9B".to_string()),
        ("AGA:A".to_string(), "12A".to_string()),
    ];
    let brickplot = BrickPlot::new()
        .with_names(vec!["read_1", "read_2"])
        .with_strigars(strigars)
        .with_x_origin(19.0)
        .with_start_positions(vec![0.0_f64, 19.0]);

    let plots = vec![Plot::Brick(brickplot)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/brickplot_stitched_gaps.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // Gap bricks should be rendered (grey color in template; SVG emits as #c8c8c8)
    assert!(svg.contains("#c8c8c8"), "gap bricks should use grey color");
}

#[test]
fn test_brick_flanked_strigars() {
    // with_flanked_strigars: left flank + STR + right flank per read.
    // Left/right flanks render with DNA colours; STR bricks use strigar colours.
    let flanked = vec![
        ("ACGTACGT", "CAG:A,C:B", "12A1B",  "TGCATGCA"),
        ("ACGTACGT", "CAG:A,C:B", "10A1B",  "TGCATGCA"),
        ("ACGT",     "CAG:A",     "8A",      "TGCA"),
    ];
    let brickplot = BrickPlot::new()
        .with_names(vec!["consensus", "read_1", "read_2"])
        .with_flanked_strigars(flanked);

    let plots = vec![Plot::Brick(brickplot)];
    let layout = Layout::auto_from_plots(&plots).with_title("BrickPlot - flanked strigars");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/brickplot_flanked.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // DNA A = rgb(0,150,0) → #009600 after SVG backend conversion; appears in flanks.
    assert!(svg.contains("#009600"), "DNA A colour should appear in left/right flanks");
    // STR primary motif colour (#1f77b4 for global letter A) should appear.
    assert!(svg.contains("#1f77b4"), "primary STR motif should use the default first palette colour");
}

#[test]
fn test_brick_right_anchor() {
    // Right-anchor: rows of different lengths should have their trailing edges aligned.
    // The SVG should still render without panic; verify it's valid SVG.
    let strigars: Vec<(String, String)> = vec![
        ("CAG:A".to_string(), "14A".to_string()),
        ("CAG:A".to_string(), "10A".to_string()),
        ("CAG:A".to_string(),  "8A".to_string()),
    ];
    let brickplot = BrickPlot::new()
        .with_names(vec!["r1", "r2", "r3"])
        .with_anchor(BrickAnchor::Right)
        .with_strigars(strigars);

    let plots = vec![Plot::Brick(brickplot)];
    let layout = Layout::auto_from_plots(&plots).with_title("BrickPlot - right anchor");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/brickplot_right_anchor.svg", svg.clone()).unwrap();
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

    assert!(svg.contains(">CAG*<"), "primary motif label must end with '*'");
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
        ("CAG:A".to_string(), "12A".to_string()),  // consensus: uses CAG
        ("AGC:A".to_string(), "10A".to_string()),  // read with rotated motif
        ("GCA:A".to_string(),  "8A".to_string()),  // another rotation
    ];
    let brickplot = BrickPlot::new()
        .with_names(vec!["consensus", "read_1", "read_2"])
        .with_consensus_row(0)
        .with_strigars(strigars);

    let plots = vec![Plot::Brick(brickplot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));

    // Legend must show CAG (consensus rotation) not AGC or GCA.
    assert!(svg.contains(">CAG<"), "legend must use the consensus row's rotation (CAG)");
    assert!(!svg.contains(">AGC<"), "AGC rotation must not appear in legend when consensus_row=0");
    assert!(!svg.contains(">GCA<"), "GCA rotation must not appear in legend when consensus_row=0");
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
    assert!(svg.contains("(CAG)12"), "per-block notation must appear for enabled row");
    // Row 1 has notations disabled → no "(CAG)10" label generated.
    assert!(!svg.contains("(CAG)10"), "disabled row must not get per-block notation");
}

#[test]
fn test_brick_stitched_per_segment_canonical() {
    // Two reads using bladerunner stitched format.
    // ACCCTA, TAACCC, CCCTAA are all rotations of the same canonical → must get the same
    // global letter and therefore the same colour across all candidates.
    // Large gaps (36@, 213@, 31@) have no motif entry; they are scaled by N nt.
    // Small-gap case exercised by the previous test.
    let strigars: Vec<(String, String)> = vec![
        (
            "ACCCTA:A | ACCCTA:A | TAACCC:A,T:B | CCCTAA:A,ACCTAACCCTTAA:B".to_string(),
            "2A | 36@ | 2A | 213@ | 2A1B3A | 31@ | 2A1B2A".to_string(),
        ),
        (
            "ACCCTA:A".to_string(),
            "5A".to_string(),
        ),
    ];
    let brickplot = BrickPlot::new()
        .with_names(vec!["read_1", "read_2"])
        .with_strigars(strigars);

    let plots = vec![Plot::Brick(brickplot)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/brickplot_stitched_canonical.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // ACCCTA, TAACCC, CCCTAA all same canonical → single motif colour in SVG
    // Gaps present → grey bricks
    assert!(svg.contains("#c8c8c8"), "gap bricks should be grey");
    // Only one non-gap motif colour should appear for the ACCCTA family
    // (global letter A = blue = #1f77b4)
    assert!(svg.contains("#1f77b4"), "ACCCTA-family should be blue (global A)");
}

// ── Bladerunner format spec tests ────────────────────────────────────────────
//
// These tests correspond to the formal bladerunner format specification covering
// motifs, STRIGAR, and traditional (human-readable) encoding.

#[test]
fn test_brick_spec_form_b_gap_width() {
    // Spec §6, Form B: N@ in STRIGAR with NO matching @:{seq} motifs entry.
    // The gap width is N nucleotides, taken directly from the STRIGAR count.
    // Spec: "gap of N nucleotides … N is already in nt"
    //
    // motifs: 2 segments — CAG:A, TGC:A
    // STRIGAR: 3 segments — 3A | 30@ | 2A
    // The 30@ has no motifs entry → form B → 30 grey bricks of width 1 each.
    //
    // Total row width = 3*3 (CAG) + 30 (gap) + 2*3 (TGC) = 9 + 30 + 6 = 45 nt.
    let bp = BrickPlot::new()
        .with_names(vec!["r1"])
        .with_strigars(vec![(
            "CAG:A | TGC:A".to_string(),
            "3A | 30@ | 2A".to_string(),
        )]);

    let x_max = Plot::Brick(bp).bounds().expect("should have bounds").0.1;
    assert!(
        (x_max - 45.0).abs() < 0.01,
        "form B 30@ gap: expected total width 45 nt, got {}",
        x_max
    );
}

#[test]
fn test_brick_spec_form_a_gap_width() {
    // Spec §6, Form A: 1@ in STRIGAR WITH a matching @:{seq} motifs entry.
    // The gap width is len(seq) × 1 nucleotides (not 1).
    // Spec: "nucleotide width = len(seq) from the @:{seq} motifs entry"
    //
    // motifs: CAG:A | @:ATGAT | TGC:A  (middle segment is the form A gap with seq "ATGAT", len=5)
    // STRIGAR: 3A | 1@ | 2A
    // Gap width = len("ATGAT") * 1 = 5 nt.
    //
    // Total row width = 3*3 (CAG) + 5 (gap) + 2*3 (TGC) = 9 + 5 + 6 = 20 nt.
    let bp = BrickPlot::new()
        .with_names(vec!["r1"])
        .with_strigars(vec![(
            "CAG:A | @:ATGAT | TGC:A".to_string(),
            "3A | 1@ | 2A".to_string(),
        )]);

    let x_max = Plot::Brick(bp).bounds().expect("should have bounds").0.1;
    assert!(
        (x_max - 20.0).abs() < 0.01,
        "form A @:ATGAT gap: expected total width 20 nt, got {}",
        x_max
    );
}

#[test]
fn test_brick_spec_form_a_vs_b_disambiguation_of_1at() {
    // Spec §6: disambiguation rule — `1@` behaves differently depending on whether
    // the current motifs position is `@:{seq}` (form A) or absent (form B).
    //
    // Form A: motifs has `@:AT` at the gap position → gap width = len("AT") = 2 nt.
    //   Total = 3*3 (CAG) + 2 (gap) + 2*3 (TGC) = 17 nt.
    //
    // Form B: motifs has no @-entry → gap width = 1 nt (count taken directly).
    //   Total = 3*3 (CAG) + 1 (gap) + 2*3 (TGC) = 16 nt.

    let form_a = BrickPlot::new()
        .with_names(vec!["r1"])
        .with_strigars(vec![(
            "CAG:A | @:AT | TGC:A".to_string(),
            "3A | 1@ | 2A".to_string(),
        )]);
    let form_b = BrickPlot::new()
        .with_names(vec!["r1"])
        .with_strigars(vec![(
            "CAG:A | TGC:A".to_string(),
            "3A | 1@ | 2A".to_string(),
        )]);

    let x_max_a = Plot::Brick(form_a).bounds().expect("form A bounds").0.1;
    let x_max_b = Plot::Brick(form_b).bounds().expect("form B bounds").0.1;

    // 3×CAG(3nt) + gap + 2×TGC(3nt) = 9 + gap + 6
    assert!(
        (x_max_a - 17.0).abs() < 0.01,
        "form A 1@ with @:AT: expected 17 nt (2-nt gap), got {}",
        x_max_a
    );
    assert!(
        (x_max_b - 16.0).abs() < 0.01,
        "form B 1@ no motifs entry: expected 16 nt (1-nt gap), got {}",
        x_max_b
    );
}

#[test]
fn test_brick_spec_bean1_sca31_renders() {
    // Full BEAN1/SCA31 locus example from the bladerunner format specification.
    // 8 segments in both motifs and STRIGAR; segments 5 and 7 are form A gaps
    // (@:AT → 2 nt wide, @:GAA → 3 nt wide).
    //
    // Cross-segment canonical unification: ATGGA (seg 3) and GAATG (seg 6) are
    // rotations of the same canonical ("AATGG") and must receive the same global
    // letter and colour. Similarly ATGA (seg 3) and AATG (seg 6) share canonical
    // "AATG".
    let strigars = vec![(
        "ATAAA:A,AT:B | ATA:A | ATGGA:A,TGGA:B,ATGA:C,AGA:D | ATA:A | @:AT | GAATG:A,AATG:B | @:GAA | TAA:A,A:B".to_string(),
        "22A1B22A | 27A | 61A1B154A1C78A1C18A1D24A1C2A1C75A1C80A1C74A1C117A | 9A | 1@ | 129A1B93A | 1@ | 11A1B1A1B2A2B1A2B2A".to_string(),
    )];
    let brickplot = BrickPlot::new()
        .with_names(vec!["SCA31_read"])
        .with_strigars(strigars);

    let plots = vec![Plot::Brick(brickplot)];
    let layout = Layout::auto_from_plots(&plots).with_title("BEAN1/SCA31 locus (spec example)");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/brickplot_bean1_sca31.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"), "must produce valid SVG");
    // Form A gaps (AT=2nt, GAA=3nt) produce grey bricks.
    assert!(svg.contains("#c8c8c8"), "form A gap segments must render as grey bricks");
    // At least the primary motif colour must appear.
    assert!(svg.contains("#1f77b4"), "primary motif (global A) must use the first palette colour");
}

#[test]
fn test_brick_spec_bean1_sca31_gap_widths() {
    // The BEAN1/SCA31 example has two form A gaps:
    //   segment 5: @:AT  → 2 nt
    //   segment 7: @:GAA → 3 nt
    // Verify the overall row width matches the expected nucleotide total.
    //
    // Segment nucleotide widths:
    //   1: ATAAA(5)×22 + AT(2)×1 + ATAAA(5)×22 = 110 + 2 + 110 = 222
    //   2: ATA(3)×27 = 81
    //   3: ATGGA(5)×(61+154+78+18+24+2+75+80+74+117)  [A counts summed]
    //      + TGGA(4)×1 + ATGA(4)×(1+1+1+1+1+1+1) + AGA(3)×1
    //      A-runs total: 61+154+78+18+24+2+75+80+74+117 = 683 → 683*5 = 3415
    //      B-runs: 1*4 = 4
    //      C-runs: (1+1+1+1+1+1+1)*4 = 7*4 = 28   ← 7 C-tokens in the STRIGAR
    //      D-runs: 1*3 = 3
    //      = 3415 + 4 + 28 + 3 = 3450
    //   4: ATA(3)×9 = 27
    //   5: gap AT = 2
    //   6: GAATG(5)×129 + AATG(4)×1 + GAATG(5)×93
    //      = (129+93)*5 + 4 = 222*5 + 4 = 1110 + 4 = 1114
    //   7: gap GAA = 3
    //   8: TAA(3)×(11+1+2+1+2) + A(1)×(1+1+2+2)
    //      = 17*3 + 6*1 = 51 + 6 = 57
    //
    // Grand total = 222 + 81 + 3450 + 27 + 2 + 1114 + 3 + 57 = 4956 nt
    //
    // Note: GAATG and ATGGA are rotations of canonical "AATGG" → same global letter
    // (5-mer, length 5). AATG and ATGA are rotations of canonical "AATG" → same
    // global letter (4-mer, length 4). AGA (canonical "AAG", 3-mer, length 3).
    // ATA and TAA are rotations of canonical "AAT" → same global letter (3-mer).
    let strigars = vec![(
        "ATAAA:A,AT:B | ATA:A | ATGGA:A,TGGA:B,ATGA:C,AGA:D | ATA:A | @:AT | GAATG:A,AATG:B | @:GAA | TAA:A,A:B".to_string(),
        "22A1B22A | 27A | 61A1B154A1C78A1C18A1D24A1C2A1C75A1C80A1C74A1C117A | 9A | 1@ | 129A1B93A | 1@ | 11A1B1A1B2A2B1A2B2A".to_string(),
    )];
    let bp = BrickPlot::new()
        .with_names(vec!["SCA31_read"])
        .with_strigars(strigars);

    let x_max = Plot::Brick(bp).bounds().expect("should have bounds").0.1;
    assert!(
        (x_max - 4956.0).abs() < 0.01,
        "BEAN1/SCA31 total width: expected 4956 nt, got {}",
        x_max
    );
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
        ("ACGTACGT", "CAG:A,CCG:B",        "8A1B10A",     "TGCATGCA"),
        ("ACGTACGT", "CAG:A",              "20A",          "TGCA"),
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
    std::fs::write("test_outputs/brickplot_spec_full_pipeline.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"), "must produce valid SVG");
    // Per-block labels appear above consensus row (auto-generated from run-length encoding).
    assert!(svg.contains("(CAG)6"),  "run of 6 A bricks must produce (CAG)6 label");
    assert!(svg.contains("(CAA)1"),  "run of 1 B brick must produce (CAA)1 label");
    assert!(svg.contains("(CAG)2"),  "run of 2 A bricks must produce (CAG)2 label");
    assert!(svg.contains("(CCG)1"),  "run of 1 C brick must produce (CCG)1 label");
    assert!(svg.contains("(CAG)10"), "run of 10 A bricks must produce (CAG)10 label");
    // Primary motif has '*' in legend.
    assert!(svg.contains("*"), "mark_primary must append * to primary motif legend label");
    // DNA flank colour must appear (A in flanks → #009600).
    assert!(svg.contains("#009600"), "DNA flank bricks must appear");
}

#[test]
fn test_brick_spec_multi_segment_single_candidate() {
    // Spec §1: single candidate (no | separator) — simple round-trip.
    // motifs: CAG:A,CAA:B,CCG:C — three motifs in one segment.
    // STRIGAR: 2A1B2A1C10A → (CAG)2(CAA)1(CAG)2(CCG)1(CAG)10.
    // Total width = (2+2+10)*3 + 1*3 + 1*3 = 42 + 3 + 3 = 48 nt.
    let bp = BrickPlot::new()
        .with_names(vec!["r1"])
        .with_strigars(vec![(
            "CAG:A,CAA:B,CCG:C".to_string(),
            "2A1B2A1C10A".to_string(),
        )]);

    let x_max = Plot::Brick(bp).bounds().expect("bounds").0.1;
    assert!(
        (x_max - 48.0).abs() < 0.01,
        "single-segment 3-motif: expected 48 nt, got {}",
        x_max
    );
}

#[test]
fn test_brick_spec_segment_count_mismatch_form_b() {
    // Spec §4: "The motifs string has one fewer segment than the STRIGAR string
    // when a form B gap is present."
    // motifs: 2 segments, STRIGAR: 3 segments (2 repeat + 1 form-B gap).
    // Both must parse without panic.
    //
    // motifs: ATAAA:A | ATA:A
    // STRIGAR: 10A | 50@ | 5A
    // Width = 10*5 + 50 + 5*3 = 50 + 50 + 15 = 115 nt.
    let bp = BrickPlot::new()
        .with_names(vec!["r1"])
        .with_strigars(vec![(
            "ATAAA:A | ATA:A".to_string(),
            "10A | 50@ | 5A".to_string(),
        )]);

    let x_max = Plot::Brick(bp).bounds().expect("bounds").0.1;
    assert!(
        (x_max - 115.0).abs() < 0.01,
        "form B 50@ gap (2 motif segs, 3 strigar segs): expected 115 nt, got {}",
        x_max
    );
}
