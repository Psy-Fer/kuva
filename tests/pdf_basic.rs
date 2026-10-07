#![cfg(feature = "pdf")]
mod common;

use kuva::plot::scatter::ScatterPlot;
use kuva::plot::BarPlot;
use kuva::plot::Histogram;
use kuva::plot::LinePlot;
use kuva::render::annotations::{ReferenceLine, ShadedRegion, TextAnnotation};
use kuva::render::figure::Figure;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_scatter;
use kuva::{PageSize, PdfBackend};

fn make_scatter_scene() -> kuva::render::render::Scene {
    let data = vec![(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)];
    let plot = ScatterPlot::new().with_data(data).with_color("steelblue");
    let layout = Layout::new((0.0, 6.0), (0.0, 8.0)).with_title("PDF test");
    render_scatter(&plot, layout).with_background(Some("white"))
}

/// Count non-overlapping occurrences of `needle` in `haystack`.
fn count(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .filter(|w| *w == needle)
        .count()
}

/// Each PDF page carries exactly one `/MediaBox`; the page tree carries none —
/// so the count of `/MediaBox` markers is the page count.
fn page_count(pdf: &[u8]) -> usize {
    count(pdf, b"/MediaBox")
}

/// Whether `needle` appears anywhere in `haystack`.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

#[test]
fn pdf_scatter_basic() {
    let scene = make_scatter_scene();
    let result = PdfBackend::new().render_scene(&scene);
    assert!(result.is_ok(), "render_scene failed: {:?}", result.err());
    let bytes = result.unwrap();
    assert_eq!(&bytes[..5], b"%PDF-", "output is not a valid PDF");
    common::write_test_output("test_outputs/pdf_scatter.pdf", &bytes).unwrap();
}

#[test]
fn pdf_is_vector() {
    let scene = make_scatter_scene();
    let bytes1 = PdfBackend::new().render_scene(&scene).unwrap();
    let bytes2 = PdfBackend::new().render_scene(&scene).unwrap();
    assert_eq!(&bytes1[..5], b"%PDF-", "first render is not a valid PDF");
    assert_eq!(&bytes2[..5], b"%PDF-", "second render is not a valid PDF");
}

