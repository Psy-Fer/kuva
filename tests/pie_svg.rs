use visus::plot::{PiePlot, PieLabelPosition};
use visus::backend::svg::SvgBackend;
use visus::render::render::{render_pie, render_multiple};
use visus::render::layout::Layout;
use visus::render::plots::Plot;


#[test]
fn test_pie_basic() {
    let pie = PiePlot::new()
                    .with_slice("hot sauce", 35.0, "green")
                    .with_slice("cheese", 25.0, "orange")
                    .with_slice("beans", 40.0, "tomato")
                    .with_inner_radius(80.0);


    let plots = vec![Plot::Pie(pie.clone())];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Pie Plot");

    let scene = render_pie(&pie, &layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/pie_builder.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
}

#[test]
fn test_pie_outside_labels_with_percent() {
    let pie = PiePlot::new()
        .with_slice("Large", 60.0, "steelblue")
        .with_slice("Small A", 3.0, "tomato")
        .with_slice("Small B", 2.0, "orange")
        .with_slice("Small C", 2.0, "gold")
        .with_slice("Medium", 15.0, "seagreen")
        .with_slice("Tiny", 1.0, "purple")
        .with_slice("Rest", 17.0, "gray")
        .with_percent()
        .with_label_position(PieLabelPosition::Outside);

    let plots = vec![Plot::Pie(pie.clone())];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Pie - Outside Labels + Percent");

    let scene = render_pie(&pie, &layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/pie_outside_percent.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // All labels should show percentages
    assert!(svg.contains("60.0%"));
    // Leader lines should be present
    assert!(svg.contains("stroke=\"#666\""));
}

#[test]
fn test_pie_auto_labels() {
    let pie = PiePlot::new()
        .with_slice("Big Slice", 70.0, "steelblue")
        .with_slice("Tiny A", 2.0, "tomato")
        .with_slice("Tiny B", 1.5, "orange")
        .with_slice("Small", 4.0, "gold")
        .with_slice("Medium", 22.5, "seagreen")
        .with_percent()
        .with_inner_radius(60.0);

    let plots = vec![Plot::Pie(pie.clone())];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Pie - Auto Label Position");

    let scene = render_pie(&pie, &layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/pie_auto_labels.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // Small slices should have leader lines (outside)
    assert!(svg.contains("stroke=\"#666\""));
}

#[test]
fn test_pie_no_labels() {
    let pie = PiePlot::new()
        .with_slice("A", 30.0, "steelblue")
        .with_slice("B", 30.0, "tomato")
        .with_slice("C", 40.0, "seagreen")
        .with_label_position(PieLabelPosition::None);

    let plots = vec![Plot::Pie(pie.clone())];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Pie - No Labels");

    let scene = render_pie(&pie, &layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/pie_no_labels.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // Should not contain slice label text (only the title text)
    assert!(!svg.contains(">A<"));
    assert!(!svg.contains(">B<"));
    assert!(!svg.contains(">C<"));
}

#[test]
fn test_pie_legend_per_slice() {
    let pie = PiePlot::new()
        .with_slice("Apples", 40.0, "green")
        .with_slice("Oranges", 35.0, "orange")
        .with_slice("Grapes", 25.0, "purple")
        .with_legend("Fruit")
        .with_percent();

    let plots = vec![Plot::Pie(pie)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Pie with Legend");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/pie_legend.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // Legend should have per-slice entries
    assert!(svg.contains("Apples"));
    assert!(svg.contains("Oranges"));
    assert!(svg.contains("Grapes"));
}
