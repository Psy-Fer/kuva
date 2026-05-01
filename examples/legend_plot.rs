//! `LegendPlot` documentation examples.
//!
//! Demonstrates placing a legend in its own figure cell and using
//! `OutsideBottomColumns` for multi-column legend placement below a plot.
//!
//! Generates SVGs used in the docs. Run with:
//!
//! ```bash
//! cargo run --example legend_plot
//! ```
//!
//! SVGs are written to `docs/src/assets/legend_plot/`.

use kuva::backend::svg::SvgBackend;
use kuva::plot::legend::{LegendEntry, LegendPosition, LegendShape};
use kuva::plot::legend_plot::LegendPlot;
use kuva::plot::scatter::ScatterPlot;
use kuva::render::figure::Figure;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::{collect_legend_entries, render_multiple};

const OUT: &str = "docs/src/assets/legend_plot";

fn write_single(name: &str, plots: Vec<Plot>, layout: Layout) {
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
    println!("  wrote {OUT}/{name}.svg");
}

fn write_figure(name: &str, fig: Figure) {
    let svg = SvgBackend.render_scene(&fig.render());
    std::fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
    println!("  wrote {OUT}/{name}.svg");
}

// ── Shared dataset ───────────────────────────────────────────────────────────

const GROUPS: &[(&str, &str, &[(f64, f64)])] = &[
    ("Alpha",   "#4477AA", &[(0.5,1.2),(1.1,2.0),(1.8,1.7),(2.5,2.8),(3.2,2.4),(3.9,3.5)]),
    ("Beta",    "#EE6677", &[(0.7,3.8),(1.3,4.5),(2.0,4.1),(2.8,5.2),(3.4,4.8),(4.1,5.8)]),
    ("Gamma",   "#228833", &[(0.6,6.5),(1.2,7.1),(1.9,6.8),(2.6,7.8),(3.3,7.3),(4.0,8.2)]),
    ("Delta",   "#CCBB44", &[(0.4,9.0),(1.0,9.6),(1.7,9.3),(2.4,10.1),(3.1,9.8),(3.8,10.7)]),
    ("Epsilon", "#66CCEE", &[(0.8,11.5),(1.4,12.0),(2.1,11.8),(2.9,12.6),(3.6,12.2),(4.3,13.0)]),
    ("Zeta",    "#AA3377", &[(0.3,13.8),(0.9,14.4),(1.6,14.0),(2.3,14.9),(3.0,14.5),(3.7,15.2)]),
];

fn scatter_plots() -> Vec<Plot> {
    GROUPS.iter().map(|(label, color, pts)| {
        Plot::Scatter(
            ScatterPlot::new()
                .with_data(pts.iter().copied())
                .with_color(*color)
                .with_legend(*label)
                .with_size(6.0),
        )
    }).collect()
}

fn scatter_plots_no_legend() -> Vec<Plot> {
    GROUPS.iter().map(|(_, color, pts)| {
        Plot::Scatter(
            ScatterPlot::new()
                .with_data(pts.iter().copied())
                .with_color(*color)
                .with_size(6.0),
        )
    }).collect()
}

// ── 1. LegendPlot cell below two data panels ─────────────────────────────────

fn legend_cell_below() {
    // Two independent panels share one LegendPlot row at the bottom.
    let panel_a = scatter_plots_no_legend();
    let panel_b: Vec<Plot> = GROUPS.iter().map(|(_, color, pts)| {
        // Same groups, slightly shifted Y values for a second cohort.
        Plot::Scatter(
            ScatterPlot::new()
                .with_data(pts.iter().map(|&(x, y)| (x, y * 0.85 + 0.5)))
                .with_color(*color)
                .with_size(6.0),
        )
    }).collect();

    // Collect entries from panel_a's colour assignments (labels live there).
    let ref_plots = scatter_plots();
    let entries = collect_legend_entries(&ref_plots);
    let legend_cell = LegendPlot::from_entries(entries);

    let layout_a = Layout::auto_from_plots(&panel_a)
        .with_title("Cohort A")
        .with_x_label("Time (h)").with_y_label("Signal");
    let layout_b = Layout::auto_from_plots(&panel_b)
        .with_title("Cohort B")
        .with_x_label("Time (h)").with_y_label("Signal");

    write_figure("legend_cell_below", Figure::new(2, 2)
        .with_cell_size(380.0, 300.0)
        // Structure: groups of flat cell indices (row*cols + col).
        // Cells 2 and 3 (bottom row) merge into one spanning legend cell.
        .with_structure(vec![
            vec![0],      // top-left: panel A
            vec![1],      // top-right: panel B
            vec![2, 3],   // bottom row merged: LegendPlot
        ])
        .with_plots(vec![
            panel_a,
            panel_b,
            vec![legend_cell.into()],
        ])
        .with_layouts(vec![layout_a, layout_b])
        .with_title("Two-panel figure with shared LegendPlot"));
}

