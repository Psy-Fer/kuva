mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::LollipopPlot;
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

#[test]
fn test_lollipop_basic() {
    let lp = LollipopPlot::new()
        .with_point(1.0, 3.0)
        .with_point(2.0, 7.0)
        .with_point(3.0, 2.0)
        .with_point(4.0, 5.0)
        .with_point(5.0, 4.0);
    let plots = vec![Plot::Lollipop(lp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Lollipop Basic")
        .with_x_label("Position")
        .with_y_label("Value");
    let svg = write_svg("lollipop_basic", plots, layout);
    assert!(svg.contains("<circle"));
    assert!(svg.contains("<line"));
}

#[test]
fn test_lollipop_baseline_visible() {
    let lp = LollipopPlot::new()
        .with_point(1.0, 3.0)
        .with_point(2.0, 5.0)
        .with_baseline(0.0)
        .with_baseline_color("#333333")
        .with_baseline_dash("4,3");
    let plots = vec![Plot::Lollipop(lp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Baseline visible");
    write_svg("lollipop_baseline", plots, layout);
}

#[test]
fn test_lollipop_baseline_hidden() {
    let lp = LollipopPlot::new()
        .with_point(1.0, 3.0)
        .with_point(2.0, 5.0)
        .with_show_baseline(false);
    let plots = vec![Plot::Lollipop(lp)];
    let layout = Layout::auto_from_plots(&plots).with_title("No baseline");
    write_svg("lollipop_no_baseline", plots, layout);
}

#[test]
fn test_lollipop_domains() {
    let lp = LollipopPlot::new()
        .with_points([
            (10.0, 3.0),
            (25.0, 7.0),
            (40.0, 2.0),
            (60.0, 5.0),
            (80.0, 4.0),
        ])
        .with_domain(1.0, 36.0, Some("N-term"), "steelblue")
        .with_domain(36.0, 70.0, Some("Kinase"), "tomato")
        .with_domain(70.0, 100.0, Some("SH2"), "seagreen")
        .with_domain_height(0.8);
    let plots = vec![Plot::Lollipop(lp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Mutation Landscape - joined domains")
        .with_x_label("Position (aa)")
        .with_y_label("Count");
    let svg = write_svg("lollipop_domains", plots, layout);
    // Background rect + 3 domain rects = at least 4 rects
    let rect_count = svg.matches("<rect").count();
    assert!(rect_count >= 4, "expected ≥4 rects, got {rect_count}");
}

#[test]
fn test_lollipop_domain_labels() {
    let lp = LollipopPlot::new()
        .with_points([(20.0, 4.0), (70.0, 6.0)])
        .with_domain(1.0, 50.0, Some("Kinase"), "steelblue")
        .with_domain(51.0, 100.0, Some("SH2"), "tomato");
    let plots = vec![Plot::Lollipop(lp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Domain labels");
    let svg = write_svg("lollipop_domain_labels", plots, layout);
    assert!(svg.contains("Kinase"));
    assert!(svg.contains("SH2"));
}

#[test]
fn test_lollipop_point_labels() {
    let lp = LollipopPlot::new()
        .with_labeled_point(10.0, 5.0, "TP53")
        .with_labeled_point(30.0, 8.0, "KRAS")
        .with_labeled_point(50.0, 3.0, "BRCA1");
    let plots = vec![Plot::Lollipop(lp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Point labels");
    let svg = write_svg("lollipop_point_labels", plots, layout);
    assert!(svg.contains("TP53"));
    assert!(svg.contains("KRAS"));
    assert!(svg.contains("BRCA1"));
}

#[test]
fn test_lollipop_per_point_colors() {
    let lp = LollipopPlot::new()
        .with_colored_point(1.0, 3.0, "tomato")
        .with_colored_point(2.0, 5.0, "seagreen")
        .with_colored_point(3.0, 2.0, "steelblue");
    let plots = vec![Plot::Lollipop(lp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Per-point colors");
    let svg = write_svg("lollipop_colors", plots, layout);
    // tomato = #ff6347 or similar; just check SVG isn't trivially wrong
    assert!(svg.contains("<circle"));
}

#[test]
fn test_lollipop_negative_values() {
    let lp = LollipopPlot::new()
        .with_point(1.0, -3.0)
        .with_point(2.0, -1.0)
        .with_point(3.0, -5.0)
        .with_baseline(0.0);
    let plots = vec![Plot::Lollipop(lp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Negative values");
    write_svg("lollipop_negative", plots, layout);
}

#[test]
fn test_lollipop_mixed_signs() {
    let lp = LollipopPlot::new()
        .with_point(1.0, 2.5)
        .with_point(2.0, -1.5)
        .with_point(3.0, 3.0)
        .with_point(4.0, -2.0)
        .with_point(5.0, 1.0)
        .with_baseline(0.0);
    let plots = vec![Plot::Lollipop(lp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Mixed signs (log2FC style)")
        .with_y_label("log2 FC");
    write_svg("lollipop_mixed", plots, layout);
}

#[test]
fn test_lollipop_single_point() {
    let lp = LollipopPlot::new().with_labeled_point(5.0, 3.0, "Only");
    let plots = vec![Plot::Lollipop(lp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Single point");
    write_svg("lollipop_single", plots, layout);
}

#[test]
fn test_lollipop_empty_no_panic() {
    let lp = LollipopPlot::new();
    // bounds() returns None for empty plot
    assert!(Plot::Lollipop(lp).bounds().is_none());
}

#[test]
fn test_lollipop_legend() {
    let lp = LollipopPlot::new()
        .with_points([(1.0, 3.0), (2.0, 5.0), (3.0, 2.0)])
        .with_legend("Mutations");
    let plots = vec![Plot::Lollipop(lp)];
    let layout = Layout::auto_from_plots(&plots).with_title("With legend");
    let svg = write_svg("lollipop_legend", plots, layout);
    assert!(svg.contains("Mutations"));
}

#[test]
fn test_lollipop_custom_style() {
    let lp = LollipopPlot::new()
        .with_points([(1.0, 3.0), (2.0, 5.0), (3.0, 2.0)])
        .with_dot_radius(8.0)
        .with_stem_width(2.5)
        .with_dot_stroke("white")
        .with_dot_stroke_width(1.5)
        .with_color("tomato");
    let plots = vec![Plot::Lollipop(lp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Custom style");
    write_svg("lollipop_custom_style", plots, layout);
}

#[test]
fn test_lollipop_labeled_colored_point() {
    let lp = LollipopPlot::new()
        .with_labeled_colored_point(5.0, 7.0, "TP53", "tomato")
        .with_labeled_colored_point(15.0, 4.0, "KRAS", "steelblue");
    let plots = vec![Plot::Lollipop(lp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Labeled + colored");
    let svg = write_svg("lollipop_labeled_colored", plots, layout);
    assert!(svg.contains("TP53"));
    assert!(svg.contains("KRAS"));
}

#[test]
fn test_lollipop_mutation_landscape() {
    // Realistic protein mutation landscape
    let mutations = vec![
        (12.0, 3.0),
        (25.0, 1.0),
        (41.0, 7.0),
        (55.0, 2.0),
        (68.0, 4.0),
        (82.0, 1.0),
        (97.0, 5.0),
        (110.0, 2.0),
        (124.0, 8.0),
        (138.0, 3.0),
        (150.0, 1.0),
        (163.0, 6.0),
    ];
    let lp = LollipopPlot::new()
        .with_points(mutations)
        .with_colored_point(41.0, 7.0, "tomato") // hotspot
        .with_labeled_colored_point(124.0, 8.0, "R248W", "tomato") // named hotspot
        .with_domain(1.0, 60.0, Some("N-term"), "#4e79a7")
        .with_domain(61.0, 120.0, Some("Kinase"), "#f28e2b")
        .with_domain(121.0, 180.0, Some("C-term"), "#59a14f")
        .with_domain_height(1.2)
        .with_color("steelblue")
        .with_dot_radius(4.5)
        .with_stem_width(1.5);
    let plots = vec![Plot::Lollipop(lp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("TP53 Mutation Landscape")
        .with_x_label("Amino acid position")
        .with_y_label("Mutation count")
        .with_width(600.0)
        .with_height(350.0)
        .with_show_grid(false);
    let svg = write_svg("lollipop_mutation_landscape", plots, layout);
    assert!(svg.contains("R248W"));
    assert!(svg.contains("N-term"));
}
