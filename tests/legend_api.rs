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