// ── 2. LegendPlot cell to the right of a chart ───────────────────────────────

fn legend_cell_right() {
    let data = scatter_plots_no_legend();
    let ref_plots = scatter_plots();
    let entries = collect_legend_entries(&ref_plots);
    let legend_cell = LegendPlot::from_entries(entries).with_cols(1);

    let data_layout = Layout::auto_from_plots(&data)
        .with_title("Expression by Group")
        .with_x_label("Time (h)").with_y_label("Expression");

    write_figure("legend_cell_right", Figure::new(1, 2)
        .with_cell_size(460.0, 340.0)
        .with_plots(vec![data, vec![legend_cell.into()]])
        .with_layouts(vec![data_layout])
        .with_title("Chart with LegendPlot to the right"));
}

// ── 3. OutsideBottomColumns — many entries, auto-column packing ───────────────

fn outside_bottom_columns() {
    // 12 groups to demonstrate multi-column layout below the plot.
    let extra_groups: &[(&str, &str, &[(f64, f64)])] = &[
        ("Alpha",   "#4477AA", &[(1.0,2.0),(2.0,2.5),(3.0,3.1)]),
        ("Beta",    "#EE6677", &[(1.0,3.5),(2.0,4.0),(3.0,4.8)]),
        ("Gamma",   "#228833", &[(1.0,5.0),(2.0,5.6),(3.0,6.2)]),
        ("Delta",   "#CCBB44", &[(1.0,1.5),(2.0,1.9),(3.0,2.4)]),
        ("Epsilon", "#66CCEE", &[(1.0,7.0),(2.0,7.5),(3.0,8.1)]),
        ("Zeta",    "#AA3377", &[(1.0,8.5),(2.0,9.0),(3.0,9.7)]),
        ("Eta",     "#BBBBBB", &[(1.0,10.0),(2.0,10.5),(3.0,11.0)]),
        ("Theta",   "#332288", &[(1.0,11.5),(2.0,12.1),(3.0,12.8)]),
        ("Iota",    "#882255", &[(1.0,13.0),(2.0,13.6),(3.0,14.3)]),
        ("Kappa",   "#44AA99", &[(1.0,14.5),(2.0,15.1),(3.0,15.8)]),
        ("Lambda",  "#DDCC77", &[(1.0,16.0),(2.0,16.6),(3.0,17.2)]),
        ("Mu",      "#117733", &[(1.0,17.5),(2.0,18.1),(3.0,18.8)]),
    ];

    let mut plots: Vec<Plot> = Vec::new();
    let mut entries: Vec<LegendEntry> = Vec::new();
    for (label, color, pts) in extra_groups {
        plots.push(Plot::Scatter(
            ScatterPlot::new()
                .with_data(pts.iter().copied())
                .with_color(*color)
                .with_size(5.0),
        ));
        entries.push(LegendEntry {
            label: (*label).to_string(),
            color: (*color).to_string(),
            shape: LegendShape::Circle,
            dasharray: None,
        });
    }

    let layout = Layout::auto_from_plots(&plots)
        .with_title("OutsideBottomColumns — 12 groups, auto-packed columns")
        .with_x_label("Time (h)").with_y_label("Expression")
        .with_legend_entries(entries)
        .with_legend_position(LegendPosition::OutsideBottomColumns);

    write_single("outside_bottom_columns", plots, layout);
}

// ── 4. Standalone LegendPlot with title ──────────────────────────────────────

fn standalone_with_title() {
    let entries: Vec<LegendEntry> = GROUPS.iter().map(|(label, color, _)| LegendEntry {
        label: (*label).to_string(),
        color: (*color).to_string(),
        shape: LegendShape::Rect,
        dasharray: None,
    }).collect();

    let lp = LegendPlot::from_entries(entries)
        .with_title("Sample groups")
        .with_cols(2);

    let layout = Layout::new((0.0, 1.0), (0.0, 1.0))
        .with_title("Standalone LegendPlot (2 columns, titled)");

    write_single("standalone_with_title", vec![lp.into()], layout);
}

// ── main ─────────────────────────────────────────────────────────────────────

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/legend_plot");

    legend_cell_below();
    legend_cell_right();
    outside_bottom_columns();
    standalone_with_title();

    println!("LegendPlot SVGs written to {OUT}/");
}
