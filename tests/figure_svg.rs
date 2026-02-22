use visus::plot::{ScatterPlot, LinePlot, LegendEntry, LegendShape};
use visus::backend::svg::SvgBackend;
use visus::render::figure::Figure;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

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
    let figure = Figure::new(2, 2)
        .with_plots(vec![
            scatter_plot("blue"),
            scatter_plot("red"),
            line_plot("green"),
            line_plot("purple"),
        ]);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/figure_basic_2x2.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<g"));
    assert!(svg.contains("</g>"));
    // Should have 4 subplot groups
    assert_eq!(svg.matches("<g ").count(), 4);
}

#[test]
fn figure_merged_cells() {
    // 2x3 grid: 3 top cells + 1 wide bottom spanning all 3 columns
    let figure = Figure::new(2, 3)
        .with_structure(vec![
            vec![0], vec![1], vec![2],
            vec![3, 4, 5],
        ])
        .with_plots(vec![
            scatter_plot("blue"),
            scatter_plot("red"),
            scatter_plot("green"),
            line_plot("purple"),
        ]);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/figure_merged_cells.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    assert_eq!(svg.matches("<g ").count(), 4);
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
    std::fs::write("test_outputs/figure_vertical_span.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    assert_eq!(svg.matches("<g ").count(), 3);
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
    std::fs::write("test_outputs/figure_shared_y_row.svg", &svg).unwrap();

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
    std::fs::write("test_outputs/figure_panel_labels.svg", &svg).unwrap();

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
    let figure = Figure::new(2, 2)
        .with_plots(vec![
            scatter_plot("blue"),
            scatter_plot("red"),
            line_plot("green"),
            // 4th slot empty
        ]);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/figure_fewer_plots.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // Only 3 subplot groups (the 4th cell is blank)
    assert_eq!(svg.matches("<g ").count(), 3);
}

#[test]
fn figure_title_and_subplot_titles() {
    let plots: Vec<Vec<Plot>> = vec![
        scatter_plot("blue"),
        scatter_plot("red"),
    ];

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
    std::fs::write("test_outputs/figure_title.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Figure Title"));
    assert!(svg.contains("Subplot A"));
    assert!(svg.contains("Subplot B"));
}

