mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::{Histogram, ScatterPlot, TextAlign, TextPlot};
use kuva::render::figure::Figure;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

fn svg(plots: Vec<Plot>, layout: Layout) -> String {
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

fn text_svg(tp: TextPlot) -> String {
    let plots = vec![Plot::Text(tp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_width(400.0)
        .with_height(200.0);
    svg(plots, layout)
}

#[test]
fn test_text_basic_renders() {
    let t = TextPlot::new().with_body("Hello world.");
    let out = text_svg(t);
    assert!(out.contains("<svg"));
    assert!(out.contains("Hello world."));
}

#[test]
fn test_text_with_title() {
    let t = TextPlot::new()
        .with_title("My Title")
        .with_body("Body text here.");
    let out = text_svg(t);
    assert!(out.contains("My Title"));
    assert!(out.contains("Body text here."));
}

#[test]
fn test_text_inline_bold() {
    let t = TextPlot::new().with_body("Normal **bold** normal.");
    let out = text_svg(t);
    assert!(out.contains("font-weight=\"bold\""));
    assert!(out.contains("bold"));
    assert!(!out.contains("**"));
}

#[test]
fn test_text_inline_italic() {
    let t = TextPlot::new().with_body("Normal *italic* normal.");
    let out = text_svg(t);
    assert!(out.contains("font-style=\"italic\""));
    assert!(!out.contains('*'));
}

#[test]
fn test_text_inline_underline() {
    let t = TextPlot::new().with_body("Normal __underline__ normal.");
    let out = text_svg(t);
    assert!(out.contains("text-decoration=\"underline\""));
    assert!(!out.contains("__"));
}

#[test]
fn test_text_inline_mixed() {
    let t = TextPlot::new().with_body("**Bold:** *italic* and __underlined__ text.");
    let out = text_svg(t);
    assert!(out.contains("font-weight=\"bold\""));
    assert!(out.contains("font-style=\"italic\""));
    assert!(out.contains("text-decoration=\"underline\""));
    assert!(!out.contains("**"));
    assert!(!out.contains("__"));
}

#[test]
fn test_text_bold_label_colon_pattern() {
    // The common "**Label:** rest of sentence" pattern
    let t = TextPlot::new().with_body("**Cohort:** 240 participants across three groups.");
    let out = text_svg(t);
    assert!(out.contains("font-weight=\"bold\""));
    assert!(out.contains("Cohort:"));
    assert!(out.contains("240 participants"));
    assert!(!out.contains("**"));
}

#[test]
fn test_text_heading1() {
    let t = TextPlot::new().with_body("# Section Heading");
    let out = text_svg(t);
    assert!(out.contains("Section Heading"));
}

#[test]
fn test_text_heading2() {
    let t = TextPlot::new().with_body("## Subsection");
    let out = text_svg(t);
    assert!(out.contains("Subsection"));
}

#[test]
fn test_text_horizontal_rule() {
    let t = TextPlot::new().with_body("Before\n---\nAfter");
    let out = text_svg(t);
    assert!(out.contains("Before"));
    assert!(out.contains("After"));
}

#[test]
fn test_text_word_wrap() {
    let long = "This sentence is intentionally long so that it will wrap to the next line when rendered in a narrow plot cell.";
    let t = TextPlot::new().with_body(long);
    let out = text_svg(t);
    assert!(out.contains("<svg"));
}

#[test]
fn test_text_background() {
    let t = TextPlot::new()
        .with_body("With background.")
        .with_background("#f0f0f0");
    let out = text_svg(t);
    assert!(out.contains("#f0f0f0"));
}

#[test]
fn test_text_border() {
    let t = TextPlot::new()
        .with_body("With border.")
        .with_border("#333333", 2.0);
    let out = text_svg(t);
    assert!(out.contains("#333333"));
}

#[test]
fn test_text_center_align() {
    let t = TextPlot::new()
        .with_body("Centered text.")
        .with_align(TextAlign::Center);
    let out = text_svg(t);
    assert!(out.contains("Centered text."));
    assert!(out.contains("middle"));
}

#[test]
fn test_text_right_align() {
    let t = TextPlot::new()
        .with_body("Right-aligned.")
        .with_align(TextAlign::Right);
    let out = text_svg(t);
    assert!(out.contains("Right-aligned."));
    assert!(out.contains("end"));
}

#[test]
fn test_text_custom_font_size() {
    let t = TextPlot::new().with_body("Big text.").with_font_size(20);
    let out = text_svg(t);
    assert!(out.contains("Big text."));
}

#[test]
fn test_text_multiline_body() {
    let t = TextPlot::new().with_body("Line one.\n\nLine two after a blank line.\nLine three.");
    let out = text_svg(t);
    assert!(out.contains("Line one."));
    assert!(out.contains("Line two after a blank line."));
    assert!(out.contains("Line three."));
}

#[test]
fn test_text_in_figure_with_scatter_and_histogram() {
    std::fs::create_dir_all("test_outputs").unwrap();

    let data_a: Vec<(f64, f64)> = (0..15)
        .map(|i| (i as f64 * 0.5, i as f64 * 0.8 + 1.0))
        .collect();
    let data_b: Vec<(f64, f64)> = (0..15)
        .map(|i| (i as f64 * 0.5 + 2.0, i as f64 * 0.5 + 3.0))
        .collect();

    let scatter_plots = vec![
        Plot::Scatter(
            ScatterPlot::new()
                .with_data(data_a)
                .with_color("steelblue")
                .with_legend("A"),
        ),
        Plot::Scatter(
            ScatterPlot::new()
                .with_data(data_b)
                .with_color("crimson")
                .with_legend("B"),
        ),
    ];
    let scatter_layout = Layout::auto_from_plots(&scatter_plots)
        .with_title("Scatter")
        .with_x_label("X")
        .with_y_label("Y");

    let bar = kuva::plot::BarPlot::new()
        .with_group("Alpha", vec![(42.0, "steelblue")])
        .with_group("Beta", vec![(28.0, "crimson")])
        .with_group("Gamma", vec![(15.0, "seagreen")]);
    let bar_plots = vec![Plot::Bar(bar)];
    let bar_layout = Layout::auto_from_plots(&bar_plots)
        .with_title("Group Counts")
        .with_x_label("Group")
        .with_y_label("n");

    let vals: Vec<f64> = (0..40)
        .map(|i| (i as f64 * 0.3).sin() * 5.0 + 10.0)
        .collect();
    let hist = Histogram::new()
        .with_data(vals)
        .with_bins(8)
        .with_range((5.0, 15.5));
    let hist_plots = vec![Plot::Histogram(hist)];
    let hist_layout = Layout::auto_from_plots(&hist_plots)
        .with_title("Value Distribution")
        .with_x_label("Value")
        .with_y_label("Count");

    let note = TextPlot::new()
        .with_body(
            "# Figure Notes\n\
            \n\
            **Scatter (A):** 15 points, slope ≈ 1.6, intercept 1.0.\n\
            **Scatter (B):** 15 points, slope ≈ 1.0, intercept 1.0.\n\
            \n\
            ---\n\
            \n\
            ## Bar chart\n\
            Alpha n = 42, Beta n = 28, Gamma n = 15.\n\
            Total: 85 observations.\n\
            \n\
            ## Histogram\n\
            40 values drawn from a sine-modulated distribution \
            centred near 10.0 (range 5–15).",
        )
        .with_background("#fafafa")
        .with_border("#cccccc", 1.0)
        .with_padding(16.0);

    let text_plots = vec![Plot::Text(note)];
    let text_layout = Layout::auto_from_plots(&text_plots).with_title("Notes");

    let all_plots = vec![scatter_plots, bar_plots, hist_plots, text_plots];
    let layouts = vec![scatter_layout, bar_layout, hist_layout, text_layout];

    let scene = Figure::new(2, 2)
        .with_plots(all_plots)
        .with_layouts(layouts)
        .with_labels()
        .with_cell_size(460.0, 340.0)
        .render();

    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/text_figure_4cell.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Figure Notes"));
    assert!(svg.contains("Total: 85 observations."));
}
