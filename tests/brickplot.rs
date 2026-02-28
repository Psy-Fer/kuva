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
