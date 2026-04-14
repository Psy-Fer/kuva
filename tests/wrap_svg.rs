use kuva::plot::ScatterPlot;
use kuva::plot::legend::{LegendEntry, LegendShape};
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::backend::svg::SvgBackend;

fn svg(plots: Vec<Plot>, layout: Layout) -> String {
    std::fs::create_dir_all("test_outputs").ok();
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

fn scatter_plots() -> Vec<Plot> {
    vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(vec![(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)])
            .with_color("steelblue"),
    )]
}

// ── Title wrapping ───────────────────────────────────────────────────────────

#[test]
fn title_wrap_splits_long_title() {
    let plots = scatter_plots();
    let layout = Layout::auto_from_plots(&plots)
        .with_title("This is a very long title that should wrap")
        .with_title_wrap(20);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/wrap_title.svg", &out).unwrap();

    // Should produce multiple <text> elements for the title lines.
    // "This is a very long" (19 chars) and "title that should" (17) and "wrap" (4)
    assert!(out.contains("This is a very long"), "first wrapped line");
    assert!(out.contains("title that should"), "second wrapped line");
    assert!(out.contains(">wrap<"), "third wrapped line");
}

#[test]
fn title_no_wrap_when_short() {
    let plots = scatter_plots();
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Short")
        .with_title_wrap(20);
    let out = svg(plots, layout);

    // Title fits in 20 chars — should appear as a single text element.
    assert!(out.contains(">Short<"));
}

#[test]
fn title_no_wrap_without_setting() {
    let plots = scatter_plots();
    let layout = Layout::auto_from_plots(&plots)
        .with_title("This is a very long title that should not wrap");
    let out = svg(plots, layout);

    // No wrap setting → title stays on one line.
    assert!(out.contains("This is a very long title that should not wrap"));
}

// ── X-label wrapping ─────────────────────────────────────────────────────────

#[test]
fn x_label_wrap_splits() {
    let plots = scatter_plots();
    let layout = Layout::auto_from_plots(&plots)
        .with_x_label("A long x-axis label for testing wrapping behavior")
        .with_x_label_wrap(25);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/wrap_x_label.svg", &out).unwrap();

    // Should split into multiple lines.
    assert!(out.contains("A long x-axis label for"), "first x-label line");
    assert!(out.contains("testing wrapping"), "second x-label line");
}

// ── Y-label wrapping ─────────────────────────────────────────────────────────

#[test]
fn y_label_wrap_produces_multiple_rotated_texts() {
    let plots = scatter_plots();
    let layout = Layout::auto_from_plots(&plots)
        .with_y_label("Long rotated y-axis label text here")
        .with_y_label_wrap(15);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/wrap_y_label.svg", &out).unwrap();

    // Each wrapped line should be a separate rotated text.
    let rotated_count = out.matches("rotate(-90").count();
    assert!(rotated_count >= 2, "expected multiple rotated y-label lines, got {rotated_count}");
}

// ── Legend wrapping ──────────────────────────────────────────────────────────

#[test]
fn legend_wrap_splits_long_labels() {
    let plots = scatter_plots();
    let entries = vec![
        LegendEntry {
            label: "A very long legend label that should wrap".into(),
            color: "steelblue".into(),
            shape: LegendShape::Rect,
            dasharray: None,
        },
        LegendEntry {
            label: "Short".into(),
            color: "tomato".into(),
            shape: LegendShape::Rect,
            dasharray: None,
        },
    ];
    let layout = Layout::auto_from_plots(&plots)
        .with_legend_entries(entries)
        .with_legend_wrap(15);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/wrap_legend.svg", &out).unwrap();

    // The long label should be split across multiple text elements.
    assert!(out.contains("A very long"), "first legend line");
    assert!(out.contains("legend label"), "continuation line");
    // Short label should appear as-is.
    assert!(out.contains(">Short<"));
}

#[test]
fn legend_wrap_title() {
    let plots = scatter_plots();
    let entries = vec![
        LegendEntry {
            label: "Item".into(),
            color: "steelblue".into(),
            shape: LegendShape::Rect,
            dasharray: None,
        },
    ];
    let layout = Layout::auto_from_plots(&plots)
        .with_legend_entries(entries)
        .with_legend_title("A long legend title that wraps")
        .with_legend_wrap(15);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/wrap_legend_title.svg", &out).unwrap();

    // At 15 chars: "A long legend" / "title that" / "wraps"
    assert!(out.contains("A long legend"), "first legend title line");
    assert!(out.contains("title that"), "second legend title line");
    assert!(out.contains(">wraps<"), "third legend title line");
}

// ── with_wrap (global) ───────────────────────────────────────────────────────

#[test]
fn global_wrap_applies_to_all() {
    let plots = scatter_plots();
    let entries = vec![
        LegendEntry {
            label: "This is a long legend entry label".into(),
            color: "steelblue".into(),
            shape: LegendShape::Rect,
            dasharray: None,
        },
    ];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("This is a long title that should wrap on all elements")
        .with_x_label("This is a long x-axis label")
        .with_legend_entries(entries)
        .with_wrap(20);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/wrap_global.svg", &out).unwrap();

    // Title should wrap.
    assert!(!out.contains("This is a long title that should wrap on all elements"),
        "title should NOT appear as single line");
    // Legend should wrap.
    assert!(!out.contains("This is a long legend entry label"),
        "legend label should NOT appear as single line");
}

