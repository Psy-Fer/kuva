use visus::plot::{VolcanoPlot, LabelStyle};
use visus::backend::svg::SvgBackend;
use visus::render::render::{render_multiple, render_volcano};
use visus::render::layout::Layout;
use visus::render::plots::Plot;

fn make_test_data() -> Vec<(&'static str, f64, f64)> {
    vec![
        // Up-regulated (log2fc >= 1.0, pvalue <= 0.05)
        ("BRCA1",   2.5,  0.001),
        ("TP53",    1.8,  0.01),
        ("EGFR",    3.2,  0.0001),
        ("MYC",     1.5,  0.03),
        ("KRAS",    2.1,  0.005),
        ("CDK4",    1.2,  0.04),
        ("PTEN",    2.8,  0.002),
        ("RB1",     1.9,  0.008),
        ("AKT1",    3.5,  0.00005),
        ("VEGFA",   2.3,  0.003),
        // Down-regulated (log2fc <= -1.0, pvalue <= 0.05)
        ("CDKN2A", -2.3,  0.002),
        ("SMAD4",  -1.9,  0.008),
        ("VHL",    -3.0,  0.0005),
        ("CASP3",  -1.6,  0.04),
        ("BCL2",   -2.7,  0.001),
        ("FAS",    -1.4,  0.035),
        ("PUMA",   -2.0,  0.007),
        ("BAX",    -1.7,  0.015),
        ("P21",    -3.2,  0.0002),
        ("MDM2",   -2.5,  0.003),
        // Not significant — low fold change
        ("GAPDH",   0.3,  0.5),
        ("ACTB",   -0.5,  0.3),
        ("TUBA1",   0.8,  0.1),
        ("HIST1",  -0.2,  0.7),
        ("RPL5",    0.6,  0.2),
        ("RPS6",   -0.9,  0.15),
        ("EEF1A",   0.1,  0.8),
        ("HNRNPA", -0.7,  0.4),
        ("SF3B1",   0.4,  0.6),
        ("SRSF1",  -0.3,  0.9),
        // NS — high fold change but not significant p-value
        ("GeneA",   1.5,  0.2),
        ("GeneB",  -1.1,  0.07),
        ("GeneC",   0.9,  0.12),
        ("GeneD",  -0.8,  0.08),
        ("GeneE",   1.3,  0.18),
    ]
}

#[test]
fn test_volcano_basic() {
    let vp = VolcanoPlot::new()
        .with_points(make_test_data());

    let plots = vec![Plot::Volcano(vp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Volcano Plot — Basic")
        .with_x_label("log2 Fold Change")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/volcano_basic.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // Dashed threshold lines should be present
    assert!(svg.contains("4 4"));
}

#[test]
fn test_volcano_labeled_nudge() {
    let vp = VolcanoPlot::new()
        .with_points(make_test_data())
        .with_label_top(10);

    let plots = vec![Plot::Volcano(vp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Volcano Plot — Nudge Labels")
        .with_x_label("log2 Fold Change")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/volcano_labeled_nudge.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // At least one gene name should appear
    assert!(svg.contains("EGFR") || svg.contains("AKT1") || svg.contains("P21"));
}

#[test]
fn test_volcano_labeled_exact() {
    let vp = VolcanoPlot::new()
        .with_points(make_test_data())
        .with_label_top(5)
        .with_label_style(LabelStyle::Exact);

    let plots = vec![Plot::Volcano(vp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Volcano Plot — Exact Labels")
        .with_x_label("log2 Fold Change")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/volcano_labeled_exact.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_volcano_labeled_arrow() {
    let vp = VolcanoPlot::new()
        .with_points(make_test_data())
        .with_label_top(8)
        .with_label_style(LabelStyle::Arrow { offset_x: 12.0, offset_y: 18.0 });

    let plots = vec![Plot::Volcano(vp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Volcano Plot — Arrow Labels")
        .with_x_label("log2 Fold Change")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/volcano_labeled_arrow.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // Leader lines drawn in gray
    assert!(svg.contains("#666666"));
}

#[test]
fn test_volcano_zero_pvalues() {
    // p=0.0 should be auto-capped at the minimum non-zero p-value
    let vp = VolcanoPlot::new()
        .with_point("ZeroP1", 4.0_f64, 0.0_f64)
        .with_point("ZeroP2", -4.0_f64, 0.0_f64)
        .with_points(make_test_data());

    let plots = vec![Plot::Volcano(vp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Volcano Plot — Zero p-values (auto-cap)")
        .with_x_label("log2 Fold Change")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/volcano_zero_pvalues.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_volcano_pvalue_floor() {
    let vp = VolcanoPlot::new()
        .with_points(make_test_data())
        .with_pvalue_floor(1e-10);

    let plots = vec![Plot::Volcano(vp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Volcano Plot — p-value floor 1e-10")
        .with_x_label("log2 Fold Change")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/volcano_pvalue_floor.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_volcano_legend() {
    let vp = VolcanoPlot::new()
        .with_points(make_test_data())
        .with_legend("Experiment 1");

    let plots = vec![Plot::Volcano(vp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Volcano Plot — Legend")
        .with_x_label("log2 Fold Change")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/volcano_legend.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    // Legend entries should appear
    assert!(svg.contains(">Up<") || svg.contains(">Up "));
    assert!(svg.contains(">Down<") || svg.contains(">Down "));
    assert!(svg.contains(">NS<") || svg.contains(">NS "));
}

#[test]
fn test_volcano_custom_colors() {
    let vp = VolcanoPlot::new()
        .with_points(make_test_data())
        .with_color_up("darkorange")
        .with_color_down("mediumpurple")
        .with_color_ns("#cccccc");

    let plots = vec![Plot::Volcano(vp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Volcano Plot — Custom Colors")
        .with_x_label("log2 Fold Change")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/volcano_custom_colors.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("darkorange"));
    assert!(svg.contains("mediumpurple"));
}

#[test]
fn test_volcano_custom_thresholds() {
    let vp = VolcanoPlot::new()
        .with_points(make_test_data())
        .with_fc_cutoff(2.0)
        .with_p_cutoff(0.01)
        .with_label_top(5);

    let plots = vec![Plot::Volcano(vp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Volcano Plot — fc>2, p<0.01")
        .with_x_label("log2 Fold Change")
        .with_y_label("-log10(p-value)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/volcano_custom_thresholds.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_volcano_render_volcano_fn() {
    let vp = VolcanoPlot::new()
        .with_points(make_test_data())
        .with_label_top(6);

    let layout = Layout::auto_from_plots(&[Plot::Volcano(
        VolcanoPlot::new().with_points(make_test_data()),
    )])
    .with_title("Volcano Plot — render_volcano()")
    .with_x_label("log2 Fold Change")
    .with_y_label("-log10(p-value)");

    let scene = render_volcano(&vp, &layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/volcano_render_fn.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}
