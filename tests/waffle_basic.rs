mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::waffle::{CellShape, FillOrder, WafflePlot};
use kuva::render::render::render_waffle;
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

fn render(wp: WafflePlot, title: &str) -> String {
    let plots = vec![Plot::Waffle(wp)];
    let layout = Layout::auto_from_plots(&plots).with_title(title);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

fn write(name: &str, svg: &str) {
    std::fs::create_dir_all("test_outputs").unwrap();
    common::write_test_output(format!("test_outputs/{name}"), svg).unwrap();
}

// ─── Basic correctness ───────────────────────────────────────────────────────

#[test]
fn test_waffle_basic() {
    let wp = WafflePlot::new()
        .with_category("Yes", 60.0, "steelblue")
        .with_category("No", 40.0, "tomato");
    let svg = render(wp, "Basic Waffle");
    write("waffle_basic.svg", &svg);
    assert!(svg.contains("<svg"), "should produce valid SVG");
    assert!(svg.contains("<rect"), "should have rect elements");
}

#[test]
fn test_waffle_empty() {
    let wp = WafflePlot::new();
    let svg = render(wp, "Empty Waffle");
    write("waffle_empty.svg", &svg);
    // Should produce a valid SVG without panicking
    assert!(svg.contains("<svg"), "should produce valid SVG");
}

#[test]
fn test_waffle_single_category() {
    let wp = WafflePlot::new().with_category("Only", 100.0, "#3498db");
    let svg = render(wp, "Single Category");
    write("waffle_single_category.svg", &svg);
    assert!(svg.contains("<rect"), "should have rect elements");
    // With a single category, all 100 cells should be that color
    assert!(svg.contains("#3498db"), "should use the category color");
}

// ─── Largest Remainder correctness ──────────────────────────────────────────

#[test]
fn test_waffle_largest_remainder_exact() {
    use kuva::render::render::waffle_largest_remainder;
    // Clean percentages: 50, 30, 20 over 10 cells
    let counts = waffle_largest_remainder(&[50.0, 30.0, 20.0], 10);
    assert_eq!(counts, vec![5, 3, 2]);
    assert_eq!(counts.iter().sum::<usize>(), 10);
}

#[test]
fn test_waffle_largest_remainder_rounding() {
    use kuva::render::render::waffle_largest_remainder;
    // 33.3% each → naive rounding gives 33+33+33=99; LR should fix to 100
    let counts = waffle_largest_remainder(&[1.0, 1.0, 1.0], 100);
    assert_eq!(counts.iter().sum::<usize>(), 100);
    // All three should be close to 33 or 34
    for &c in &counts {
        assert!(c >= 33 && c <= 34);
    }
}

#[test]
fn test_waffle_largest_remainder_single() {
    use kuva::render::render::waffle_largest_remainder;
    let counts = waffle_largest_remainder(&[100.0], 10);
    assert_eq!(counts, vec![10]);
}

#[test]
fn test_waffle_largest_remainder_empty() {
    use kuva::render::render::waffle_largest_remainder;
    let counts = waffle_largest_remainder(&[], 10);
    assert!(counts.is_empty());
}

#[test]
fn test_waffle_largest_remainder_zero_total() {
    use kuva::render::render::waffle_largest_remainder;
    let counts = waffle_largest_remainder(&[0.0, 0.0], 10);
    assert_eq!(counts, vec![0, 0]);
}

// ─── Grid options ────────────────────────────────────────────────────────────

#[test]
fn test_waffle_custom_grid() {
    let wp = WafflePlot::new()
        .with_grid(5, 20)
        .with_category("A", 70.0, "steelblue")
        .with_category("B", 30.0, "tomato");
    let svg = render(wp, "5×20 Grid");
    write("waffle_custom_grid.svg", &svg);
    assert!(svg.contains("<rect"), "should have rect elements");
}

#[test]
fn test_waffle_rows_cols_builders() {
    let wp = WafflePlot::new()
        .with_rows(4)
        .with_cols(25)
        .with_category("A", 50.0, "#2ecc71")
        .with_category("B", 50.0, "#e74c3c");
    assert_eq!(wp.rows, 4);
    assert_eq!(wp.cols, 25);
    assert_eq!(wp.total_cells(), 100);
    let svg = render(wp, "4×25 Grid");
    write("waffle_rows_cols.svg", &svg);
    assert!(svg.contains("<rect"));
}

// ─── Fill order ──────────────────────────────────────────────────────────────

#[test]
fn test_waffle_fill_order_bottom_left() {
    let wp = WafflePlot::new()
        .with_fill_order(FillOrder::RowMajorBottomLeft)
        .with_category("A", 60.0, "steelblue")
        .with_category("B", 40.0, "tomato");
    let svg = render(wp, "Bottom-Left Fill");
    write("waffle_fill_bottom_left.svg", &svg);
    assert!(svg.contains("<rect"));
}

#[test]
fn test_waffle_fill_order_col_major() {
    let wp = WafflePlot::new()
        .with_fill_order(FillOrder::ColMajorTopLeft)
        .with_category("A", 60.0, "steelblue")
        .with_category("B", 40.0, "tomato");
    let svg = render(wp, "Column-Major Fill");
    write("waffle_fill_col_major.svg", &svg);
    assert!(svg.contains("<rect"));
}

// ─── Cell shape ──────────────────────────────────────────────────────────────

#[test]
fn test_waffle_circle_shape() {
    let wp = WafflePlot::new()
        .with_shape(CellShape::Circle)
        .with_category("A", 70.0, "#9b59b6")
        .with_category("B", 30.0, "#f39c12");
    let svg = render(wp, "Circle Cells");
    write("waffle_circles.svg", &svg);
    // Circle cells use <circle> elements
    assert!(svg.contains("<circle"), "should have circle elements");
}

// ─── Gap control ─────────────────────────────────────────────────────────────

#[test]
fn test_waffle_no_gap() {
    let wp = WafflePlot::new()
        .with_gap(0.0)
        .with_category("A", 50.0, "steelblue")
        .with_category("B", 50.0, "tomato");
    let svg = render(wp, "No Gap");
    write("waffle_no_gap.svg", &svg);
    assert!(svg.contains("<rect"));
}

#[test]
fn test_waffle_large_gap() {
    let wp = WafflePlot::new()
        .with_gap(0.3)
        .with_category("A", 50.0, "steelblue")
        .with_category("B", 50.0, "tomato");
    let svg = render(wp, "Large Gap");
    write("waffle_large_gap.svg", &svg);
    assert!(svg.contains("<rect"));
}

// ─── Legend ──────────────────────────────────────────────────────────────────

#[test]
fn test_waffle_legend() {
    let wp = WafflePlot::new()
        .with_category("Treated", 45.0, "#3498db")
        .with_category("Partial", 30.0, "#f1c40f")
        .with_category("Untreated", 25.0, "#e74c3c")
        .with_legend("Status");
    let svg = render(wp, "With Legend");
    write("waffle_legend.svg", &svg);
    assert!(svg.contains("Treated"), "legend should contain 'Treated'");
    assert!(svg.contains("Partial"), "legend should contain 'Partial'");
    assert!(
        svg.contains("Untreated"),
        "legend should contain 'Untreated'"
    );
}

#[test]
fn test_waffle_legend_with_percents() {
    let wp = WafflePlot::new()
        .with_category("Yes", 75.0, "steelblue")
        .with_category("No", 25.0, "tomato")
        .with_legend("Response")
        .with_show_percents();
    let svg = render(wp, "Legend + Percents");
    write("waffle_legend_percents.svg", &svg);
    assert!(svg.contains("75.0%"), "legend should show 75.0%");
    assert!(svg.contains("25.0%"), "legend should show 25.0%");
}

#[test]
fn test_waffle_legend_with_counts() {
    let wp = WafflePlot::new()
        .with_category("A", 40.0, "steelblue")
        .with_category("B", 60.0, "tomato")
        .with_legend("Category")
        .with_show_counts();
    let svg = render(wp, "Legend + Counts");
    write("waffle_legend_counts.svg", &svg);
    assert!(
        svg.contains("40 cells"),
        "legend should show cell count for A"
    );
    assert!(
        svg.contains("60 cells"),
        "legend should show cell count for B"
    );
}

#[test]
fn test_waffle_legend_percents_and_counts() {
    let wp = WafflePlot::new()
        .with_category("Alpha", 50.0, "#2ecc71")
        .with_category("Beta", 50.0, "#e74c3c")
        .with_legend("Group")
        .with_show_percents()
        .with_show_counts();
    let svg = render(wp, "Legend + Both");
    write("waffle_legend_both.svg", &svg);
    assert!(svg.contains("50 cells"), "should show cell count");
    assert!(svg.contains("50.0%"), "should show percentage");
}

// ─── Unit label ──────────────────────────────────────────────────────────────

#[test]
fn test_waffle_unit_label() {
    let wp = WafflePlot::new()
        .with_category("Vaccinated", 68.0, "#27ae60")
        .with_category("Unvaccinated", 32.0, "#e74c3c")
        .with_unit_label("■ = 1% of population");
    let svg = render(wp, "With Unit Label");
    write("waffle_unit_label.svg", &svg);
    assert!(
        svg.contains("1% of population"),
        "should include unit annotation"
    );
}

// ─── render_waffle convenience ───────────────────────────────────────────────

#[test]
fn test_render_waffle_convenience() {
    let wp = WafflePlot::new()
        .with_category("A", 60.0, "steelblue")
        .with_category("B", 40.0, "tomato");
    let layout =
        Layout::auto_from_plots(&[Plot::Waffle(wp.clone())]).with_title("Convenience render");
    let scene = render_waffle(wp, layout);
    let svg = SvgBackend.render_scene(&scene);
    write("waffle_render_convenience.svg", &svg);
    assert!(svg.contains("<rect"));
}

// ─── Many categories ─────────────────────────────────────────────────────────

#[test]
fn test_waffle_many_categories() {
    let wp = WafflePlot::new()
        .with_categories([
            ("A", 20.0, "#e74c3c"),
            ("B", 18.0, "#e67e22"),
            ("C", 15.0, "#f1c40f"),
            ("D", 14.0, "#2ecc71"),
            ("E", 12.0, "#1abc9c"),
            ("F", 11.0, "#3498db"),
            ("G", 6.0, "#9b59b6"),
            ("H", 4.0, "#34495e"),
        ])
        .with_legend("Category")
        .with_show_percents();
    let svg = render(wp, "Many Categories");
    write("waffle_many_categories.svg", &svg);
    assert!(svg.contains("<rect"));
    assert!(svg.contains("A"), "legend should contain category A");
}

// ─── Full-feature showcase tests ─────────────────────────────────────────────

/// Public health vaccination coverage — classic waffle use case.
/// Demonstrates: circles, bottom-left fill, unit label, legend with percents.
#[test]
fn test_waffle_vaccination_showcase() {
    let wp = WafflePlot::new()
        .with_category("Fully vaccinated", 62.0, "#27ae60")
        .with_category("Partially vaccinated", 13.0, "#f39c12")
        .with_category("Unvaccinated", 25.0, "#e74c3c")
        .with_shape(CellShape::Circle)
        .with_fill_order(FillOrder::RowMajorBottomLeft)
        .with_gap(0.15)
        .with_legend("Vaccination status")
        .with_show_percents()
        .with_unit_label("■ = 1% of adult population");
    let svg = render(wp, "COVID-19 Vaccination Coverage");
    write("waffle_vaccination_showcase.svg", &svg);
    assert!(svg.contains("<circle"), "should use circle cells");
    assert!(
        svg.contains("Fully vaccinated"),
        "legend should contain 'Fully vaccinated'"
    );
    assert!(svg.contains("62.0%"), "legend should show 62.0%");
    assert!(svg.contains("25.0%"), "legend should show 25.0%");
    assert!(
        svg.contains("adult population"),
        "unit label should be present"
    );
}

/// Budget allocation — demonstrates rectangular cells, column-major fill,
/// many categories, show_counts in legend.
#[test]
fn test_waffle_budget_showcase() {
    let wp = WafflePlot::new()
        .with_categories([
            ("Healthcare", 32.0, "#3498db"),
            ("Education", 22.0, "#2ecc71"),
            ("Defence", 16.0, "#e74c3c"),
            ("Transport", 12.0, "#f39c12"),
            ("Social care", 9.0, "#9b59b6"),
            ("Other", 9.0, "#95a5a6"),
        ])
        .with_fill_order(FillOrder::ColMajorTopLeft)
        .with_gap(0.08)
        .with_legend("Budget allocation")
        .with_show_percents()
        .with_show_counts()
        .with_unit_label("■ = 1% of public spending");
    let svg = render(wp, "Government Budget Allocation");
    write("waffle_budget_showcase.svg", &svg);
    assert!(svg.contains("<rect"), "should have rect elements");
    assert!(
        svg.contains("Healthcare"),
        "legend should contain Healthcare"
    );
    assert!(svg.contains("Education"), "legend should contain Education");
    assert!(
        svg.contains("32.0%"),
        "legend should show Healthcare percentage"
    );
    assert!(
        svg.contains("1% of public spending"),
        "unit label should be present"
    );
    // Cell counts: with LR, 32+22+16+12+9+9 = 100, so all exactly filled
    assert!(
        svg.contains("32 cells"),
        "legend should show Healthcare cell count"
    );
}

/// Survey results — 5×20 wide grid to match landscape aspect ratio.
/// Demonstrates: with_grid, empty_color override, no-gap cells.
#[test]
fn test_waffle_survey_wide_grid() {
    let wp = WafflePlot::new()
        .with_grid(5, 20)
        .with_categories([
            ("Strongly agree", 28.0, "#1a5276"),
            ("Agree", 35.0, "#2980b9"),
            ("Neutral", 17.0, "#aab7b8"),
            ("Disagree", 12.0, "#e67e22"),
            ("Strongly disagree", 8.0, "#c0392b"),
        ])
        .with_gap(0.05)
        .with_empty_color("#f2f3f4")
        .with_legend("Agreement")
        .with_show_percents();
    let svg = render(wp, "Survey: \"The product meets my needs\"");
    write("waffle_survey_wide.svg", &svg);
    assert!(svg.contains("<rect"), "should have rect elements");
    assert!(
        svg.contains("Strongly agree"),
        "legend should contain Strongly agree"
    );
    assert!(svg.contains("28.0%"), "legend should show 28.0%");
    assert!(svg.contains("35.0%"), "legend should show 35.0%");
    // 5×20 = 100 cells, same as 10×10
    let wp2 = WafflePlot::new().with_grid(5, 20);
    assert_eq!(wp2.total_cells(), 100);
}

/// Small multiples via Figure — two waffle charts side by side.
/// Demonstrates the primary use case for Figure + WafflePlot.
#[test]
fn test_waffle_figure_small_multiples() {
    use kuva::backend::svg::SvgBackend;
    use kuva::render::figure::{Figure, LabelConfig};

    let waffle_2020 = WafflePlot::new()
        .with_category("Renewable", 29.0, "#27ae60")
        .with_category("Nuclear", 19.0, "#8e44ad")
        .with_category("Gas", 39.0, "#e67e22")
        .with_category("Coal", 13.0, "#7f8c8d")
        .with_legend("Source");

    let waffle_2023 = WafflePlot::new()
        .with_category("Renewable", 42.0, "#27ae60")
        .with_category("Nuclear", 17.0, "#8e44ad")
        .with_category("Gas", 35.0, "#e67e22")
        .with_category("Coal", 6.0, "#7f8c8d")
        .with_legend("Source");

    let fig = Figure::new(1, 2)
        .with_plots(vec![
            vec![Plot::Waffle(waffle_2020)],
            vec![Plot::Waffle(waffle_2023)],
        ])
        .with_labels_custom(vec!["2020", "2023"], LabelConfig::default())
        .with_title("UK Electricity Generation Mix");

    let svg = SvgBackend.render_scene(&fig.render());
    write("waffle_figure_small_multiples.svg", &svg);
    assert!(svg.contains("2020"), "should contain first panel label");
    assert!(svg.contains("2023"), "should contain second panel label");
    assert!(svg.contains("Renewable"), "should contain category label");
    assert!(svg.contains("<rect"), "should have rect elements");
}

/// Absolute count mode with unit label — clinical trial showing actual counts.
#[test]
fn test_waffle_clinical_trial_showcase() {
    // 270 responders, 130 non-responders; 1 cell = 4 patients (100 cells total)
    let wp = WafflePlot::new()
        .with_categories([
            ("Responders", 270.0, "#2980b9"),
            ("Non-responders", 130.0, "#e74c3c"),
        ])
        .with_shape(CellShape::Circle)
        .with_gap(0.12)
        .with_legend("Outcome")
        .with_show_percents()
        .with_show_counts()
        .with_unit_label("● = 4 patients  (n = 400 total)");
    let svg = render(wp, "Clinical Trial Response Rate");
    write("waffle_clinical_trial.svg", &svg);
    assert!(svg.contains("<circle"), "should use circle cells");
    assert!(
        svg.contains("Responders"),
        "legend should contain Responders"
    );
    assert!(svg.contains("67.5%"), "legend should show response rate");
    // 270/400 = 67.5%, 130/400 = 32.5%; LR over 100 cells: 68 and 32
    assert!(
        svg.contains("68 cells"),
        "LR should give 68 cells to responders"
    );
    assert!(
        svg.contains("32 cells"),
        "LR should give 32 cells to non-responders"
    );
    assert!(svg.contains("4 patients"), "unit label should be present");
}
