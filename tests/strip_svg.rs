use kuva::plot::{BoxPlot, ViolinPlot, LegendEntry, LegendShape, LegendPosition};
use kuva::plot::StripPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::Palette;

#[test]
fn test_strip_basic() {
    let strip = StripPlot::new()
        .with_group("A", vec![1.0, 2.0, 2.5, 3.1, 4.0, 3.5, 2.8])
        .with_group("B", vec![2.0, 2.1, 3.5, 3.8, 4.0, 4.2, 5.0])
        .with_group("C", vec![0.5, 1.5, 1.8, 2.2, 3.0, 3.3, 4.5])
        .with_color("steelblue");

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Strip Plot Basic")
        .with_y_label("Values");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/strip_basic.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_strip_swarm() {
    let strip = StripPlot::new()
        .with_group("Control", vec![1.0, 1.2, 1.5, 1.8, 2.0, 2.1, 2.3, 2.5, 2.7, 3.0])
        .with_group("Treatment", vec![2.5, 2.7, 3.0, 3.2, 3.5, 3.8, 4.0, 4.2, 4.5, 5.0])
        .with_color("coral")
        .with_swarm()
        .with_point_size(5.0);

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Strip Plot - Swarm Layout")
        .with_y_label("Measurement");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/strip_swarm.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_strip_center() {
    let strip = StripPlot::new()
        .with_group("Group1", vec![1.0, 2.0, 3.0, 4.0, 5.0])
        .with_group("Group2", vec![1.5, 2.5, 3.5, 4.5])
        .with_color("purple")
        .with_center()
        .with_point_size(3.0);

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Strip Plot - Center Layout")
        .with_y_label("Values");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/strip_center.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_strip_legend_palette() {
    let strip_a = StripPlot::new()
        .with_group("WT", vec![1.0, 1.5, 2.0, 2.5, 3.0])
        .with_legend("Wild Type");

    let strip_b = StripPlot::new()
        .with_group("KO", vec![2.0, 2.5, 3.0, 3.5, 4.0])
        .with_legend("Knockout");

    let plots = vec![Plot::Strip(strip_a), Plot::Strip(strip_b)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Strip Plot - Palette + Legend")
        .with_y_label("Expression")
        .with_palette(Palette::wong());

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/strip_legend_palette.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_box_with_strip_overlay() {
    let boxplot = BoxPlot::new()
        .with_group("A", vec![1.0, 2.0, 2.5, 3.0, 4.0, 5.0, 2.2, 3.3])
        .with_group("B", vec![2.0, 2.1, 3.5, 3.8, 4.0, 4.2, 5.5, 3.0])
        .with_group("C", vec![0.5, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0])
        .with_color("steelblue")
        .with_strip(0.25);

    let plots = vec![Plot::Box(boxplot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Box Plot with Strip Overlay")
        .with_y_label("Values");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/box_with_strip_overlay.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_box_with_swarm_overlay() {
    let boxplot = BoxPlot::new()
        .with_group("Control", vec![1.0, 1.2, 1.5, 1.8, 2.0, 2.1, 2.3, 2.5, 2.7, 3.0])
        .with_group("Treated", vec![2.5, 2.7, 3.0, 3.2, 3.5, 3.8, 4.0, 4.2, 4.5, 5.0])
        .with_color("lightblue")
        .with_swarm_overlay()
        .with_overlay_color("rgba(30,100,200,0.7)")
        .with_overlay_size(4.0);

    let plots = vec![Plot::Box(boxplot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Box Plot with Swarm Overlay")
        .with_y_label("Measurement");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/box_with_swarm_overlay.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_violin_with_strip_overlay() {
    let violin = ViolinPlot::new()
        .with_group("Alpha", vec![1.0, 1.5, 2.0, 2.2, 2.8, 3.0, 3.5, 4.0])
        .with_group("Beta", vec![2.0, 2.5, 3.0, 3.1, 3.5, 4.0, 4.2, 5.0])
        .with_color("mediumpurple")
        .with_strip(0.2)
        .with_overlay_color("rgba(0,0,0,0.5)")
        .with_overlay_size(3.0);

    let plots = vec![Plot::Violin(violin)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Violin Plot with Strip Overlay")
        .with_y_label("Values");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/violin_with_strip_overlay.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_strip_group_colors() {
    let strip = StripPlot::new()
        .with_group("A", vec![1.0, 2.0, 2.5, 3.1])
        .with_group("B", vec![2.0, 2.1, 3.5, 3.8])
        .with_group("C", vec![0.5, 1.5, 1.8, 2.2])
        .with_color("black")
        .with_group_colors(vec!["red", "green", "blue"]);

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Strip Plot Group Colors")
        .with_y_label("Values");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/strip_group_colors.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains(r##"fill="#ff0000""##));
    assert!(svg.contains(r##"fill="#008000""##));
    assert!(svg.contains(r##"fill="#0000ff""##));
}

#[test]
fn test_strip_and_box_composed() {
    // Box and Strip sharing the same categorical x-axis
    let boxplot = BoxPlot::new()
        .with_group("A", vec![1.0, 2.0, 2.5, 3.0, 4.0, 5.0])
        .with_group("B", vec![2.0, 2.5, 3.5, 4.0, 4.5, 5.0])
        .with_color("lightblue")
        .with_legend("Boxes");

    let strip = StripPlot::new()
        .with_group("A", vec![1.0, 2.0, 2.5, 3.0, 4.0, 5.0])
        .with_group("B", vec![2.0, 2.5, 3.5, 4.0, 4.5, 5.0])
        .with_color("rgba(200,50,50,0.7)")
        .with_jitter(0.15)
        .with_point_size(3.5)
        .with_legend("Points");

    let plots = vec![Plot::Box(boxplot), Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Box + Strip Composed")
        .with_y_label("Values");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/strip_and_box_composed.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_strip_point_colors_full() {
    // All points carry individual colors — simulates coloring by motif type
    let strip = StripPlot::new()
        .with_colored_group("Sample A", vec![
            (1.2, "steelblue"),
            (2.4, "tomato"),
            (1.8, "seagreen"),
            (3.1, "goldenrod"),
            (2.0, "mediumpurple"),
            (2.7, "steelblue"),
            (1.5, "tomato"),
        ])
        .with_colored_group("Sample B", vec![
            (2.2, "tomato"),
            (3.3, "seagreen"),
            (2.8, "steelblue"),
            (1.9, "goldenrod"),
            (3.5, "mediumpurple"),
        ])
        .with_swarm()
        .with_point_size(5.0);

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Strip — Per-point Colors")
        .with_y_label("Value");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/strip_point_colors_full.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("#ff6347"),   "SVG should contain tomato (#ff6347)");
    assert!(svg.contains("seagreen"),  "SVG should contain seagreen color");
    assert!(svg.contains("goldenrod"), "SVG should contain goldenrod color");
}

#[test]
fn test_strip_point_colors_mixed() {
    // One colored group alongside a plain group — plain group uses uniform color fallback
    let strip = StripPlot::new()
        .with_colored_group("Motifs", vec![
            (1.0, "tomato"),
            (2.0, "seagreen"),
            (3.0, "goldenrod"),
            (2.5, "mediumpurple"),
        ])
        .with_group("Control", vec![1.5, 2.5, 3.5])
        .with_color("steelblue")  // fallback for the plain group
        .with_jitter(0.2);

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Strip — Mixed Colored and Plain Groups")
        .with_y_label("Value");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/strip_point_colors_mixed.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("#ff6347"),   "SVG should contain tomato (#ff6347) from colored group");
    assert!(svg.contains("#4682b4"),   "SVG should contain steelblue (#4682b4) fallback for plain group");
}

#[test]
fn test_strip_point_colors_with_legend() {
    // Motif categories — each gets a color and a legend entry
    let motifs = [
        ("ATTC", "tomato"),
        ("GCGC", "seagreen"),
        ("ATAT", "goldenrod"),
        ("CGCG", "mediumpurple"),
    ];

    // Build (value, color) points — simulate repeat counts per motif occurrence
    let points: Vec<(f64, &str)> = vec![
        (4.0, "tomato"), (5.0, "tomato"), (4.5, "tomato"),
        (7.0, "seagreen"), (8.0, "seagreen"), (7.5, "seagreen"),
        (3.0, "goldenrod"), (3.5, "goldenrod"),
        (9.0, "mediumpurple"), (10.0, "mediumpurple"), (9.5, "mediumpurple"),
    ];

    let strip = StripPlot::new()
        .with_colored_group("Sample", points)
        .with_swarm()
        .with_point_size(5.0);

    // Manual legend — one circle swatch per motif category
    let legend_entries: Vec<LegendEntry> = motifs.iter().map(|(label, color)| LegendEntry {
        label: label.to_string(),
        color: color.to_string(),
        shape: LegendShape::Circle,
        dasharray: None,
    }).collect();

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("STR Motif Repeat Counts")
        .with_x_label("Sample")
        .with_y_label("Repeat count")
        .with_legend_title("Motif")
        .with_legend_entries(legend_entries)
        .with_legend_position(LegendPosition::OutsideRightTop);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/strip_point_colors_legend.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("STR Motif"), "SVG should contain the title");
    assert!(svg.contains("Motif"),     "SVG should contain the legend title");
    assert!(svg.contains("ATTC"),      "SVG should contain ATTC legend entry");
    assert!(svg.contains("GCGC"),      "SVG should contain GCGC legend entry");
    assert!(svg.contains("ATAT"),      "SVG should contain ATAT legend entry");
    assert!(svg.contains("CGCG"),      "SVG should contain CGCG legend entry");
    // All four motif colors should appear in the SVG
    assert!(svg.contains("#ff6347"),   "tomato (#ff6347) should appear");
    assert!(svg.contains("seagreen"),  "seagreen should appear");
    assert!(svg.contains("goldenrod"), "goldenrod should appear");
    assert!(svg.contains("mediumpurple"), "mediumpurple should appear");
}
