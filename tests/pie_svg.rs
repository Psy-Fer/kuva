mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::{PieLabelPosition, PiePlot};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::{render_multiple, render_pie};

#[test]
fn test_pie_basic() {
    let pie = PiePlot::new()
        .with_slice("hot sauce", 35.0, "green")
        .with_slice("cheese", 25.0, "orange")
        .with_slice("beans", 40.0, "tomato")
        .with_inner_radius(60.0);

    let plots = vec![Plot::Pie(pie.clone())];

    let layout = Layout::auto_from_plots(&plots).with_title("Pie Plot");

    let scene = render_pie(&pie, &layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/pie_builder.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
}

#[test]
fn test_pie_single_slice_is_a_full_circle() {
    let pie = PiePlot::new()
        .with_slice("Only slice", 100.0, "steelblue")
        .with_label_position(PieLabelPosition::None);
    let plots = vec![Plot::Pie(pie.clone())];
    let layout = Layout::auto_from_plots(&plots);

    let svg = SvgBackend.render_scene(&render_pie(&pie, &layout));
    let path_data = svg
        .split("<path d=\"")
        .nth(1)
        .and_then(|path| path.split('"').next())
        .expect("single slice should render a path");

    assert_eq!(svg.matches("<path ").count(), 1);
    assert_eq!(path_data.matches(" A").count(), 2);
    assert!(!path_data.contains(" L"));
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
    let layout = Layout::auto_from_plots(&plots).with_title("Pie - Outside Labels + Percent");

    let scene = render_pie(&pie, &layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/pie_outside_percent.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // All labels should show percentages
    assert!(svg.contains("60.0%"));
    // Leader lines should be present
    assert!(svg.contains("stroke=\"#666666\""));
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
    let layout = Layout::auto_from_plots(&plots).with_title("Pie - Auto Label Position");

    let scene = render_pie(&pie, &layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/pie_auto_labels.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // Small slices should have leader lines (outside)
    assert!(svg.contains("stroke=\"#666666\""));
}

#[test]
fn test_pie_no_labels() {
    let pie = PiePlot::new()
        .with_slice("A", 30.0, "steelblue")
        .with_slice("B", 30.0, "tomato")
        .with_slice("C", 40.0, "seagreen")
        .with_label_position(PieLabelPosition::None);

    let plots = vec![Plot::Pie(pie.clone())];
    let layout = Layout::auto_from_plots(&plots).with_title("Pie - No Labels");

    let scene = render_pie(&pie, &layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/pie_no_labels.svg", svg.clone()).unwrap();

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
    let layout = Layout::auto_from_plots(&plots).with_title("Pie with Legend");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/pie_legend.svg", svg.clone()).unwrap();

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
    common::write_test_output("test_outputs/pie_font_family.svg", svg.clone()).unwrap();

    assert!(svg.contains(r#"font-family="Helvetica, Arial, sans-serif""#));
    // All labels should be present
    assert!(svg.contains("Alpha"));
    assert!(svg.contains("Epsilon"));
    // Leader lines should be present
    assert!(svg.contains("stroke=\"#666666\""));
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
    let layout_default =
        Layout::auto_from_plots(&plots_default).with_title("Pie - Default Font Size");
    let scene_default = render_pie(&pie, &layout_default);
    let svg_default = SvgBackend.render_scene(&scene_default);
    common::write_test_output(
        "test_outputs/pie_outside_default_font.svg",
        svg_default.clone(),
    )
    .unwrap();

    let plots_large = vec![Plot::Pie(pie.clone())];
    let layout_large = Layout::auto_from_plots(&plots_large)
        .with_title("Pie - Large Font Size")
        .with_body_size(20);
    let scene_large = render_pie(&pie, &layout_large);
    let svg_large = SvgBackend.render_scene(&scene_large);
    common::write_test_output("test_outputs/pie_outside_large_font.svg", svg_large.clone())
        .unwrap();

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
    let label_count_default = svg_default.matches("stroke=\"#666666\"").count();
    let label_count_large = svg_large.matches("stroke=\"#666666\"").count();
    // Both should have the same number of leader line segments
    assert_eq!(label_count_default, label_count_large);
}

// Companion to `test_pie_single_slice_is_a_full_circle` for the donut branch
// (`with_inner_radius`), which the plain-pie test never reaches.
//
// Two regressions live here. The path was built with `\\` at a line break in
// the format string, which is an *escaped backslash* in a normal Rust string
// literal, not a line continuation: a literal `\` plus the newline and its
// indentation landed in the `d` attribute and truncated the path at the first
// parser that hit it. And the hole was drawn as a second subpath, which needs
// the nonzero rule applied across subpaths; kuva's raster and terminal
// backends fill each subpath on its own, so the hole came out solid.
#[test]
fn test_pie_single_slice_donut_is_a_ring() {
    let pie = PiePlot::new()
        .with_slice("Only slice", 100.0, "steelblue")
        .with_inner_radius(60.0)
        .with_label_position(PieLabelPosition::None);
    let plots = vec![Plot::Pie(pie.clone())];
    let layout = Layout::auto_from_plots(&plots);

    let svg = SvgBackend.render_scene(&render_pie(&pie, &layout));
    let path_data = svg
        .split("<path d=\"")
        .nth(1)
        .and_then(|path| path.split('"').next())
        .expect("single slice should render a path");

    // No stray backslash or newline: both are invalid in path data and make
    // renderers abandon the rest of the path.
    assert!(
        !path_data.contains('\\'),
        "path data must not contain a backslash: {path_data}"
    );
    assert!(
        !path_data.contains('\n'),
        "path data must not contain a newline: {path_data}"
    );

    // One subpath (a single `M`), so the hole does not depend on cross-subpath
    // winding that kuva's own backends never evaluate.
    assert_eq!(
        path_data.matches('M').count(),
        1,
        "donut must be one subpath: {path_data}"
    );

    // Four half-arcs (two outer, two inner) joined by the `L` that steps in to
    // the inner radius.
    assert_eq!(
        path_data.matches(" A").count(),
        4,
        "expected two outer + two inner half-arcs: {path_data}"
    );
    assert_eq!(
        path_data.matches(" L").count(),
        1,
        "expected one line in to the inner radius: {path_data}"
    );
}

// The SVG assertions above check path *structure*; this checks what actually
// gets painted. A two-subpath donut still produces plausible-looking SVG, but
// kuva's rasterizer fills each subpath independently, so the hole comes out
// solid. Only a rendered pixel catches that.
#[cfg(feature = "png")]
#[test]
fn test_pie_single_slice_donut_hole_is_not_filled() {
    use image::GenericImageView;

    let pie = PiePlot::new()
        .with_slice("Only slice", 100.0, "steelblue")
        .with_inner_radius(60.0)
        .with_label_position(PieLabelPosition::None);
    let plots = vec![Plot::Pie(pie)];
    let layout = Layout::auto_from_plots(&plots)
        .with_width(400.0)
        .with_height(400.0);
    let png = kuva::render_to_raster(plots, layout, 1.0).expect("raster render");

    let img = image::load_from_memory(&png).expect("decode png");
    let (w, h) = img.dimensions();
    let centre = img.get_pixel(w / 2, h / 2);
    let steelblue = [70u8, 130, 180];

    assert_ne!(
        [centre[0], centre[1], centre[2]],
        steelblue,
        "the donut hole at ({}, {}) is filled with the slice colour, so the \
         inner ring did not subtract; got {:?}",
        w / 2,
        h / 2,
        centre
    );
}

#[test]
fn test_pie_without_valid_total() {
    for values in [[0.0, 0.0], [3.0, -1.0], [3.0, f64::NAN]] {
        let pie = PiePlot::new()
            .with_slice("a", values[0], "red")
            .with_slice("b", values[1], "blue")
            .with_legend("Slices")
            .with_percent();
        let plots = vec![Plot::Pie(pie.clone())];
        let layout = Layout::auto_from_plots(&plots);
        let direct = SvgBackend.render_scene(&render_pie(&pie, &layout));
        let combined = SvgBackend.render_scene(&render_multiple(plots, layout));
        for svg in [&direct, &combined] {
            assert!(!svg.contains("NaN") && !svg.contains("inf"), "{values:?}");
            assert!(!svg.contains("<path"), "{values:?}");
            assert!(
                !svg.contains("%)") && !svg.contains("%</text>"),
                "{values:?}"
            );
        }
        assert!(combined.contains(">a</text>"), "{values:?}");
    }
}
