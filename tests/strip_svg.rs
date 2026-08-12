mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::StripPlot;
use kuva::plot::{BoxPlot, LegendEntry, LegendPosition, LegendShape, ViolinPlot};
use kuva::render::layout::{ComputedLayout, Layout};
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
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
    common::write_test_output("test_outputs/strip_basic.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_strip_swarm() {
    let strip = StripPlot::new()
        .with_group(
            "Control",
            vec![1.0, 1.2, 1.5, 1.8, 2.0, 2.1, 2.3, 2.5, 2.7, 3.0],
        )
        .with_group(
            "Treatment",
            vec![2.5, 2.7, 3.0, 3.2, 3.5, 3.8, 4.0, 4.2, 4.5, 5.0],
        )
        .with_color("coral")
        .with_swarm()
        .with_point_size(5.0);

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Strip Plot - Swarm Layout")
        .with_y_label("Measurement");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/strip_swarm.svg", svg.clone()).unwrap();
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
    common::write_test_output("test_outputs/strip_center.svg", svg.clone()).unwrap();
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
    common::write_test_output("test_outputs/strip_legend_palette.svg", svg.clone()).unwrap();
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
    common::write_test_output("test_outputs/box_with_strip_overlay.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_box_with_swarm_overlay() {
    let boxplot = BoxPlot::new()
        .with_group(
            "Control",
            vec![1.0, 1.2, 1.5, 1.8, 2.0, 2.1, 2.3, 2.5, 2.7, 3.0],
        )
        .with_group(
            "Treated",
            vec![2.5, 2.7, 3.0, 3.2, 3.5, 3.8, 4.0, 4.2, 4.5, 5.0],
        )
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
    common::write_test_output("test_outputs/box_with_swarm_overlay.svg", svg.clone()).unwrap();
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
    common::write_test_output("test_outputs/violin_with_strip_overlay.svg", svg.clone()).unwrap();
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
    common::write_test_output("test_outputs/strip_group_colors.svg", svg.clone()).unwrap();

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
    common::write_test_output("test_outputs/strip_and_box_composed.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_strip_point_colors_full() {
    // All points carry individual colors — simulates coloring by motif type
    let strip = StripPlot::new()
        .with_colored_group(
            "Sample A",
            vec![
                (1.2, "steelblue"),
                (2.4, "tomato"),
                (1.8, "seagreen"),
                (3.1, "goldenrod"),
                (2.0, "mediumpurple"),
                (2.7, "steelblue"),
                (1.5, "tomato"),
            ],
        )
        .with_colored_group(
            "Sample B",
            vec![
                (2.2, "tomato"),
                (3.3, "seagreen"),
                (2.8, "steelblue"),
                (1.9, "goldenrod"),
                (3.5, "mediumpurple"),
            ],
        )
        .with_swarm()
        .with_point_size(5.0);

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Strip — Per-point Colors")
        .with_y_label("Value");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/strip_point_colors_full.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(
        svg.contains("#ff6347"),
        "SVG should contain tomato (#ff6347)"
    );
    assert!(
        svg.contains("seagreen") || svg.contains("#2e8b57"),
        "SVG should contain seagreen color"
    );
    assert!(
        svg.contains("goldenrod") || svg.contains("#daa520"),
        "SVG should contain goldenrod color"
    );
}

#[test]
fn test_strip_point_colors_mixed() {
    // One colored group alongside a plain group — plain group uses uniform color fallback
    let strip = StripPlot::new()
        .with_colored_group(
            "Motifs",
            vec![
                (1.0, "tomato"),
                (2.0, "seagreen"),
                (3.0, "goldenrod"),
                (2.5, "mediumpurple"),
            ],
        )
        .with_group("Control", vec![1.5, 2.5, 3.5])
        .with_color("steelblue") // fallback for the plain group
        .with_jitter(0.2);

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Strip — Mixed Colored and Plain Groups")
        .with_y_label("Value");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/strip_point_colors_mixed.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(
        svg.contains("#ff6347"),
        "SVG should contain tomato (#ff6347) from colored group"
    );
    assert!(
        svg.contains("#4682b4"),
        "SVG should contain steelblue (#4682b4) fallback for plain group"
    );
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
        (4.0, "tomato"),
        (5.0, "tomato"),
        (4.5, "tomato"),
        (7.0, "seagreen"),
        (8.0, "seagreen"),
        (7.5, "seagreen"),
        (3.0, "goldenrod"),
        (3.5, "goldenrod"),
        (9.0, "mediumpurple"),
        (10.0, "mediumpurple"),
        (9.5, "mediumpurple"),
    ];

    let strip = StripPlot::new()
        .with_colored_group("Sample", points)
        .with_swarm()
        .with_point_size(5.0);

    // Manual legend — one circle swatch per motif category
    let legend_entries: Vec<LegendEntry> = motifs
        .iter()
        .map(|(label, color)| LegendEntry {
            label: label.to_string(),
            color: color.to_string(),
            shape: LegendShape::Circle,
            dasharray: None,
        })
        .collect();

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
    common::write_test_output("test_outputs/strip_point_colors_legend.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("STR Motif"), "SVG should contain the title");
    assert!(svg.contains("Motif"), "SVG should contain the legend title");
    assert!(svg.contains("ATTC"), "SVG should contain ATTC legend entry");
    assert!(svg.contains("GCGC"), "SVG should contain GCGC legend entry");
    assert!(svg.contains("ATAT"), "SVG should contain ATAT legend entry");
    assert!(svg.contains("CGCG"), "SVG should contain CGCG legend entry");
    // All four motif colors should appear in the SVG
    assert!(svg.contains("#ff6347"), "tomato (#ff6347) should appear");
    assert!(
        svg.contains("seagreen") || svg.contains("#2e8b57"),
        "seagreen should appear"
    );
    assert!(
        svg.contains("goldenrod") || svg.contains("#daa520"),
        "goldenrod should appear"
    );
    assert!(
        svg.contains("mediumpurple") || svg.contains("#9370db"),
        "mediumpurple should appear"
    );
}

// ── Per-point marker shapes ──────────────────────────────────────────────────

#[test]
fn test_strip_shaped_group() {
    use kuva::plot::MarkerShape;

    let strip = StripPlot::new()
        .with_shaped_group(
            "Sample",
            vec![
                (1.5, MarkerShape::Circle),
                (2.3, MarkerShape::Triangle),
                (1.8, MarkerShape::Square),
                (3.1, MarkerShape::Diamond),
                (2.7, MarkerShape::Cross),
                (3.5, MarkerShape::Plus),
            ],
        )
        .with_color("steelblue")
        .with_swarm()
        .with_point_size(5.0);

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Strip — Per-Point Shapes")
        .with_y_label("Value");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/strip_shaped.svg", svg.clone()).unwrap();

    // Shapes render as: Circle→<circle>, Square→<rect>, Triangle/Diamond→<path>,
    // Cross/Plus→<line>.  Verify each shape type is present.
    assert!(
        svg.contains("<circle"),
        "Circle marker should render as <circle>"
    );
    // Triangle + Diamond → at least 2 <path> elements from data markers
    // (SVG also has the clip-path in <defs>, so total ≥ 3 but marker paths ≥ 2)
    let path_count = svg.matches("<path").count();
    assert!(
        path_count >= 2,
        "Triangle and Diamond should each produce a <path>, got {path_count}"
    );
    // Square → at least 1 <rect> data marker (beyond the background rect)
    let rect_count = svg.matches("<rect").count();
    assert!(
        rect_count >= 2,
        "Square marker should add a <rect> data element, got {rect_count}"
    );
    // Cross + Plus each produce 2 <line> elements
    let line_count = svg.matches("<line").count();
    assert!(
        line_count >= 4,
        "Cross and Plus should each produce 2 <line> elements, got {line_count}"
    );
}

#[test]
fn test_strip_styled_group() {
    use kuva::plot::MarkerShape;

    let strip = StripPlot::new()
        .with_styled_group(
            "Control",
            vec![
                (1.0, "steelblue", MarkerShape::Circle),
                (1.5, "steelblue", MarkerShape::Circle),
                (2.0, "steelblue", MarkerShape::Circle),
            ],
        )
        .with_styled_group(
            "Treatment",
            vec![
                (2.5, "tomato", MarkerShape::Triangle),
                (3.0, "tomato", MarkerShape::Triangle),
                (3.5, "tomato", MarkerShape::Triangle),
            ],
        )
        .with_swarm()
        .with_point_size(5.0);

    let legend_entries = vec![
        LegendEntry {
            label: "Control".into(),
            color: "steelblue".into(),
            shape: LegendShape::Circle,
            dasharray: None,
        },
        LegendEntry {
            label: "Treatment".into(),
            color: "tomato".into(),
            shape: LegendShape::Marker(MarkerShape::Triangle),
            dasharray: None,
        },
    ];

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Strip — Per-Point Color + Shape")
        .with_y_label("Value")
        .with_legend_entries(legend_entries);

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/strip_styled.svg", svg.clone()).unwrap();

    // Circles (control) and triangles (treatment) both present.
    assert!(
        svg.contains("<circle"),
        "circle markers for Control should be present"
    );
    assert!(
        svg.contains("<path"),
        "triangle/path markers for Treatment should be present"
    );
    assert!(
        svg.contains("Control"),
        "legend entry 'Control' should appear"
    );
    assert!(
        svg.contains("Treatment"),
        "legend entry 'Treatment' should appear"
    );
}

#[test]
fn test_strip_shaped_fallback_to_circle() {
    use kuva::plot::MarkerShape;

    // With an empty point_shapes list, all points should fall back to circles.
    let strip = StripPlot::new()
        .with_shaped_group(
            "A",
            vec![(1.0, MarkerShape::Circle), (2.0, MarkerShape::Circle)],
        )
        .with_color("seagreen");

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots);

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/strip_shaped_fallback.svg", svg.clone()).unwrap();

    assert!(
        svg.contains("<circle"),
        "all-circle shapes should render as <circle>"
    );
    // No <path> data markers expected (only axis/clip paths, not marker paths).
    let circle_count = svg.matches("<circle").count();
    assert_eq!(
        circle_count, 2,
        "expected exactly 2 circle markers, got {circle_count}"
    );
}

#[test]
fn test_strip_group_colors_legend_width() {
    // Regression: with group_colors, legend entries are the group labels.
    // Previously, auto_from_plots measured only legend_label.len() for width,
    // but the renderer uses group.label — causing long labels to overflow the box.
    let long_label = "A Much Longer Category Label";
    let strip = StripPlot::new()
        .with_group("Short", vec![1.0, 2.0, 3.0])
        .with_group(long_label, vec![1.5, 2.5, 3.5])
        .with_group("Medium Label", vec![2.0, 3.0, 4.0])
        .with_group_colors(vec!["steelblue", "tomato", "seagreen"])
        .with_legend("groups");

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/strip_group_legend_width.svg", svg.clone()).unwrap();

    // --- Parse the legend box rect (white fill) ---
    // Format: <rect x="NNN" ... width="NNN" ... fill="#ffffff" .../>
    let legend_rect_x = {
        let mut found = None;
        for chunk in svg.split("<rect") {
            if chunk.contains("fill=\"#ffffff\"") {
                let x: f64 = chunk
                    .split("x=\"")
                    .nth(1)
                    .and_then(|s| s.split('"').next())
                    .and_then(|s| s.parse().ok())
                    .expect("legend rect should have numeric x");
                let w: f64 = chunk
                    .split("width=\"")
                    .nth(1)
                    .and_then(|s| s.split('"').next())
                    .and_then(|s| s.parse().ok())
                    .expect("legend rect should have numeric width");
                found = Some((x, w));
                break;
            }
        }
        found.expect("legend background rect (fill=#ffffff) not found in SVG")
    };
    let (_box_x, box_w) = legend_rect_x;

    // The box is sized from the real measured widths of the GROUP labels; the precise
    // "widest label fits" invariant is unit-tested in `render::layout` against actual
    // DejaVu advances (an integration test can't measure glyphs without re-deriving
    // them). Here we only guard the original regression: the width must come from the
    // group labels, not the short legend label "groups" — otherwise the box collapses
    // to the ~80px minimum and the long label overflows.
    assert!(
        svg.contains(">A Much Longer Category Label<"),
        "the long group label should be rendered in the legend"
    );
    assert!(
        box_w > 120.0,
        "legend box ({box_w:.1}px) collapsed near the minimum width — it was likely \
         sized from the wrong string instead of the group labels"
    );
}

// Collects every `<circle cx="..." cy="...">` center from an SVG string.
fn circle_centers(svg: &str) -> Vec<(f64, f64)> {
    let mut centers = Vec::new();
    for chunk in svg.split("<circle") {
        let cx = chunk
            .split("cx=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .and_then(|s| s.parse::<f64>().ok());
        let cy = chunk
            .split("cy=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .and_then(|s| s.parse::<f64>().ok());
        if let (Some(cx), Some(cy)) = (cx, cy) {
            centers.push((cx, cy));
        }
    }
    centers
}

// A large `with_jitter` value used to push points from the outermost category
// past the plot's left/right edge, since the axis has no nice-rounded margin
// to absorb the overflow (categorical axes use their exact [0.5, n+0.5]
// extent — see PR #99). The offset must be clamped to the category's own
// half-slot so points stay within the plot area regardless of jitter size.
#[test]
fn test_strip_large_jitter_stays_within_plot_bounds() {
    let strip = StripPlot::new()
        .with_group("A", vec![1.0, 2.0, 3.0, 4.0, 5.0])
        .with_group("B", vec![1.5, 2.5, 3.5, 4.5, 5.5])
        .with_group("C", vec![1.2, 2.2, 3.2, 4.2, 5.2])
        .with_jitter(5.0) // far beyond the documented [0, 1] range
        .with_point_size(3.0);

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots);
    let computed = ComputedLayout::from_layout(&layout);
    let (left, right) = (computed.margin_left, computed.width - computed.margin_right);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/strip_large_jitter.svg", svg.clone()).unwrap();

    let centers = circle_centers(&svg);
    assert!(!centers.is_empty(), "expected strip points to be rendered");
    // A small tolerance absorbs the SVG backend's 2-decimal-place float
    // formatting, not a real bounds violation.
    const TOL: f64 = 0.05;
    for (cx, _cy) in centers {
        assert!(
            cx >= left - TOL && cx <= right + TOL,
            "jittered point at cx={cx:.2} fell outside plot bounds [{left:.2}, {right:.2}]"
        );
    }
}

// A dense Swarm column packed with many tightly-clustered points used to
// spread wide enough (in pixel space) to bleed into a neighboring category's
// slot, since `beeswarm_positions` has no cap tied to slot width. Offsets
// must be capped at half the category's own pixel slot, so each category's
// points stay within [center - slot/2, center + slot/2].
#[test]
fn test_strip_swarm_stays_within_own_category_slot() {
    // Identical values maximize collisions, forcing beeswarm_positions to
    // walk as far outward as it can — the scenario that needs the cap.
    let n = 60;
    let dense: Vec<f64> = vec![3.0; n];
    let strip = StripPlot::new()
        .with_group("A", dense.clone())
        .with_group("B", dense.clone())
        .with_group("C", dense)
        .with_swarm()
        .with_point_size(5.0);

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots);
    let computed = ComputedLayout::from_layout(&layout);
    let slot_px = (computed.width - computed.margin_left - computed.margin_right) / 3.0;
    let max_offset = slot_px / 2.0;
    let centers_of = |i: f64| computed.margin_left + slot_px * (i - 0.5);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/strip_swarm_dense.svg", svg.clone()).unwrap();

    let centers = circle_centers(&svg);
    assert_eq!(
        centers.len(),
        3 * n,
        "expected all swarm points to be rendered"
    );
    // Points are emitted group-by-group (A's n points, then B's, then C's),
    // so each contiguous chunk of `n` circles belongs to one category and
    // must stay within that category's own half-slot around its center.
    const TOL: f64 = 0.05;
    for (group_idx, chunk) in centers.chunks(n).enumerate() {
        let center = centers_of(group_idx as f64 + 1.0);
        for &(cx, _cy) in chunk {
            assert!(
                cx >= center - max_offset - TOL && cx <= center + max_offset + TOL,
                "group {group_idx} swarm point at cx={cx:.2} fell outside its own \
                 category slot [{:.2}, {:.2}]",
                center - max_offset,
                center + max_offset
            );
        }
    }
}

// ── Horizontal orientation (coord_flip) ─────────────────────────────────────────
// Categories move to the Y axis, values to X. The renderer already supported the
// swap for every style; these assert it end-to-end and write SVGs for inspection.

/// The category (group) label text-elements, returned as `(x, y)` screen coords.
fn category_label_positions(svg: &str, labels: &[&str]) -> Vec<(f64, f64)> {
    labels
        .iter()
        .map(|lab| {
            let needle = format!(">{lab}</text>");
            let end = svg
                .find(&needle)
                .unwrap_or_else(|| panic!("no label {lab}"));
            let tag = &svg[svg[..end].rfind("<text").unwrap()..end];
            let get = |attr: &str| -> f64 {
                let s = tag.find(attr).unwrap() + attr.len();
                let e = tag[s..].find('"').unwrap() + s;
                tag[s..e].parse().unwrap()
            };
            (get(" x=\""), get(" y=\""))
        })
        .collect()
}

#[test]
fn test_strip_horizontal_jitter() {
    let strip = StripPlot::new()
        .with_group("A", vec![1.0, 2.0, 2.5, 3.1, 4.0, 3.5, 2.8])
        .with_group("B", vec![2.0, 2.1, 3.5, 3.8, 4.0, 4.2, 5.0])
        .with_group("C", vec![0.5, 1.5, 1.8, 2.2, 3.0, 3.3, 4.5])
        .with_color("steelblue")
        .with_horizontal(true);

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Horizontal Strip (jitter)")
        .with_x_label("Values");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/strip_horizontal.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // Categories are on the Y axis: the three labels share one x (left gutter) and
    // sit at distinct, monotonically-changing y positions.
    let pos = category_label_positions(&svg, &["A", "B", "C"]);
    assert!(
        (pos[0].0 - pos[1].0).abs() < 0.5 && (pos[1].0 - pos[2].0).abs() < 0.5,
        "horizontal strip: category labels must share one x (Y-axis gutter), got {pos:?}"
    );
    assert!(
        pos[0].1 > pos[1].1 && pos[1].1 > pos[2].1,
        "categories should stack down the Y axis, got {pos:?}"
    );
}

#[test]
fn test_strip_horizontal_swarm() {
    let strip = StripPlot::new()
        .with_group("Control", vec![4.1, 5.0, 5.3, 5.8, 6.2, 4.7, 5.5, 5.1, 4.9])
        .with_group(
            "Treatment",
            vec![5.5, 6.1, 6.4, 7.2, 7.8, 6.9, 7.0, 6.6, 6.2],
        )
        .with_color("darkorange")
        .with_swarm()
        .with_horizontal(true);

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Horizontal Beeswarm")
        .with_x_label("Value");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/strip_horizontal_swarm.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    let pos = category_label_positions(&svg, &["Control", "Treatment"]);
    assert!(
        (pos[0].0 - pos[1].0).abs() < 0.5,
        "swarm category labels must share the Y-axis gutter x, got {pos:?}"
    );
}

#[test]
fn test_strip_horizontal_center() {
    let strip = StripPlot::new()
        .with_group("A", vec![1.0, 2.0, 3.0, 4.0])
        .with_group("B", vec![2.0, 3.0, 4.0, 5.0])
        .with_center()
        .with_horizontal(true);

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Horizontal Center")
        .with_x_label("Value");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/strip_horizontal_center.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle") || svg.contains("<path"));
}

/// Marker opacity at density: overlapping points build up visible density. Asserts
/// the fill-opacity reaches the SVG. Writes an SVG for visual inspection at scale.
#[test]
fn test_strip_opacity_dense() {
    // A deterministic dense group (120 points) so overlap is real.
    let dense: Vec<f64> = (0..120)
        .map(|i| {
            let t = i as f64;
            5.0 + (t * 0.7).sin() * 1.2 + (t * 0.13).cos() * 0.6
        })
        .collect();
    let strip = StripPlot::new()
        .with_group("Dense", dense)
        .with_color("steelblue")
        .with_point_size(4.0)
        .with_marker_opacity(0.3)
        .with_swarm();

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Dense beeswarm with marker opacity")
        .with_y_label("Value");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/strip_opacity_dense.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    assert!(
        svg.contains("fill-opacity=\"0.3"),
        "marker opacity must reach the SVG as fill-opacity"
    );
}
