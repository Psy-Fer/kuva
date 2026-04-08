use kuva::plot::ScatterPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::backend::svg::SvgBackend;
use kuva::plot::legend::LegendPosition;
use std::fs;

fn write_svg(name: &str, plots: Vec<Plot>, layout: Layout) -> String {
    fs::create_dir_all("test_outputs").unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("test_outputs/{name}.svg"), &svg).unwrap();
    assert!(svg.contains("<svg"));
    svg
}

fn make_scatter() -> (Vec<Plot>, Layout) {
    let sp = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0_f64), (2.0, 3.5), (3.0, 5.0), (4.0, 6.5), (5.0, 8.0)]);
    let plots = vec![Plot::Scatter(sp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Scatter")
        .with_x_label("X")
        .with_y_label("Y");
    (plots, layout)
}

#[test]
fn test_stats_box_scatter() {
    let (plots, layout) = make_scatter();
    let layout = layout
        .with_stats_box(vec!["R² = 0.992", "p < 0.0001", "n = 5"]);
    let svg = write_svg("stats_box_scatter", plots, layout);
    assert!(svg.contains("R²"));
    assert!(svg.contains("0.992"));
    assert!(svg.contains("p &lt; 0.0001") || svg.contains("p < 0.0001"));
    assert!(svg.contains("n = 5"));
}

#[test]
fn test_stats_box_with_title() {
    let (plots, layout) = make_scatter();
    let layout = layout
        .with_stats_title("Statistics")
        .with_stats_box(vec!["R² = 0.847", "AUC = 0.923"]);
    let svg = write_svg("stats_box_with_title", plots, layout);
    assert!(svg.contains("Statistics"));
    assert!(svg.contains("R²"));
    assert!(svg.contains("AUC = 0.923"));
}

#[test]
fn test_stats_box_position_inside_bottom_right() {
    let (plots, layout) = make_scatter();
    let layout = layout
        .with_stats_box_at(
            LegendPosition::InsideBottomRight,
            vec!["R² = 0.91", "RMSE = 0.32"],
        );
    let svg = write_svg("stats_box_position", plots, layout);
    assert!(svg.contains("RMSE = 0.32"));
}

#[test]
fn test_stats_box_no_border() {
    let (plots, layout) = make_scatter();
    let layout = layout
        .with_stats_box(vec!["AUC = 0.923"])
        .with_stats_box_border(false);
    let svg = write_svg("stats_box_no_border", plots, layout);
    assert!(svg.contains("AUC = 0.923"));
}

#[test]
fn test_stats_box_with_legend_no_collision() {
    // Both stats box and legend at InsideTopLeft — stats should appear, then legend below.
    let sp = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0_f64), (2.0, 3.5)])
        .with_legend("Group A");
    let plots = vec![Plot::Scatter(sp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Scatter with legend and stats")
        .with_legend_position(LegendPosition::InsideTopLeft)
        .with_stats_box_at(
            LegendPosition::InsideTopLeft,
            vec!["R² = 0.99"],
        );
    let svg = write_svg("stats_box_with_legend_no_collision", plots, layout);
    // Both should appear
    assert!(svg.contains("R²"));
    assert!(svg.contains("Group A"));
}

#[test]
fn test_stats_box_single_entry() {
    let (plots, layout) = make_scatter();
    let layout = layout.with_stats_entry("p = 0.023");
    let svg = write_svg("stats_box_single_entry", plots, layout);
    assert!(svg.contains("p = 0.023"));
}

#[test]
fn test_stats_box_survival() {
    use kuva::plot::SurvivalPlot;
    let sp = SurvivalPlot::new()
        .with_group("Control",
            vec![2.0, 4.0, 6.0, 8.0, 10.0],
            vec![true, true, false, true, false])
        .with_group("Treatment",
            vec![3.0, 5.0, 7.0, 9.0, 12.0],
            vec![true, false, true, false, true]);
    let plots = vec![Plot::Survival(sp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Survival")
        .with_x_label("Time (months)")
        .with_y_label("Survival probability")
        .with_stats_box(vec!["log-rank p = 0.041", "HR = 0.62 [0.31, 1.24]"]);
    let svg = write_svg("stats_box_survival", plots, layout);
    assert!(svg.contains("log-rank p = 0.041"));
    assert!(svg.contains("HR = 0.62"));
}
