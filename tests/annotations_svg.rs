use visus::plot::scatter::ScatterPlot;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;
use visus::render::annotations::{TextAnnotation, ReferenceLine, ShadedRegion};

#[test]
fn test_text_annotation_with_arrow() {
    let data = vec![
        (1.0, 2.0),
        (2.0, 4.0),
        (3.0, 3.0),
        (4.0, 7.0),
        (5.0, 5.0),
        (6.0, 9.0),
    ];

    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_size(4.0);

    let plots = vec![Plot::Scatter(scatter)];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Text Annotation with Arrow")
        .with_x_label("X")
        .with_y_label("Y")
        .with_annotation(
            TextAnnotation::new("Outlier!", 5.0, 7.5)
                .with_arrow(6.0, 9.0)
                .with_color("red")
        );

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/annotation_arrow.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Outlier!"));
    assert!(svg.contains("<path")); // arrowhead
}

#[test]
fn test_reference_lines() {
    let data = vec![
        (1.0, 2.0),
        (2.0, 5.0),
        (3.0, 3.0),
        (4.0, 8.0),
        (5.0, 6.0),
    ];

    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("navy")
        .with_size(4.0);

    let plots = vec![Plot::Scatter(scatter)];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Reference Lines")
        .with_x_label("X")
        .with_y_label("Y")
        .with_reference_line(
            ReferenceLine::horizontal(5.0)
                .with_color("red")
                .with_label("y = 5")
        )
        .with_reference_line(
            ReferenceLine::vertical(3.0)
                .with_color("green")
                .with_label("x = 3")
        );

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/reference_lines.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("stroke-dasharray"));
    assert!(svg.contains("y = 5"));
    assert!(svg.contains("x = 3"));
}

#[test]
fn test_shaded_regions() {
    let data = vec![
        (1.0, 1.0),
        (2.0, 4.0),
        (3.0, 2.0),
        (4.0, 6.0),
        (5.0, 3.0),
        (6.0, 8.0),
    ];

    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("darkgreen")
        .with_size(4.0);

    let plots = vec![Plot::Scatter(scatter)];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Shaded Regions")
        .with_x_label("X")
        .with_y_label("Y")
        .with_shaded_region(
            ShadedRegion::horizontal(3.0, 5.0)
                .with_color("orange")
                .with_opacity(0.2)
        )
        .with_shaded_region(
            ShadedRegion::vertical(2.0, 4.0)
                .with_color("blue")
                .with_opacity(0.1)
        );

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/shaded_regions.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("fill-opacity"));
}

#[test]
fn test_all_annotations_combined() {
    let data = vec![
        (1.0, 2.0),
        (2.0, 5.0),
        (3.0, 3.0),
        (4.0, 8.0),
        (5.0, 6.0),
        (6.0, 10.0),
    ];

    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("purple")
        .with_size(4.0);

    let plots = vec![Plot::Scatter(scatter)];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("All Annotations Combined")
        .with_x_label("X")
        .with_y_label("Y")
        .with_shaded_region(
            ShadedRegion::horizontal(4.0, 7.0)
                .with_color("yellow")
                .with_opacity(0.15)
        )
        .with_reference_line(
            ReferenceLine::horizontal(5.0)
                .with_color("red")
                .with_label("threshold")
        )
        .with_reference_line(
            ReferenceLine::vertical(3.5)
                .with_color("blue")
                .with_label("midpoint")
        )
        .with_annotation(
            TextAnnotation::new("Peak", 5.0, 8.5)
                .with_arrow(6.0, 10.0)
                .with_color("darkred")
                .with_font_size(14)
        );

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/annotations_combined.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("fill-opacity"));      // shaded region
    assert!(svg.contains("stroke-dasharray"));  // reference line
    assert!(svg.contains("Peak"));              // text annotation
    assert!(svg.contains("threshold"));         // reference line label
    assert!(svg.contains("midpoint"));          // reference line label
}
