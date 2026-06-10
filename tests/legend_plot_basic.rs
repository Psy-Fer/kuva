mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::{LegendEntry, LegendPlot, LegendShape, ScatterPlot};
use kuva::render::figure::Figure;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render_to_svg;

fn write(name: &str, svg: &str) {
    std::fs::create_dir_all("test_outputs").ok();
    common::write_test_output(format!("test_outputs/{name}.svg"), svg).unwrap();
}

fn palette() -> Vec<&'static str> {
    vec![
        "#4477AA", "#EE6677", "#228833", "#CCBB44", "#66CCEE", "#AA3377", "#BBBBBB", "#332288",
        "#882255", "#44AA99", "#DDCC77", "#117733", "#999933", "#AA4499", "#88CCEE", "#CC6677",
        "#DDDDDD", "#44BB99", "#AAAA00", "#EE8866",
    ]
}

fn make_entries(n: usize) -> Vec<LegendEntry> {
    let pal = palette();
    (0..n)
        .map(|i| LegendEntry {
            label: format!("Category {}", i + 1),
            color: pal[i % pal.len()].to_string(),
            shape: LegendShape::Rect,
            dasharray: None,
        })
        .collect()
}

fn scatter() -> ScatterPlot {
    ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (3.0, 4.5), (5.0, 3.0), (2.0, 1.5)])
        .with_color("steelblue")
}

#[test]
fn test_legend_plot_standalone_12() {
    let lp = LegendPlot::from_entries(make_entries(12));
    let layout =
        Layout::new((0.0, 1.0), (0.0, 1.0)).with_title("LegendPlot standalone – 12 entries");
    let plots: Vec<Plot> = vec![lp.into()];
    let svg = render_to_svg(plots, layout);
    write("legend_plot_standalone_12", &svg);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_legend_plot_with_explicit_cols() {
    let lp = LegendPlot::from_entries(make_entries(20)).with_cols(4);
    let layout = Layout::new((0.0, 1.0), (0.0, 1.0))
        .with_title("LegendPlot – 20 entries, 4 columns explicit");
    let plots: Vec<Plot> = vec![lp.into()];
    let svg = render_to_svg(plots, layout);
    write("legend_plot_4cols_20entries", &svg);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_legend_plot_with_title() {
    let lp = LegendPlot::from_entries(make_entries(8)).with_title("Sample groups");
    let plots: Vec<Plot> = vec![lp.into()];
    let svg = render_to_svg(plots, Layout::new((0.0, 1.0), (0.0, 1.0)));
    write("legend_plot_with_title", &svg);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_legend_plot_in_figure_below() {
    // 2-row figure: scatter on top, legend cell below
    let lp = LegendPlot::from_entries(make_entries(12));
    let fig = Figure::new(2, 1)
        .with_cell_size(500.0, 300.0)
        .with_plots(vec![vec![scatter().into()], vec![lp.into()]])
        .with_title("Figure with LegendPlot below");
    let scene = fig.render();
    let svg = SvgBackend::default().render_scene(&scene);
    write("legend_plot_figure_below", &svg);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_legend_plot_in_figure_right() {
    // 1-row, 2-col figure: scatter left, legend right
    let lp = LegendPlot::from_entries(make_entries(10));
    let fig = Figure::new(1, 2)
        .with_cell_size(400.0, 380.0)
        .with_plots(vec![vec![scatter().into()], vec![lp.into()]])
        .with_title("Figure with LegendPlot right");
    let scene = fig.render();
    let svg = SvgBackend::default().render_scene(&scene);
    write("legend_plot_figure_right", &svg);
    assert!(svg.contains("<svg"));
}