#[test]
fn per_element_overrides_global() {
    let plots = scatter_plots();
    let layout = Layout::auto_from_plots(&plots)
        .with_title("A medium-length title for testing")
        .with_wrap(10)          // global: aggressive wrap
        .with_title_wrap(40);   // override: title gets more room
    let out = svg(plots, layout);

    // Title is 32 chars, title_wrap is 40 → should NOT wrap.
    assert!(out.contains("A medium-length title for testing"),
        "title should stay on one line with per-element override");
}

// ── Edge cases ───────────────────────────────────────────────────────────────

#[test]
fn wrap_with_zero_is_disabled() {
    let plots = scatter_plots();
    let layout = Layout::auto_from_plots(&plots)
        .with_title("This should not wrap at all even though wrap is called")
        .with_wrap(0);
    let out = svg(plots, layout);

    assert!(out.contains("This should not wrap at all even though wrap is called"));
}

#[test]
fn wrap_margin_grows_for_multiline_title() {
    let plots1 = scatter_plots();
    let layout_no_wrap = Layout::auto_from_plots(&plots1)
        .with_title("Short title");
    let svg_no_wrap = svg(plots1, layout_no_wrap);

    let plots2 = scatter_plots();
    let layout_wrap = Layout::auto_from_plots(&plots2)
        .with_title("A very long title that definitely needs to wrap onto many lines")
        .with_title_wrap(15);
    let svg_wrap = svg(plots2, layout_wrap);

    // Extract the SVG height to verify margins grew.
    let height_no_wrap = extract_height(&svg_no_wrap);
    let height_wrap = extract_height(&svg_wrap);
    // The wrapped version should have the same or larger canvas
    // (margin_top grows to accommodate extra title lines).
    assert!(height_wrap >= height_no_wrap,
        "wrapped height ({height_wrap}) should be >= no-wrap height ({height_no_wrap})");
}

fn extract_height(svg: &str) -> f64 {
    // Parse height="NNN" from the SVG root element.
    let start = svg.find("height=\"").unwrap() + 8;
    let end = start + svg[start..].find('"').unwrap();
    svg[start..end].parse().unwrap()
}

// ── Y2-label wrapping ────────────────────────────────────────────────────────

#[test]
fn y2_label_wrap_produces_multiple_rotated_texts() {
    use kuva::render::render::render_twin_y;

    let primary = vec![Plot::Scatter(ScatterPlot::new()
        .with_data(vec![(1.0, 2.0), (3.0, 4.0)])
        .with_color("steelblue"))];
    let secondary = vec![Plot::Scatter(ScatterPlot::new()
        .with_data(vec![(1.0, 10.0), (3.0, 20.0)])
        .with_color("tomato"))];

    let layout = Layout::new((0.0, 5.0), (0.0, 5.0))
        .with_y_label("Primary axis")
        .with_y2_range(0.0, 25.0)
        .with_y2_label("A very long secondary y-axis label that wraps")
        .with_y2_label_wrap(15);

    let scene = render_twin_y(primary, secondary, layout);
    let out = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/wrap_y2_label.svg", &out).unwrap();

    let rotated_90_count = out.matches("rotate(90").count();
    assert!(rotated_90_count >= 2,
        "expected multiple +90° rotated y2-label lines, got {rotated_90_count}");
}

// ── Grouped legend wrapping ──────────────────────────────────────────────────

#[test]
fn grouped_legend_wrap() {
    let plots = scatter_plots();
    let layout = Layout::auto_from_plots(&plots)
        .with_legend_group(
            "A long group title that wraps",
            vec![
                LegendEntry { label: "Entry with a long label text".into(), color: "steelblue".into(), shape: LegendShape::Rect, dasharray: None },
                LegendEntry { label: "Short".into(), color: "tomato".into(), shape: LegendShape::Rect, dasharray: None },
            ],
        )
        .with_legend_wrap(15);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/wrap_grouped_legend.svg", &out).unwrap();

    // Group title wraps: "A long group" / "title that" / "wraps"
    assert!(out.contains("A long group"), "group title first line");
    assert!(out.contains("title that"), "group title second line");
    // Long entry wraps.
    assert!(out.contains("Entry with a"), "entry first line");
    assert!(out.contains("long label text"), "entry continuation");
    // Short entry stays on one line.
    assert!(out.contains(">Short<"));
}

// ── Outside-bottom legend with wrapping ──────────────────────────────────────

#[test]
fn outside_bottom_legend_wrap_adjusts_height() {
    use kuva::plot::LegendPosition;

    let plots1 = scatter_plots();
    let entries = vec![
        LegendEntry { label: "A very long legend entry that should definitely wrap".into(), color: "steelblue".into(), shape: LegendShape::Rect, dasharray: None },
        LegendEntry { label: "Another long entry for good measure here".into(), color: "tomato".into(), shape: LegendShape::Rect, dasharray: None },
    ];
    let layout = Layout::auto_from_plots(&plots1)
        .with_legend_entries(entries.clone())
        .with_legend_position(LegendPosition::OutsideBottomCenter)
        .with_legend_wrap(20);
    let svg_wrap = svg(plots1, layout);
    std::fs::write("test_outputs/wrap_outside_bottom.svg", &svg_wrap).unwrap();

    let plots2 = scatter_plots();
    let layout_no = Layout::auto_from_plots(&plots2)
        .with_legend_entries(entries)
        .with_legend_position(LegendPosition::OutsideBottomCenter);
    let svg_no_wrap = svg(plots2, layout_no);

    // The wrapped version needs more vertical space for the taller legend.
    let h_wrap = extract_height(&svg_wrap);
    let h_no = extract_height(&svg_no_wrap);
    assert!(h_wrap >= h_no,
        "wrapped outside-bottom height ({h_wrap}) should be >= no-wrap ({h_no})");
}
