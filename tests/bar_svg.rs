use visus::plot::BarPlot;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

#[test]
fn test_bar_svg_output_builder() {
    let bar = BarPlot::new()
                        .with_bar("A", 3.2)
                        .with_bar("B", 4.7)
                        .with_bar("Longform_C", 2.8)
                        .with_color("green");
    
    let plots = vec![Plot::Bar(bar)];

    let layout = Layout::auto_from_plots(&plots)
                        .with_title("Exciting Bar Plot")
                        .with_y_label("Value");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/bar_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}

#[test]
fn test_bar_vec_svg_output_builder() {
    let barvec = vec![("A", 3.2), ("B", 4.7), ("Longform_C", 2.8)];
    let bar = BarPlot::new()
                        .with_bars(barvec)
                        .with_color("purple");
    
    let plots = vec![Plot::Bar(bar)];

    let layout = Layout::auto_from_plots(&plots)
                        .with_title("Exciting Bar Plot")
                        .with_y_label("Value");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/bar_vec_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}

#[test]
fn test_bar_categories_svg_output_builder() {
    let bar = BarPlot::new()
                    .with_group("Laptop", vec![(3.2, "tomato"), (7.8, "skyblue")])
                    .with_group("Server", vec![(5.8, "tomato"), (9.1, "skyblue")])
                    .with_legend(vec!["blow5", "pod5"]);

    let plots = vec![Plot::Bar(bar)];

    let layout = Layout::auto_from_plots(&plots)
                        .with_title("Software Performance")
                        .with_y_label("Time")
                        .with_ticks(20);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/bar_categories_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}

#[test]
fn test_bar_stacked() {
    let bar = BarPlot::new()
        .with_group("Q1", vec![(10.0, "tomato"), (15.0, "skyblue"), (8.0, "gold")])
        .with_group("Q2", vec![(12.0, "tomato"), (10.0, "skyblue"), (14.0, "gold")])
        .with_group("Q3", vec![(8.0, "tomato"), (18.0, "skyblue"), (6.0, "gold")])
        .with_legend(vec!["Product A", "Product B", "Product C"])
        .with_stacked();

    let plots = vec![Plot::Bar(bar)];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Stacked Bar Plot")
        .with_y_label("Revenue");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/bar_stacked.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("tomato"));
    assert!(svg.contains("skyblue"));
    assert!(svg.contains("gold"));
}