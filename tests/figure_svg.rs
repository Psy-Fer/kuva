mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::{LegendEntry, LegendPlot, LegendShape, LinePlot, ScatterPlot};
use kuva::render::figure::{Figure, FigureLegendPosition};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

fn scatter_plot(color: &str) -> Vec<Plot> {
    vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(vec![(1.0, 2.0), (3.0, 5.0), (5.0, 3.0), (7.0, 8.0)])
            .with_color(color),
    )]
}

fn line_plot(color: &str) -> Vec<Plot> {
    vec![Plot::Line(
        LinePlot::new()
            .with_data(vec![(0.0, 0.0), (2.0, 4.0), (4.0, 3.0), (6.0, 7.0)])
            .with_color(color),
    )]
}

#[test]
fn figure_basic_2x2() {
    let figure = Figure::new(2, 2).with_plots(vec![
        scatter_plot("blue"),
        scatter_plot("red"),
        line_plot("green"),
        line_plot("purple"),
    ]);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_basic_2x2.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<g"));
    assert!(svg.contains("</g>"));
    // Each panel has a translate group + a clip group → 2 × 4 = 8
    assert_eq!(svg.matches("<g ").count(), 8);
}

#[test]
fn figure_merged_cells() {
    // 2x3 grid: 3 top cells + 1 wide bottom spanning all 3 columns
    let figure = Figure::new(2, 3)
        .with_structure(vec![vec![0], vec![1], vec![2], vec![3, 4, 5]])
        .with_plots(vec![
            scatter_plot("blue"),
            scatter_plot("red"),
            scatter_plot("green"),
            line_plot("purple"),
        ]);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_merged_cells.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // Each panel has a translate group + a clip group → 2 × 4 = 8
    assert_eq!(svg.matches("<g ").count(), 8);
}

#[test]
fn figure_vertical_span() {
    // 2x2 grid: tall left cell spanning both rows + 2 right cells
    let figure = Figure::new(2, 2)
        .with_structure(vec![
            vec![0, 2], // left column, both rows
            vec![1],    // top right
            vec![3],    // bottom right
        ])
        .with_plots(vec![
            line_plot("blue"),
            scatter_plot("red"),
            scatter_plot("green"),
        ]);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_vertical_span.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // Each panel has a translate group + a clip group → 2 × 3 = 6
    assert_eq!(svg.matches("<g ").count(), 6);
}

#[test]
fn figure_shared_y_row() {
    // 1x3 grid with shared y axis
    let plots: Vec<Vec<Plot>> = vec![
        scatter_plot("blue"),
        scatter_plot("red"),
        scatter_plot("green"),
    ];

    let layouts = vec![
        Layout::auto_from_plots(&plots[0])
            .with_y_label("Shared Y")
            .with_x_label("X1"),
        Layout::auto_from_plots(&plots[1])
            .with_y_label("Y2")
            .with_x_label("X2"),
        Layout::auto_from_plots(&plots[2])
            .with_y_label("Y3")
            .with_x_label("X3"),
    ];

    let figure = Figure::new(1, 3)
        .with_plots(plots)
        .with_layouts(layouts)
        .with_shared_y(0);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_shared_y_row.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // Only the leftmost subplot should have the y label
    assert!(svg.contains("Shared Y"));
    // The other y labels should be suppressed
    assert!(!svg.contains("Y2"));
    assert!(!svg.contains("Y3"));
}

