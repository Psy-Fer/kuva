use visus::plot::PiePlot;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_pie;
use visus::render::layout::Layout;
use visus::render::plots::Plot;


#[test]
fn test_boxplot_groups_svg_output_builder() {
    let pie = PiePlot::new()
                    .with_slice("hot sauce", 35.0, "green")
                    .with_slice("cheese", 25.0, "orange")
                    .with_slice("beans", 40.0, "tomato")
                    .with_inner_radius(80.0);


    // let x_labels: Vec<String> = boxplot.groups.iter().map(|g| g.label.clone()).collect();
    
    let plots = vec![Plot::Pie(pie.clone())];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Pie Plot");
        // .with_x_categories(x_labels);

    let scene = render_pie(&pie, &layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/pie_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}