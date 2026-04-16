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

// ── Per-point marker shapes ──────────────────────────────────────────────────

#[test]
fn test_strip_shaped_group() {
    use kuva::plot::MarkerShape;

    let strip = StripPlot::new()
        .with_shaped_group("Sample", vec![
            (1.5, MarkerShape::Circle),
            (2.3, MarkerShape::Triangle),
            (1.8, MarkerShape::Square),
            (3.1, MarkerShape::Diamond),
            (2.7, MarkerShape::Cross),
            (3.5, MarkerShape::Plus),
        ])
        .with_color("steelblue")
        .with_swarm()
        .with_point_size(5.0);

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Strip — Per-Point Shapes")
        .with_y_label("Value");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/strip_shaped.svg", svg.clone()).unwrap();

    // Shapes render as: Circle→<circle>, Square→<rect>, Triangle/Diamond→<path>,
    // Cross/Plus→<line>.  Verify each shape type is present.
    assert!(svg.contains("<circle"), "Circle marker should render as <circle>");
    // Triangle + Diamond → at least 2 <path> elements from data markers
    // (SVG also has the clip-path in <defs>, so total ≥ 3 but marker paths ≥ 2)
    let path_count = svg.matches("<path").count();
    assert!(path_count >= 2, "Triangle and Diamond should each produce a <path>, got {path_count}");
    // Square → at least 1 <rect> data marker (beyond the background rect)
    let rect_count = svg.matches("<rect").count();
    assert!(rect_count >= 2, "Square marker should add a <rect> data element, got {rect_count}");
    // Cross + Plus each produce 2 <line> elements
    let line_count = svg.matches("<line").count();
    assert!(line_count >= 4, "Cross and Plus should each produce 2 <line> elements, got {line_count}");
}

#[test]
fn test_strip_styled_group() {
    use kuva::plot::MarkerShape;

    let strip = StripPlot::new()
        .with_styled_group("Control", vec![
            (1.0, "steelblue", MarkerShape::Circle),
            (1.5, "steelblue", MarkerShape::Circle),
            (2.0, "steelblue", MarkerShape::Circle),
        ])
        .with_styled_group("Treatment", vec![
            (2.5, "tomato", MarkerShape::Triangle),
            (3.0, "tomato", MarkerShape::Triangle),
            (3.5, "tomato", MarkerShape::Triangle),
        ])
        .with_swarm()
        .with_point_size(5.0);

    let legend_entries = vec![
        LegendEntry { label: "Control".into(),   color: "steelblue".into(), shape: LegendShape::Circle,                        dasharray: None },
        LegendEntry { label: "Treatment".into(), color: "tomato".into(),    shape: LegendShape::Marker(MarkerShape::Triangle), dasharray: None },
    ];

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Strip — Per-Point Color + Shape")
        .with_y_label("Value")
        .with_legend_entries(legend_entries);

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/strip_styled.svg", svg.clone()).unwrap();

    // Circles (control) and triangles (treatment) both present.
    assert!(svg.contains("<circle"), "circle markers for Control should be present");
    assert!(svg.contains("<path"),   "triangle/path markers for Treatment should be present");
    assert!(svg.contains("Control"),   "legend entry 'Control' should appear");
    assert!(svg.contains("Treatment"), "legend entry 'Treatment' should appear");
}

#[test]
fn test_strip_shaped_fallback_to_circle() {
    use kuva::plot::MarkerShape;

    // With an empty point_shapes list, all points should fall back to circles.
    let strip = StripPlot::new()
        .with_shaped_group("A", vec![
            (1.0, MarkerShape::Circle),
            (2.0, MarkerShape::Circle),
        ])
        .with_color("seagreen");

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots);

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/strip_shaped_fallback.svg", svg.clone()).unwrap();

    assert!(svg.contains("<circle"), "all-circle shapes should render as <circle>");
    // No <path> data markers expected (only axis/clip paths, not marker paths).
    let circle_count = svg.matches("<circle").count();
    assert_eq!(circle_count, 2, "expected exactly 2 circle markers, got {circle_count}");
}

#[test]
fn test_strip_group_colors_legend_width() {
    // Regression: with group_colors, legend entries are the group labels.
    // Previously, auto_from_plots measured only legend_label.len() for width,
    // but the renderer uses group.label — causing long labels to overflow the box.
    let long_label = "A Much Longer Category Label";
    let strip = StripPlot::new()
        .with_group("Short",        vec![1.0, 2.0, 3.0])
        .with_group(long_label,     vec![1.5, 2.5, 3.5])
        .with_group("Medium Label", vec![2.0, 3.0, 4.0])
        .with_group_colors(vec!["steelblue", "tomato", "seagreen"])
        .with_legend("groups");

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/strip_group_legend_width.svg", svg.clone()).unwrap();

    // --- Parse the legend box rect (white fill) ---
    // Format: <rect x="NNN" ... width="NNN" ... fill="#ffffff" .../>
    let legend_rect_x = {
        let mut found = None;
        for chunk in svg.split("<rect") {
            if chunk.contains("fill=\"#ffffff\"") {
                let x: f64 = chunk.split("x=\"").nth(1)
                    .and_then(|s| s.split('"').next())
                    .and_then(|s| s.parse().ok())
                    .expect("legend rect should have numeric x");
                let w: f64 = chunk.split("width=\"").nth(1)
                    .and_then(|s| s.split('"').next())
                    .and_then(|s| s.parse().ok())
                    .expect("legend rect should have numeric width");
                found = Some((x, w));
                break;
            }
        }
        found.expect("legend background rect (fill=#ffffff) not found in SVG")
    };
    let (box_x, box_w) = legend_rect_x;
    let box_right = box_x + box_w;

    // --- Parse legend text entries (text-anchor="start") and check they fit ---
    // Format: <text x="NNN" ... text-anchor="start">LABEL</text>
    let px_per_char = 8.0_f64; // same heuristic used by layout width formula
    for chunk in svg.split("<text") {
        if !chunk.contains("text-anchor=\"start\"") { continue; }
        let text_x: f64 = match chunk.split("x=\"").nth(1)
            .and_then(|s| s.split('"').next())
            .and_then(|s| s.parse().ok())
        {
            Some(v) => v,
            None => continue,
        };
        let label = match chunk.split('>').nth(1).and_then(|s| s.split('<').next()) {
            Some(l) => l,
            None => continue,
        };
        let estimated_right = text_x + label.len() as f64 * px_per_char;
        assert!(
            estimated_right <= box_right + 1.0, // +1 for floating-point rounding
            "legend label {:?} estimated right edge ({estimated_right:.1}px) \
             exceeds legend box right edge ({box_right:.1}px)",
            label,
        );
    }
}