#[test]
fn figure_shared_y_row_slice() {
    // 2x3 grid, share y only for columns 1-2 in row 0
    let plots: Vec<Vec<Plot>> = vec![
        scatter_plot("blue"),   // row 0, col 0 — independent
        scatter_plot("red"),    // row 0, col 1 — shared
        scatter_plot("green"),  // row 0, col 2 — shared
        line_plot("purple"),    // row 1, col 0
        line_plot("orange"),    // row 1, col 1
        line_plot("teal"),      // row 1, col 2
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
    std::fs::write("test_outputs/figure_shared_y_slice.svg", &svg).unwrap();

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
        scatter_plot("blue"),   // top
        line_plot("red"),       // bottom
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
    std::fs::write("test_outputs/figure_shared_x_column.svg", &svg).unwrap();

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
    let plots: Vec<Vec<Plot>> = vec![
        vec![Plot::Scatter(
            ScatterPlot::new()
                .with_data(vec![(1.0, -3.0), (3.0, 2.0), (5.0, -7.0), (7.0, 4.0)])
                .with_color("blue"),
        )],
    ];

    let figure = Figure::new(1, 1)
        .with_plots(plots);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/figure_negative_y.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // x min should be 0 (all positive, clamped)
    assert!(svg.contains(">0<"));
    // y should go below -7: pad_min(-7.0) = -7 - 1 = -8
    assert!(svg.contains(">-8<"));
}

#[test]
fn figure_negative_x_only() {
    // X spans negative, Y stays positive
    let plots: Vec<Vec<Plot>> = vec![
        vec![Plot::Scatter(
            ScatterPlot::new()
                .with_data(vec![(-6.0, 1.0), (-2.0, 5.0), (3.0, 3.0), (8.0, 7.0)])
                .with_color("red"),
        )],
    ];

    let figure = Figure::new(1, 1)
        .with_plots(plots);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/figure_negative_x.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // x min = -6, pad_min(-6) = -7, rounded by auto_nice_range to -8
    assert!(svg.contains(">-8<"));
    // y min should be 0 (all positive, clamped)
    assert!(svg.contains(">0<"));
}

#[test]
fn figure_both_axes_negative() {
    // Both axes span negative ranges
    let plots: Vec<Vec<Plot>> = vec![
        vec![Plot::Line(
            LinePlot::new()
                .with_data(vec![(-5.0, -4.0), (-1.0, -8.0), (3.0, -2.0), (6.0, -6.0)])
                .with_color("green"),
        )],
    ];

    let figure = Figure::new(1, 1)
        .with_plots(plots);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/figure_both_negative.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // x: min=-5, pad_min(-5) = -5 - 1 = -6; max=6, pad_max(6) = 7
    assert!(svg.contains(">-6<"));
    assert!(svg.contains(">7<"));
    // y: all negative. min=-8, pad_min(-8) = -8 - 1 = -9; max=-2, pad_max(-2) = -1
    assert!(svg.contains(">-9<"));
    assert!(svg.contains(">-1<"));
}

#[test]
fn figure_large_negative_values() {
    // Values below -10 should use the 5% rule
    let plots: Vec<Vec<Plot>> = vec![
        vec![Plot::Scatter(
            ScatterPlot::new()
                .with_data(vec![(-50.0, -20.0), (-10.0, 30.0), (40.0, -40.0), (80.0, 60.0)])
                .with_color("purple"),
        )],
    ];

    let figure = Figure::new(1, 1)
        .with_plots(plots);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/figure_large_negatives.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // x: min=-50, pad_min(-50) = -50 * 1.05 = -52.5; max=80, pad_max(80) = 80 * 1.05 = 84
    // y: min=-40, pad_min(-40) = -40 * 1.05 = -42; max=60, pad_max(60) = 60 * 1.05 = 63
    // These get rounded by auto_nice_range, so just verify the SVG renders
    // and that negative ticks appear
    assert!(svg.contains(">-"));
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
    let figure = Figure::new(1, 2)
        .with_plots(vec![
            scatter_with_legend("blue", "Blue data"),
            scatter_with_legend("red", "Red data"),
        ]);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/figure_panel_legends.svg", &svg).unwrap();

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
    std::fs::write("test_outputs/figure_shared_legend_right.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // Should have the shared legend entries (deduplicated: Blue, Red, Green)
    assert!(svg.contains("Blue"));
    assert!(svg.contains("Red"));
    assert!(svg.contains("Green"));
    // Count occurrences of "Blue" legend label — should appear once (deduplicated)
    // in the shared legend, not in per-panel legends (suppressed)
    let blue_count = svg.matches(">Blue<").count();
    assert_eq!(blue_count, 1, "Expected 1 occurrence of Blue in shared legend, got {blue_count}");
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
    std::fs::write("test_outputs/figure_shared_legend_bottom.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Sample A"));
    assert!(svg.contains("Sample B"));
    // Shared legend should increase the figure height (bottom position)
    // Per-panel legends should be suppressed
    let a_count = svg.matches(">Sample A<").count();
    assert_eq!(a_count, 1, "Expected 1 occurrence of Sample A, got {a_count}");
}

#[test]
fn figure_shared_legend_manual_entries() {
    // Provide custom legend entries manually
    let manual_entries = vec![
        LegendEntry { label: "Custom 1".into(), color: "orange".into(), shape: LegendShape::Circle, dasharray: None },
        LegendEntry { label: "Custom 2".into(), color: "purple".into(), shape: LegendShape::Line, dasharray: None },
    ];

    let figure = Figure::new(1, 2)
        .with_plots(vec![
            scatter_plot("blue"),
            scatter_plot("red"),
        ])
        .with_shared_legend()
        .with_shared_legend_entries(manual_entries);

    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/figure_shared_legend_manual.svg", &svg).unwrap();

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
    std::fs::write("test_outputs/figure_keep_panel_legends.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    // Both shared legend and per-panel legends should render
    // "Blue" appears in shared legend + panel legend = at least 2
    let blue_count = svg.matches(">Blue<").count();
    assert!(blue_count >= 2, "Expected Blue in both shared and panel legends, got {blue_count}");
}