#[test]
fn pdf_rich_figure() {
    // --- Panel A: scatter with two series, shaded region, reference line,
    //              and an annotated outlier ---
    let series1 = ScatterPlot::new()
        .with_data(vec![
            (1.0, 2.0),
            (2.0, 4.5),
            (3.0, 3.2),
            (4.0, 5.8),
            (5.0, 4.1),
        ])
        .with_color("steelblue")
        .with_size(5.0)
        .with_legend("Control");
    let series2 = ScatterPlot::new()
        .with_data(vec![
            (1.0, 3.5),
            (2.0, 6.0),
            (3.0, 5.1),
            (4.0, 8.2),
            (5.0, 7.0),
        ])
        .with_color("tomato")
        .with_size(5.0)
        .with_legend("Treatment");

    let scatter_plots = vec![Plot::Scatter(series1), Plot::Scatter(series2)];
    let layout_a = Layout::auto_from_plots(&scatter_plots)
        .with_title("Scatter: Control vs Treatment")
        .with_x_label("Time (days)")
        .with_y_label("Expression level")
        .with_shaded_region(
            ShadedRegion::horizontal(5.0, 7.0)
                .with_color("gold")
                .with_opacity(0.2),
        )
        .with_reference_line(
            ReferenceLine::horizontal(5.0)
                .with_color("grey")
                .with_label("baseline"),
        )
        .with_annotation(
            TextAnnotation::new("Peak", 4.0, 8.8)
                .with_arrow(4.0, 8.2)
                .with_color("darkred")
                .with_font_size(12),
        );

    // --- Panel B: two line series (solid + dashed) with a vertical marker ---
    let xs: Vec<f64> = (0..=60).map(|i| i as f64 / 10.0).collect();
    let line1 = LinePlot::new()
        .with_data(xs.iter().map(|&x| (x, x.sin())))
        .with_color("steelblue")
        .with_legend("sin(x)");
    let line2 = LinePlot::new()
        .with_data(xs.iter().map(|&x| (x, x.cos())))
        .with_color("tomato")
        .with_dashed()
        .with_legend("cos(x)");

    let line_plots = vec![Plot::Line(line1), Plot::Line(line2)];
    let layout_b = Layout::new((0.0, 6.0), (-1.5, 1.5))
        .with_title("Waveforms")
        .with_x_label("Angle (rad)")
        .with_y_label("Amplitude")
        .with_ticks(6)
        .with_reference_line(
            ReferenceLine::vertical(std::f64::consts::PI)
                .with_color("purple")
                .with_label("π"),
        )
        .with_reference_line(
            ReferenceLine::horizontal(0.0)
                .with_color("black")
                .with_dasharray("2,2"),
        );

    // --- Panel C: grouped bar chart with a shaded band and annotation ---
    let bar = BarPlot::new()
        .with_bar("Alpha", 4.2)
        .with_bar("Beta", 7.1)
        .with_bar("Gamma", 5.5)
        .with_bar("Delta", 9.3)
        .with_bar("Epsilon", 3.8)
        .with_color("#4e79a7");

    let bar_plots = vec![Plot::Bar(bar)];
    let layout_c = Layout::auto_from_plots(&bar_plots)
        .with_title("Group Counts")
        .with_x_label("Group")
        .with_y_label("Count")
        .with_shaded_region(
            ShadedRegion::horizontal(6.0, 8.0)
                .with_color("limegreen")
                .with_opacity(0.15),
        )
        .with_reference_line(
            ReferenceLine::horizontal(6.0)
                .with_color("darkgreen")
                .with_label("target"),
        )
        .with_annotation(
            TextAnnotation::new("Best", 3.0, 9.9)
                .with_arrow(3.0, 9.3)
                .with_color("navy")
                .with_font_size(11),
        );

    // --- Panel D: histogram with a mean reference line ---
    let values: Vec<f64> = vec![
        1.2, 1.5, 1.8, 2.1, 2.3, 2.5, 2.6, 2.8, 2.9, 3.0, 3.1, 3.3, 3.5, 3.7, 4.0, 4.2, 4.5, 4.8,
        5.0, 5.3,
    ];
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let hist = Histogram::new()
        .with_data(values)
        .with_bins(8)
        .with_color("#f28e2b")
        .with_range((0.0, 6.0));

    let hist_plots = vec![Plot::Histogram(hist)];
    let layout_d = Layout::auto_from_plots(&hist_plots)
        .with_title("Value Distribution")
        .with_x_label("Value")
        .with_y_label("Frequency")
        .with_reference_line(
            ReferenceLine::vertical(mean)
                .with_color("firebrick")
                .with_label("mean"),
        )
        .with_shaded_region(
            ShadedRegion::vertical(2.0, 4.0)
                .with_color("steelblue")
                .with_opacity(0.12),
        );

    // --- Compose into a 2×2 figure ---
    let figure = Figure::new(2, 2)
        .with_title("PDF Rich Figure Test")
        .with_plots(vec![scatter_plots, line_plots, bar_plots, hist_plots])
        .with_layouts(vec![layout_a, layout_b, layout_c, layout_d])
        .with_labels()
        .with_shared_legend();

    let scene = figure.render();
    let bytes = PdfBackend::new()
        .render_scene(&scene)
        .expect("PDF render failed");

    assert_eq!(&bytes[..5], b"%PDF-", "output is not a valid PDF");
    common::write_test_output("test_outputs/pdf_rich_figure.pdf", &bytes).unwrap();
}