#[test]
fn figure_panel_labels() {
    let figure = Figure::new(2, 2)
        .with_plots(vec![
            scatter_plot("blue"),
            scatter_plot("red"),
            line_plot("green"),
            line_plot("purple"),
        ])
        .with_labels();

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_panel_labels.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains(">A<"));
    assert!(svg.contains(">B<"));
    assert!(svg.contains(">C<"));
    assert!(svg.contains(">D<"));
    assert!(svg.contains(r#"font-weight="bold""#));
}

#[test]
fn figure_fewer_plots_than_slots() {
    // 2x2 grid with only 3 plots, 4th cell blank
    let figure = Figure::new(2, 2).with_plots(vec![
        scatter_plot("blue"),
        scatter_plot("red"),
        line_plot("green"),
        // 4th slot empty
    ]);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_fewer_plots.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // 3 panels × (translate group + clip group) = 6; 4th cell is blank
    assert_eq!(svg.matches("<g ").count(), 6);
}

#[test]
fn figure_title_and_subplot_titles() {
    let plots: Vec<Vec<Plot>> = vec![scatter_plot("blue"), scatter_plot("red")];

    let layouts = vec![
        Layout::auto_from_plots(&plots[0]).with_title("Subplot A"),
        Layout::auto_from_plots(&plots[1]).with_title("Subplot B"),
    ];

    let figure = Figure::new(1, 2)
        .with_title("Figure Title")
        .with_plots(plots)
        .with_layouts(layouts);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_title.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Figure Title"));
    assert!(svg.contains("Subplot A"));
    assert!(svg.contains("Subplot B"));
}

#[test]
fn figure_shared_y_row_slice() {
    // 2x3 grid, share y only for columns 1-2 in row 0
    let plots: Vec<Vec<Plot>> = vec![
        scatter_plot("blue"),  // row 0, col 0 — independent
        scatter_plot("red"),   // row 0, col 1 — shared
        scatter_plot("green"), // row 0, col 2 — shared
        line_plot("purple"),   // row 1, col 0
        line_plot("orange"),   // row 1, col 1
        line_plot("teal"),     // row 1, col 2
    ];

    let layouts = vec![
        Layout::auto_from_plots(&plots[0]).with_y_label("Y0"),
        Layout::auto_from_plots(&plots[1]).with_y_label("Y1"),
        Layout::auto_from_plots(&plots[2]).with_y_label("Y2"),
        Layout::auto_from_plots(&plots[3]).with_y_label("Y3"),
        Layout::auto_from_plots(&plots[4]).with_y_label("Y4"),
        Layout::auto_from_plots(&plots[5]).with_y_label("Y5"),
    ];

    let figure = Figure::new(2, 3)
        .with_plots(plots)
        .with_layouts(layouts)
        .with_shared_y_slice(0, 1, 2);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_shared_y_slice.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // Y0 should remain (col 0, not in slice)
    assert!(svg.contains("Y0"));
    // Y1 should remain (leftmost in shared slice)
    assert!(svg.contains("Y1"));
    // Y2 should be suppressed (non-leftmost in shared slice)
    assert!(!svg.contains("Y2"));
    // Bottom row labels should all remain
    assert!(svg.contains("Y3"));
    assert!(svg.contains("Y4"));
    assert!(svg.contains("Y5"));
}

#[test]
fn figure_shared_x_column() {
    // 2x1 vertical stack with shared x axis (e.g. stacked time series)
    let plots: Vec<Vec<Plot>> = vec![
        scatter_plot("blue"), // top
        line_plot("red"),     // bottom
    ];

    let layouts = vec![
        Layout::auto_from_plots(&plots[0])
            .with_x_label("X top")
            .with_y_label("Y1"),
        Layout::auto_from_plots(&plots[1])
            .with_x_label("Time")
            .with_y_label("Y2"),
    ];

    let figure = Figure::new(2, 1)
        .with_plots(plots)
        .with_layouts(layouts)
        .with_shared_x(0);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_shared_x_column.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // Only the bottommost subplot should have the x label
    assert!(svg.contains("Time"));
    // The top x label should be suppressed
    assert!(!svg.contains("X top"));
    // Both y labels should remain
    assert!(svg.contains("Y1"));
    assert!(svg.contains("Y2"));
}

#[test]
fn figure_negative_y_only() {
    // Y spans negative, X stays positive — y-axis should pad below zero
    let plots: Vec<Vec<Plot>> = vec![vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(vec![(1.0, -3.0), (3.0, 2.0), (5.0, -7.0), (7.0, 4.0)])
            .with_color("blue"),
    )]];

    let figure = Figure::new(1, 1).with_plots(plots);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_negative_y.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // x min should be 0 (all positive, clamped)
    assert!(svg.contains(">0<"));
    // y: ticks=5 → auto_nice_range(-7.11, 4.11, 5) → step=2 → (-8, 6) range.
    // generate_ticks(-8, 6, 5) → step=2.5 → first tick at -7.5
    assert!(svg.contains(">-7.5<"));
}

#[test]
fn figure_negative_x_only() {
    // X spans negative, Y stays positive
    let plots: Vec<Vec<Plot>> = vec![vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(vec![(-6.0, 1.0), (-2.0, 5.0), (3.0, 3.0), (8.0, 7.0)])
            .with_color("red"),
    )]];

    let figure = Figure::new(1, 1).with_plots(plots);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_negative_x.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // x: ticks=5 → auto_nice_range(-6.14, 8.14, 5) → step=2.5 → (-7.5, 10) range.
    // generate_ticks(-7.5, 10, 5) → step=5 → first tick at -5
    assert!(svg.contains(">-5<"));
    // y min should be 0 (all positive, clamped)
    assert!(svg.contains(">0<"));
}

#[test]
fn figure_both_axes_negative() {
    // Both axes span negative ranges
    let plots: Vec<Vec<Plot>> = vec![vec![Plot::Line(
        LinePlot::new()
            .with_data(vec![(-5.0, -4.0), (-1.0, -8.0), (3.0, -2.0), (6.0, -6.0)])
            .with_color("green"),
    )]];

    let figure = Figure::new(1, 1).with_plots(plots);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_both_negative.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // x: ticks=5 → auto_nice_range(-5.11, 6.11, 5) → step=2 → (-6, 8) range.
    // generate_ticks(-6, 8, 5) → step=2.5 → ticks: -5, -2.5, 0, 2.5, 5, 7.5
    // y: ticks=5 → auto_nice_range(-8.06, -1.94, 5) → step=1 → (-9, -1) range.
    // generate_ticks(-9, -1, 5) → step=2 → ticks: -8, -6, -4, -2
    assert!(svg.contains(">-6<")); // y tick: confirms negative y range
    assert!(svg.contains(">7.5<")); // max x tick (was ">7<")
    assert!(svg.contains(">-8<")); // most-negative y tick
    assert!(svg.contains(">-5<")); // first negative x tick (was ">-1<")
}

#[test]
fn figure_large_negative_values() {
    // Values below -10 should use the 5% rule
    let plots: Vec<Vec<Plot>> = vec![vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(vec![
                (-50.0, -20.0),
                (-10.0, 30.0),
                (40.0, -40.0),
                (80.0, 60.0),
            ])
            .with_color("purple"),
    )]];

    let figure = Figure::new(1, 1).with_plots(plots);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_large_negatives.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // x: min=-50, pad_min(-50) = -50 * 1.05 = -52.5; max=80, pad_max(80) = 80 * 1.05 = 84
    // y: min=-40, pad_min(-40) = -40 * 1.05 = -42; max=60, pad_max(60) = 60 * 1.05 = 63
    // These get rounded by auto_nice_range, so just verify the SVG renders
    // and that negative ticks appear
    assert!(svg.contains(">-"));
}

