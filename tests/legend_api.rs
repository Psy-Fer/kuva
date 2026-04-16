use kuva::plot::{ScatterPlot, LinePlot, LegendPosition};
use kuva::plot::legend::{LegendEntry, LegendShape};
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::backend::svg::SvgBackend;
use kuva::render::figure::Figure;

fn svg(plots: Vec<Plot>, layout: Layout) -> String {
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

/// Manual entries appear in the order supplied (not auto-collect order).
#[test]
fn test_manual_entries_override() {
    let scatter = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
        .with_color("steelblue")
        .with_legend("Alpha");

    let line = LinePlot::new()
        .with_data(vec![(0.0_f64, 0.0), (5.0, 5.0)])
        .with_color("tomato")
        .with_legend("Beta");

    let plots = vec![Plot::Scatter(scatter), Plot::Line(line)];

    // Supply entries in reverse order: Beta first, Alpha second.
    let entries = vec![
        LegendEntry { label: "Beta".into(), color: "tomato".into(), shape: LegendShape::Line, dasharray: None },
        LegendEntry { label: "Alpha".into(), color: "steelblue".into(), shape: LegendShape::Circle, dasharray: None },
    ];

    let layout = Layout::auto_from_plots(&plots).with_legend_entries(entries);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_api_override.svg", &out).unwrap();

    let beta_pos  = out.find("Beta").expect("'Beta' not in SVG");
    let alpha_pos = out.find("Alpha").expect("'Alpha' not in SVG");
    assert!(beta_pos < alpha_pos, "'Beta' should appear before 'Alpha' in SVG");
}

/// Manual entries completely replace auto-collected entries.
#[test]
fn test_manual_entries_bypasses_auto() {
    let scatter = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
        .with_color("steelblue")
        .with_legend("auto-label");

    let plots = vec![Plot::Scatter(scatter)];

    let entries = vec![
        LegendEntry { label: "manual-label".into(), color: "steelblue".into(), shape: LegendShape::Circle, dasharray: None },
    ];

    let layout = Layout::auto_from_plots(&plots).with_legend_entries(entries);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_api_bypass_auto.svg", &out).unwrap();

    assert!(!out.contains("auto-label"), "'auto-label' should not appear when manual entries are set");
    assert!(out.contains("manual-label"), "'manual-label' should appear in SVG");
}

/// with_legend_at: legend floats at the given pixel position; right-margin unchanged.
#[test]
fn test_legend_at_no_margin() {
    let data = vec![(1.0_f64, 2.0), (3.0, 4.0)];

    // Reference: scatter without any legend — gives us baseline width.
    let layout_no_legend = Layout::auto_from_plots(&[Plot::Scatter(
        ScatterPlot::new().with_data(data.clone()).with_color("steelblue"),
    )]);
    let svg_no_legend = svg(
        vec![Plot::Scatter(ScatterPlot::new().with_data(data.clone()).with_color("steelblue"))],
        layout_no_legend,
    );

    // Scatter with legend at absolute position.
    let plots = vec![Plot::Scatter(
        ScatterPlot::new().with_data(data.clone()).with_color("steelblue").with_legend("My label"),
    )];
    let entries = vec![
        LegendEntry { label: "My label".into(), color: "steelblue".into(), shape: LegendShape::Circle, dasharray: None },
    ];
    let layout = Layout::auto_from_plots(&plots)
        .with_legend_entries(entries)
        .with_legend_at(50.0, 50.0);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_api_at.svg", &out).unwrap();

    // Extract SVG width from `<svg ... width="NNN"` attribute.
    fn extract_width(s: &str) -> f64 {
        let w_start = s.find("width=\"").expect("no width attr") + 7;
        let w_end = s[w_start..].find('"').unwrap() + w_start;
        s[w_start..w_end].parse().expect("width parse")
    }

    let width_no_legend = extract_width(&svg_no_legend);
    let width_with_at   = extract_width(&out);
    assert_eq!(
        width_no_legend as u64, width_with_at as u64,
        "with_legend_at should not widen the canvas (no right-margin reserved)"
    );

    // Legend text should appear near x=75 (50 + 25 swatch width).
    assert!(out.contains("<text"), "Legend text element missing");
    assert!(out.contains("My label"), "'My label' not found in SVG");
}

/// RightMiddle places the legend vertically centred on the plot area.
#[test]
fn test_right_middle_preset() {
    let scatter = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0), (5.0, 6.0)])
        .with_color("steelblue")
        .with_legend("Series A");

    let plots = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plots)
        .with_legend_position(LegendPosition::OutsideRightMiddle);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_api_right_middle.svg", &out).unwrap();

    assert!(out.contains("Series A"), "'Series A' not found in SVG");

    // Extract SVG height to determine expected vertical band.
    let h_start = out.find("height=\"").expect("no height attr") + 8;
    let h_end = out[h_start..].find('"').unwrap() + h_start;
    let svg_height: f64 = out[h_start..h_end].parse().expect("height parse");

    // Find the legend rect y: scan `<rect ... >` tags outside of `<defs>` blocks.
    // We skip the canvas background rect (no x= attribute) and clipPath rects
    // (which are inside <defs>...</defs> and have small y values near the top).
    let legend_rect_y = {
        let mut y_val: Option<f64> = None;
        // Start scanning after the closing </defs> tag to skip all def-block rects.
        let scan_start = out.find("</defs>").map(|i| i + 7).unwrap_or(0);
        let mut search = &out[scan_start..];
        while let Some(rect_pos) = search.find("<rect") {
            let tag_end = search[rect_pos..].find('>').map(|e| rect_pos + e).unwrap_or(search.len());
            let tag = &search[rect_pos..=tag_end];
            // Skip the canvas background rect (has no x= attribute)
            if tag.contains(" x=\"") {
                if let Some(y_pos) = tag.find(" y=\"") {
                    let y_start = y_pos + 4;
                    if let Some(y_end) = tag[y_start..].find('"') {
                        if let Ok(y) = tag[y_start..y_start + y_end].parse::<f64>() {
                            y_val = Some(y);
                            break;
                        }
                    }
                }
            }
            search = &search[rect_pos + 1..];
        }
        y_val
    };

    // Legend background rect y = legend_y - padding (10). The legend_y is centred on the plot area.
    // Plot area spans roughly margin_top to (height - margin_bottom). The legend rect y must be
    // in the middle half of the SVG (between 25% and 75% of total height).
    let y = legend_rect_y.expect("Legend rect not found in SVG");
    assert!(
        y > svg_height * 0.25 && y < svg_height * 0.75,
        "Legend y={y} should be in middle half of SVG (height={svg_height})"
    );
}

