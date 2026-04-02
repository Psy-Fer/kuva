use kuva::plot::ForestPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

#[test]
fn test_forest_basic() {
    let forest = ForestPlot::new()
        .with_row("Study A", 0.50, 0.10, 0.90)
        .with_row("Study B", -0.30, -0.80, 0.20)
        .with_row("Study C", 0.20, -0.10, 0.50);

    let plots = vec![Plot::Forest(forest)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Forest Plot Basic")
        .with_x_label("Effect Size");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/forest_basic.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // Should contain circles (point estimates) and lines (whiskers)
    assert!(svg.contains("<rect"), "SVG should contain rect elements (point estimates)");
    assert!(svg.contains("<line"), "SVG should contain line elements");
    // Row labels should appear
    assert!(svg.contains("Study A"), "SVG should contain row label Study A");
    assert!(svg.contains("Study B"), "SVG should contain row label Study B");
    assert!(svg.contains("Study C"), "SVG should contain row label Study C");
}

#[test]
fn test_forest_null_line() {
    let forest = ForestPlot::new()
        .with_row("Study A", 0.50, 0.10, 0.90)
        .with_row("Study B", -0.30, -0.80, 0.20)
        .with_null_value(0.0);

    let plots = vec![Plot::Forest(forest)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Forest Plot Null Line");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/forest_null_line.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // The null line should be dashed
    assert!(svg.contains("stroke-dasharray"), "SVG should contain a dashed null reference line");
}

#[test]
fn test_forest_weighted() {
    let forest = ForestPlot::new()
        .with_weighted_row("Study A", 0.50, 0.10, 0.90, 5.2)
        .with_weighted_row("Study B", -0.30, -0.80, 0.20, 3.8)
        .with_weighted_row("Study C", 0.20, -0.10, 0.50, 8.1);

    let plots = vec![Plot::Forest(forest)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Forest Plot Weighted");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/forest_weighted.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<rect"), "SVG should contain rect elements for weighted markers");
}

#[test]
fn test_forest_custom_color() {
    let forest = ForestPlot::new()
        .with_row("Study A", 0.50, 0.10, 0.90)
        .with_row("Study B", -0.30, -0.80, 0.20)
        .with_color("crimson");

    let plots = vec![Plot::Forest(forest)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Forest Plot Custom Color");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/forest_custom_color.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // Check the color appears in some form (hex, named, or rgb)
    assert!(
        svg.contains("#dc143c") || svg.contains("crimson") || svg.contains("rgb(220,20,60)"),
        "SVG should contain crimson color in some encoding"
    );
}

#[test]
fn test_forest_single_row() {
    let forest = ForestPlot::new()
        .with_row("Only Study", 0.30, -0.10, 0.70);

    let plots = vec![Plot::Forest(forest)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Forest Plot Single Row");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/forest_single_row.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Only Study"), "SVG should contain the single row label");
}

// ── edge cases ───────────────────────────────────────────────────────────────

/// 30 rows — stress the y-axis label layout and spacing.
#[test]
fn test_forest_many_rows() {
    let mut forest = ForestPlot::new();
    for i in 0..30 {
        let est = (i as f64 - 15.0) * 0.1;
        forest = forest.with_row(format!("Study {i:02}"), est, est - 0.5, est + 0.5);
    }

    let plots = vec![Plot::Forest(forest)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/forest_many_rows.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Study 00"));
    assert!(svg.contains("Study 29"));
}

/// Very wide CI — one study spans a huge range.
#[test]
fn test_forest_wide_ci() {
    let forest = ForestPlot::new()
        .with_row("Tight",  0.5,  0.4, 0.6)
        .with_row("Medium", 0.3, -0.5, 1.1)
        .with_row("Huge",   0.1, -10.0, 10.0);

    let plots = vec![Plot::Forest(forest)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/forest_wide_ci.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<rect"), "even tiny-relative CIs should have markers");
}

/// All estimates identical — marker squares should stack at the same x.
#[test]
fn test_forest_identical_estimates() {
    let forest = ForestPlot::new()
        .with_row("A", 0.0, -0.5, 0.5)
        .with_row("B", 0.0, -1.0, 1.0)
        .with_row("C", 0.0, -0.2, 0.2);

    let plots = vec![Plot::Forest(forest)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/forest_identical_est.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

/// Estimate at the CI boundary — marker should still render.
#[test]
fn test_forest_estimate_at_ci_edge() {
    let forest = ForestPlot::new()
        .with_row("At lower", -1.0, -1.0, 0.5)
        .with_row("At upper",  0.5, -0.5, 0.5)
        .with_row("Zero width", 0.3, 0.3, 0.3);

    let plots = vec![Plot::Forest(forest)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/forest_edge_est.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // All three rows should still produce markers
    let rect_count = svg.matches("<rect").count();
    // background rect + 3 markers = at least 4
    assert!(rect_count >= 4, "expected at least 4 rects (bg + 3 markers), got {rect_count}");
}

/// Negative null value (e.g. log-odds ratio with null at 1 → ln(1)=0,
/// but testing with null_value = -1 to verify arbitrary placement).
#[test]
fn test_forest_negative_null() {
    let forest = ForestPlot::new()
        .with_row("A", 0.5, -0.5, 1.5)
        .with_row("B", 1.2, 0.2, 2.2)
        .with_null_value(-1.0);

    let plots = vec![Plot::Forest(forest)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/forest_neg_null.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("stroke-dasharray"), "null line should be present");
}

/// No null line.
#[test]
fn test_forest_no_null_line() {
    let forest = ForestPlot::new()
        .with_row("A", 0.5, 0.1, 0.9)
        .with_show_null_line(false);

    let plots = vec![Plot::Forest(forest)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/forest_no_null.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(!svg.contains("stroke-dasharray"), "no dashed line when null line is disabled");
}

/// Extreme weight disparity — one study 1000x heavier than another.
#[test]
fn test_forest_extreme_weights() {
    let forest = ForestPlot::new()
        .with_weighted_row("Tiny",   0.1, -0.5, 0.7, 0.001)
        .with_weighted_row("Normal", 0.3, -0.2, 0.8, 1.0)
        .with_weighted_row("Giant",  0.5,  0.2, 0.8, 1000.0);

    let plots = vec![Plot::Forest(forest)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/forest_extreme_weights.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // All three should still produce visible rects
    let rect_count = svg.matches("<rect").count();
    assert!(rect_count >= 4, "expected at least 4 rects, got {rect_count}");
}

/// Very long label names.
#[test]
fn test_forest_long_labels() {
    let forest = ForestPlot::new()
        .with_row("Randomized Controlled Trial of Extended Duration Therapy (Phase III)", 0.3, 0.1, 0.5)
        .with_row("Short", -0.1, -0.4, 0.2);

    let plots = vec![Plot::Forest(forest)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/forest_long_labels.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Phase III"), "long label should be present");
}

/// All positive CIs — null line at 0 should be outside data range
/// but still visible because bounds include null_value.
#[test]
fn test_forest_all_positive() {
    let forest = ForestPlot::new()
        .with_row("A", 2.0, 1.5, 2.5)
        .with_row("B", 3.0, 2.0, 4.0)
        .with_row("C", 1.8, 1.2, 2.4)
        .with_null_value(0.0);

    let plots = vec![Plot::Forest(forest)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/forest_all_positive.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("stroke-dasharray"), "null line at 0 should be visible even when all CIs are positive");
}

/// Caps enabled — verify the cap lines render.
#[test]
fn test_forest_with_caps() {
    let forest = ForestPlot::new()
        .with_row("A", 0.5, 0.1, 0.9)
        .with_row("B", -0.3, -0.8, 0.2)
        .with_cap_size(4.0);

    let plots = vec![Plot::Forest(forest)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/forest_caps.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // With 2 rows and caps: each row has 1 whisker + 2 cap lines + 1 rect = 3 lines per row
    // Plus null line + axis lines. Should have many more lines than without caps.
    let line_count = svg.matches("<line").count();
    assert!(line_count >= 8, "expected at least 8 lines with caps, got {line_count}");
}

#[test]
fn test_forest_colored_rows() {
    let forest = ForestPlot::new()
        .with_colored_row("Favours treatment", 0.50, 0.10, 0.90, "seagreen")
        .with_colored_row("Favours control", -0.30, -0.80, 0.20, "tomato")
        .with_row("Neutral", 0.05, -0.20, 0.30);

    let plots = vec![Plot::Forest(forest)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Forest Plot Colored Rows")
        .with_x_label("Effect Size");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/forest_colored_rows.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // Per-row colors should appear in the SVG.
    // "seagreen" is not in the named-color table so it stays as a CSS string;
    // "tomato" is resolved to its Rgb value and emitted as hex.
    assert!(svg.contains("seagreen"), "SVG should contain seagreen");
    assert!(svg.contains("#ff6347"), "SVG should contain #ff6347 (tomato)");
    // Row labels should be present
    assert!(svg.contains("Favours treatment"));
    assert!(svg.contains("Favours control"));
    assert!(svg.contains("Neutral"));
}

#[test]
fn test_forest_weighted_colored_rows() {
    let forest = ForestPlot::new()
        .with_weighted_colored_row("Large study",  0.60, 0.30, 0.90, 10.0, "steelblue")
        .with_weighted_colored_row("Small study", -0.20, -0.70, 0.30,  2.0, "orange")
        .with_weighted_colored_row("Medium study", 0.10, -0.15, 0.35,  5.0, "purple");

    let plots = vec![Plot::Forest(forest)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Forest Plot Weighted + Colored");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/forest_weighted_colored.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // All three per-row colors should appear.
    // These names are in the named-color table and are emitted as hex.
    assert!(svg.contains("#4682b4"), "SVG should contain #4682b4 (steelblue)");
    assert!(svg.contains("#ffa500"), "SVG should contain #ffa500 (orange)");
    assert!(svg.contains("#800080"), "SVG should contain #800080 (purple)");
    // Weighted rows produce rects of different sizes; there should be at least 3 rects
    let rect_count = svg.matches("<rect").count();
    assert!(rect_count >= 3, "expected at least 3 rects for weighted markers, got {rect_count}");
}