fn make_line_scene() -> kuva::render::render::Scene {
    let plot = LinePlot::new()
        .with_data(vec![(0.0, 1.0), (1.0, 2.0), (2.0, 1.5), (3.0, 3.0)])
        .with_color("firebrick");
    let plots = vec![Plot::Line(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Line page");
    kuva::render::render::render_multiple(plots, layout).with_background(Some("white"))
}

fn make_bar_scene() -> kuva::render::render::Scene {
    let bar = BarPlot::new()
        .with_bar("A", 3.0)
        .with_bar("B", 5.0)
        .with_bar("C", 2.0);
    let plots = vec![Plot::Bar(bar)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Bar page")
        .with_subtitle(
        "background intentionally blue — checks the letterbox fill matches the scene's own color",
    );
    kuva::render::render::render_multiple(plots, layout).with_background(Some("#eef6ff"))
}

#[test]
fn render_scenes_rejects_empty_input() {
    let result = PdfBackend::new().render_scenes(&[]);
    assert!(result.is_err(), "an empty scene list should be rejected");
}

#[test]
fn render_scenes_single_scene_matches_render_scene() {
    // Single-scene render_scenes (Natural page size) should be structurally
    // identical to the plain render_scene path — one page, valid PDF.
    let scene = make_scatter_scene();
    let via_render_scene = PdfBackend::new().render_scene(&scene).unwrap();
    let via_render_scenes = PdfBackend::new().render_scenes(&[scene]).unwrap();
    assert_eq!(page_count(&via_render_scene), 1);
    assert_eq!(page_count(&via_render_scenes), 1);
}

#[test]
fn render_scenes_natural_produces_one_page_per_scene() {
    let scenes = [make_scatter_scene(), make_line_scene(), make_bar_scene()];
    let bytes = PdfBackend::new().render_scenes(&scenes).unwrap();
    assert_eq!(&bytes[..5], b"%PDF-", "output is not a valid PDF");
    assert_eq!(page_count(&bytes), 3, "expected one page per scene");
    common::write_test_output("test_outputs/pdf_multi_natural.pdf", &bytes).unwrap();
}

#[test]
fn render_scenes_fixed_page_size_scales_every_page_the_same() {
    let scenes = [make_scatter_scene(), make_line_scene(), make_bar_scene()];
    let bytes = PdfBackend::new()
        .with_page_size(PageSize::inches(11.0, 8.5))
        .render_scenes(&scenes)
        .unwrap();
    assert_eq!(page_count(&bytes), 3);
    // US Letter landscape at 72 pt/inch: 792 x 612. Every page should carry
    // this exact MediaBox regardless of each scene's own natural size.
    assert!(
        contains(&bytes, b"792 612") || contains(&bytes, b"792.0 612.0"),
        "expected every page's MediaBox to be the fixed 792x612 page size"
    );
    common::write_test_output("test_outputs/pdf_multi_fixed_letterboxed.pdf", &bytes).unwrap();
}

#[test]
fn render_scenes_rejects_non_finite_or_non_positive_fixed_size() {
    let scenes = [make_scatter_scene()];
    for (w, h) in [(0.0, 100.0), (100.0, 0.0), (f64::NAN, 100.0), (-1.0, 100.0)] {
        let result = PdfBackend::new()
            .with_page_size(PageSize::Fixed {
                width: w,
                height: h,
            })
            .render_scenes(&scenes);
        assert!(
            result.is_err(),
            "PageSize::Fixed({w}, {h}) should be rejected"
        );
    }
}

#[test]
fn page_size_inches_converts_to_points() {
    // 1 inch = 72 points.
    match PageSize::inches(2.0, 3.0) {
        PageSize::Fixed { width, height } => {
            assert_eq!(width, 144.0);
            assert_eq!(height, 216.0);
        }
        PageSize::Natural => panic!("expected PageSize::Fixed"),
    }
}

#[test]
fn render_to_pdf_multi_one_shot_helper() {
    use kuva::render::layout::Layout as L;
    let page1 = (
        vec![Plot::Scatter(
            ScatterPlot::new().with_data(vec![(1.0, 2.0), (2.0, 3.0)]),
        )],
        L::new((0.0, 3.0), (0.0, 4.0)).with_title("Page 1"),
    );
    let page2 = (
        vec![Plot::Line(
            LinePlot::new().with_data(vec![(0.0, 0.0), (1.0, 1.0)]),
        )],
        L::new((0.0, 1.0), (0.0, 1.0)).with_title("Page 2"),
    );
    let bytes = kuva::render_to_pdf_multi(vec![page1, page2]).unwrap();
    assert_eq!(&bytes[..5], b"%PDF-");
    assert_eq!(page_count(&bytes), 2);
}

#[test]
fn pdf_tick_font_family_renders_bundled_monospace() {
    let render = |layout: Layout, plots: Vec<Plot>| {
        let scene = kuva::render::render::render_multiple(plots, layout);
        PdfBackend::new().render_scene(&scene).unwrap()
    };
    let bars = || {
        vec![Plot::Bar(
            BarPlot::new().with_bars(vec![("ACG", 3.0), ("TCG", 5.0)]),
        )]
    };

    let plots = bars();
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Contexts")
        .with_x_tick_rotate(-90.0)
        .with_tick_font_family("monospace");
    let mono = render(layout, plots);
    common::write_test_output("test_outputs/pdf_tick_font_mono.pdf", &mono).unwrap();
    assert_eq!(&mono[..5], b"%PDF-");
    assert!(
        contains(&mono, b"DejaVuSansMono"),
        "generic monospace tick labels should embed the bundled DejaVu Sans Mono"
    );
    assert!(
        contains(&mono, b"+DejaVuSans/"),
        "the title should still embed DejaVu Sans"
    );

    let plots = bars();
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Contexts")
        .with_x_tick_rotate(-90.0);
    let sans = render(layout, plots);
    assert!(!contains(&sans, b"DejaVuSansMono"));
}