// ── with_figure_size ────────────────────────────────────────────────────────

/// Parse `width="NNN"` or `height="NNN"` from an SVG string.
fn svg_dim(svg: &str, attr: &str) -> f64 {
    let needle = format!(r#"{attr}=""#);
    let start = svg
        .find(&needle)
        .unwrap_or_else(|| panic!("no {attr} in SVG"))
        + needle.len();
    let end = svg[start..].find('"').unwrap() + start;
    svg[start..end].parse().unwrap()
}

#[test]
fn figure_size_basic() {
    // 2×2 grid, no title, no legend — total size should be exactly 800×600.
    // With default padding=10, spacing=15:
    //   cell_w = (800 - 2·10 - 1·15) / 2 = 382.5
    //   cell_h = (600 - 2·10 - 1·15) / 2 = 282.5
    let plots: Vec<Vec<Plot>> = vec![
        scatter_plot("steelblue"),
        scatter_plot("crimson"),
        line_plot("seagreen"),
        line_plot("darkorange"),
    ];
    let layouts = vec![
        Layout::auto_from_plots(&plots[0]).with_title("A"),
        Layout::auto_from_plots(&plots[1]).with_title("B"),
        Layout::auto_from_plots(&plots[2]).with_title("C"),
        Layout::auto_from_plots(&plots[3]).with_title("D"),
    ];

    let scene = Figure::new(2, 2)
        .with_plots(plots)
        .with_layouts(layouts)
        .with_labels()
        .with_figure_size(800.0, 600.0)
        .render();

    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_size_basic.svg", &svg).unwrap();

    assert_eq!(
        svg_dim(&svg, "width"),
        800.0,
        "SVG width should match requested figure width"
    );
    assert_eq!(
        svg_dim(&svg, "height"),
        600.0,
        "SVG height should match requested figure height"
    );
}

#[test]
fn figure_size_with_title() {
    // 1×3 grid with a title — total size should still be exactly 900×400.
    // Title height=30 is deducted from the cell height budget:
    //   cell_w = (900 - 2·10 - 2·15) / 3 = 283.33…
    //   cell_h = (400 - 2·10 - 0·15 - 30) / 1 = 350
    let plots: Vec<Vec<Plot>> = vec![
        scatter_plot("steelblue"),
        scatter_plot("crimson"),
        line_plot("seagreen"),
    ];
    let layouts = vec![
        Layout::auto_from_plots(&plots[0]).with_title("Panel A"),
        Layout::auto_from_plots(&plots[1]).with_title("Panel B"),
        Layout::auto_from_plots(&plots[2]).with_title("Panel C"),
    ];

    let scene = Figure::new(1, 3)
        .with_title("Figure with title — cells auto-fit")
        .with_plots(plots)
        .with_layouts(layouts)
        .with_figure_size(900.0, 400.0)
        .render();

    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_size_with_title.svg", &svg).unwrap();

    assert_eq!(svg_dim(&svg, "width"), 900.0);
    assert_eq!(svg_dim(&svg, "height"), 400.0);
    assert!(svg.contains("Figure with title"));
}

#[test]
fn figure_size_with_shared_legend() {
    // 1×2 grid with a right-side shared legend — total size 760×380.
    // Legend width (~87px) + legend_spacing(20) is deducted from cell width budget.
    let plots: Vec<Vec<Plot>> = vec![
        scatter_with_legend("steelblue", "Control"),
        scatter_with_legend("crimson", "Treatment"),
    ];
    let layouts = vec![
        Layout::auto_from_plots(&plots[0])
            .with_title("Experiment 1")
            .with_x_label("Time")
            .with_y_label("Response"),
        Layout::auto_from_plots(&plots[1])
            .with_title("Experiment 2")
            .with_x_label("Time"),
    ];

    let scene = Figure::new(1, 2)
        .with_plots(plots)
        .with_layouts(layouts)
        .with_shared_legend()
        .with_figure_size(760.0, 380.0)
        .render();

    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_size_with_legend.svg", &svg).unwrap();

    assert_eq!(svg_dim(&svg, "width"), 760.0);
    assert_eq!(svg_dim(&svg, "height"), 380.0);
    assert!(svg.contains("Control"));
    assert!(svg.contains("Treatment"));
}

fn scatter_with_legend(color: &str, label: &str) -> Vec<Plot> {
    vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(vec![(1.0, 2.0), (3.0, 5.0), (5.0, 3.0), (7.0, 8.0)])
            .with_color(color)
            .with_legend(label),
    )]
}

#[test]
fn figure_panel_legends() {
    // 1x2 figure where each subplot has its own legend
    let figure = Figure::new(1, 2).with_plots(vec![
        scatter_with_legend("blue", "Blue data"),
        scatter_with_legend("red", "Red data"),
    ]);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_panel_legends.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Blue data"));
    assert!(svg.contains("Red data"));
}

#[test]
fn figure_shared_legend_right() {
    // 2x2 figure with shared legend on the right
    let figure = Figure::new(2, 2)
        .with_plots(vec![
            scatter_with_legend("blue", "Blue"),
            scatter_with_legend("red", "Red"),
            scatter_with_legend("green", "Green"),
            scatter_with_legend("blue", "Blue"), // duplicate label
        ])
        .with_shared_legend();

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_shared_legend_right.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // Should have the shared legend entries (deduplicated: Blue, Red, Green)
    assert!(svg.contains("Blue"));
    assert!(svg.contains("Red"));
    assert!(svg.contains("Green"));
    // Count occurrences of "Blue" legend label — should appear once (deduplicated)
    // in the shared legend, not in per-panel legends (suppressed)
    let blue_count = svg.matches(">Blue<").count();
    assert_eq!(
        blue_count, 1,
        "Expected 1 occurrence of Blue in shared legend, got {blue_count}"
    );
}

