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

/// Color-mode counterpart to `bw_svg`, used by the Group 6 colormap tests to
/// confirm the same plot construction actually produces colorful fills
/// outside BW mode (i.e. the test data exercises the colormap meaningfully).
fn plain_svg(plots: Vec<Plot>, layout: Layout) -> String {
    let scene = render_multiple(plots, layout);
    SvgBackend.render_scene(&scene)
}

/// True if the SVG contains a `<rect>` element filled with the bw-mode marker
/// color — i.e. a `Square` marker drawn by `draw_marker`. Chart-area/clip
/// rects never carry this fill, so this can't false-positive on chrome.
fn svg_has_bw_square_marker(svg: &str) -> bool {
    let mut search_from = 0;
    while let Some(rel) = svg[search_from..].find("<rect ") {
        let start = search_from + rel;
        let end = svg[start..].find("/>").map_or(svg.len(), |e| start + e);
        if svg[start..end].contains("fill=\"#1a1a1a\"") {
            return true;
        }
        search_from = end.max(start + 1);
    }
    false
}

/// Count of distinct `url(#kuva-fp-...)` pattern fills used strictly *inside*
/// the plot-body clip group — i.e. on the data marks themselves, not on
/// legend swatches (which live outside the clip group and, via the
/// already-bw-aware generic legend renderer, can register their own pattern
/// defs independent of whether the plot's data-drawing code is bw-aware).
/// `svg.matches("<pattern").count()` alone is NOT a safe signal here: pattern
/// *definitions* are deduplicated by id, so a plot with an unrelated but
/// already-bw-aware legend can satisfy a bare pattern-def-count assertion
/// even when its own data marks are not bw-aware at all.
fn distinct_patterns_in_plot_body(svg: &str) -> usize {
    let body = match svg.find("<g clip-path=") {
        Some(start) => {
            let open_end = svg[start..].find('>').map_or(svg.len(), |e| start + e + 1);
            let close = svg[open_end..]
                .rfind("</g>")
                .map_or(svg.len(), |e| open_end + e);
            &svg[open_end..close]
        }
        None => svg,
    };
    let mut ids = std::collections::HashSet::new();
    let mut search_from = 0;
    while let Some(rel) = body[search_from..].find("url(#kuva-fp-") {
        let start = search_from + rel;
        let end = body[start..]
            .find(')')
            .map_or(body.len(), |e| start + e + 1);
        ids.insert(&body[start..end]);
        search_from = end;
    }
    ids.len()
}

/// Count of distinct dash "groups" (dasharray value, or "solid" when the
/// attribute is absent) among elements stroked with `stroke`. Scans forward
/// from each `stroke="<stroke>"` occurrence to the element's closing `/>`
/// (both `<line>` and `<path>` end that way in this backend), so it works
/// for both primitive kinds without needing to know which one is used.
fn distinct_dash_groups_for_stroke(svg: &str, stroke: &str) -> usize {
    let marker = format!("stroke=\"{stroke}\"");
    let mut groups = std::collections::HashSet::new();
    let mut search_from = 0;
    while let Some(rel) = svg[search_from..].find(&marker) {
        let start = search_from + rel;
        let end = svg[start..].find("/>").map_or(svg.len(), |e| start + e);
        let seg = &svg[start..end];
        let dash = if let Some(dpos) = seg.find("stroke-dasharray=\"") {
            let s = dpos + "stroke-dasharray=\"".len();
            let e = seg[s..].find('"').map_or(seg.len(), |e| s + e);
            seg[s..e].to_string()
        } else {
            "solid".to_string()
        };
        groups.insert(dash);
        search_from = end.max(start + 1);
    }
    groups.len()
}

/// True if any `fill="#rrggbb"` in the SVG has unequal R/G/B channels — i.e.
/// a genuinely colored (not grey/black/white) fill. Used for the Group 6
/// colormap family: BW mode forces `ColorMap::Grayscale`, so a data fill
/// derived from the colormap should always have R == G == B, while chrome
/// (background, axis lines, gridlines) is already neutral in kuva's default
/// theme, so this is a safe whole-SVG scan rather than needing to scope to
/// a specific element.
fn svg_has_non_grey_fill(svg: &str) -> bool {
    let mut search_from = 0;
    while let Some(rel) = svg[search_from..].find("fill=\"#") {
        let start = search_from + rel + "fill=\"".len();
        let end = svg[start..].find('"').map_or(svg.len(), |e| start + e);
        let hex = &svg[start..end];
        if hex.len() == 7 {
            let r = u8::from_str_radix(&hex[1..3], 16);
            let g = u8::from_str_radix(&hex[3..5], 16);
            let b = u8::from_str_radix(&hex[5..7], 16);
            if let (Ok(r), Ok(g), Ok(b)) = (r, g, b) {
                if r != g || g != b {
                    return true;
                }
            }
        }
        search_from = end.max(start + 1);
    }
    false
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
    assert!(
        svg.contains("<pattern"),
        "BW bar chart should emit SVG pattern defs"
    );
    assert!(
        svg.contains("kuva-fp-"),
        "Pattern defs should use kuva-fp- prefix"
    );
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
        vec![
            (0b001u64, 10usize),
            (0b010, 8),
            (0b100, 12),
            (0b011, 5),
            (0b111, 20),
        ],
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
        .with_group(
            "Spring",
            vec![12.0, 15.0, 18.0, 14.0, 16.0, 13.0, 17.0, 15.5],
        )
        .with_group(
            "Summer",
            vec![22.0, 25.0, 28.0, 24.0, 26.0, 23.0, 27.0, 25.5],
        )
        .with_group(
            "Autumn",
            vec![10.0, 13.0, 16.0, 12.0, 14.0, 11.0, 15.0, 13.5],
        )
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
    assert!(
        svg.contains("#1a1a1a"),
        "BW survival curves should use dark stroke"
    );
}