/// with_shared_legend_at places the shared legend at a custom pixel position;
/// the total SVG width equals the grid width (no extra right-margin).
#[test]
fn test_figure_shared_legend_at() {
    let make_scatter = |color: &str, label: &str| -> Vec<Plot> {
        vec![Plot::Scatter(
            ScatterPlot::new()
                .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
                .with_color(color)
                .with_legend(label),
        )]
    };

    let cell_w = 300.0_f64;
    let cell_h = 250.0_f64;

    let figure = Figure::new(1, 2)
        .with_plots(vec![
            make_scatter("steelblue", "Alpha"),
            make_scatter("tomato", "Beta"),
        ])
        .with_cell_size(cell_w, cell_h)
        .with_shared_legend_at(20.0, 20.0);

    let out = SvgBackend.render_scene(&figure.render());
    std::fs::write("test_outputs/legend_api_figure_at.svg", &out).unwrap();

    assert!(out.contains("Alpha"), "'Alpha' not found in SVG");
    assert!(out.contains("Beta"), "'Beta' not found in SVG");

    // Extract SVG width.
    let w_start = out.find("width=\"").expect("no width attr") + 7;
    let w_end = out[w_start..].find('"').unwrap() + w_start;
    let svg_width: f64 = out[w_start..w_end].parse().expect("width parse");

    // With Custom placement there is no extra legend margin — width should be the grid width.
    // Grid width = 2 cells + (2-1)*spacing + 2*padding (defaults: spacing=15, padding=10).
    let expected_grid_width = 2.0 * cell_w + 1.0 * 15.0 + 2.0 * 10.0;
    assert!(
        (svg_width - expected_grid_width).abs() < 2.0,
        "SVG width {svg_width} should equal grid width {expected_grid_width} (no legend margin)"
    );
}

