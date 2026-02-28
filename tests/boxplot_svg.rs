use kuva::plot::BoxPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;


#[test]
fn test_boxplot_groups_svg_output_builder() {
    let boxplot = BoxPlot::new()
    .with_group("A", vec![1.0, 2.0, 2.5, 3.0, 4.0, 5.0])
    .with_group("B", vec![2.0, 2.1, 3.5, 3.8, 4.0, 4.2])
    .with_color("darkred");

    // let x_labels: Vec<String> = boxplot.groups.iter().map(|g| g.label.clone()).collect();
    
    let plots = vec![Plot::Box(boxplot)];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Box Plot")
        .with_y_label("Values");
        // .with_x_categories(x_labels);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/boxplot_groups_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}

#[test]
fn test_boxplot_svg_output_builder() {
    let boxplot = BoxPlot::new()
    .with_group("A", vec![1.0, 2.0, 2.5, 3.0, 4.0, 5.0])
    .with_color("darkred");

    let plots = vec![Plot::Box(boxplot)];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Box Plot")
        .with_y_label("Values");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/boxplot_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}