#[test]
fn figure_shared_legend_bottom() {
    // 1x2 figure with shared legend at bottom
    let figure = Figure::new(1, 2)
        .with_plots(vec![
            scatter_with_legend("blue", "Sample A"),
            scatter_with_legend("red", "Sample B"),
        ])
        .with_shared_legend_bottom();

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_shared_legend_bottom.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Sample A"));
    assert!(svg.contains("Sample B"));
    // Shared legend should increase the figure height (bottom position)
    // Per-panel legends should be suppressed
    let a_count = svg.matches(">Sample A<").count();
    assert_eq!(
        a_count, 1,
        "Expected 1 occurrence of Sample A, got {a_count}"
    );
}

#[test]
fn figure_shared_legend_manual_entries() {
    // Provide custom legend entries manually
    let manual_entries = vec![
        LegendEntry {
            label: "Custom 1".into(),
            color: "orange".into(),
            shape: LegendShape::Circle,
            dasharray: None,
        },
        LegendEntry {
            label: "Custom 2".into(),
            color: "purple".into(),
            shape: LegendShape::Line,
            dasharray: None,
        },
    ];

    let figure = Figure::new(1, 2)
        .with_plots(vec![scatter_plot("blue"), scatter_plot("red")])
        .with_shared_legend()
        .with_shared_legend_entries(manual_entries);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_shared_legend_manual.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Custom 1"));
    assert!(svg.contains("Custom 2"));
}

#[test]
fn figure_keep_panel_legends() {
    // Shared legend + keep panel legends
    let figure = Figure::new(1, 2)
        .with_plots(vec![
            scatter_with_legend("blue", "Blue"),
            scatter_with_legend("red", "Red"),
        ])
        .with_shared_legend()
        .with_keep_panel_legends();

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_keep_panel_legends.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // Both shared legend and per-panel legends should render
    // "Blue" appears in shared legend + panel legend = at least 2
    let blue_count = svg.matches(">Blue<").count();
    assert!(
        blue_count >= 2,
        "Expected Blue in both shared and panel legends, got {blue_count}"
    );
}

#[test]
fn figure_explicit_axis_bounds_preserved() {
    // Regression: clone_layout must carry x_axis_{min,max} and y_axis_{min,max}
    // so that explicit bounds survive into the rendered subplot.
    let plots: Vec<Vec<Plot>> = vec![scatter_plot("blue"), scatter_plot("red")];

    let layouts = vec![
        Layout::auto_from_plots(&plots[0])
            .with_y_axis_min(-10.0)
            .with_y_axis_max(20.0),
        Layout::auto_from_plots(&plots[1])
            .with_x_axis_min(-10.0)
            .with_x_axis_max(20.0),
    ];

    let figure = Figure::new(1, 2).with_plots(plots).with_layouts(layouts);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_explicit_bounds.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // Panel 0: y range forced to [-10, 20] → ticks at -10 and 20
    assert!(
        svg.contains(">-10<"),
        "y_axis_min=-10 should produce a -10 tick"
    );
    assert!(
        svg.contains(">20<"),
        "y_axis_max=20 should produce a 20 tick"
    );
    // Panel 1: x range forced to [-10, 20] → same boundary ticks
    // (both panels share the -10 / 20 assertions above, which is fine)
}

#[test]
fn figure_twin_y_cell() {
    // 1×2 figure: left cell is a regular scatter, right cell is a twin-Y (line + bar).
    let primary = vec![Plot::Line(
        LinePlot::new()
            .with_data(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 2.0), (3.0, 4.0)])
            .with_color("steelblue")
            .with_legend("Primary"),
    )];
    let secondary = vec![Plot::Line(
        LinePlot::new()
            .with_data(vec![(0.0, 100.0), (1.0, 250.0), (2.0, 180.0), (3.0, 320.0)])
            .with_color("crimson")
            .with_legend("Secondary"),
    )];

    let figure = Figure::new(1, 2)
        .with_plots(vec![scatter_plot("green")]) // cell 0: regular
        .with_twin_y_plots(1, primary, secondary); // cell 1: twin-Y

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_twin_y_cell.svg", &svg).unwrap();

    assert!(svg.contains("<svg"), "should produce valid SVG");
    // Twin-Y emits a right-side y2 axis line; check the SVG has more than 2 axis line elements
    assert!(svg.contains("<line"), "should have axis lines");
}

#[test]
fn figure_twin_y_auto_layout() {
    // Twin-Y cell with no explicit layout — auto_from_twin_y_plots should fire.
    let primary = vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(vec![(0.0f64, 1.0f64), (1.0, 2.0), (2.0, 1.5)])
            .with_color("steelblue"),
    )];
    let secondary = vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(vec![(0.0f64, 500.0f64), (1.0, 750.0), (2.0, 600.0)])
            .with_color("crimson"),
    )];

    let figure = Figure::new(1, 1).with_twin_y_plots(0, primary, secondary);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_twin_y_auto.svg", &svg).unwrap();

    assert!(svg.contains("<svg"), "should produce valid SVG");
    assert!(svg.contains("<circle"), "scatter points should be present");
}