fn extract_width(s: &str) -> f64 {
    let w_start = s.find("width=\"").expect("no width attr") + 7;
    let w_end = s[w_start..].find('"').unwrap() + w_start;
    s[w_start..w_end].parse().expect("width parse")
}

/// InsideTopLeft / InsideBottomLeft legend must not overlap y-axis tick labels.
///
/// Tick labels use text-anchor="end", so their x attribute IS their right edge.
/// The legend box rect x must be strictly greater than every tick label x.
#[test]
fn inside_left_legend_clears_tick_labels() {
    let make_plots = || vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(vec![(1.0, 100.0), (2.0, 200.0), (3.0, 150.0)])
            .with_color("steelblue"),
    )];

    let entries = vec![
        LegendEntry { label: "Series A".into(), color: "steelblue".into(), shape: LegendShape::Circle, dasharray: None },
    ];

    for (pos, name) in [
        (LegendPosition::InsideTopLeft,    "top_left"),
        (LegendPosition::InsideBottomLeft, "bottom_left"),
    ] {
        let plots = make_plots();
        let layout = Layout::auto_from_plots(&plots)
            .with_legend_entries(entries.clone())
            .with_legend_position(pos);
        let out = svg(plots, layout);
        std::fs::write(format!("test_outputs/legend_inside_left_{name}.svg"), &out).ok();

        // Find the legend box rect (fill="#ffffff") and extract its x.
        let legend_box_x = out
            .split("<rect ")
            .skip(1)
            .find(|seg| seg.contains("fill=\"#ffffff\""))
            .and_then(|seg| {
                let x_start = seg.find("x=\"")? + 3;
                let x_end = x_start + seg[x_start..].find('"')?;
                seg[x_start..x_end].parse::<f64>().ok()
            });

        if let Some(box_x) = legend_box_x {
            // Collect x values of all text elements with text-anchor="end" (tick labels).
            let tick_xs: Vec<f64> = out
                .split("<text ")
                .skip(1)
                .filter(|seg| seg.contains("text-anchor=\"end\""))
                .filter_map(|seg| {
                    let x_start = seg.find("x=\"")? + 3;
                    let x_end = x_start + seg[x_start..].find('"')?;
                    seg[x_start..x_end].parse::<f64>().ok()
                })
                .collect();

            if !tick_xs.is_empty() {
                let max_tick_x = tick_xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                assert!(
                    box_x > max_tick_x,
                    "InsideLeft ({name}): legend box x ({box_x:.1}) must be > max tick label x ({max_tick_x:.1})"
                );
            }
        }
    }
}

/// with_legend_box(false): background/border rects suppressed; entries still present.
#[test]
fn test_legend_box_suppress() {
    let scatter = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
        .with_color("steelblue")
        .with_legend("Alpha");
    let plots = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plots).with_legend_box(false);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_api_no_box.svg", &out).unwrap();

    assert!(out.contains("Alpha"), "'Alpha' should appear in SVG");
    // The legend border rect is the only element with fill="none"; with show_box=false it is absent.
    // (Scatter data circles use a real fill color, grid lines don't use <rect>.)
    assert!(!out.contains("fill=\"none\""),
        "legend border rect (fill=none) should not appear when show_box=false");
}

/// with_legend_title: a bold title row is rendered above entries.
#[test]
fn test_legend_title() {
    let scatter = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
        .with_color("steelblue")
        .with_legend("Series A");
    let plots = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plots).with_legend_title("Groups");
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_api_title.svg", &out).unwrap();

    assert!(out.contains("Groups"), "'Groups' title should appear in SVG");
    assert!(out.contains("Series A"), "'Series A' entry should appear in SVG");
}

