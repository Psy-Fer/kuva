use kuva::plot::brick::BrickTemplate;
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