#[test]
fn figure_twin_y_with_layout() {
    // Twin-Y cell with an explicit layout passed via with_layouts.
    let primary = vec![Plot::Line(
        LinePlot::new()
            .with_data(vec![(0.0, 0.0), (10.0, 1.0)])
            .with_color("steelblue"),
    )];
    let secondary = vec![Plot::Line(
        LinePlot::new()
            .with_data(vec![(0.0, 0.0), (10.0, 1000.0)])
            .with_color("crimson"),
    )];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary)
        .with_title("Twin-Y in Figure")
        .with_x_label("Time")
        .with_y_label("Primary")
        .with_y2_label("Secondary");

    let figure = Figure::new(1, 1)
        .with_layouts(vec![layout])
        .with_twin_y_plots(0, primary, secondary);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_twin_y_layout.svg", &svg).unwrap();

    assert!(svg.contains("Twin-Y in Figure"), "title should appear");
    assert!(svg.contains("Time"), "x label should appear");
}

// ── FigureLegendPosition presets ────────────────────────────────────────────
//
// 1×2 figure, cells 500×380, padding=10, spacing=15:
//   grid_width  = 2·500 + 15 + 2·10 = 1035
//   grid_height = 380 + 2·10        = 400
//
// Legend entries "Alpha"/"Beta" (5 chars max):
//   legend_width  = (5·7+35).max(80) = 80
//   legend_height = 2·18+20          = 56
//   legend_spacing = 20
//
// Right / Left → total 1135 × 400
// Top  / Bottom → total 1035 × 476

fn legend_plots() -> Vec<Vec<Plot>> {
    vec![
        scatter_with_legend("steelblue", "Alpha"),
        scatter_with_legend("crimson", "Beta"),
    ]
}

fn render_legend_pos(pos: FigureLegendPosition, name: &str) -> String {
    std::fs::create_dir_all("test_outputs").unwrap();
    let scene = Figure::new(1, 2)
        .with_plots(legend_plots())
        .with_shared_legend_position(pos)
        .render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output(format!("test_outputs/{name}.svg"), &svg).unwrap();
    svg
}

// ── Right side ───────────────────────────────────────────────────────────────

#[test]
fn figure_legend_right_top() {
    let svg = render_legend_pos(FigureLegendPosition::RightTop, "figure_legend_right_top");
    assert!(
        svg.contains("Alpha") && svg.contains("Beta"),
        "legend entries missing"
    );
    assert_eq!(svg_dim(&svg, "width"), 1135.0, "right legend expands width");
    assert_eq!(
        svg_dim(&svg, "height"),
        400.0,
        "right legend does not expand height"
    );
}

#[test]
fn figure_legend_right_middle() {
    let svg = render_legend_pos(
        FigureLegendPosition::RightMiddle,
        "figure_legend_right_middle",
    );
    assert!(svg.contains("Alpha") && svg.contains("Beta"));
    assert_eq!(svg_dim(&svg, "width"), 1135.0);
    assert_eq!(svg_dim(&svg, "height"), 400.0);
}

#[test]
fn figure_legend_right_bottom() {
    let svg = render_legend_pos(
        FigureLegendPosition::RightBottom,
        "figure_legend_right_bottom",
    );
    assert!(svg.contains("Alpha") && svg.contains("Beta"));
    assert_eq!(svg_dim(&svg, "width"), 1135.0);
    assert_eq!(svg_dim(&svg, "height"), 400.0);
}

// ── Left side ────────────────────────────────────────────────────────────────

#[test]
fn figure_legend_left_top() {
    let svg = render_legend_pos(FigureLegendPosition::LeftTop, "figure_legend_left_top");
    assert!(
        svg.contains("Alpha") && svg.contains("Beta"),
        "legend entries missing"
    );
    assert_eq!(svg_dim(&svg, "width"), 1135.0, "left legend expands width");
    assert_eq!(
        svg_dim(&svg, "height"),
        400.0,
        "left legend does not expand height"
    );
}

#[test]
fn figure_legend_left_middle() {
    let svg = render_legend_pos(
        FigureLegendPosition::LeftMiddle,
        "figure_legend_left_middle",
    );
    assert!(svg.contains("Alpha") && svg.contains("Beta"));
    assert_eq!(svg_dim(&svg, "width"), 1135.0);
    assert_eq!(svg_dim(&svg, "height"), 400.0);
}

#[test]
fn figure_legend_left_bottom() {
    let svg = render_legend_pos(
        FigureLegendPosition::LeftBottom,
        "figure_legend_left_bottom",
    );
    assert!(svg.contains("Alpha") && svg.contains("Beta"));
    assert_eq!(svg_dim(&svg, "width"), 1135.0);
    assert_eq!(svg_dim(&svg, "height"), 400.0);
}

// ── Top edge ─────────────────────────────────────────────────────────────────

#[test]
fn figure_legend_top_left() {
    let svg = render_legend_pos(FigureLegendPosition::TopLeft, "figure_legend_top_left");
    assert!(
        svg.contains("Alpha") && svg.contains("Beta"),
        "legend entries missing"
    );
    assert_eq!(
        svg_dim(&svg, "width"),
        1035.0,
        "top legend does not expand width"
    );
    assert_eq!(svg_dim(&svg, "height"), 476.0, "top legend expands height");
}

#[test]
fn figure_legend_top_center() {
    let svg = render_legend_pos(FigureLegendPosition::TopCenter, "figure_legend_top_center");
    assert!(svg.contains("Alpha") && svg.contains("Beta"));
    assert_eq!(svg_dim(&svg, "width"), 1035.0);
    assert_eq!(svg_dim(&svg, "height"), 476.0);
}

#[test]
fn figure_legend_top_right() {
    let svg = render_legend_pos(FigureLegendPosition::TopRight, "figure_legend_top_right");
    assert!(svg.contains("Alpha") && svg.contains("Beta"));
    assert_eq!(svg_dim(&svg, "width"), 1035.0);
    assert_eq!(svg_dim(&svg, "height"), 476.0);
}