/// with_legend_group: two groups render all titles and all entry labels.
#[test]
fn test_legend_groups() {
    let scatter = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
        .with_color("steelblue");
    let plots = vec![Plot::Scatter(scatter)];

    let group_a = vec![
        LegendEntry { label: "Apple".into(), color: "red".into(), shape: LegendShape::Rect, dasharray: None },
        LegendEntry { label: "Apricot".into(), color: "orange".into(), shape: LegendShape::Rect, dasharray: None },
    ];
    let group_b = vec![
        LegendEntry { label: "Banana".into(), color: "yellow".into(), shape: LegendShape::Circle, dasharray: None },
    ];

    let layout = Layout::auto_from_plots(&plots)
        .with_legend_group("Fruits A", group_a)
        .with_legend_group("Fruits B", group_b);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_api_groups.svg", &out).unwrap();

    assert!(out.contains("Fruits A"), "'Fruits A' group title should appear");
    assert!(out.contains("Fruits B"), "'Fruits B' group title should appear");
    assert!(out.contains("Apple"), "'Apple' should appear");
    assert!(out.contains("Apricot"), "'Apricot' should appear");
    assert!(out.contains("Banana"), "'Banana' should appear");

    // Group A entries appear before group B entries in SVG output order
    let fruits_a_pos = out.find("Fruits A").expect("Fruits A missing");
    let fruits_b_pos = out.find("Fruits B").expect("Fruits B missing");
    assert!(fruits_a_pos < fruits_b_pos, "'Fruits A' should appear before 'Fruits B'");
}

/// InsideTopRight: legend x should be less than plot_right (inside the axes).
#[test]
fn test_inside_top_right() {
    let scatter = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
        .with_color("steelblue")
        .with_legend("Series A");
    let plots = vec![Plot::Scatter(scatter)];
    let layout_ref = Layout::auto_from_plots(&plots);
    // Canvas width without legend (InsideTopRight adds no right margin)
    let layout = layout_ref
        .with_legend_position(LegendPosition::InsideTopRight);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_api_inside_top_right.svg", &out).unwrap();

    assert!(out.contains("Series A"), "'Series A' should appear in SVG");

    // Canvas width for InsideTopRight should equal a plot without any legend
    // (no extra right margin). We check it's narrower than OutsideRightTop width.
    let scatter2 = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
        .with_color("steelblue")
        .with_legend("Series A");
    let plots2 = vec![Plot::Scatter(scatter2)];
    let layout_outside = Layout::auto_from_plots(&plots2);  // default OutsideRightTop
    let out_outside = svg(plots2, layout_outside);
    let width_inside  = extract_width(&out);
    let width_outside = extract_width(&out_outside);
    assert!(
        width_inside < width_outside,
        "InsideTopRight (width={width_inside}) should be narrower than OutsideRightTop (width={width_outside})"
    );
}

/// OutsideLeftTop: legend near x=5; left margin expanded.
#[test]
fn test_outside_left() {
    let scatter = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
        .with_color("steelblue")
        .with_legend("Left Label");
    let plots = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plots)
        .with_legend_position(LegendPosition::OutsideLeftTop);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_api_outside_left.svg", &out).unwrap();

    assert!(out.contains("Left Label"), "'Left Label' should appear in SVG");
}

/// with_legend_at(x, y): Custom placement, no right-margin reserved.
#[test]
fn test_custom_position() {
    let scatter = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
        .with_color("steelblue")
        .with_legend("Custom");
    let plots = vec![Plot::Scatter(scatter)];

    // Baseline: same plot with InsideTopRight (also no right margin)
    let scatter2 = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
        .with_color("steelblue")
        .with_legend("Custom");
    let plots2 = vec![Plot::Scatter(scatter2)];
    let layout_baseline = Layout::auto_from_plots(&plots2)
        .with_legend_position(LegendPosition::InsideTopRight);
    let out_baseline = svg(plots2, layout_baseline);

    let layout = Layout::auto_from_plots(&plots).with_legend_at(50.0, 50.0);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_api_custom_position.svg", &out).unwrap();

    assert!(out.contains("Custom"), "'Custom' label should appear in SVG");

    // Custom position adds no right margin — width should match InsideTopRight baseline.
    let width_custom   = extract_width(&out);
    let width_baseline = extract_width(&out_baseline);
    assert_eq!(
        width_custom as u64, width_baseline as u64,
        "Custom position should not widen canvas (width={width_custom} vs baseline={width_baseline})"
    );
}

