#![cfg(feature = "pdf")]

use visus::plot::scatter::ScatterPlot;
use visus::plot::LinePlot;
use visus::plot::BarPlot;
use visus::plot::Histogram;
use visus::PdfBackend;
use visus::render::render::render_scatter;
use visus::render::layout::Layout;
use visus::render::plots::Plot;
use visus::render::figure::Figure;
use visus::render::annotations::{TextAnnotation, ReferenceLine, ShadedRegion};

fn make_scatter_scene() -> visus::render::render::Scene {
    let data = vec![(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)];
    let plot = ScatterPlot::new()
        .with_data(data)
        .with_color("steelblue");
    let layout = Layout::new((0.0, 6.0), (0.0, 8.0))
        .with_title("PDF test");
    render_scatter(&plot, layout).with_background(Some("white"))
}

#[test]
fn pdf_scatter_basic() {
    let scene = make_scatter_scene();
    let result = PdfBackend::new().render_scene(&scene);
    assert!(result.is_ok(), "render_scene failed: {:?}", result.err());
    let bytes = result.unwrap();
    assert_eq!(&bytes[..5], b"%PDF-", "output is not a valid PDF");
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/pdf_scatter.pdf", &bytes).unwrap();
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
        .with_data(vec![(1.0, 2.0), (2.0, 4.5), (3.0, 3.2), (4.0, 5.8), (5.0, 4.1)])
        .with_color("steelblue")
        .with_size(5.0)
        .with_legend("Control");
    let series2 = ScatterPlot::new()
        .with_data(vec![(1.0, 3.5), (2.0, 6.0), (3.0, 5.1), (4.0, 8.2), (5.0, 7.0)])
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
        1.2, 1.5, 1.8, 2.1, 2.3, 2.5, 2.6, 2.8, 2.9, 3.0,
        3.1, 3.3, 3.5, 3.7, 4.0, 4.2, 4.5, 4.8, 5.0, 5.3,
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
    let bytes = PdfBackend::new().render_scene(&scene).expect("PDF render failed");

    assert_eq!(&bytes[..5], b"%PDF-", "output is not a valid PDF");
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/pdf_rich_figure.pdf", &bytes).unwrap();
}