// ── Bottom edge ───────────────────────────────────────────────────────────────

#[test]
fn figure_legend_bottom_left() {
    let svg = render_legend_pos(
        FigureLegendPosition::BottomLeft,
        "figure_legend_bottom_left",
    );
    assert!(
        svg.contains("Alpha") && svg.contains("Beta"),
        "legend entries missing"
    );
    assert_eq!(
        svg_dim(&svg, "width"),
        1035.0,
        "bottom legend does not expand width"
    );
    assert_eq!(
        svg_dim(&svg, "height"),
        476.0,
        "bottom legend expands height"
    );
}

#[test]
fn figure_legend_bottom_center() {
    let svg = render_legend_pos(
        FigureLegendPosition::BottomCenter,
        "figure_legend_bottom_center",
    );
    assert!(svg.contains("Alpha") && svg.contains("Beta"));
    assert_eq!(svg_dim(&svg, "width"), 1035.0);
    assert_eq!(svg_dim(&svg, "height"), 476.0);
}

#[test]
fn figure_legend_bottom_right() {
    let svg = render_legend_pos(
        FigureLegendPosition::BottomRight,
        "figure_legend_bottom_right",
    );
    assert!(svg.contains("Alpha") && svg.contains("Beta"));
    assert_eq!(svg_dim(&svg, "width"), 1035.0);
    assert_eq!(svg_dim(&svg, "height"), 476.0);
}

// ── Legend height scales with body_size ─────────────────────────────────────
//
// Regression for the hardcoded `18.0` in render_legend_at and figure.rs.
// With body_size=16: line_height = 16*1.5 = 24 (not 18).
// Legend entries "Alpha"/"Beta" (2 entries):
//   legend_height = 2*24+20 = 68   (was 2*18+20 = 56 at default body_size=12)
//   legend_spacing = 20
// Top legend: total height = 400 + 68 + 20 = 488  (was 476 at body_size=12)

#[test]
fn figure_legend_height_scales_with_body_size() {
    let plots = legend_plots();
    let layouts = vec![
        Layout::auto_from_plots(&plots[0]).with_body_size(16),
        Layout::auto_from_plots(&plots[1]).with_body_size(16),
    ];
    let scene = Figure::new(1, 2)
        .with_plots(plots)
        .with_layouts(layouts)
        .with_shared_legend_position(FigureLegendPosition::TopCenter)
        .render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output(
        "test_outputs/figure_legend_scaled_body_size.svg",
        &svg,
    )
    .unwrap();
    assert!(svg.contains("Alpha") && svg.contains("Beta"));
    assert_eq!(
        svg_dim(&svg, "height"),
        488.0,
        "legend height must scale with body_size (16*1.5=24 per entry, 2*24+20=68 + 20 spacing)"
    );
}

// ── Backward-compat aliases still work ───────────────────────────────────────

#[test]
fn figure_legend_right_compat() {
    // `Right` is a backward-compat alias for `RightMiddle` — same dimensions and entries.
    let svg = render_legend_pos(FigureLegendPosition::Right, "figure_legend_right_compat");
    assert!(svg.contains("Alpha") && svg.contains("Beta"));
    assert_eq!(svg_dim(&svg, "width"), 1135.0);
    assert_eq!(svg_dim(&svg, "height"), 400.0);
}

#[test]
fn figure_legend_bottom_compat() {
    // `Bottom` is a backward-compat alias for `BottomCenter` — same dimensions and entries.
    let svg = render_legend_pos(FigureLegendPosition::Bottom, "figure_legend_bottom_compat");
    assert!(svg.contains("Alpha") && svg.contains("Beta"));
    assert_eq!(svg_dim(&svg, "width"), 1035.0);
    assert_eq!(svg_dim(&svg, "height"), 476.0);
}

// ── Left/Top offsets actually shift the grid ─────────────────────────────────
//
// For Left positions the grid cells must be offset rightward by
// (legend_width + legend_spacing) = 100 px.  A cell that would normally start
// at x = padding = 10 should now start at x = 110.  We verify this by
// checking that the `translate(110,…)` group appears in the SVG.
//
// For Top positions the grid cells must be offset downward by
// (legend_height + legend_spacing) = 76 px.  Cell y normally starts at
// y = padding = 10, so with top legend it should be y = 86.
// We verify `translate(…,86)` appears.

#[test]
fn figure_legend_left_grid_offset() {
    let svg = render_legend_pos(
        FigureLegendPosition::LeftMiddle,
        "figure_legend_left_offset_check",
    );
    // Cell 0 translate: x = cell_x_offset(100) + padding(10) + col(0)*(500+15) = 110
    assert!(
        svg.contains("translate(110,"),
        "left legend should shift grid cells right by 100px"
    );
}

#[test]
fn figure_legend_top_grid_offset() {
    let svg = render_legend_pos(
        FigureLegendPosition::TopCenter,
        "figure_legend_top_offset_check",
    );
    // Cell 0 translate: x=10, y = cell_y_offset(76) + padding(10) + figure_title(0) = 86
    assert!(
        svg.contains(",86)"),
        "top legend should shift grid cells down by 76px"
    );
}

// ── Per-row / per-col sizing ─────────────────────────────────────────────────

fn make_legend_entries(labels: &[&str], colors: &[&str]) -> Vec<LegendEntry> {
    labels
        .iter()
        .zip(colors.iter())
        .map(|(&label, &color)| LegendEntry {
            label: label.into(),
            color: color.into(),
            shape: LegendShape::Circle,
            dasharray: None,
        })
        .collect()
}