/// with_legend_at_data: data-space placement; label present; no right margin added.
#[test]
fn test_data_coords_position() {
    let scatter = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0), (5.0, 6.0)])
        .with_color("steelblue")
        .with_legend("Data Coords");
    let plots = vec![Plot::Scatter(scatter)];

    // Baseline: same plot with InsideTopRight (no right margin)
    let scatter2 = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0), (5.0, 6.0)])
        .with_color("steelblue")
        .with_legend("Data Coords");
    let plots2 = vec![Plot::Scatter(scatter2)];
    let layout_baseline = Layout::auto_from_plots(&plots2)
        .with_legend_position(LegendPosition::InsideTopRight);
    let out_baseline = svg(plots2, layout_baseline);

    let layout = Layout::auto_from_plots(&plots).with_legend_at_data(2.0, 4.0);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_api_data_coords.svg", &out).unwrap();

    assert!(out.contains("Data Coords"), "'Data Coords' label should appear in SVG");

    let width_data     = extract_width(&out);
    let width_baseline = extract_width(&out_baseline);
    assert_eq!(
        width_data as u64, width_baseline as u64,
        "DataCoords position should not widen canvas (width={width_data} vs baseline={width_baseline})"
    );
}

// ── Legend position preset tests ─────────────────────────────────────────────
//
// One test per named LegendPosition variant.  Every plot has a title, x-label,
// and y-label so margin computation is exercised fully.  SVGs are written to
// test_outputs/ for visual inspection.
//
// Assertions per group:
//   Inside*     — entries present; canvas same size as a no-legend baseline.
//   OutsideRight* — entries present; canvas is wider than the no-legend baseline.
//   OutsideLeft*  — entries present; canvas is wider than the no-legend baseline.
//   OutsideTop*   — entries present; canvas is taller than the no-legend baseline.
//   OutsideBottom*— entries present; canvas is taller than the no-legend baseline.

/// Scatter plot without `.with_legend()` so `auto_from_plots` does not
/// auto-detect entries and default to `OutsideRightTop` margins.
/// Legend entries are supplied explicitly via `with_legend_entries` in each test.
fn preset_scatter() -> Vec<Plot> {
    vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(vec![(1.0, 2.0), (2.0, 4.0), (3.0, 3.0), (4.0, 5.0)])
            .with_color("steelblue"),
    )]
}

fn preset_entries() -> Vec<LegendEntry> {
    vec![LegendEntry {
        label: "Series A".into(),
        color: "steelblue".into(),
        shape: LegendShape::Rect,
        dasharray: None,
    }]
}

fn preset_base_layout(plots: &[Plot]) -> Layout {
    Layout::auto_from_plots(plots)
        .with_title("Legend Position Test")
        .with_x_label("X Axis Label")
        .with_y_label("Y Axis Label")
}

/// Canvas size with no legend — the reference for Inside* (same) and Outside* (larger) checks.
fn preset_baseline() -> (f64, f64) {
    let plots = preset_scatter();
    let layout = preset_base_layout(&plots);
    let out = svg(plots, layout);
    (extract_width(&out), extract_height_val(&out))
}

fn preset_layout(plots: &[Plot], pos: LegendPosition) -> Layout {
    preset_base_layout(plots)
        .with_legend_entries(preset_entries())
        .with_legend_position(pos)
}

fn extract_height_val(s: &str) -> f64 {
    let start = s.find("height=\"").unwrap() + 8;
    let end = start + s[start..].find('"').unwrap();
    s[start..end].parse().unwrap()
}

fn check_entries_present(out: &str) {
    assert!(out.contains("Series A"), "legend entry 'Series A' should be present");
}

// ── Inside positions ──────────────────────────────────────────────────────────

#[test]
fn legend_position_inside_top_right() {
    let plots = preset_scatter();
    let (bw, bh) = preset_baseline();
    let layout = preset_layout(&plots, LegendPosition::InsideTopRight);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_pos_inside_top_right.svg", &out).unwrap();
    check_entries_present(&out);
    assert_eq!(extract_width(&out) as u64, bw as u64, "InsideTopRight should not widen canvas");
    assert_eq!(extract_height_val(&out) as u64, bh as u64, "InsideTopRight should not change canvas height");
}

#[test]
fn legend_position_inside_top_left() {
    let plots = preset_scatter();
    let (bw, bh) = preset_baseline();
    let layout = preset_layout(&plots, LegendPosition::InsideTopLeft);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_pos_inside_top_left.svg", &out).unwrap();
    check_entries_present(&out);
    assert_eq!(extract_width(&out) as u64, bw as u64, "InsideTopLeft should not widen canvas");
    assert_eq!(extract_height_val(&out) as u64, bh as u64, "InsideTopLeft should not change canvas height");
}

