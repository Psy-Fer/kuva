use kuva::plot::mosaic::MosaicPlot;
use kuva::render::{plots::Plot, layout::Layout, render::render_multiple};
use kuva::backend::svg::SvgBackend;

fn render(mp: MosaicPlot, title: &str) -> String {
    let plots = vec![Plot::Mosaic(mp)];
    let layout = Layout::auto_from_plots(&plots).with_title(title);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

#[test]
fn test_mosaic_basic() {
    // 3 cols × 3 rows — epidemiology data
    let mp = MosaicPlot::new()
        .with_cell("Unexposed", "Healthy", 500.0)
        .with_cell("Unexposed", "Mild", 80.0)
        .with_cell("Unexposed", "Severe", 20.0)
        .with_cell("Low Dose", "Healthy", 300.0)
        .with_cell("Low Dose", "Mild", 150.0)
        .with_cell("Low Dose", "Severe", 50.0)
        .with_cell("High Dose", "Healthy", 100.0)
        .with_cell("High Dose", "Mild", 200.0)
        .with_cell("High Dose", "Severe", 200.0);
    let svg = render(mp, "Exposure vs Severity");
    std::fs::write("test_outputs/mosaic_basic.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "SVG should contain rects");
    assert!(svg.contains('%'), "SVG should contain percent labels");
}

#[test]
fn test_mosaic_two_cols_unequal() {
    // Col A total=10, Col B total=90 — B should be much wider
    let mp = MosaicPlot::new()
        .with_cell("A", "Yes", 4.0)
        .with_cell("A", "No", 6.0)
        .with_cell("B", "Yes", 63.0)
        .with_cell("B", "No", 27.0);
    let plots = vec![Plot::Mosaic(mp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Unequal Column Widths");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/mosaic_two_cols_unequal.svg", &svg).unwrap();
    // Find width attributes — column B should have a larger width value than A.
    // Since the SVG uses rect elements, check that we have rects and they render.
    assert!(svg.contains("<rect"));
    // Both column labels should appear
    assert!(svg.contains(">A<") || svg.contains("A"), "Col A label");
    assert!(svg.contains(">B<") || svg.contains("B"), "Col B label");
}

#[test]
fn test_mosaic_no_percents() {
    let mp = MosaicPlot::new()
        .with_cell("X", "Yes", 40.0)
        .with_cell("X", "No", 60.0)
        .with_cell("Y", "Yes", 70.0)
        .with_cell("Y", "No", 30.0)
        .with_percents(false)
        .with_values(false);
    let svg = render(mp, "No Labels");
    std::fs::write("test_outputs/mosaic_no_percents.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "should still have rects");
    // No percent sign in cell labels
    assert!(!svg.contains("%."), "should not have percent labels in cells");
}

#[test]
fn test_mosaic_show_values() {
    let mp = MosaicPlot::new()
        .with_cell("Group1", "Cat A", 120.0)
        .with_cell("Group1", "Cat B", 80.0)
        .with_cell("Group2", "Cat A", 30.0)
        .with_cell("Group2", "Cat B", 170.0)
        .with_percents(false)
        .with_values(true);
    let svg = render(mp, "Raw Values");
    std::fs::write("test_outputs/mosaic_show_values.svg", &svg).unwrap();
    assert!(svg.contains("<rect"));
    // Raw values should appear as text
    assert!(svg.contains("120") || svg.contains("80"), "raw value labels");
}

#[test]
fn test_mosaic_normalize_false() {
    let mp = MosaicPlot::new()
        .with_cell("Small", "A", 5.0)
        .with_cell("Small", "B", 5.0)
        .with_cell("Large", "A", 45.0)
        .with_cell("Large", "B", 45.0)
        .with_normalize(false);
    let svg = render(mp, "Non-normalized");
    std::fs::write("test_outputs/mosaic_normalize_false.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "should render without panic");
}

#[test]
fn test_mosaic_explicit_ordering() {
    let mp = MosaicPlot::new()
        .with_cell("A", "X", 10.0)
        .with_cell("A", "Y", 20.0)
        .with_cell("A", "Z", 30.0)
        .with_cell("B", "X", 40.0)
        .with_cell("B", "Y", 25.0)
        .with_cell("B", "Z", 35.0)
        .with_col_order(["B", "A"])
        .with_row_order(["Z", "Y", "X"]);
    let svg = render(mp, "Explicit Ordering");
    std::fs::write("test_outputs/mosaic_explicit_ordering.svg", &svg).unwrap();
    assert!(svg.contains("<rect"));
    // Both column names should appear
    assert!(svg.contains('A'));
    assert!(svg.contains('B'));
}

#[test]
fn test_mosaic_custom_colors() {
    let mp = MosaicPlot::new()
        .with_cell("Alpha", "R", 50.0)
        .with_cell("Alpha", "G", 30.0)
        .with_cell("Alpha", "B", 20.0)
        .with_cell("Beta", "R", 20.0)
        .with_cell("Beta", "G", 50.0)
        .with_cell("Beta", "B", 30.0)
        .with_group_colors(["#ff0000", "#00ff00", "#0000ff"]);
    let svg = render(mp, "Custom Colors");
    std::fs::write("test_outputs/mosaic_custom_colors.svg", &svg).unwrap();
    assert!(svg.contains("ff0000"), "SVG should contain custom color ff0000");
}

#[test]
fn test_mosaic_legend() {
    let mp = MosaicPlot::new()
        .with_cell("Control", "Positive", 30.0)
        .with_cell("Control", "Negative", 70.0)
        .with_cell("Treated", "Positive", 60.0)
        .with_cell("Treated", "Negative", 40.0)
        .with_legend("Response");
    let plots = vec![Plot::Mosaic(mp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Treatment vs Response");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/mosaic_legend.svg", &svg).unwrap();
    // Legend row labels should appear in SVG
    assert!(svg.contains("Positive"), "legend label Positive");
    assert!(svg.contains("Negative"), "legend label Negative");
}

#[test]
fn test_mosaic_gap() {
    let mp = MosaicPlot::new()
        .with_cell("P", "a", 50.0)
        .with_cell("P", "b", 50.0)
        .with_cell("Q", "a", 30.0)
        .with_cell("Q", "b", 70.0)
        .with_gap(5.0);
    let svg = render(mp, "Wide Gap");
    std::fs::write("test_outputs/mosaic_gap.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "should render without panic");
}

#[test]
fn test_mosaic_single_column() {
    let mp = MosaicPlot::new()
        .with_cell("Only", "Cat1", 25.0)
        .with_cell("Only", "Cat2", 25.0)
        .with_cell("Only", "Cat3", 25.0)
        .with_cell("Only", "Cat4", 25.0);
    let svg = render(mp, "Single Column");
    std::fs::write("test_outputs/mosaic_single_column.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "should render without panic");
}

#[test]
fn test_mosaic_empty() {
    let mp = MosaicPlot::new();
    let plots = vec![Plot::Mosaic(mp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Empty");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write("test_outputs/mosaic_empty.svg", &svg).unwrap();
    // Should render without panicking — empty mosaic just emits nothing
    assert!(svg.contains("<svg"), "should still produce an SVG element");
}

#[test]
fn test_mosaic_market_share() {
    // 4 regions × 3 products — realistic market share data
    let mp = MosaicPlot::new()
        .with_cells([
            ("North",   "Product A", 120.0),
            ("North",   "Product B",  80.0),
            ("North",   "Product C",  50.0),
            ("South",   "Product A",  90.0),
            ("South",   "Product B", 110.0),
            ("South",   "Product C",  40.0),
            ("East",    "Product A",  60.0),
            ("East",    "Product B",  70.0),
            ("East",    "Product C",  90.0),
            ("West",    "Product A", 200.0),
            ("West",    "Product B", 150.0),
            ("West",    "Product C",  80.0),
        ])
        .with_legend("Product");
    let svg = render(mp, "Market Share by Region");
    std::fs::write("test_outputs/mosaic_market_share.svg", &svg).unwrap();
    assert!(svg.contains("<rect"));
    assert!(svg.contains("North") || svg.contains("South") || svg.contains("East") || svg.contains("West"));
}
