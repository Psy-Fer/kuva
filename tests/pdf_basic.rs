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
fn render_scenes_rejects_empty_input() {
    let result = PdfBackend::new().render_scenes(&[]);
    assert!(result.is_err(), "empty scene list should be an error");
}

#[test]
fn single_scene_matches_render_scene() {
    let scene = make_scatter_scene();
    let single = PdfBackend::new().render_scene(&scene).unwrap();
    let multi = PdfBackend::new()
        .render_scenes(std::slice::from_ref(&scene))
        .unwrap();
    assert_eq!(
        single, multi,
        "render_scenes of one natural-sized scene should equal render_scene"
    );
}

#[test]
fn two_scenes_produce_a_two_page_pdf() {
    let a = make_scatter_scene();
    let b = make_scatter_scene();
    let bytes = PdfBackend::new()
        .render_scenes(&[a, b])
        .expect("render_scenes failed");
    assert_eq!(&bytes[..5], b"%PDF-", "output is not a valid PDF");
    assert_eq!(page_count(&bytes), 2, "expected two pages");
    common::write_test_output("test_outputs/pdf_multipage_two.pdf", &bytes).unwrap();
}

#[test]
fn three_scenes_produce_a_three_page_pdf() {
    let a = make_scatter_scene();
    let b = make_scatter_scene();
    let c = make_scatter_scene();
    let bytes = PdfBackend::new().render_scenes(&[a, b, c]).unwrap();
    assert_eq!(page_count(&bytes), 3, "expected three pages");
}

#[test]
fn fixed_page_size_coerces_every_page_to_us_letter() {
    // Two scenes of different natural sizes, both coerced onto 11×8.5" pages.
    let wide = make_scatter_scene();
    let mut tall = make_scatter_scene();
    tall.height *= 2.0;

    let bytes = PdfBackend::new()
        .with_page_size(PageSize::inches(11.0, 8.5))
        .render_scenes(&[wide, tall])
        .expect("fixed-size render failed");

    assert_eq!(&bytes[..5], b"%PDF-", "output is not a valid PDF");
    assert_eq!(page_count(&bytes), 2, "expected two pages");
    // 11×8.5in = 792×612pt; both pages share that media box rect. The literal
    // form (no decimal point) relies on pdf-writer's integer-valued-float
    // formatting; pdf-writer is version-pinned, so this won't shift underfoot.
    assert_eq!(
        count(&bytes, b"[0 0 792 612]"),
        2,
        "both pages should be exactly US Letter landscape"
    );
    common::write_test_output("test_outputs/pdf_multipage_letter.pdf", &bytes).unwrap();
}

#[test]
fn render_to_pdf_multi_smoke() {
    let page1 = vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(vec![(1.0, 2.0), (3.0, 4.0)])
            .with_color("steelblue"),
    )];
    let page2 = vec![Plot::Bar(
        BarPlot::new().with_bar("A", 3.0).with_bar("B", 5.0),
    )];
    // page2's layout must be derived from page2's own plots (auto-ranged),
    // computed before page2 is moved into the pages vec.
    let layout1 = Layout::new((0.0, 6.0), (0.0, 8.0)).with_title("Page 1");
    let layout2 = Layout::auto_from_plots(&page2).with_title("Page 2");
    let bytes = kuva::render_to_pdf_multi(vec![(page1, layout1), (page2, layout2)])
        .expect("render_to_pdf_multi failed");
    assert_eq!(&bytes[..5], b"%PDF-");
    assert_eq!(page_count(&bytes), 2);
}

#[test]
fn natural_pages_preserve_each_scene_size_and_placement() {
    // Distinct, clean dimensions so both the MediaBox and the content-stream
    // placement matrix are exactly predictable.
    let mut a = make_scatter_scene();
    a.width = 320.0;
    a.height = 240.0;
    let mut b = make_scatter_scene();
    b.width = 500.0;
    b.height = 400.0;

    let bytes = PdfBackend::new().render_scenes(&[a, b]).unwrap();
    assert_eq!(page_count(&bytes), 2, "expected two pages");
    // Each page takes its own scene's size, so pages differ in size.
    assert!(contains(&bytes, b"[0 0 320 240]"), "page A media box");
    assert!(contains(&bytes, b"[0 0 500 400]"), "page B media box");
    // The 1×1-pt SVG XObject is scaled to the full scene size with no offset.
    assert!(
        contains(&bytes, b"320 0 0 240 0 0 cm"),
        "page A placement matrix"
    );
    assert!(
        contains(&bytes, b"500 0 0 400 0 0 cm"),
        "page B placement matrix"
    );
}

#[test]
fn fixed_page_fill_uses_scene_background_color() {
    // A non-white, RGB-resolvable background so the fill differs from the white
    // fallback — proves the fill color is read from the scene, not defaulted.
    let mut scene = make_scatter_scene();
    scene.background_color = Some("#ff0000".to_string());

    let bytes = PdfBackend::new()
        .with_page_size(PageSize::inches(8.5, 11.0))
        .render_scenes(&[scene])
        .unwrap();

    assert!(
        contains(&bytes, b"1 0 0 rg"),
        "letterbox fill should use the scene's red background, not white"
    );
}

#[test]
fn points_constructor_sets_exact_page_size() {
    let scene = make_scatter_scene();
    let bytes = PdfBackend::new()
        .with_page_size(PageSize::points(300.0, 400.0))
        .render_scenes(&[scene])
        .unwrap();
    assert!(
        contains(&bytes, b"[0 0 300 400]"),
        "media box should be 300×400 pt"
    );
}

#[test]
fn fixed_page_size_rejects_degenerate_dimensions() {
    let scene = make_scatter_scene();
    // Zero, negative, and non-finite page dimensions must error rather than
    // emit a corrupt PDF. Built via the `Fixed` literal to bypass the
    // debug-only assertion in the points/inches constructors.
    for bad in [
        PageSize::Fixed {
            width: 0.0,
            height: 400.0,
        },
        PageSize::Fixed {
            width: -300.0,
            height: 400.0,
        },
        PageSize::Fixed {
            width: f64::NAN,
            height: 400.0,
        },
    ] {
        let result = PdfBackend::new()
            .with_page_size(bad)
            .render_scenes(std::slice::from_ref(&scene));
        assert!(result.is_err(), "degenerate page size {bad:?} should error");
    }
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