#[test]
fn legend_position_inside_bottom_right() {
    let plots = preset_scatter();
    let (bw, bh) = preset_baseline();
    let layout = preset_layout(&plots, LegendPosition::InsideBottomRight);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_pos_inside_bottom_right.svg", &out).unwrap();
    check_entries_present(&out);
    assert_eq!(extract_width(&out) as u64, bw as u64, "InsideBottomRight should not widen canvas");
    assert_eq!(extract_height_val(&out) as u64, bh as u64, "InsideBottomRight should not change canvas height");
}

#[test]
fn legend_position_inside_bottom_left() {
    let plots = preset_scatter();
    let (bw, bh) = preset_baseline();
    let layout = preset_layout(&plots, LegendPosition::InsideBottomLeft);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_pos_inside_bottom_left.svg", &out).unwrap();
    check_entries_present(&out);
    assert_eq!(extract_width(&out) as u64, bw as u64, "InsideBottomLeft should not widen canvas");
    assert_eq!(extract_height_val(&out) as u64, bh as u64, "InsideBottomLeft should not change canvas height");
}

#[test]
fn legend_position_inside_top_center() {
    let plots = preset_scatter();
    let (bw, bh) = preset_baseline();
    let layout = preset_layout(&plots, LegendPosition::InsideTopCenter);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_pos_inside_top_center.svg", &out).unwrap();
    check_entries_present(&out);
    assert_eq!(extract_width(&out) as u64, bw as u64, "InsideTopCenter should not widen canvas");
    assert_eq!(extract_height_val(&out) as u64, bh as u64, "InsideTopCenter should not change canvas height");
}

#[test]
fn legend_position_inside_bottom_center() {
    let plots = preset_scatter();
    let (bw, bh) = preset_baseline();
    let layout = preset_layout(&plots, LegendPosition::InsideBottomCenter);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_pos_inside_bottom_center.svg", &out).unwrap();
    check_entries_present(&out);
    assert_eq!(extract_width(&out) as u64, bw as u64, "InsideBottomCenter should not widen canvas");
    assert_eq!(extract_height_val(&out) as u64, bh as u64, "InsideBottomCenter should not change canvas height");
}

// ── Outside Right ─────────────────────────────────────────────────────────────

#[test]
fn legend_position_outside_right_top() {
    let plots = preset_scatter();
    let (bw, _) = preset_baseline();
    let layout = preset_layout(&plots, LegendPosition::OutsideRightTop);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_pos_outside_right_top.svg", &out).unwrap();
    check_entries_present(&out);
    assert!(extract_width(&out) > bw, "OutsideRightTop should widen canvas");
}

#[test]
fn legend_position_outside_right_middle() {
    let plots = preset_scatter();
    let (bw, _) = preset_baseline();
    let layout = preset_layout(&plots, LegendPosition::OutsideRightMiddle);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_pos_outside_right_middle.svg", &out).unwrap();
    check_entries_present(&out);
    assert!(extract_width(&out) > bw, "OutsideRightMiddle should widen canvas");
}

#[test]
fn legend_position_outside_right_bottom() {
    let plots = preset_scatter();
    let (bw, _) = preset_baseline();
    let layout = preset_layout(&plots, LegendPosition::OutsideRightBottom);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_pos_outside_right_bottom.svg", &out).unwrap();
    check_entries_present(&out);
    assert!(extract_width(&out) > bw, "OutsideRightBottom should widen canvas");
}

// ── Outside Left ──────────────────────────────────────────────────────────────

#[test]
fn legend_position_outside_left_top() {
    let plots = preset_scatter();
    let (bw, _) = preset_baseline();
    let layout = preset_layout(&plots, LegendPosition::OutsideLeftTop);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_pos_outside_left_top.svg", &out).unwrap();
    check_entries_present(&out);
    assert!(extract_width(&out) > bw, "OutsideLeftTop should widen canvas");
}

