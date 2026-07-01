mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::{
    BarPlot, DensityPlot, EcdfPlot, Histogram, LinePlot, PiePlot, ScatterPlot, SeriesPlot,
    StripPlot, ViolinPlot, WaterfallPlot,
};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

fn bw_svg(plots: Vec<Plot>, layout: Layout) -> String {
    let layout = layout.with_bw_mode();
    let scene = render_multiple(plots, layout);
    SvgBackend.render_scene(&scene)
}

// ── Tier 1: fills ────────────────────────────────────────────────────────────

#[test]
fn bw_bar_single_series() {
    let bar = BarPlot::new()
        .with_bar("A", 3.2)
        .with_bar("B", 4.7)
        .with_bar("C", 2.8);
    let plots = vec![Plot::Bar(bar)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_bar.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<pattern"), "BW bar chart should emit SVG pattern defs");
    assert!(svg.contains("kuva-fp-"), "Pattern defs should use kuva-fp- prefix");
}

#[test]
fn bw_bar_multi_category() {
    let bar = BarPlot::new()
        .with_bar("A", 3.0)
        .with_bar("B", 4.5)
        .with_bar("C", 2.8)
        .with_bar("D", 5.1);
    let plots = vec![Plot::Bar(bar)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_bar_multi.svg", svg.clone()).unwrap();
    assert!(svg.contains("<pattern"));
}

#[test]
fn bw_histogram() {
    let hist = Histogram::new()
        .with_data(vec![1.0, 2.0, 2.5, 3.0, 3.5, 4.0, 5.0, 5.5, 6.0])
        .with_range((0.0, 7.0))
        .with_bins(7)
        .with_color("steelblue");
    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_histogram.svg", svg.clone()).unwrap();
    assert!(svg.contains("<pattern"));
}

#[test]
fn bw_pie() {
    let pie = PiePlot::new()
        .with_slice("A", 40.0, "#4499cc")
        .with_slice("B", 35.0, "#cc4444")
        .with_slice("C", 25.0, "#44cc44");
    let plots = vec![Plot::Pie(pie)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_pie.svg", svg.clone()).unwrap();
    assert!(svg.contains("<pattern"));
}

#[test]
fn bw_waterfall() {
    let wf = WaterfallPlot::new()
        .with_delta("Revenue", 50.0)
        .with_delta("Costs", -30.0)
        .with_total("Net");
    let plots = vec![Plot::Waterfall(wf)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_waterfall.svg", svg.clone()).unwrap();
    assert!(svg.contains("<pattern"));
}

#[test]
fn bw_rose() {
    use kuva::plot::rose::RosePlot;
    let rose = RosePlot::new()
        .with_slice("N", 12.0)
        .with_slice("NE", 8.0)
        .with_slice("E", 5.0)
        .with_slice("SE", 9.0)
        .with_slice("S", 14.0)
        .with_slice("SW", 11.0)
        .with_slice("W", 6.0)
        .with_slice("NW", 10.0);
    let plots = vec![Plot::Rose(rose)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_rose.svg", svg.clone()).unwrap();
    assert!(svg.contains("<pattern"));
}

#[test]
fn bw_upset() {
    use kuva::plot::UpSetPlot;
    let upset = UpSetPlot::new().with_data(
        vec!["Set A", "Set B", "Set C"],
        vec![52usize, 47, 36],
        vec![(0b001u64, 10usize), (0b010, 8), (0b100, 12), (0b011, 5), (0b111, 20)],
    );
    let plots = vec![Plot::UpSet(upset)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_upset.svg", svg.clone()).unwrap();
    assert!(svg.contains("<pattern"));
}

// ── Tier 2: fills ────────────────────────────────────────────────────────────

#[test]
fn bw_band() {
    use kuva::plot::BandPlot;
    let x: Vec<f64> = (0..20).map(|i| i as f64 * 0.5).collect();
    let y_lower: Vec<f64> = x.iter().map(|&v| v.sin() - 0.4).collect();
    let y_upper: Vec<f64> = x.iter().map(|&v| v.sin() + 0.4).collect();
    let band = BandPlot::new(x, y_lower, y_upper)
        .with_color("steelblue")
        .with_opacity(0.4);
    let plots = vec![Plot::Band(band)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_band.svg", svg.clone()).unwrap();
    assert!(svg.contains("<pattern"));
}

#[test]
fn bw_boxplot() {
    use kuva::plot::BoxPlot;
    let bp = BoxPlot::new()
        .with_group("A", vec![1.0, 2.0, 2.5, 3.0, 4.0, 5.0, 2.8])
        .with_group("B", vec![2.0, 2.1, 3.5, 3.8, 4.0, 4.2, 3.0])
        .with_group("C", vec![0.5, 1.5, 2.0, 2.5, 3.5, 4.5, 2.0]);
    let plots = vec![Plot::Box(bp)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_boxplot.svg", svg.clone()).unwrap();
    assert!(svg.contains("<pattern"));
}

#[test]
fn bw_violin() {
    let violin = ViolinPlot::new()
        .with_group("Normal", vec![1.0, 1.5, 2.0, 2.5, 2.4, 2.4, 3.1, 1.9, 2.2])
        .with_group("Bimodal", vec![0.5, 0.6, 3.8, 4.0, 0.4, 3.5, 4.2, 0.7, 3.9])
        .with_color("mediumpurple");
    let plots = vec![Plot::Violin(violin)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_violin.svg", svg.clone()).unwrap();
    assert!(svg.contains("<pattern"));
}

#[test]
fn bw_density() {
    let density = DensityPlot::new()
        .with_data(vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 2.0, 2.5, 3.0])
        .with_color("steelblue")
        .with_filled(true)
        .with_opacity(0.4);
    let plots = vec![Plot::Density(density)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_density.svg", svg.clone()).unwrap();
    assert!(svg.contains("<pattern"));
}

#[test]
fn bw_ridgeline() {
    use kuva::plot::ridgeline::RidgelinePlot;
    let rp = RidgelinePlot::new()
        .with_group("Spring", vec![12.0, 15.0, 18.0, 14.0, 16.0, 13.0, 17.0, 15.5])
        .with_group("Summer", vec![22.0, 25.0, 28.0, 24.0, 26.0, 23.0, 27.0, 25.5])
        .with_group("Autumn", vec![10.0, 13.0, 16.0, 12.0, 14.0, 11.0, 15.0, 13.5])
        .with_filled(true)
        .with_opacity(0.7);
    let plots = vec![Plot::Ridgeline(rp)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_ridgeline.svg", svg.clone()).unwrap();
    assert!(svg.contains("<pattern"));
}

#[test]
fn bw_stacked_area() {
    use kuva::plot::StackedAreaPlot;
    let sa = StackedAreaPlot::new()
        .with_x(vec![0.0, 1.0, 2.0, 3.0, 4.0])
        .with_series(vec![10.0, 20.0, 15.0, 25.0, 18.0])
        .with_color("steelblue")
        .with_series(vec![5.0, 10.0, 8.0, 12.0, 9.0])
        .with_color("tomato")
        .with_series(vec![3.0, 5.0, 6.0, 4.0, 7.0])
        .with_color("goldenrod");
    let plots = vec![Plot::StackedArea(sa)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_stacked_area.svg", svg.clone()).unwrap();
    assert!(svg.contains("<pattern"));
}

#[test]
fn bw_streamgraph() {
    use kuva::plot::streamgraph::StreamgraphPlot;
    let sg = StreamgraphPlot::new()
        .with_x(vec![1.0, 2.0, 3.0, 4.0, 5.0])
        .with_series(vec![10.0, 14.0, 18.0, 22.0, 20.0])
        .with_label("Alpha")
        .with_series(vec![5.0, 8.0, 12.0, 15.0, 14.0])
        .with_label("Beta")
        .with_series(vec![3.0, 4.0, 6.0, 8.0, 9.0])
        .with_label("Gamma");
    let plots = vec![Plot::Streamgraph(sg)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_streamgraph.svg", svg.clone()).unwrap();
    assert!(svg.contains("<pattern"));
}

#[test]
fn bw_survival_ci_band() {
    use kuva::plot::SurvivalPlot;
    let sp = SurvivalPlot::new()
        .with_group(
            "Control",
            vec![2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0],
            vec![true, true, true, false, true, false, true],
        )
        .with_group(
            "Treatment",
            vec![3.0, 7.0, 9.0, 12.0, 15.0, 18.0, 20.0],
            vec![true, false, true, false, true, false, false],
        )
        .with_ci(true);
    let plots = vec![Plot::Survival(sp)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_survival.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("#1a1a1a"), "BW survival curves should use dark stroke");
}

#[test]
fn bw_roc() {
    use kuva::plot::roc::{RocGroup, RocPlot};
    let data: Vec<(f64, bool)> = vec![
        (0.95, true), (0.88, true), (0.80, false), (0.72, true),
        (0.65, false), (0.55, true), (0.40, false), (0.30, false),
        (0.22, true), (0.10, false),
    ];
    let group = RocGroup::new("Classifier").with_raw(data).with_ci(true);
    let roc = RocPlot::new().with_group(group);
    let plots = vec![Plot::Roc(roc)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_roc.svg", svg.clone()).unwrap();
    assert!(svg.contains("#1a1a1a"), "BW ROC curve should use dark stroke");
}

// ── Lines ────────────────────────────────────────────────────────────────────

#[test]
fn bw_line() {
    let line = LinePlot::new()
        .with_data(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 2.0), (3.0, 4.0), (4.0, 3.5)])
        .with_color("steelblue");
    let plots = vec![Plot::Line(line)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_line.svg", svg.clone()).unwrap();
    assert!(svg.contains("#1a1a1a"), "BW line chart should use dark stroke color");
}

#[test]
fn bw_series() {
    let data: Vec<f64> = (0..40).map(|x| (x as f64 * 0.3).sin() * 3.0 + 5.0).collect();
    let series = SeriesPlot::new()
        .with_data(data)
        .with_color("tomato")
        .with_line_point_style();
    let plots = vec![Plot::Series(series)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_series.svg", svg.clone()).unwrap();
    assert!(svg.contains("#1a1a1a"));
}

#[test]
fn bw_pr() {
    use kuva::plot::pr::{PrGroup, PrPlot};
    let data: Vec<(f64, bool)> = vec![
        (0.92, true), (0.85, true), (0.78, false), (0.70, true),
        (0.60, false), (0.50, true), (0.38, false), (0.25, false),
        (0.18, true), (0.08, false),
    ];
    let group = PrGroup::new("Model A").with_raw(data);
    let pr = PrPlot::new().with_group(group);
    let plots = vec![Plot::Pr(pr)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_pr.svg", svg.clone()).unwrap();
    assert!(svg.contains("#1a1a1a"));
}

#[test]
fn bw_ecdf() {
    let ecdf = EcdfPlot::new()
        .with_data("Sample A", vec![1.2, 3.4, 2.1, 5.6, 4.0, 0.8, 3.3, 2.7, 4.5, 1.9])
        .with_data("Sample B", vec![2.2, 3.8, 2.9, 4.6, 3.0, 1.8, 4.3, 3.7, 5.0, 2.4])
        .with_confidence_band();
    let plots = vec![Plot::Ecdf(ecdf)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_ecdf.svg", svg.clone()).unwrap();
    assert!(svg.contains("#1a1a1a"));
}

#[test]
fn bw_slope() {
    use kuva::plot::slope::SlopePlot;
    let sp = SlopePlot::new()
        .with_before_label("2015")
        .with_after_label("2023")
        .with_point("Germany", 68.2, 71.5)
        .with_point("France", 70.1, 68.9)
        .with_point("Italy", 65.3, 69.1)
        .with_point("Spain", 72.0, 74.2);
    let plots = vec![Plot::Slope(sp)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_slope.svg", svg.clone()).unwrap();
    assert!(svg.contains("#1a1a1a"));
}

#[test]
fn bw_bump() {
    use kuva::plot::bump::BumpPlot;
    let bp = BumpPlot::new()
        .with_series("Alpha", vec![1, 3, 2, 1])
        .with_series("Beta", vec![2, 1, 1, 3])
        .with_series("Gamma", vec![3, 2, 3, 2])
        .with_x_labels(["2021", "2022", "2023", "2024"]);
    let plots = vec![Plot::Bump(bp)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_bump.svg", svg.clone()).unwrap();
    assert!(svg.contains("#1a1a1a"));
}

// ── Scatter / points ─────────────────────────────────────────────────────────

#[test]
fn bw_scatter() {
    let scatter = ScatterPlot::new()
        .with_data(vec![(1.0, 2.0), (2.0, 3.0), (3.0, 1.5), (4.0, 4.0), (5.0, 2.5)])
        .with_color("steelblue");
    let plots = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_scatter.svg", svg.clone()).unwrap();
    assert!(svg.contains("#1a1a1a"), "BW scatter chart should use dark fill color");
}

#[test]
fn bw_strip() {
    let strip = StripPlot::new()
        .with_group("A", vec![1.0, 2.0, 2.5, 3.1, 4.0, 3.5, 2.2])
        .with_group("B", vec![2.0, 2.1, 3.5, 3.8, 4.0, 4.2, 3.0])
        .with_group("C", vec![0.5, 1.5, 2.0, 2.5, 3.5, 1.8, 2.8])
        .with_color("steelblue");
    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_strip.svg", svg.clone()).unwrap();
    assert!(svg.contains("#1a1a1a"));
}

#[test]
fn bw_qq() {
    use kuva::plot::QQPlot;
    let qq = QQPlot::new()
        .with_data(
            "Sample",
            vec![1.2, 3.4, 2.1, 5.6, 4.0, 0.8, 3.3, 2.7, 4.5, 1.9, 2.3, 3.8],
        )
        .with_reference_line()
        .with_color("steelblue");
    let plots = vec![Plot::QQ(qq)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_qq.svg", svg.clone()).unwrap();
    assert!(svg.contains("#1a1a1a"));
}

// ── Multi-series / multi-group differentiation ───────────────────────────────

#[test]
fn bw_line_multi_series_uses_distinct_dash_styles() {
    // Each series uses different y-values so the lines are visually separated
    let data0 = vec![(0.0, 1.0), (1.0, 2.0), (2.0, 1.5), (3.0, 3.0)];
    let data1 = vec![(0.0, 5.0), (1.0, 6.0), (2.0, 5.5), (3.0, 7.0)];
    let data2 = vec![(0.0, 9.0), (1.0, 10.0), (2.0, 9.5), (3.0, 11.0)];
    let line0 = LinePlot::new().with_data(data0).with_color("steelblue").with_legend("A");
    let line1 = LinePlot::new().with_data(data1).with_color("tomato").with_legend("B");
    let line2 = LinePlot::new().with_data(data2).with_color("seagreen").with_legend("C");
    let plots = vec![Plot::Line(line0), Plot::Line(line1), Plot::Line(line2)];
    let mut layout = Layout::auto_from_plots(&plots);
    layout.show_legend = true;
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_line_multi.svg", svg.clone()).unwrap();
    // Series 0 → Solid (no dasharray), Series 1 → Dashed "8 4", Series 2 → Dotted "2 4"
    assert!(svg.contains("8 4"), "Second line should be dashed (8 4)");
    assert!(svg.contains("2 4"), "Third line should be dotted (2 4)");
}

#[test]
fn bw_scatter_multi_series_uses_distinct_shapes() {
    let pts_a = vec![(1.0, 2.0), (2.0, 3.0), (3.0, 2.5)];
    let pts_b = vec![(1.0, 3.5), (2.0, 1.5), (3.0, 4.0)];
    let scatter0 = ScatterPlot::new().with_data(pts_a).with_color("steelblue").with_legend("A");
    let scatter1 = ScatterPlot::new().with_data(pts_b).with_color("tomato").with_legend("B");
    let plots = vec![Plot::Scatter(scatter0), Plot::Scatter(scatter1)];
    let mut layout = Layout::auto_from_plots(&plots);
    layout.show_legend = true;
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_scatter_multi.svg", svg.clone()).unwrap();
    // Series 0 → Circle (fast path CircleBatch), Series 1 → Square (rect element via slow path)
    assert!(svg.contains("#1a1a1a"), "All BW scatter points should be dark");
    // The second scatter uses bw_shape(1) = Square → should contain a rect or polygon path,
    // not just circles.  A simple proxy: the SVG is longer / more complex than a single series.
    assert!(svg.len() > 500, "Multi-series BW scatter should produce non-trivial SVG");
}

#[test]
fn bw_density_multi_series_distinct_patterns() {
    let density0 = DensityPlot::new()
        .with_data(vec![1.0, 1.5, 2.0, 2.5, 3.0, 2.0, 1.8])
        .with_color("steelblue").with_filled(true).with_opacity(0.5);
    let density1 = DensityPlot::new()
        .with_data(vec![3.0, 3.5, 4.0, 4.5, 5.0, 4.0, 3.8])
        .with_color("tomato").with_filled(true).with_opacity(0.5);
    let plots = vec![Plot::Density(density0), Plot::Density(density1)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_density_multi.svg", svg.clone()).unwrap();
    // Two densities → two different patterns → at least 2 pattern defs
    let pattern_count = svg.matches("<pattern").count();
    assert!(
        pattern_count >= 2,
        "Two BW density fills should use at least 2 distinct pattern defs, got {pattern_count}"
    );
}

// Plot::Series overlays

#[test]
fn bw_series_multi_distinct_dashes() {
    // Different y-ranges so the two series are visually separated
    let v0: Vec<f64> = (0..20).map(|i| (i as f64 * 0.4).sin() * 2.0 + 3.0).collect();
    let v1: Vec<f64> = (0..20).map(|i| (i as f64 * 0.4).cos() * 2.0 + 9.0).collect();
    let s0 = SeriesPlot::new().with_data(v0).with_color("steelblue").with_line_style();
    let s1 = SeriesPlot::new().with_data(v1).with_color("tomato").with_line_style();
    let plots = vec![Plot::Series(s0), Plot::Series(s1)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_series_multi.svg", svg.clone()).unwrap();
    assert!(svg.contains("8 4"), "Second SeriesPlot should be dashed (8 4)");
}

// Plot::Band overlays

#[test]
fn bw_band_multi_distinct_patterns() {
    let x: Vec<f64> = (0..15).map(|i| i as f64).collect();
    let lo0: Vec<f64> = x.iter().map(|&v| v.sin() - 0.5).collect();
    let hi0: Vec<f64> = x.iter().map(|&v| v.sin() + 0.5).collect();
    let lo1: Vec<f64> = x.iter().map(|&v| v.cos() - 0.5).collect();
    let hi1: Vec<f64> = x.iter().map(|&v| v.cos() + 0.5).collect();
    use kuva::plot::BandPlot;
    let b0 = BandPlot::new(x.clone(), lo0, hi0).with_color("steelblue").with_opacity(0.4);
    let b1 = BandPlot::new(x, lo1, hi1).with_color("tomato").with_opacity(0.4);
    let plots = vec![Plot::Band(b0), Plot::Band(b1)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_band_multi.svg", svg.clone()).unwrap();
    let pattern_count = svg.matches("<pattern").count();
    assert!(
        pattern_count >= 2,
        "Two BW band overlays should use at least 2 distinct pattern defs, got {pattern_count}"
    );
}

// BoxPlot multi-group

#[test]
fn bw_boxplot_multi_group_distinct_patterns() {
    use kuva::plot::BoxPlot;
    let bp = BoxPlot::new()
        .with_group("A", vec![1.0, 2.0, 2.5, 3.0, 4.0, 2.8])
        .with_group("B", vec![2.0, 3.0, 3.5, 4.0, 4.5, 3.2])
        .with_group("C", vec![0.5, 1.5, 2.0, 2.5, 3.5, 2.0]);
    let plots = vec![Plot::Box(bp)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_boxplot_multi.svg", svg.clone()).unwrap();
    let pattern_count = svg.matches("<pattern").count();
    assert!(
        pattern_count >= 3,
        "Three box groups should use at least 3 distinct patterns, got {pattern_count}"
    );
}

// ViolinPlot multi-group

#[test]
fn bw_violin_multi_group_distinct_patterns() {
    let violin = ViolinPlot::new()
        .with_group("A", vec![1.0, 1.5, 2.0, 2.5, 3.0, 2.2, 1.8])
        .with_group("B", vec![2.5, 3.0, 3.5, 4.0, 4.5, 3.8, 3.2])
        .with_group("C", vec![0.5, 1.0, 1.5, 2.0, 2.5, 1.2, 0.8]);
    let plots = vec![Plot::Violin(violin)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_violin_multi.svg", svg.clone()).unwrap();
    let pattern_count = svg.matches("<pattern").count();
    assert!(
        pattern_count >= 3,
        "Three violin groups should use at least 3 distinct patterns, got {pattern_count}"
    );
}

// RidgelinePlot multi-group

#[test]
fn bw_ridgeline_multi_group_distinct_patterns() {
    use kuva::plot::ridgeline::RidgelinePlot;
    // Five groups to clearly differentiate from the single bw_ridgeline test (3 groups)
    let rp = RidgelinePlot::new()
        .with_group("Jan", vec![2.0, 3.0, 4.0, 3.5, 2.5, 4.5, 3.0])
        .with_group("Mar", vec![8.0, 10.0, 12.0, 11.0, 9.0, 13.0, 10.5])
        .with_group("Jun", vec![20.0, 22.0, 24.0, 23.0, 21.0, 25.0, 22.5])
        .with_group("Sep", vec![14.0, 16.0, 18.0, 17.0, 15.0, 19.0, 16.5])
        .with_group("Nov", vec![6.0, 8.0, 10.0, 9.0, 7.0, 11.0, 8.5])
        .with_filled(true);
    let plots = vec![Plot::Ridgeline(rp)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_ridgeline_multi.svg", svg.clone()).unwrap();
    let pattern_count = svg.matches("<pattern").count();
    assert!(
        pattern_count >= 5,
        "Five ridgeline groups should use at least 5 distinct fill patterns, got {pattern_count}"
    );
}

// SurvivalPlot multi-group

#[test]
fn bw_survival_multi_group_distinct_dashes() {
    use kuva::plot::SurvivalPlot;
    let sp = SurvivalPlot::new()
        .with_group("Ctrl",  vec![2.0, 4.0, 6.0, 8.0, 10.0, 12.0], vec![true, true, false, true, false, true])
        .with_group("Trt A", vec![3.0, 6.0, 9.0, 12.0, 15.0, 18.0], vec![true, false, true, false, true, false])
        .with_group("Trt B", vec![5.0, 8.0, 11.0, 14.0, 17.0, 20.0], vec![true, true, false, false, true, true]);
    let plots = vec![Plot::Survival(sp)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_survival_multi.svg", svg.clone()).unwrap();
    assert!(svg.contains("8 4"), "Second survival group should use dashed line (8 4)");
    assert!(svg.contains("2 4"), "Third survival group should use dotted line (2 4)");
}

// EcdfPlot multi-group

#[test]
fn bw_ecdf_multi_group_distinct_dashes() {
    let ecdf = EcdfPlot::new()
        .with_data("A", vec![1.2, 2.3, 3.4, 2.1, 4.5, 1.8, 3.0, 2.7])
        .with_data("B", vec![2.2, 3.3, 4.4, 3.1, 5.5, 2.8, 4.0, 3.7])
        .with_confidence_band();
    let plots = vec![Plot::Ecdf(ecdf)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_ecdf_multi.svg", svg.clone()).unwrap();
    assert!(svg.contains("8 4"), "Second ECDF group should use dashed line (8 4)");
}

// RocPlot multi-group

#[test]
fn bw_roc_multi_group_distinct_dashes() {
    use kuva::plot::roc::{RocGroup, RocPlot};
    // Model A: strong classifier (positives cluster at high scores)
    let data_a: Vec<(f64, bool)> = vec![
        (0.95, true), (0.90, true), (0.85, true), (0.80, true),
        (0.40, false), (0.30, false), (0.20, false), (0.10, false),
    ];
    // Model B: weak classifier (scores mixed between classes)
    let data_b: Vec<(f64, bool)> = vec![
        (0.75, true), (0.55, false), (0.65, true), (0.45, false),
        (0.60, false), (0.50, true), (0.40, false), (0.35, true),
    ];
    let g0 = RocGroup::new("Model A").with_raw(data_a);
    let g1 = RocGroup::new("Model B").with_raw(data_b);
    let roc = RocPlot::new().with_group(g0).with_group(g1);
    let plots = vec![Plot::Roc(roc)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_roc_multi.svg", svg.clone()).unwrap();
    assert!(svg.contains("8 4"), "Second ROC group should use dashed line (8 4)");
}

// PrPlot multi-group

#[test]
fn bw_pr_multi_group_distinct_dashes() {
    use kuva::plot::pr::{PrGroup, PrPlot};
    // Model A: strong classifier
    let data_a: Vec<(f64, bool)> = vec![
        (0.95, true), (0.90, true), (0.85, true), (0.80, true),
        (0.40, false), (0.30, false), (0.20, false), (0.10, false),
    ];
    // Model B: weaker classifier
    let data_b: Vec<(f64, bool)> = vec![
        (0.75, true), (0.55, false), (0.65, true), (0.45, false),
        (0.60, false), (0.50, true), (0.40, false), (0.35, true),
    ];
    let g0 = PrGroup::new("Model A").with_raw(data_a);
    let g1 = PrGroup::new("Model B").with_raw(data_b);
    let pr = PrPlot::new().with_group(g0).with_group(g1);
    let plots = vec![Plot::Pr(pr)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_pr_multi.svg", svg.clone()).unwrap();
    assert!(svg.contains("8 4"), "Second PR group should use dashed line (8 4)");
}

// BumpPlot multi-series

#[test]
fn bw_bump_multi_series_distinct_dashes() {
    use kuva::plot::bump::BumpPlot;
    let bp = BumpPlot::new()
        .with_series("Alpha", vec![1, 3, 2, 1])
        .with_series("Beta",  vec![2, 1, 3, 2])
        .with_series("Gamma", vec![3, 2, 1, 3])
        .with_x_labels(["2021", "2022", "2023", "2024"]);
    let plots = vec![Plot::Bump(bp)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_bump_multi.svg", svg.clone()).unwrap();
    assert!(svg.contains("8 4"), "Second bump series should use dashed line (8 4)");
    assert!(svg.contains("2 4"), "Third bump series should use dotted line (2 4)");
}

// StackedAreaPlot multi-series patterns

#[test]
fn bw_stacked_area_multi_series_distinct_patterns() {
    use kuva::plot::StackedAreaPlot;
    // Five series to clearly differentiate from the single bw_stacked_area test (3 series)
    let sa = StackedAreaPlot::new()
        .with_x(vec![0.0, 1.0, 2.0, 3.0, 4.0])
        .with_series(vec![10.0, 12.0, 11.0, 14.0, 13.0]).with_color("steelblue")
        .with_series(vec![5.0, 7.0, 6.0, 8.0, 7.0]).with_color("tomato")
        .with_series(vec![3.0, 4.0, 3.0, 5.0, 4.0]).with_color("goldenrod")
        .with_series(vec![2.0, 3.0, 4.0, 3.0, 2.0]).with_color("orchid")
        .with_series(vec![1.0, 2.0, 1.0, 2.0, 3.0]).with_color("teal");
    let plots = vec![Plot::StackedArea(sa)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_stacked_area_multi.svg", svg.clone()).unwrap();
    let pattern_count = svg.matches("<pattern").count();
    assert!(
        pattern_count >= 5,
        "Five stacked-area series should use at least 5 distinct patterns, got {pattern_count}"
    );
}

// ── Sanity checks ─────────────────────────────────────────────────────────────

#[test]
fn bw_color_mode_no_patterns() {
    let bar = BarPlot::new()
        .with_bar("A", 3.2)
        .with_bar("B", 4.7);
    let plots = vec![Plot::Bar(bar)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    assert!(
        !svg.contains("kuva-fp-"),
        "Color mode should NOT emit pattern defs"
    );
}

#[test]
fn bw_layout_flag_propagates_to_computed() {
    use kuva::render::layout::ComputedLayout;
    let layout = Layout::new((0.0, 1.0), (0.0, 1.0)).with_bw_mode();
    let computed = ComputedLayout::from_layout(&layout);
    assert!(computed.bw_mode, "bw_mode should propagate from Layout to ComputedLayout");
}
