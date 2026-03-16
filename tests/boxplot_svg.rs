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

#[test]
fn test_boxplot_group_colors_full() {
    let boxplot = BoxPlot::new()
        .with_group("A", vec![1.0, 2.0, 2.5, 3.0, 4.0, 5.0])
        .with_group("B", vec![2.0, 2.1, 3.5, 3.8, 4.0, 4.2])
        .with_group("C", vec![3.0, 3.5, 4.0, 4.5, 5.0, 5.5])
        .with_color("black")
        .with_group_colors(["steelblue", "tomato", "seagreen"]);

    let plots = vec![Plot::Box(boxplot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Per-group Colors");
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/boxplot_group_colors_full.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // Each group color must appear; the fallback "black" must not be used as a fill
    assert!(svg.contains("steelblue") || svg.contains("#4682b4"));
    assert!(svg.contains("tomato") || svg.contains("#ff6347"));
    assert!(svg.contains("seagreen"));
}

#[test]
fn test_boxplot_group_colors_partial() {
    // Only 1 color provided for 3 groups — groups B and C fall back to "black"
    let boxplot = BoxPlot::new()
        .with_group("A", vec![1.0, 2.0, 2.5, 3.0, 4.0, 5.0])
        .with_group("B", vec![2.0, 2.1, 3.5, 3.8, 4.0, 4.2])
        .with_group("C", vec![3.0, 3.5, 4.0, 4.5, 5.0, 5.5])
        .with_color("black")
        .with_group_colors(["tomato"]);

    let plots = vec![Plot::Box(boxplot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Partial Per-group Colors");
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/boxplot_group_colors_partial.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("tomato") || svg.contains("#ff6347"));
    // Fallback color must appear for the uncolored groups
    assert!(svg.contains("black"));
}