#[test]
fn legend_position_outside_left_middle() {
    let plots = preset_scatter();
    let (bw, _) = preset_baseline();
    let layout = preset_layout(&plots, LegendPosition::OutsideLeftMiddle);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_pos_outside_left_middle.svg", &out).unwrap();
    check_entries_present(&out);
    assert!(extract_width(&out) > bw, "OutsideLeftMiddle should widen canvas");
}

#[test]
fn legend_position_outside_left_bottom() {
    let plots = preset_scatter();
    let (bw, _) = preset_baseline();
    let layout = preset_layout(&plots, LegendPosition::OutsideLeftBottom);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_pos_outside_left_bottom.svg", &out).unwrap();
    check_entries_present(&out);
    assert!(extract_width(&out) > bw, "OutsideLeftBottom should widen canvas");
}

// ── Outside Top ───────────────────────────────────────────────────────────────

#[test]
fn legend_position_outside_top_left() {
    let plots = preset_scatter();
    let (_, bh) = preset_baseline();
    let layout = preset_layout(&plots, LegendPosition::OutsideTopLeft);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_pos_outside_top_left.svg", &out).unwrap();
    check_entries_present(&out);
    assert!(extract_height_val(&out) > bh, "OutsideTopLeft should increase canvas height");
}

#[test]
fn legend_position_outside_top_center() {
    let plots = preset_scatter();
    let (_, bh) = preset_baseline();
    let layout = preset_layout(&plots, LegendPosition::OutsideTopCenter);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_pos_outside_top_center.svg", &out).unwrap();
    check_entries_present(&out);
    assert!(extract_height_val(&out) > bh, "OutsideTopCenter should increase canvas height");
}

#[test]
fn legend_position_outside_top_right() {
    let plots = preset_scatter();
    let (_, bh) = preset_baseline();
    let layout = preset_layout(&plots, LegendPosition::OutsideTopRight);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_pos_outside_top_right.svg", &out).unwrap();
    check_entries_present(&out);
    assert!(extract_height_val(&out) > bh, "OutsideTopRight should increase canvas height");
}

// ── Outside Bottom ────────────────────────────────────────────────────────────

#[test]
fn legend_position_outside_bottom_left() {
    let plots = preset_scatter();
    let (_, bh) = preset_baseline();
    let layout = preset_layout(&plots, LegendPosition::OutsideBottomLeft);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_pos_outside_bottom_left.svg", &out).unwrap();
    check_entries_present(&out);
    assert!(extract_height_val(&out) > bh, "OutsideBottomLeft should increase canvas height");
}

#[test]
fn legend_position_outside_bottom_center() {
    let plots = preset_scatter();
    let (_, bh) = preset_baseline();
    let layout = preset_layout(&plots, LegendPosition::OutsideBottomCenter);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_pos_outside_bottom_center.svg", &out).unwrap();
    check_entries_present(&out);
    assert!(extract_height_val(&out) > bh, "OutsideBottomCenter should increase canvas height");
}

#[test]
fn legend_position_outside_bottom_right() {
    let plots = preset_scatter();
    let (_, bh) = preset_baseline();
    let layout = preset_layout(&plots, LegendPosition::OutsideBottomRight);
    let out = svg(plots, layout);
    std::fs::write("test_outputs/legend_pos_outside_bottom_right.svg", &out).unwrap();
    check_entries_present(&out);
    assert!(extract_height_val(&out) > bh, "OutsideBottomRight should increase canvas height");
}

// ── Twin-Y OutsideRight legend tests ─────────────────────────────────────────
//
// Verify that OutsideRight{Top,Middle,Bottom} legends do not overlap the y2
// axis tick labels when a secondary y-axis is present.  The legend box must
// start to the RIGHT of every `text-anchor="start"` text element (y2 tick
// labels and the y2 axis label all use start/non-rotated anchoring on the
// right side of the plot).

use kuva::render::render::render_twin_y;