#[test]
fn figure_per_row_height() {
    // 2×1 grid: scatter plot in row 0 (300 px), LegendPlot in row 1 (80 px).
    // This is the canonical "thin legend strip below a data plot" use case.
    // Expected total height:
    //   per_row_heights.sum() + (rows-1)*spacing + 2*padding
    //   = (300 + 80) + 1*15 + 2*10 = 415
    // Width: cell_width(500) + 0*spacing + 2*padding = 520
    let entries = make_legend_entries(&["Control", "Treatment"], &["steelblue", "crimson"]);
    let data_plots: Vec<Plot> = vec![
        scatter_with_legend("steelblue", "Control"),
        scatter_with_legend("crimson", "Treatment"),
    ]
    .into_iter()
    .flatten()
    .collect();
    let mut layout = Layout::auto_from_plots(&data_plots)
        .with_title("Response over time")
        .with_x_label("Time")
        .with_y_label("Value");
    layout.show_legend = false; // legend is in the LegendPlot row below

    let figure = Figure::new(2, 1)
        .with_plots(vec![
            data_plots,
            vec![Plot::LegendPlot(LegendPlot::from_entries(entries))],
        ])
        .with_layouts(vec![layout])
        .with_row_height(0, 300.0)
        .with_row_height(1, 80.0);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_per_row_height.svg", &svg).unwrap();

    assert!(svg.contains("<circle"), "scatter points should be present");
    assert!(
        svg.contains("Control") && svg.contains("Treatment"),
        "legend labels should appear"
    );
    assert_eq!(
        svg_dim(&svg, "width"),
        520.0,
        "width should be cell_width + 2*padding"
    );
    assert_eq!(
        svg_dim(&svg, "height"),
        415.0,
        "height should reflect explicit row heights"
    );
}

#[test]
fn figure_per_col_width() {
    // 1×2 grid: col 0 = 600 px (wide scatter), col 1 = 300 px (compact line).
    // Expected total width:
    //   (600 + 300) + 1*spacing + 2*padding = 935
    // Height: cell_height(380) + 0*spacing + 2*padding = 400
    let scatter_data = vec![
        (1.0, 2.0),
        (2.0, 3.5),
        (3.0, 2.8),
        (4.0, 4.1),
        (5.0, 5.0),
        (6.0, 4.6),
    ];
    let line_data = vec![
        (0.0, 0.0),
        (1.0, 1.2),
        (2.0, 2.5),
        (3.0, 2.0),
        (4.0, 3.8),
        (5.0, 4.5),
    ];

    let sp = vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(scatter_data)
            .with_color("steelblue"),
    )];
    let lp = vec![Plot::Line(
        LinePlot::new().with_data(line_data).with_color("crimson"),
    )];

    let layouts = vec![
        Layout::auto_from_plots(&sp)
            .with_title("Wide panel")
            .with_x_label("X")
            .with_y_label("Y"),
        Layout::auto_from_plots(&lp)
            .with_title("Compact panel")
            .with_x_label("X"),
    ];

    let figure = Figure::new(1, 2)
        .with_plots(vec![sp, lp])
        .with_layouts(layouts)
        .with_col_width(0, 600.0)
        .with_col_width(1, 300.0);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_per_col_width.svg", &svg).unwrap();

    assert!(svg.contains("<circle"), "scatter points should render");
    assert!(svg.contains("<path"), "line path should render");
    assert!(svg.contains("Wide panel") && svg.contains("Compact panel"));
    assert_eq!(
        svg_dim(&svg, "width"),
        935.0,
        "width should reflect explicit col widths"
    );
    assert_eq!(
        svg_dim(&svg, "height"),
        400.0,
        "height should be cell_height + 2*padding"
    );
}

#[test]
fn figure_per_row_col_mixed() {
    // 2×2 grid: row 0 has two data panels, row 1 (80 px) has a LegendPlot spanning
    // both columns; col 0 is narrower (250 px), col 1 uses default (500 px).
    // Width:  (250 + 500) + 1*15 + 2*10 = 785
    // Height: (380 +  80) + 1*15 + 2*10 = 495
    let entries = make_legend_entries(&["Alpha", "Beta"], &["steelblue", "crimson"]);
    let plots_a = scatter_with_legend("steelblue", "Alpha");
    let plots_b = scatter_with_legend("crimson", "Beta");
    let mut layout_a = Layout::auto_from_plots(&plots_a)
        .with_title("Group A")
        .with_x_label("X")
        .with_y_label("Y");
    layout_a.show_legend = false;
    let mut layout_b = Layout::auto_from_plots(&plots_b)
        .with_title("Group B")
        .with_x_label("X");
    layout_b.show_legend = false;

    let figure = Figure::new(2, 2)
        .with_structure(vec![
            vec![0],
            vec![1],    // row 0: two data panels
            vec![2, 3], // row 1: legend spans both cols
        ])
        .with_plots(vec![
            plots_a,
            plots_b,
            vec![Plot::LegendPlot(LegendPlot::from_entries(entries))],
        ])
        .with_layouts(vec![layout_a, layout_b])
        .with_row_height(1, 80.0)
        .with_col_width(0, 250.0);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_per_row_col_mixed.svg", &svg).unwrap();

    assert!(svg.contains("Group A") && svg.contains("Group B"));
    assert!(svg.contains("Alpha") && svg.contains("Beta"));
    assert_eq!(
        svg_dim(&svg, "width"),
        785.0,
        "col 0 explicit + col 1 default"
    );
    assert_eq!(
        svg_dim(&svg, "height"),
        495.0,
        "row 0 default + row 1 explicit"
    );
}

