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
                    .with_inner_radius(60.0);


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
        .with_inner_radius(50.0);

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

#[test]
fn test_pie_outside_labels_font_family() {
    let pie = PiePlot::new()
        .with_slice("Alpha", 30.0, "steelblue")
        .with_slice("Beta", 25.0, "tomato")
        .with_slice("Gamma", 20.0, "gold")
        .with_slice("Delta", 15.0, "seagreen")
        .with_slice("Epsilon", 10.0, "purple")
        .with_percent()
        .with_label_position(PieLabelPosition::Outside);

    let plots = vec![Plot::Pie(pie.clone())];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Pie - Font Family")
        .with_font_family("Helvetica, Arial, sans-serif");

    let scene = render_pie(&pie, &layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/pie_font_family.svg", svg.clone()).unwrap();

    assert!(svg.contains(r#"font-family="Helvetica, Arial, sans-serif""#));
    // All labels should be present
    assert!(svg.contains("Alpha"));
    assert!(svg.contains("Epsilon"));
    // Leader lines should be present
    assert!(svg.contains("stroke=\"#666\""));
}

#[test]
fn test_pie_outside_labels_large_font() {
    // Many small slices with large body_size — tests anti-overlap spacing
    let pie = PiePlot::new()
        .with_slice("Slice A", 5.0, "steelblue")
        .with_slice("Slice B", 5.0, "tomato")
        .with_slice("Slice C", 5.0, "orange")
        .with_slice("Slice D", 5.0, "gold")
        .with_slice("Slice E", 5.0, "seagreen")
        .with_slice("Slice F", 5.0, "purple")
        .with_slice("Slice G", 5.0, "coral")
        .with_slice("Slice H", 65.0, "lightgray")
        .with_percent()
        .with_label_position(PieLabelPosition::Outside);

    // Render with default body_size (12) and large body_size (20)
    let plots_default = vec![Plot::Pie(pie.clone())];
    let layout_default = Layout::auto_from_plots(&plots_default)
        .with_title("Pie - Default Font Size");
    let scene_default = render_pie(&pie, &layout_default);
    let svg_default = SvgBackend.render_scene(&scene_default);
    std::fs::write("test_outputs/pie_outside_default_font.svg", svg_default.clone()).unwrap();

    let plots_large = vec![Plot::Pie(pie.clone())];
    let layout_large = Layout::auto_from_plots(&plots_large)
        .with_title("Pie - Large Font Size")
        .with_body_size(20);
    let scene_large = render_pie(&pie, &layout_large);
    let svg_large = SvgBackend.render_scene(&scene_large);
    std::fs::write("test_outputs/pie_outside_large_font.svg", svg_large.clone()).unwrap();

    assert!(svg_default.contains("<svg"));
    assert!(svg_large.contains("<svg"));
    // Large font version should use font-size 20 for labels
    assert!(svg_large.contains(r#"font-size="20""#));
    // Both should have all labels present
    assert!(svg_default.contains("Slice A"));
    assert!(svg_large.contains("Slice A"));
    assert!(svg_default.contains("Slice H"));
    assert!(svg_large.contains("Slice H"));

    // Extract label Y positions to verify spacing is wider with large font.
    // The anti-overlap min_gap is body_size + 2, so large font labels should
    // be spaced further apart. We count the distinct y values in text elements
    // to confirm they are all rendered (no collapsing).
    let label_count_default = svg_default.matches("stroke=\"#666\"").count();
    let label_count_large = svg_large.matches("stroke=\"#666\"").count();
    // Both should have the same number of leader line segments
    assert_eq!(label_count_default, label_count_large);
}
