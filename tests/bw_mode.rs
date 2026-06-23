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