#[test]
fn figure_figure_size_with_explicit_row() {
    // 2×1 grid: scatter plot in row 0, LegendPlot in row 1 (60 px).
    // with_figure_size forces total 800×600; row 0 absorbs the remaining height.
    let entries = make_legend_entries(&["Series A", "Series B"], &["steelblue", "crimson"]);
    let data_plots: Vec<Plot> = vec![
        scatter_with_legend("steelblue", "Series A"),
        scatter_with_legend("crimson", "Series B"),
    ]
    .into_iter()
    .flatten()
    .collect();
    let mut layout = Layout::auto_from_plots(&data_plots)
        .with_title("Experiment results")
        .with_x_label("Time")
        .with_y_label("Response");
    layout.show_legend = false; // legend is in the LegendPlot row below

    let figure = Figure::new(2, 1)
        .with_plots(vec![
            data_plots,
            vec![Plot::LegendPlot(LegendPlot::from_entries(entries))],
        ])
        .with_layouts(vec![layout])
        .with_row_height(1, 60.0)
        .with_figure_size(800.0, 600.0);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_size_explicit_row.svg", &svg).unwrap();

    assert!(svg.contains("Experiment results"));
    assert!(svg.contains("Series A") && svg.contains("Series B"));
    assert_eq!(
        svg_dim(&svg, "width"),
        800.0,
        "figure_size width must be honoured"
    );
    assert_eq!(
        svg_dim(&svg, "height"),
        600.0,
        "figure_size height must be honoured"
    );
}

// ── LegendPlot auto-sizing ────────────────────────────────────────────────────

#[test]
fn figure_legend_plot_fits_short_cell() {
    // 2×1 grid: scatter data in row 0 (default height), LegendPlot in row 1 (120 px).
    // The legend has 12 entries; at 18 px/row a single column would need 216 px > 120 px,
    // so the renderer must bump up columns until all rows fit.
    let colors = [
        "steelblue",
        "crimson",
        "seagreen",
        "darkorange",
        "mediumpurple",
        "teal",
        "coral",
        "goldenrod",
        "slateblue",
        "peru",
        "cadetblue",
        "indianred",
    ];
    let labels: Vec<String> = (1..=12).map(|i| format!("Group {i}")).collect();
    let entries: Vec<LegendEntry> = labels
        .iter()
        .zip(colors.iter())
        .map(|(l, c)| LegendEntry {
            label: l.clone(),
            color: (*c).into(),
            shape: LegendShape::Circle,
            dasharray: None,
        })
        .collect();

    // Data panel: one scatter series per color so the plot area is non-empty.
    // No .with_legend() calls — the LegendPlot row below carries the labels.
    let data_plots: Vec<Plot> = colors
        .iter()
        .map(|&c| {
            Plot::Scatter(
                ScatterPlot::new()
                    .with_data(vec![(1.0, 2.0), (3.0, 4.0), (5.0, 3.0)])
                    .with_color(c),
            )
        })
        .collect();
    let layout = Layout::auto_from_plots(&data_plots)
        .with_title("12-group scatter")
        .with_x_label("X")
        .with_y_label("Y");

    let figure = Figure::new(2, 1)
        .with_plots(vec![
            data_plots,
            vec![Plot::LegendPlot(LegendPlot::from_entries(entries))],
        ])
        .with_layouts(vec![layout])
        .with_row_height(1, 120.0);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_legend_plot_fits.svg", &svg).unwrap();

    assert!(svg.contains("<circle"), "scatter points should render");
    assert!(
        svg.contains("12-group scatter"),
        "panel title should appear"
    );
    for i in 1..=12 {
        assert!(
            svg.contains(&format!("Group {i}")),
            "legend entry Group {i} missing"
        );
    }
}

#[test]
fn figure_legend_plot_single_col_when_tall_enough() {
    // 2×1 grid: line data in row 0, LegendPlot in row 1 (200 px).
    // 4 entries × 18 px = 72 px < 200 px available — should stay in 1 column.
    let colors = ["steelblue", "crimson", "seagreen", "darkorange"];
    let labels = ["Alpha", "Beta", "Gamma", "Delta"];
    let entries: Vec<LegendEntry> = labels
        .iter()
        .zip(colors.iter())
        .map(|(&l, &c)| LegendEntry {
            label: l.into(),
            color: c.into(),
            shape: LegendShape::Rect,
            dasharray: None,
        })
        .collect();

    // No .with_legend() — the LegendPlot row carries the labels.
    let data_plots: Vec<Plot> = colors
        .iter()
        .enumerate()
        .map(|(k, &c)| {
            let data: Vec<(f64, f64)> = (0..6)
                .map(|i| (i as f64, i as f64 * 0.8 + k as f64))
                .collect();
            Plot::Line(LinePlot::new().with_data(data).with_color(c))
        })
        .collect();
    let layout = Layout::auto_from_plots(&data_plots)
        .with_title("Four series")
        .with_x_label("Time")
        .with_y_label("Value");

    let figure = Figure::new(2, 1)
        .with_plots(vec![
            data_plots,
            vec![Plot::LegendPlot(LegendPlot::from_entries(entries))],
        ])
        .with_layouts(vec![layout])
        .with_row_height(1, 200.0);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/figure_legend_plot_single_col.svg", &svg).unwrap();

    assert!(svg.contains("<path"), "line paths should render");
    assert!(svg.contains("Four series"), "panel title should appear");
    assert!(
        svg.contains("Alpha")
            && svg.contains("Beta")
            && svg.contains("Gamma")
            && svg.contains("Delta")
    );
}
