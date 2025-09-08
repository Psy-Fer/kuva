use visus::plot::brick::BrickTemplate;
use visus::plot::BrickPlot;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;


#[test]
fn test_brickplot_svg_output_builder() {


    let sequences: Vec<String> = vec![
       "ACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATTCAT".to_string(),
       "ACTCATCATCATCATCATCATCATCATCATCATCATCATCATCATTCAT".to_string(),
       "ACTCATCATCATCATCATCATCATCATCATCATCATCATCATCATTCAT".to_string(),
       "ACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATTCAT".to_string(),
       "ACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATTCAT".to_string(),
       "ACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATCATCATCATCATGGTCATCATCATCATCAT".to_string(),
       "ACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATCAT".to_string(),
       "ACTCATCATCATCATCATCATCATCATCATCATCCATCATCATCATTCAT".to_string(),
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
                        .with_template(b.template);
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
