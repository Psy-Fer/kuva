use visus::plot::{ScatterPlot, LinePlot};
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