fn twin_y_svg(pos: LegendPosition) -> String {
    std::fs::create_dir_all("test_outputs").ok();
    let primary = vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(vec![(1.0, 2.0), (2.0, 4.0), (3.0, 3.0)])
            .with_color("steelblue"),
    )];
    let secondary = vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(vec![(1.0, 100.0), (2.0, 200.0), (3.0, 150.0)])
            .with_color("tomato"),
    )];
    let entries = vec![
        LegendEntry { label: "Primary".into(),   color: "steelblue".into(), shape: LegendShape::Circle, dasharray: None },
        LegendEntry { label: "Secondary".into(), color: "tomato".into(),    shape: LegendShape::Circle, dasharray: None },
    ];
    let layout = Layout::new((0.0, 4.0), (0.0, 5.0))
        .with_title("Twin-Y Legend Test")
        .with_x_label("X Axis")
        .with_y_label("Primary Y")
        .with_y2_range(0.0, 250.0)
        .with_y2_label("Secondary Y")
        .with_legend_entries(entries)
        .with_legend_position(pos);
    SvgBackend.render_scene(&render_twin_y(primary, secondary, layout))
}

/// Parse the x attribute of the legend background rect (fill="#ffffff").
fn legend_box_x(svg: &str) -> Option<f64> {
    svg.split("<rect ")
        .skip(1)
        .find(|seg| seg.contains("fill=\"#ffffff\""))
        .and_then(|seg| {
            let x_start = seg.find("x=\"")? + 3;
            let x_end = x_start + seg[x_start..].find('"')?;
            seg[x_start..x_end].parse::<f64>().ok()
        })
}

/// Find the maximum x value among y2 tick-label texts.
///
/// y2 tick labels use text-anchor="start".  Legend entry texts also use
/// text-anchor="start", but they appear to the RIGHT of the legend box.
/// We only want elements that sit to the LEFT of the legend box, i.e.
/// the axis tick labels — so we cap to `box_x - 1` to exclude the legend
/// entries themselves from the measurement.
fn max_y2_tick_x(svg: &str, box_x: f64) -> f64 {
    svg.split("<text ")
        .skip(1)
        .filter(|seg| seg.contains("text-anchor=\"start\""))
        .filter_map(|seg| {
            let x_start = seg.find("x=\"")? + 3;
            let x_end = x_start + seg[x_start..].find('"')?;
            seg[x_start..x_end].parse::<f64>().ok()
        })
        .filter(|&x| x < box_x) // exclude legend entry texts (right of the box)
        .fold(f64::NEG_INFINITY, f64::max)
}

fn check_twin_entries(out: &str) {
    assert!(out.contains("Primary"),   "legend entry 'Primary' should be present");
    assert!(out.contains("Secondary"), "legend entry 'Secondary' should be present");
}

#[test]
fn twin_y_outside_right_top_clears_y2_axis() {
    let out = twin_y_svg(LegendPosition::OutsideRightTop);
    std::fs::write("test_outputs/twin_y_legend_right_top.svg", &out).unwrap();

    check_twin_entries(&out);
    let box_x = legend_box_x(&out).expect("legend box rect not found");
    let max_y2_x = max_y2_tick_x(&out, box_x);
    assert!(
        box_x > max_y2_x,
        "OutsideRightTop legend box x ({box_x:.1}) must be > max y2 tick label x ({max_y2_x:.1})"
    );
}

#[test]
fn twin_y_outside_right_middle_clears_y2_axis() {
    let out = twin_y_svg(LegendPosition::OutsideRightMiddle);
    std::fs::write("test_outputs/twin_y_legend_right_middle.svg", &out).unwrap();

    check_twin_entries(&out);
    let box_x = legend_box_x(&out).expect("legend box rect not found");
    let max_y2_x = max_y2_tick_x(&out, box_x);
    assert!(
        box_x > max_y2_x,
        "OutsideRightMiddle legend box x ({box_x:.1}) must be > max y2 tick label x ({max_y2_x:.1})"
    );
}

#[test]
fn twin_y_outside_right_bottom_clears_y2_axis() {
    let out = twin_y_svg(LegendPosition::OutsideRightBottom);
    std::fs::write("test_outputs/twin_y_legend_right_bottom.svg", &out).unwrap();

    check_twin_entries(&out);
    let box_x = legend_box_x(&out).expect("legend box rect not found");
    let max_y2_x = max_y2_tick_x(&out, box_x);
    assert!(
        box_x > max_y2_x,
        "OutsideRightBottom legend box x ({box_x:.1}) must be > max y2 tick label x ({max_y2_x:.1})"
    );
}
