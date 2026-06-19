mod common;
use kuva::plot::{BrickPlot, LegendEntry, LegendPosition, LegendShape};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render_to_svg;
use std::collections::HashMap;

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

fn make_entries(n: usize, label_prefix: &str) -> Vec<LegendEntry> {
    let pal = palette();
    (0..n)
        .map(|i| LegendEntry {
            label: format!("{} {}", label_prefix, i + 1),
            color: pal[i % pal.len()].to_string(),
            shape: LegendShape::Rect,
            dasharray: None,
        })
        .collect()
}

fn make_brick_with_entries(n_entries: usize) -> (Vec<Plot>, Vec<LegendEntry>) {
    let mut tmpl: HashMap<char, String> = HashMap::new();
    let pal = palette();
    for (i, ch) in "ABCDE".chars().enumerate() {
        tmpl.insert(ch, pal[i].to_string());
    }
    let bp = BrickPlot::new()
        .with_sequences(vec![
            "ABCDEABCDE".to_string(),
            "CDABECDABE".to_string(),
            "EABCDEABCD".to_string(),
        ])
        .with_template(tmpl);
    let entries = make_entries(n_entries, "Motif");
    (vec![bp.into()], entries)
}

#[test]
fn test_legend_columns_basic() {
    let (plots, entries) = make_brick_with_entries(10);
    let layout = Layout::auto_from_plots(&plots)
        .with_legend_entries(entries)
        .with_legend_position(LegendPosition::OutsideBottomColumns)
        .with_title("BrickPlot – OutsideBottomColumns (10 entries)");
    let svg = render_to_svg(plots, layout);
    write("brick_legend_columns_10", &svg);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_legend_columns_many_entries() {
    let (plots, entries) = make_brick_with_entries(40);
    let layout = Layout::auto_from_plots(&plots)
        .with_legend_entries(entries)
        .with_legend_position(LegendPosition::OutsideBottomColumns)
        .with_title("BrickPlot – OutsideBottomColumns (40 entries)");
    let svg = render_to_svg(plots, layout);
    write("brick_legend_columns_40", &svg);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_legend_columns_long_labels() {
    let pal = palette();
    let entries: Vec<LegendEntry> = (0..20)
        .map(|i| LegendEntry {
            label: format!("Long motif label number {:02}", i + 1),
            color: pal[i % pal.len()].to_string(),
            shape: LegendShape::Rect,
            dasharray: None,
        })
        .collect();
    let (plots, _) = make_brick_with_entries(0);
    let layout = Layout::auto_from_plots(&plots)
        .with_legend_entries(entries)
        .with_legend_position(LegendPosition::OutsideBottomColumns)
        .with_title("BrickPlot – OutsideBottomColumns (long labels)");
    let svg = render_to_svg(plots, layout);
    write("brick_legend_columns_long_labels", &svg);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_legend_columns_extends_canvas() {
    use kuva::render::layout::ComputedLayout;
    let scatter = kuva::plot::ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
        .with_color("steelblue");
    let plots_base: Vec<Plot> = vec![scatter.into()];
    let h_default = ComputedLayout::from_layout(&Layout::auto_from_plots(&plots_base)).height;

    let scatter2 = kuva::plot::ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
        .with_color("steelblue");
    let plots2: Vec<Plot> = vec![scatter2.into()];
    let layout_cols = Layout::auto_from_plots(&plots2)
        .with_legend_entries(make_entries(20, "Group"))
        .with_legend_position(LegendPosition::OutsideBottomColumns);
    let h_cols = ComputedLayout::from_layout(&layout_cols).height;

    assert!(
        h_cols > h_default,
        "OutsideBottomColumns must extend canvas height"
    );
}