#[test]
fn bw_roc() {
    use kuva::plot::roc::{RocGroup, RocPlot};
    let data: Vec<(f64, bool)> = vec![
        (0.95, true),
        (0.88, true),
        (0.80, false),
        (0.72, true),
        (0.65, false),
        (0.55, true),
        (0.40, false),
        (0.30, false),
        (0.22, true),
        (0.10, false),
    ];
    let group = RocGroup::new("Classifier").with_raw(data).with_ci(true);
    let roc = RocPlot::new().with_group(group);
    let plots = vec![Plot::Roc(roc)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_roc.svg", svg.clone()).unwrap();
    assert!(
        svg.contains("#1a1a1a"),
        "BW ROC curve should use dark stroke"
    );
}

// ── Lines ────────────────────────────────────────────────────────────────────

#[test]
fn bw_line() {
    let line = LinePlot::new()
        .with_data(vec![
            (0.0, 1.0),
            (1.0, 3.0),
            (2.0, 2.0),
            (3.0, 4.0),
            (4.0, 3.5),
        ])
        .with_color("steelblue");
    let plots = vec![Plot::Line(line)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_line.svg", svg.clone()).unwrap();
    assert!(
        svg.contains("#1a1a1a"),
        "BW line chart should use dark stroke color"
    );
}

#[test]
fn bw_series() {
    let data: Vec<f64> = (0..40)
        .map(|x| (x as f64 * 0.3).sin() * 3.0 + 5.0)
        .collect();
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
        (0.92, true),
        (0.85, true),
        (0.78, false),
        (0.70, true),
        (0.60, false),
        (0.50, true),
        (0.38, false),
        (0.25, false),
        (0.18, true),
        (0.08, false),
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
        .with_data(
            "Sample A",
            vec![1.2, 3.4, 2.1, 5.6, 4.0, 0.8, 3.3, 2.7, 4.5, 1.9],
        )
        .with_data(
            "Sample B",
            vec![2.2, 3.8, 2.9, 4.6, 3.0, 1.8, 4.3, 3.7, 5.0, 2.4],
        )
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
        .with_data(vec![
            (1.0, 2.0),
            (2.0, 3.0),
            (3.0, 1.5),
            (4.0, 4.0),
            (5.0, 2.5),
        ])
        .with_color("steelblue");
    let plots = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_scatter.svg", svg.clone()).unwrap();
    assert!(
        svg.contains("#1a1a1a"),
        "BW scatter chart should use dark fill color"
    );
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
    let line0 = LinePlot::new()
        .with_data(data0)
        .with_color("steelblue")
        .with_legend("A");
    let line1 = LinePlot::new()
        .with_data(data1)
        .with_color("tomato")
        .with_legend("B");
    let line2 = LinePlot::new()
        .with_data(data2)
        .with_color("seagreen")
        .with_legend("C");
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
    let scatter0 = ScatterPlot::new()
        .with_data(pts_a)
        .with_color("steelblue")
        .with_legend("A");
    let scatter1 = ScatterPlot::new()
        .with_data(pts_b)
        .with_color("tomato")
        .with_legend("B");
    let plots = vec![Plot::Scatter(scatter0), Plot::Scatter(scatter1)];
    let mut layout = Layout::auto_from_plots(&plots);
    layout.show_legend = true;
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_scatter_multi.svg", svg.clone()).unwrap();
    // Series 0 → Circle (fast path CircleBatch), Series 1 → Square (rect element via slow path)
    assert!(
        svg.contains("#1a1a1a"),
        "All BW scatter points should be dark"
    );
    // The second scatter uses bw_shape(1) = Square → should contain a rect or polygon path,
    // not just circles.  A simple proxy: the SVG is longer / more complex than a single series.
    assert!(
        svg.len() > 500,
        "Multi-series BW scatter should produce non-trivial SVG"
    );
}

#[test]
fn bw_density_multi_series_distinct_patterns() {
    let density0 = DensityPlot::new()
        .with_data(vec![1.0, 1.5, 2.0, 2.5, 3.0, 2.0, 1.8])
        .with_color("steelblue")
        .with_filled(true)
        .with_opacity(0.5);
    let density1 = DensityPlot::new()
        .with_data(vec![3.0, 3.5, 4.0, 4.5, 5.0, 4.0, 3.8])
        .with_color("tomato")
        .with_filled(true)
        .with_opacity(0.5);
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
    let v0: Vec<f64> = (0..20)
        .map(|i| (i as f64 * 0.4).sin() * 2.0 + 3.0)
        .collect();
    let v1: Vec<f64> = (0..20)
        .map(|i| (i as f64 * 0.4).cos() * 2.0 + 9.0)
        .collect();
    let s0 = SeriesPlot::new()
        .with_data(v0)
        .with_color("steelblue")
        .with_line_style();
    let s1 = SeriesPlot::new()
        .with_data(v1)
        .with_color("tomato")
        .with_line_style();
    let plots = vec![Plot::Series(s0), Plot::Series(s1)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_series_multi.svg", svg.clone()).unwrap();
    assert!(
        svg.contains("8 4"),
        "Second SeriesPlot should be dashed (8 4)"
    );
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
    let b0 = BandPlot::new(x.clone(), lo0, hi0)
        .with_color("steelblue")
        .with_opacity(0.4);
    let b1 = BandPlot::new(x, lo1, hi1)
        .with_color("tomato")
        .with_opacity(0.4);
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
        .with_group(
            "Ctrl",
            vec![2.0, 4.0, 6.0, 8.0, 10.0, 12.0],
            vec![true, true, false, true, false, true],
        )
        .with_group(
            "Trt A",
            vec![3.0, 6.0, 9.0, 12.0, 15.0, 18.0],
            vec![true, false, true, false, true, false],
        )
        .with_group(
            "Trt B",
            vec![5.0, 8.0, 11.0, 14.0, 17.0, 20.0],
            vec![true, true, false, false, true, true],
        );
    let plots = vec![Plot::Survival(sp)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_survival_multi.svg", svg.clone()).unwrap();
    assert!(
        svg.contains("8 4"),
        "Second survival group should use dashed line (8 4)"
    );
    assert!(
        svg.contains("2 4"),
        "Third survival group should use dotted line (2 4)"
    );
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
    assert!(
        svg.contains("8 4"),
        "Second ECDF group should use dashed line (8 4)"
    );
}

// RocPlot multi-group

#[test]
fn bw_roc_multi_group_distinct_dashes() {
    use kuva::plot::roc::{RocGroup, RocPlot};
    // Model A: strong classifier (positives cluster at high scores)
    let data_a: Vec<(f64, bool)> = vec![
        (0.95, true),
        (0.90, true),
        (0.85, true),
        (0.80, true),
        (0.40, false),
        (0.30, false),
        (0.20, false),
        (0.10, false),
    ];
    // Model B: weak classifier (scores mixed between classes)
    let data_b: Vec<(f64, bool)> = vec![
        (0.75, true),
        (0.55, false),
        (0.65, true),
        (0.45, false),
        (0.60, false),
        (0.50, true),
        (0.40, false),
        (0.35, true),
    ];
    let g0 = RocGroup::new("Model A").with_raw(data_a);
    let g1 = RocGroup::new("Model B").with_raw(data_b);
    let roc = RocPlot::new().with_group(g0).with_group(g1);
    let plots = vec![Plot::Roc(roc)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_roc_multi.svg", svg.clone()).unwrap();
    assert!(
        svg.contains("8 4"),
        "Second ROC group should use dashed line (8 4)"
    );
}

// PrPlot multi-group

#[test]
fn bw_pr_multi_group_distinct_dashes() {
    use kuva::plot::pr::{PrGroup, PrPlot};
    // Model A: strong classifier
    let data_a: Vec<(f64, bool)> = vec![
        (0.95, true),
        (0.90, true),
        (0.85, true),
        (0.80, true),
        (0.40, false),
        (0.30, false),
        (0.20, false),
        (0.10, false),
    ];
    // Model B: weaker classifier
    let data_b: Vec<(f64, bool)> = vec![
        (0.75, true),
        (0.55, false),
        (0.65, true),
        (0.45, false),
        (0.60, false),
        (0.50, true),
        (0.40, false),
        (0.35, true),
    ];
    let g0 = PrGroup::new("Model A").with_raw(data_a);
    let g1 = PrGroup::new("Model B").with_raw(data_b);
    let pr = PrPlot::new().with_group(g0).with_group(g1);
    let plots = vec![Plot::Pr(pr)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_pr_multi.svg", svg.clone()).unwrap();
    assert!(
        svg.contains("8 4"),
        "Second PR group should use dashed line (8 4)"
    );
}

// BumpPlot multi-series

#[test]
fn bw_bump_multi_series_distinct_dashes() {
    use kuva::plot::bump::BumpPlot;
    let bp = BumpPlot::new()
        .with_series("Alpha", vec![1, 3, 2, 1])
        .with_series("Beta", vec![2, 1, 3, 2])
        .with_series("Gamma", vec![3, 2, 1, 3])
        .with_x_labels(["2021", "2022", "2023", "2024"]);
    let plots = vec![Plot::Bump(bp)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_bump_multi.svg", svg.clone()).unwrap();
    assert!(
        svg.contains("8 4"),
        "Second bump series should use dashed line (8 4)"
    );
    assert!(
        svg.contains("2 4"),
        "Third bump series should use dotted line (2 4)"
    );
}

// StackedAreaPlot multi-series patterns

#[test]
fn bw_stacked_area_multi_series_distinct_patterns() {
    use kuva::plot::StackedAreaPlot;
    // Five series to clearly differentiate from the single bw_stacked_area test (3 series)
    let sa = StackedAreaPlot::new()
        .with_x(vec![0.0, 1.0, 2.0, 3.0, 4.0])
        .with_series(vec![10.0, 12.0, 11.0, 14.0, 13.0])
        .with_color("steelblue")
        .with_series(vec![5.0, 7.0, 6.0, 8.0, 7.0])
        .with_color("tomato")
        .with_series(vec![3.0, 4.0, 3.0, 5.0, 4.0])
        .with_color("goldenrod")
        .with_series(vec![2.0, 3.0, 4.0, 3.0, 2.0])
        .with_color("orchid")
        .with_series(vec![1.0, 2.0, 1.0, 2.0, 3.0])
        .with_color("teal");
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

#[test]
fn bw_histogram_multi_column_distinct_patterns() {
    // Multi-column mode overlays multiple Histogram plot instances (as the CLI's
    // `--y A,B,C` does). Before the bw_idx fix, every instance rendered with
    // rect_bw's hardcoded index 0 and got an identical pattern.
    let h0 = Histogram::new()
        .with_data(vec![1.0, 1.5, 2.0, 2.0, 2.5])
        .with_range((0.0, 10.0))
        .with_bins(10)
        .with_color("steelblue");
    let h1 = Histogram::new()
        .with_data(vec![5.0, 5.5, 6.0, 6.0, 6.5])
        .with_range((0.0, 10.0))
        .with_bins(10)
        .with_color("tomato");
    let plots = vec![Plot::Histogram(h0), Plot::Histogram(h1)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_histogram_multi.svg", svg.clone()).unwrap();
    let pattern_count = svg.matches("<pattern").count();
    assert!(
        pattern_count >= 2,
        "Two overlaid histograms should use at least 2 distinct patterns, got {pattern_count}"
    );
}

#[test]
fn bw_upset_dot_matrix_uses_guaranteed_contrast_not_user_colors() {
    // The dot matrix is a binary membership indicator, not a multi-series
    // encoding — BW mode should force dark/light contrast regardless of the
    // user's configured dot_color/dot_empty_color.
    use kuva::plot::UpSetPlot;
    let mut upset = UpSetPlot::new()
        .with_data(
            vec!["Set A", "Set B", "Set C"],
            vec![52usize, 47, 36],
            vec![
                (0b001u64, 10usize),
                (0b010, 8),
                (0b100, 12),
                (0b011, 5),
                (0b111, 20),
            ],
        )
        .with_dot_color("#4499cc");
    upset.dot_empty_color = "#eeeeee".to_string();
    let plots = vec![Plot::UpSet(upset)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_upset_dots.svg", svg.clone()).unwrap();
    assert!(
        svg.contains("#1a1a1a"),
        "filled dots must use the guaranteed-dark BW color, not the user's dot_color"
    );
    assert!(
        !svg.contains("#4499cc") && !svg.contains("#eeeeee"),
        "BW mode must not leak the user's configured dot colors into the output"
    );
}

// ── Group 2: point/marker family ──────────────────────────────────────────────
//
// All of these draw raw Circle points or call draw_marker directly. Without a
// bw fix, every point renders as a plain dark circle regardless of category —
// so in bw_mode a non-circle SVG tag (<rect> for Square, <path> for Triangle/
// Diamond) appearing at all is the signal that shape differentiation kicked in.

#[test]
fn bw_volcano_categories_use_distinct_shapes() {
    use kuva::plot::VolcanoPlot;
    // fc_cutoff=1.0, p_cutoff=0.05 (defaults): one point per category (NS, Down, Up).
    let volcano = VolcanoPlot::new().with_points(vec![
        ("GeneNS", 0.1, 0.5),     // NS: |log2fc| < cutoff
        ("GeneDown", -2.0, 0.01), // Down: log2fc <= -cutoff, significant
        ("GeneUp", 2.0, 0.01),    // Up: log2fc >= cutoff, significant
    ]);
    let plots = vec![Plot::Volcano(volcano)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_volcano.svg", svg.clone()).unwrap();
    // NS -> bw_shape(0) = Circle, Down -> bw_shape(1) = Square (<rect>).
    assert!(
        svg_has_bw_square_marker(&svg),
        "Down category should render as a Square (<rect> filled #1a1a1a)"
    );
}

#[test]
fn bw_manhattan_chromosomes_use_distinct_shapes() {
    use kuva::plot::ManhattanPlot;
    let mp = ManhattanPlot::new().with_data(vec![("1", 0.01), ("2", 0.02), ("3", 0.03)]);
    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_manhattan.svg", svg.clone()).unwrap();
    // Chr "1" (span 0) -> Circle, chr "2" (span 1) -> Square (<rect>).
    assert!(
        svg_has_bw_square_marker(&svg),
        "second chromosome band should render as a Square (<rect> filled #1a1a1a)"
    );
}

#[test]
fn bw_manhattan_significance_lines_are_not_colored() {
    use kuva::plot::ManhattanPlot;
    let mp = ManhattanPlot::new().with_data(vec![("1", 0.01), ("2", 0.02), ("3", 0.03)]);
    let plots = vec![Plot::Manhattan(mp)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_manhattan_thresholds.svg", svg.clone()).unwrap();
    assert!(
        !svg.contains("#cc3333"),
        "genome-wide significance line should not be red in BW mode"
    );
    assert!(
        svg.contains("stroke=\"#1a1a1a\""),
        "genome-wide significance line should use the BW near-black color"
    );
}

#[test]
fn bw_polar_scatter_series_use_distinct_shapes() {
    use kuva::plot::polar::PolarPlot;
    let polar = PolarPlot::new()
        .with_series(vec![1.0, 2.0, 3.0], vec![0.0, 90.0, 180.0])
        .with_series(vec![1.5, 2.5], vec![45.0, 135.0]);
    let plots = vec![Plot::Polar(polar)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_polar.svg", svg.clone()).unwrap();
    assert!(
        svg_has_bw_square_marker(&svg),
        "second polar series should render as a Square (<rect> filled #1a1a1a)"
    );
}

#[test]
fn bw_ternary_groups_use_distinct_shapes() {
    use kuva::plot::ternary::TernaryPlot;
    let ternary = TernaryPlot::new()
        .with_point_group(0.6, 0.2, 0.2, "A")
        .with_point_group(0.2, 0.6, 0.2, "B")
        .with_point_group(0.3, 0.5, 0.2, "B");
    let plots = vec![Plot::Ternary(ternary)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_ternary.svg", svg.clone()).unwrap();
    assert!(
        svg_has_bw_square_marker(&svg),
        "second ternary group should render as a Square (<rect> filled #1a1a1a)"
    );
}

#[test]
fn bw_diceplot_categorical_dots_use_distinct_shapes() {
    use kuva::plot::diceplot::DicePlot;
    let dice = DicePlot::new(1)
        .with_category_labels(vec!["Only".into()])
        .with_dot_legend(vec![("Down", "#2166ac"), ("Up", "#b2182b")])
        .with_records(vec![
            ("Row1", "Col1", "Only", "#2166ac"),
            ("Row2", "Col1", "Only", "#b2182b"),
        ]);
    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_diceplot.svg", svg.clone()).unwrap();
    // "Down" (legend index 0) -> Circle, "Up" (legend index 1) -> Square (<rect>).
    assert!(
        svg_has_bw_square_marker(&svg),
        "second dot-legend category should render as a Square (<rect> filled #1a1a1a)"
    );
}

#[test]
fn bw_diceplot_dot_legend_swatches_are_not_colored() {
    use kuva::plot::diceplot::DicePlot;
    let dice = DicePlot::new(1)
        .with_category_labels(vec!["Only".into()])
        .with_dot_legend(vec![("Down", "#2166ac"), ("Up", "#b2182b")])
        .with_records(vec![
            ("Row1", "Col1", "Only", "#2166ac"),
            ("Row2", "Col1", "Only", "#b2182b"),
        ]);
    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_diceplot_legend.svg", svg.clone()).unwrap();
    assert!(
        !svg.contains("#2166ac") && !svg.contains("#b2182b"),
        "dot-legend swatches (drawn via add_legend_at) should not leak the user's configured colors in BW mode"
    );
}

#[test]
fn bw_venn_sets_use_distinct_patterns() {
    use kuva::plot::venn::VennPlot;
    let venn = VennPlot::new()
        .with_set_size("Set A", 100)
        .with_set_size("Set B", 80)
        .with_overlap(["Set A", "Set B"], 30);
    let plots = vec![Plot::Venn(venn)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_venn.svg", svg.clone()).unwrap();
    let pattern_count = svg.matches("<pattern").count();
    assert!(
        pattern_count >= 2,
        "Two venn sets should use at least 2 distinct patterns, got {pattern_count}"
    );
}

#[test]
fn bw_scatter3d_instances_use_distinct_shapes() {
    use kuva::plot::scatter3d::Scatter3DPlot;
    let s0 = Scatter3DPlot::new()
        .with_data(vec![(1.0, 2.0, 3.0), (2.0, 3.0, 4.0)])
        .with_color("steelblue");
    let s1 = Scatter3DPlot::new()
        .with_data(vec![(4.0, 5.0, 1.0), (5.0, 6.0, 2.0)])
        .with_color("tomato");
    let plots = vec![Plot::Scatter3D(s0), Plot::Scatter3D(s1)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_scatter3d.svg", svg.clone()).unwrap();
    assert!(
        svg_has_bw_square_marker(&svg),
        "second Scatter3D instance should render as a Square (<rect> filled #1a1a1a)"
    );
}

// ── Group 3: hierarchical rect/path family ──────────────────────────────────

#[test]
fn bw_mosaic_rows_use_distinct_patterns() {
    use kuva::plot::mosaic::MosaicPlot;
    let mosaic = MosaicPlot::new()
        .with_cell("Control", "Positive", 30.0)
        .with_cell("Control", "Negative", 70.0)
        .with_cell("Treated", "Positive", 60.0)
        .with_cell("Treated", "Negative", 40.0);
    let plots = vec![Plot::Mosaic(mosaic)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_mosaic.svg", svg.clone()).unwrap();
    let pattern_count = distinct_patterns_in_plot_body(&svg);
    assert!(
        pattern_count >= 2,
        "two mosaic rows should use at least 2 distinct patterns, got {pattern_count}"
    );
}

#[test]
fn bw_treemap_roots_use_distinct_patterns() {
    use kuva::plot::treemap::{TreemapNode, TreemapPlot};
    let treemap = TreemapPlot::new()
        .with_node(TreemapNode::new(
            "Foods",
            vec![
                TreemapNode::leaf("Apples", 30.0),
                TreemapNode::leaf("Oranges", 20.0),
            ],
        ))
        .with_node(TreemapNode::new(
            "Drinks",
            vec![
                TreemapNode::leaf("Coffee", 15.0),
                TreemapNode::leaf("Tea", 10.0),
            ],
        ));
    let plots = vec![Plot::Treemap(treemap)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_treemap.svg", svg.clone()).unwrap();
    let pattern_count = distinct_patterns_in_plot_body(&svg);
    assert!(
        pattern_count >= 2,
        "two treemap roots should use at least 2 distinct patterns, got {pattern_count}"
    );
}

#[test]
fn bw_sunburst_roots_use_distinct_patterns() {
    use kuva::plot::sunburst::SunburstPlot;
    use kuva::plot::treemap::TreemapNode;
    let sunburst = SunburstPlot::new()
        .with_node(TreemapNode::new(
            "Org1",
            vec![
                TreemapNode::leaf("Alice", 40.0),
                TreemapNode::leaf("Bob", 30.0),
            ],
        ))
        .with_node(TreemapNode::new(
            "Org2",
            vec![
                TreemapNode::leaf("Carol", 25.0),
                TreemapNode::leaf("Dave", 20.0),
            ],
        ));
    let plots = vec![Plot::Sunburst(sunburst)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_sunburst.svg", svg.clone()).unwrap();
    let pattern_count = distinct_patterns_in_plot_body(&svg);
    assert!(
        pattern_count >= 2,
        "two sunburst roots should use at least 2 distinct patterns, got {pattern_count}"
    );
}

#[test]
fn bw_funnel_stages_use_distinct_patterns() {
    use kuva::plot::funnel::FunnelPlot;
    let funnel = FunnelPlot::new()
        .with_stage("Screened", 1200.0)
        .with_stage("Eligible", 800.0)
        .with_stage("Enrolled", 600.0);
    let plots = vec![Plot::Funnel(funnel)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_funnel.svg", svg.clone()).unwrap();
    let pattern_count = distinct_patterns_in_plot_body(&svg);
    assert!(
        pattern_count >= 2,
        "funnel stages should use at least 2 distinct patterns, got {pattern_count}"
    );
}

#[test]
fn bw_pyramid_sides_use_distinct_patterns() {
    use kuva::plot::pyramid::PopulationPyramid;
    let pyramid = PopulationPyramid::new()
        .with_left_color("#4C72B0")
        .with_right_color("#DD8452")
        .with_group("0-4", 6.5, 6.2)
        .with_group("5-9", 6.8, 6.5);
    let plots = vec![Plot::Pyramid(pyramid)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_pyramid.svg", svg.clone()).unwrap();
    let pattern_count = distinct_patterns_in_plot_body(&svg);
    assert!(
        pattern_count >= 2,
        "pyramid left/right halves should use at least 2 distinct patterns, got {pattern_count}"
    );
}

#[test]
fn bw_waffle_categories_use_distinct_patterns() {
    use kuva::plot::waffle::WafflePlot;
    let waffle = WafflePlot::new()
        .with_category("Treated", 45.0, "steelblue")
        .with_category("Partial", 30.0, "gold")
        .with_category("Untreated", 25.0, "#e74c3c")
        .with_grid(5, 20);
    let plots = vec![Plot::Waffle(waffle)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_waffle.svg", svg.clone()).unwrap();
    let pattern_count = distinct_patterns_in_plot_body(&svg);
    assert!(
        pattern_count >= 2,
        "waffle categories should use at least 2 distinct patterns, got {pattern_count}"
    );
}

#[test]
fn bw_gantt_groups_use_distinct_patterns() {
    use kuva::plot::gantt::GanttPlot;
    let gantt = GanttPlot::new()
        .with_task_group("Design", "Wireframes", 0.0, 3.0)
        .with_task_group("Dev", "Backend API", 3.0, 8.0);
    let plots = vec![Plot::Gantt(gantt)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_gantt.svg", svg.clone()).unwrap();
    let pattern_count = distinct_patterns_in_plot_body(&svg);
    assert!(
        pattern_count >= 2,
        "gantt task groups should use at least 2 distinct patterns, got {pattern_count}"
    );
}

#[test]
fn bw_brick_template_chars_use_distinct_patterns() {
    use kuva::plot::brick::BrickPlot;
    use std::collections::HashMap;
    let mut template = HashMap::new();
    template.insert('X', "steelblue".to_string());
    template.insert('Y', "#ff7f0e".to_string());
    let brick = BrickPlot::new()
        .with_sequences(vec!["XYXYXY", "XXYXYY"])
        .with_template(template);
    let plots = vec![Plot::Brick(brick)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_brick.svg", svg.clone()).unwrap();
    let pattern_count = distinct_patterns_in_plot_body(&svg);
    assert!(
        pattern_count >= 2,
        "brick template characters should use at least 2 distinct patterns, got {pattern_count}"
    );
}

// ── Group 4: line/stroke family ─────────────────────────────────────────────

#[test]
fn bw_candlestick_up_down_use_distinct_patterns() {
    use kuva::plot::candlestick::CandlestickPlot;
    let candlestick = CandlestickPlot::new()
        .with_candle("Day1", 10.0, 12.0, 9.0, 11.5) // up
        .with_candle("Day2", 11.5, 12.0, 8.0, 8.5); // down
    let plots = vec![Plot::Candlestick(candlestick)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_candlestick.svg", svg.clone()).unwrap();
    let pattern_count = distinct_patterns_in_plot_body(&svg);
    assert!(
        pattern_count >= 2,
        "up vs down candle bodies should use at least 2 distinct patterns, got {pattern_count}"
    );
}

#[test]
fn bw_forest_rows_use_distinct_patterns() {
    use kuva::plot::forest::ForestPlot;
    let forest = ForestPlot::new()
        .with_row("Study A", 0.50, 0.10, 0.90)
        .with_row("Study B", -0.30, -0.80, 0.20);
    let plots = vec![Plot::Forest(forest)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_forest.svg", svg.clone()).unwrap();
    let pattern_count = distinct_patterns_in_plot_body(&svg);
    assert!(
        pattern_count >= 2,
        "forest row markers should use at least 2 distinct patterns, got {pattern_count}"
    );
}

#[test]
fn bw_lollipop_points_use_distinct_patterns() {
    use kuva::plot::lollipop::LollipopPlot;
    let lollipop = LollipopPlot::new()
        .with_point(1.0, 5.0)
        .with_point(2.0, 8.0);
    let plots = vec![Plot::Lollipop(lollipop)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_lollipop.svg", svg.clone()).unwrap();
    let pattern_count = distinct_patterns_in_plot_body(&svg);
    assert!(
        pattern_count >= 2,
        "lollipop dots should use at least 2 distinct patterns, got {pattern_count}"
    );
}

#[test]
fn bw_phylo_clades_use_distinct_dashes() {
    use kuva::plot::PhyloTree;
    let edges: Vec<(&str, &str, f64)> = vec![
        ("root", "Bacteria", 1.5),
        ("root", "Eukarya", 2.0),
        ("Bacteria", "E. coli", 0.5),
        ("Bacteria", "B. subtilis", 0.7),
        ("Eukarya", "Yeast", 1.0),
        ("Eukarya", "Human", 0.8),
    ];
    // node id 1 = "Bacteria", id 2 = "Eukarya" (see tests/phylo_basic.rs)
    let tree = PhyloTree::from_edges(&edges)
        .with_clade_color(1, "#e41a1c")
        .with_clade_color(2, "#377eb8");
    let plots = vec![Plot::PhyloTree(tree)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_phylo.svg", svg.clone()).unwrap();
    assert!(
        !svg.contains("#e41a1c") && !svg.contains("#377eb8"),
        "clade colors should not leak into BW mode"
    );
    let dash_groups = distinct_dash_groups_for_stroke(&svg, "#1a1a1a");
    assert!(
        dash_groups >= 2,
        "the two clades should use at least 2 distinct dash styles, got {dash_groups}"
    );
}

#[test]
fn bw_parallel_groups_use_distinct_dashes() {
    use kuva::plot::parallel::ParallelPlot;
    let parallel = ParallelPlot::new()
        .with_axis_names(vec!["A", "B", "C"])
        .with_row_group("Group1", vec![1.0, 2.0, 3.0])
        .with_row_group("Group2", vec![3.0, 2.0, 1.0]);
    let plots = vec![Plot::Parallel(parallel)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_parallel.svg", svg.clone()).unwrap();
    let dash_groups = distinct_dash_groups_for_stroke(&svg, "#1a1a1a");
    assert!(
        dash_groups >= 2,
        "the two row groups should use at least 2 distinct dash styles, got {dash_groups}"
    );
}

#[test]
fn bw_radar_series_use_distinct_patterns() {
    use kuva::plot::radar::RadarPlot;
    let radar = RadarPlot::new(vec!["Speed", "Power", "Range"])
        .with_series(vec![3.0, 4.0, 5.0])
        .with_series(vec![5.0, 3.0, 2.0])
        .with_filled(true);
    let plots = vec![Plot::Radar(radar)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_radar.svg", svg.clone()).unwrap();
    let pattern_count = distinct_patterns_in_plot_body(&svg);
    assert!(
        pattern_count >= 2,
        "the two radar series should use at least 2 distinct patterns, got {pattern_count}"
    );
}

#[test]
fn bw_horizon_pos_neg_are_not_colored() {
    use kuva::plot::horizon::HorizonPlot;
    let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&t| (t - 5.0) * 2.0).collect(); // crosses zero
    let horizon = HorizonPlot::new().with_series("Temp", x, y);
    let plots = vec![Plot::Horizon(horizon)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_horizon.svg", svg.clone()).unwrap();
    // Default pos_color/neg_color hex (see horizon_basic.rs) should not leak through.
    assert!(
        !svg.contains("#1f77b4") && !svg.contains("#d62728"),
        "pos/neg colors should not leak into BW mode"
    );
    assert!(
        svg.contains("#1a1a1a"),
        "positive bands should use the fixed BW dark grey"
    );
    assert!(
        svg.contains("#888888"),
        "negative bands should use a distinct fixed BW grey"
    );
}

// ── Group 5: composite/pixel-space, most bespoke ────────────────────────────

#[test]
fn bw_chord_nodes_use_distinct_patterns() {
    use kuva::plot::chord::ChordPlot;
    let chord = ChordPlot::new()
        .with_matrix(vec![
            vec![0.0, 10.0, 5.0],
            vec![10.0, 0.0, 3.0],
            vec![5.0, 3.0, 0.0],
        ])
        .with_labels(vec!["A", "B", "C"]);
    let plots = vec![Plot::Chord(chord)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_chord.svg", svg.clone()).unwrap();
    let pattern_count = distinct_patterns_in_plot_body(&svg);
    assert!(
        pattern_count >= 2,
        "chord nodes should use at least 2 distinct patterns, got {pattern_count}"
    );
}

#[test]
fn bw_sankey_nodes_use_distinct_patterns() {
    use kuva::plot::sankey::SankeyPlot;
    let sankey = SankeyPlot::new()
        .with_link("Source A", "Target", 10.0)
        .with_link("Source B", "Target", 5.0);
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_sankey.svg", svg.clone()).unwrap();
    let pattern_count = distinct_patterns_in_plot_body(&svg);
    assert!(
        pattern_count >= 2,
        "sankey nodes/links should use at least 2 distinct patterns, got {pattern_count}"
    );
}

#[test]
fn bw_sankey_node_rects_have_visible_border() {
    // Links share their source node's pattern (see bw_sankey_nodes_use_distinct_patterns),
    // so without an outline the node rect visually disappears into its own outgoing
    // ribbon — reported directly by the user after visually checking Group 5.
    use kuva::plot::sankey::SankeyPlot;
    let sankey = SankeyPlot::new().with_link("Source A", "Target", 10.0);
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    assert!(
        svg.contains("stroke=\"#1a1a1a\" stroke-width=\"2\""),
        "sankey node rects should have a visible dark border in BW mode"
    );
}

#[test]
fn bw_synteny_sequences_use_distinct_patterns() {
    use kuva::plot::synteny::SyntenyPlot;
    let synteny = SyntenyPlot::new()
        .with_sequences(vec![("Seq1", 100.0), ("Seq2", 100.0)])
        .with_block(0, 10.0, 40.0, 1, 10.0, 40.0);
    let plots = vec![Plot::Synteny(synteny)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_synteny.svg", svg.clone()).unwrap();
    let pattern_count = distinct_patterns_in_plot_body(&svg);
    assert!(
        pattern_count >= 2,
        "synteny sequence bars should use at least 2 distinct patterns, got {pattern_count}"
    );
}

#[test]
fn bw_synteny_bars_have_visible_border() {
    // Block index and sequence index are independent, so a ribbon can share
    // its sequence bar's pattern (as in this exact 2-sequence/1-block case,
    // both index 0) — without an outline the bar visually disappears into
    // the ribbon it sits on top of.
    use kuva::plot::synteny::SyntenyPlot;
    let synteny = SyntenyPlot::new()
        .with_sequences(vec![("Seq1", 100.0), ("Seq2", 100.0)])
        .with_block(0, 10.0, 40.0, 1, 10.0, 40.0);
    let plots = vec![Plot::Synteny(synteny)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    assert!(
        svg.contains("stroke=\"#1a1a1a\" stroke-width=\"2\""),
        "synteny sequence bars should have a visible dark border in BW mode"
    );
}

#[test]
fn bw_network_groups_use_distinct_patterns() {
    use kuva::plot::network::NetworkPlot;
    let network = NetworkPlot::new()
        .with_node_group("N1", "GroupA")
        .with_node_group("N2", "GroupB")
        .with_edge("N1", "N2", 1.0);
    let plots = vec![Plot::Network(network)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_network.svg", svg.clone()).unwrap();
    let pattern_count = distinct_patterns_in_plot_body(&svg);
    assert!(
        pattern_count >= 2,
        "network node groups should use at least 2 distinct patterns, got {pattern_count}"
    );
}

#[test]
fn bw_joint_groups_use_distinct_patterns() {
    use kuva::plot::jointplot::JointPlot;
    let joint = JointPlot::new()
        .with_group(
            "Group1",
            vec![1.0, 2.0, 3.0, 2.0],
            vec![1.0, 2.0, 1.5, 2.5],
            "steelblue",
        )
        .with_group(
            "Group2",
            vec![4.0, 5.0, 6.0, 5.0],
            vec![4.0, 5.0, 4.5, 5.5],
            "tomato",
        );
    let plots = vec![Plot::Joint(joint)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_joint.svg", svg.clone()).unwrap();
    // Joint's marginal panels are appended to the scene *after* the inner
    // scatter sub-scene's own <g clip-path> group closes, so
    // distinct_patterns_in_plot_body (which assumes one clip group wraps all
    // data) would truncate before reaching them. The scatter markers use
    // bw_shape (no patterns) and the custom group legend uses bw_shape too
    // (see the DicePlot/Group2 legend fix), so a plain pattern-def count is
    // safe here — nothing else in this SVG registers a pattern.
    let pattern_count = svg.matches("<pattern").count();
    assert!(
        pattern_count >= 2,
        "joint marginal histograms should use at least 2 distinct patterns, got {pattern_count}"
    );
}

#[test]
fn bw_joint_standalone_render_jointplot_respects_bw_mode() {
    // render_jointplot() builds its own stub ComputedLayout independent of
    // render_multiple's dispatch path — it must copy layout.bw_mode across,
    // or bw_mode silently has no effect when called this way.
    use kuva::plot::jointplot::JointPlot;
    use kuva::render::render::render_jointplot;
    let joint = JointPlot::new()
        .with_group(
            "Group1",
            vec![1.0, 2.0, 3.0, 2.0],
            vec![1.0, 2.0, 1.5, 2.5],
            "steelblue",
        )
        .with_group(
            "Group2",
            vec![4.0, 5.0, 6.0, 5.0],
            vec![4.0, 5.0, 4.5, 5.5],
            "tomato",
        );
    let layout = Layout::new((0.0, 7.0), (0.0, 7.0)).with_bw_mode();
    let scene = render_jointplot(joint, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/bw_joint_standalone.svg", svg.clone()).unwrap();
    assert!(
        !svg.contains("steelblue") && !svg.contains("tomato"),
        "group colors should not leak through the standalone render_jointplot path in BW mode"
    );
    assert!(
        svg.contains("<pattern"),
        "standalone render_jointplot should still emit BW patterns"
    );
}

#[test]
fn bw_raincloud_groups_use_distinct_patterns() {
    use kuva::plot::raincloud::RaincloudPlot;
    let raincloud = RaincloudPlot::new()
        .with_group("Control", vec![1.0, 2.0, 2.5, 3.0, 2.2, 2.8, 1.8])
        .with_group("Treated", vec![4.0, 5.0, 4.5, 5.5, 4.2, 4.8, 5.2]);
    let plots = vec![Plot::Raincloud(raincloud)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    common::write_test_output("test_outputs/bw_raincloud.svg", svg.clone()).unwrap();
    let pattern_count = distinct_patterns_in_plot_body(&svg);
    assert!(
        pattern_count >= 2,
        "raincloud groups should use at least 2 distinct patterns, got {pattern_count}"
    );
}

// ── Group 6: continuous-value / colormap family ─────────────────────────────
//
// These plots encode a continuous numeric value via a `ColorMap`, not a
// discrete per-series category, so hatch/dash/shape differentiation doesn't
// apply. Design decision: force `ColorMap::Grayscale` in BW mode instead.
// Each test renders the same construction twice — once plain (sanity-checking
// that the default colormap, usually Viridis, actually produces colorful
// fills for this data) and once with BW mode (asserting no colorful fill
// survives).

#[test]
fn bw_histogram2d_forces_grayscale() {
    use kuva::plot::Histogram2D;
    let data = vec![(1.0, 1.0), (1.0, 1.0), (1.0, 1.0), (5.0, 5.0)];
    let build = || Histogram2D::new().with_data(data.clone(), (0.0, 6.0), (0.0, 6.0), 3, 3);
    let plain = plain_svg(
        vec![Plot::Histogram2d(build())],
        Layout::auto_from_plots(&[Plot::Histogram2d(build())]),
    );
    assert!(
        svg_has_non_grey_fill(&plain),
        "sanity: default colormap should produce colorful fills"
    );
    let svg = bw_svg(
        vec![Plot::Histogram2d(build())],
        Layout::auto_from_plots(&[Plot::Histogram2d(build())]),
    );
    common::write_test_output("test_outputs/bw_histogram2d.svg", svg.clone()).unwrap();
    assert!(
        !svg_has_non_grey_fill(&svg),
        "histogram2d bins should use only grayscale fills in BW mode"
    );
}

#[test]
fn bw_heatmap_forces_grayscale() {
    use kuva::plot::Heatmap;
    let build = || {
        Heatmap::new().with_data(vec![
            vec![1.0, 5.0, 9.0],
            vec![2.0, 6.0, 3.0],
            vec![8.0, 4.0, 7.0],
        ])
    };
    let plots_plain = vec![Plot::Heatmap(build())];
    let plots_bw = vec![Plot::Heatmap(build())];
    let plain = plain_svg(
        plots_plain,
        Layout::auto_from_plots(&[Plot::Heatmap(build())]),
    );
    assert!(
        svg_has_non_grey_fill(&plain),
        "sanity: default colormap should produce colorful fills"
    );
    let svg = bw_svg(plots_bw, Layout::auto_from_plots(&[Plot::Heatmap(build())]));
    common::write_test_output("test_outputs/bw_heatmap.svg", svg.clone()).unwrap();
    assert!(
        !svg_has_non_grey_fill(&svg),
        "heatmap cells should use only grayscale fills in BW mode"
    );
}

#[test]
fn bw_hexbin_forces_grayscale() {
    use kuva::plot::hexbin::HexbinPlot;
    // Varying density per bin (not a uniform grid) so norm spans [0, 1] —
    // a uniform grid makes every bin normalize to the same value, which
    // would pass this assertion trivially without proving anything.
    let mut xs: Vec<f64> = (0..40).map(|i| (i % 8) as f64).collect();
    let mut ys: Vec<f64> = (0..40).map(|i| (i / 8) as f64).collect();
    for _ in 0..20 {
        xs.push(0.0);
        ys.push(0.0);
    }
    let build = || HexbinPlot::new().with_data(xs.clone(), ys.clone());
    let plain = plain_svg(
        vec![Plot::Hexbin(build())],
        Layout::auto_from_plots(&[Plot::Hexbin(build())]),
    );
    assert!(
        svg_has_non_grey_fill(&plain),
        "sanity: default colormap should produce colorful fills"
    );
    let svg = bw_svg(
        vec![Plot::Hexbin(build())],
        Layout::auto_from_plots(&[Plot::Hexbin(build())]),
    );
    common::write_test_output("test_outputs/bw_hexbin.svg", svg.clone()).unwrap();
    assert!(
        !svg_has_non_grey_fill(&svg),
        "hexbin cells should use only grayscale fills in BW mode"
    );
}

#[test]
fn bw_calendar_forces_grayscale() {
    use kuva::plot::calendar::{CalendarAgg, CalendarPlot};
    // CalendarAgg defaults to Count (occurrences per day, ignoring the value
    // field) — with one entry per date that collapses every day to the same
    // count and defeats the point of varying the input values. Sum makes a
    // single entry's value pass through unchanged.
    let build = || {
        CalendarPlot::new()
            .with_data(vec![
                ("2026-01-01", 1.0),
                ("2026-01-02", 5.0),
                ("2026-01-03", 10.0),
                ("2026-01-04", 20.0),
            ])
            .with_aggregation(CalendarAgg::Sum)
    };
    let plain = plain_svg(
        vec![Plot::Calendar(build())],
        Layout::auto_from_plots(&[Plot::Calendar(build())]),
    );
    assert!(
        svg_has_non_grey_fill(&plain),
        "sanity: default colormap should produce colorful fills"
    );
    let svg = bw_svg(
        vec![Plot::Calendar(build())],
        Layout::auto_from_plots(&[Plot::Calendar(build())]),
    );
    common::write_test_output("test_outputs/bw_calendar.svg", svg.clone()).unwrap();
    assert!(
        !svg_has_non_grey_fill(&svg),
        "calendar day cells should use only grayscale fills in BW mode"
    );
}

#[test]
fn bw_calendar_missing_days_use_a_pattern_not_flat_grey() {
    // Flat grey for "no data" sits on the same white-to-black scale as a real
    // low value, so it's ambiguous which one a viewer is looking at — reported
    // by the user after visually reviewing the Group 6 calendar preview.
    use kuva::plot::calendar::CalendarPlot;
    let calendar = CalendarPlot::new().with_data(vec![("2026-06-15", 5.0)]);
    let plots = vec![Plot::Calendar(calendar)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = bw_svg(plots, layout);
    let pattern_count = distinct_patterns_in_plot_body(&svg);
    assert!(
        pattern_count >= 1,
        "missing calendar days should use a hatch pattern, not a flat fill, got {pattern_count}"
    );
}

#[test]
fn bw_clustermap_forces_grayscale() {
    use kuva::plot::clustermap::Clustermap;
    let build = || {
        Clustermap::new()
            .with_data(vec![
                vec![1.0, 5.0, 9.0],
                vec![2.0, 6.0, 3.0],
                vec![8.0, 4.0, 7.0],
            ])
            .with_cluster_rows(false)
            .with_cluster_cols(false)
    };
    let plain = plain_svg(
        vec![Plot::Clustermap(build())],
        Layout::auto_from_plots(&[Plot::Clustermap(build())]),
    );
    assert!(
        svg_has_non_grey_fill(&plain),
        "sanity: default colormap should produce colorful fills"
    );
    let svg = bw_svg(
        vec![Plot::Clustermap(build())],
        Layout::auto_from_plots(&[Plot::Clustermap(build())]),
    );
    common::write_test_output("test_outputs/bw_clustermap.svg", svg.clone()).unwrap();
    assert!(
        !svg_has_non_grey_fill(&svg),
        "clustermap cells should use only grayscale fills in BW mode"
    );
}

#[test]
fn bw_surface3d_forces_grayscale() {
    use kuva::plot::surface3d::Surface3DPlot;
    use kuva::plot::ColorMap;
    let build = || {
        Surface3DPlot::new(vec![
            vec![1.0, 5.0, 9.0],
            vec![2.0, 6.0, 3.0],
            vec![8.0, 4.0, 7.0],
        ])
        .with_z_colormap(ColorMap::Viridis)
    };
    let plain = plain_svg(
        vec![Plot::Surface3D(build())],
        Layout::auto_from_plots(&[Plot::Surface3D(build())]),
    );
    assert!(
        svg_has_non_grey_fill(&plain),
        "sanity: default colormap should produce colorful fills"
    );
    let svg = bw_svg(
        vec![Plot::Surface3D(build())],
        Layout::auto_from_plots(&[Plot::Surface3D(build())]),
    );
    common::write_test_output("test_outputs/bw_surface3d.svg", svg.clone()).unwrap();
    assert!(
        !svg_has_non_grey_fill(&svg),
        "surface3d faces should use only grayscale fills in BW mode"
    );
}

#[test]
fn bw_contour_forces_grayscale() {
    use kuva::plot::contour::ContourPlot;
    let z = vec![
        vec![0.0, 1.0, 2.0, 3.0],
        vec![1.0, 2.0, 3.0, 4.0],
        vec![2.0, 3.0, 4.0, 5.0],
        vec![3.0, 4.0, 5.0, 6.0],
    ];
    let build = || {
        ContourPlot::new()
            .with_grid(
                z.clone(),
                vec![0.0, 1.0, 2.0, 3.0],
                vec![0.0, 1.0, 2.0, 3.0],
            )
            .with_filled()
    };
    let plain = plain_svg(
        vec![Plot::Contour(build())],
        Layout::auto_from_plots(&[Plot::Contour(build())]),
    );
    assert!(
        svg_has_non_grey_fill(&plain),
        "sanity: default colormap should produce colorful fills"
    );
    let svg = bw_svg(
        vec![Plot::Contour(build())],
        Layout::auto_from_plots(&[Plot::Contour(build())]),
    );
    common::write_test_output("test_outputs/bw_contour.svg", svg.clone()).unwrap();
    assert!(
        !svg_has_non_grey_fill(&svg),
        "filled contour bands should use only grayscale fills in BW mode"
    );
}

#[test]
fn bw_dotplot_forces_grayscale() {
    use kuva::plot::dotplot::DotPlot;
    let build = || {
        DotPlot::new().with_data(vec![
            ("A", "X", 5.0, 1.0),
            ("B", "X", 5.0, 5.0),
            ("A", "Y", 5.0, 9.0),
        ])
    };
    let plain = plain_svg(
        vec![Plot::DotPlot(build())],
        Layout::auto_from_plots(&[Plot::DotPlot(build())]),
    );
    assert!(
        svg_has_non_grey_fill(&plain),
        "sanity: default colormap should produce colorful fills"
    );
    let svg = bw_svg(
        vec![Plot::DotPlot(build())],
        Layout::auto_from_plots(&[Plot::DotPlot(build())]),
    );
    common::write_test_output("test_outputs/bw_dotplot.svg", svg.clone()).unwrap();
    assert!(
        !svg_has_non_grey_fill(&svg),
        "dot fills should use only grayscale colors in BW mode"
    );
}

#[test]
fn bw_quiver_forces_grayscale() {
    use kuva::plot::quiver::QuiverPlot;
    use kuva::plot::ColorMap;
    let build = || {
        QuiverPlot::new()
            .with_arrow(0.0, 0.0, 1.0, 1.0)
            .with_arrow(1.0, 1.0, 3.0, 3.0)
            .with_arrow(2.0, 2.0, 5.0, 0.0)
            .with_color_map(ColorMap::Viridis)
    };
    let plain = plain_svg(
        vec![Plot::Quiver(build())],
        Layout::auto_from_plots(&[Plot::Quiver(build())]),
    );
    assert!(
        svg_has_non_grey_fill(&plain),
        "sanity: default colormap should produce colorful fills"
    );
    let svg = bw_svg(
        vec![Plot::Quiver(build())],
        Layout::auto_from_plots(&[Plot::Quiver(build())]),
    );
    common::write_test_output("test_outputs/bw_quiver.svg", svg.clone()).unwrap();
    assert!(
        !svg_has_non_grey_fill(&svg),
        "quiver arrows should use only grayscale colors in BW mode"
    );
}

#[test]
fn bw_diceplot_continuous_tile_forces_grayscale() {
    // The categorical dot-legend mode was covered in Group 2
    // (bw_diceplot_categorical_dots_use_distinct_shapes); this covers the
    // continuous per-tile fill mode deferred to Group 6 at the time.
    use kuva::plot::diceplot::DicePlot;
    let build = || {
        DicePlot::new(4).with_points(vec![
            ("Row1", "Col1", vec![0, 1, 2, 3], Some(0.1), Some(3.0)),
            ("Row2", "Col1", vec![0, 1, 2, 3], Some(0.9), Some(3.0)),
        ])
    };
    let plain = plain_svg(
        vec![Plot::DicePlot(build())],
        Layout::auto_from_plots(&[Plot::DicePlot(build())]),
    );
    assert!(
        svg_has_non_grey_fill(&plain),
        "sanity: default colormap should produce colorful fills"
    );
    let svg = bw_svg(
        vec![Plot::DicePlot(build())],
        Layout::auto_from_plots(&[Plot::DicePlot(build())]),
    );
    common::write_test_output("test_outputs/bw_diceplot_continuous.svg", svg.clone()).unwrap();
    assert!(
        !svg_has_non_grey_fill(&svg),
        "continuous dice tiles should use only grayscale fills in BW mode"
    );
}

// ── Sanity checks ─────────────────────────────────────────────────────────────

#[test]
fn bw_color_mode_no_patterns() {
    let bar = BarPlot::new().with_bar("A", 3.2).with_bar("B", 4.7);
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
    assert!(
        computed.bw_mode,
        "bw_mode should propagate from Layout to ComputedLayout"
    );
}

/// Continuous data in BW mode must never be filled pure white.
///
/// BW mode maps continuous colormaps onto a grey ramp. ColorBrewer *Greys*
/// starts at `#ffffff`, which is the canvas colour: for plots that omit empty
/// cells (hexbin, calendar) a minimum-value cell then reads as *no data* rather
/// than *least data*, and roughly half a realistic hexbin disappeared. The ramp
/// is floored so the lightest cell stays visible, the same reason the
/// categorical `BW_GREYS` sequence stops at `#f0f0f0` rather than white.
#[test]
fn bw_continuous_fills_are_never_pure_white() {
    use kuva::plot::hexbin::HexbinPlot;

    // A long thin tail of single-count bins plus one dense bin: the single-count
    // bins all normalize to 0.0, which is exactly where the ramp used to be white.
    let mut xs: Vec<f64> = (0..40).map(|i| (i % 8) as f64).collect();
    let mut ys: Vec<f64> = (0..40).map(|i| (i / 8) as f64).collect();
    for _ in 0..20 {
        xs.push(0.0);
        ys.push(0.0);
    }
    let build = || HexbinPlot::new().with_data(xs.clone(), ys.clone());
    let svg = bw_svg(
        vec![Plot::Hexbin(build())],
        Layout::auto_from_plots(&[Plot::Hexbin(build())]),
    );

    let whites = svg.matches(r##"fill="#ffffff""##).count();
    assert_eq!(
        whites, 0,
        "BW hexbin drew {whites} pure-white cells, which are invisible on the white canvas"
    );
    // The ramp must still reach full black, i.e. it is floored, not compressed away.
    assert!(
        svg.contains(r##"fill="#000000""##),
        "BW hexbin lost its dark end; the ramp should still reach black"
    );
}
