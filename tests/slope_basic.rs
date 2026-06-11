mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::slope::{SlopePlot, SlopeValueFormat};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use std::fs;

fn write_svg(name: &str, plots: Vec<Plot>, layout: Layout) -> String {
    fs::create_dir_all("test_outputs").unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output(format!("test_outputs/{name}.svg"), &svg).unwrap();
    assert!(svg.contains("<svg"));
    svg
}

fn countries() -> Vec<(&'static str, f64, f64)> {
    vec![
        ("Germany", 68.2, 71.5),
        ("France", 70.1, 68.9),
        ("Italy", 65.3, 69.1),
        ("Spain", 72.4, 74.8),
        ("Poland", 58.6, 63.2),
        ("Netherlands", 74.3, 76.1),
    ]
}

#[test]
fn test_slope_basic() {
    let mut sp = SlopePlot::new()
        .with_before_label("2015")
        .with_after_label("2023");
    for (label, before, after) in countries() {
        sp = sp.with_point(label, before, after);
    }
    let plots = vec![Plot::Slope(sp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Employment Rate");
    let svg = write_svg("slope_basic", plots, layout);
    assert!(svg.contains("<path") || svg.contains("<line"));
    assert!(svg.contains("<circle") || svg.contains("circle"));
}

#[test]
fn test_slope_show_values() {
    let mut sp = SlopePlot::new()
        .with_before_label("2015")
        .with_after_label("2023")
        .with_values(true);
    for (label, before, after) in countries() {
        sp = sp.with_point(label, before, after);
    }
    let plots = vec![Plot::Slope(sp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Employment Rate (with values)")
        .with_x_label("Employment rate (%)");
    let svg = write_svg("slope_show_values", plots, layout);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_slope_uniform_color() {
    let mut sp = SlopePlot::new()
        .with_direction_colors(false)
        .with_color("steelblue");
    for (label, before, after) in countries() {
        sp = sp.with_point(label, before, after);
    }
    let plots = vec![Plot::Slope(sp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Employment Rate (uniform color)");
    let svg = write_svg("slope_uniform_color", plots, layout);
    assert!(svg.contains("steelblue") || svg.contains("#"));
}

#[test]
fn test_slope_group_colors() {
    let colors = vec![
        "#e41a1c", "#377eb8", "#4daf4a", "#984ea3", "#ff7f00", "#a65628",
    ];
    let mut sp = SlopePlot::new().with_group_colors(colors.iter().copied());
    for (label, before, after) in countries() {
        sp = sp.with_point(label, before, after);
    }
    let plots = vec![Plot::Slope(sp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Employment Rate (per-group colors)");
    let svg = write_svg("slope_group_colors", plots, layout);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_slope_no_legend() {
    let mut sp = SlopePlot::new();
    for (label, before, after) in countries() {
        sp = sp.with_point(label, before, after);
    }
    // No .with_legend() call
    let plots = vec![Plot::Slope(sp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Employment Rate (no legend)");
    let svg = write_svg("slope_no_legend", plots, layout);
    assert!(svg.contains("<svg"));
    // Legend entries should not be present
    assert!(!svg.contains("Increase"));
}

#[test]
fn test_slope_with_legend() {
    let mut sp = SlopePlot::new().with_legend("Direction");
    for (label, before, after) in countries() {
        sp = sp.with_point(label, before, after);
    }
    let plots = vec![Plot::Slope(sp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Employment Rate (legend)");
    let svg = write_svg("slope_with_legend", plots, layout);
    // Direction color legend should have Increase and Decrease entries
    assert!(
        svg.contains("Increase"),
        "Expected 'Increase' legend entry in SVG"
    );
    assert!(
        svg.contains("Decrease"),
        "Expected 'Decrease' legend entry in SVG"
    );
}

#[test]
fn test_slope_integer_format() {
    let data = vec![
        ("A", 10.0_f64, 15.0_f64),
        ("B", 20.0_f64, 18.0_f64),
        ("C", 30.0_f64, 35.0_f64),
    ];
    let mut sp = SlopePlot::new()
        .with_values(true)
        .with_value_format(SlopeValueFormat::Integer);
    for (label, before, after) in data {
        sp = sp.with_point(label, before, after);
    }
    let plots = vec![Plot::Slope(sp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Integer format values");
    let svg = write_svg("slope_integer_format", plots, layout);
    assert!(svg.contains("<svg"));
}

#[test]
fn test_slope_gene_expression() {
    let genes = vec![
        ("BRCA1", 4.2_f64, 7.8_f64),
        ("TP53", 6.1, 5.4),
        ("MYC", 3.3, 8.9),
        ("EGFR", 7.5, 6.2),
        ("VEGFA", 2.8, 5.1),
        ("CDKN2A", 5.9, 4.3),
        ("KRAS", 8.1, 7.6),
        ("PIK3CA", 3.6, 6.7),
    ];
    let mut sp = SlopePlot::new()
        .with_before_label("Control")
        .with_after_label("Treatment")
        .with_values(true);
    for (g, b, a) in genes {
        sp = sp.with_point(g, b, a);
    }
    let plots = vec![Plot::Slope(sp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Gene Expression: Control vs Treatment")
        .with_x_label("log2 Expression");
    write_svg("slope_gene_expression", plots, layout);
}
