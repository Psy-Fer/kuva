mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::line::LinePlot;
use kuva::plot::scatter::ScatterPlot;
use kuva::plot::{PieLabelPosition, PiePlot};
use kuva::render::layout::{ComputedLayout, Layout};
use kuva::render::plots::Plot;
use kuva::render::render::{render_multiple, render_twin_y};

/// Extract the `x` attribute value from the SVG `<text>` element whose content
/// matches `text`. Finds `>text<`, walks back to the nearest `x="..."`.
fn extract_text_x(svg: &str, text: &str) -> Option<f64> {
    let needle = format!(">{}<", text);
    let pos = svg.find(&needle)?;
    let before = &svg[..pos];
    let x_attr = before.rfind("x=\"")?;
    let after_quote = &before[x_attr + 3..];
    let end = after_quote.find('"')?;
    after_quote[..end].parse::<f64>().ok()
}

/// Extract the `y` attribute value from the SVG `<text>` element whose content
/// matches `text`. Finds `>text<`, walks back to the nearest `y="..."`.
fn extract_text_y(svg: &str, text: &str) -> Option<f64> {
    let needle = format!(">{}<", text);
    let pos = svg.find(&needle)?;
    let before = &svg[..pos];
    let y_attr = before.rfind("y=\"")?;
    let after_quote = &before[y_attr + 3..];
    let end = after_quote.find('"')?;
    after_quote[..end].parse::<f64>().ok()
}

#[test]
fn test_title_centred_with_legend() {
    let data = vec![(1.0f64, 2.0f64), (3.0, 4.0), (5.0, 6.0)];
    let plot = ScatterPlot::new().with_data(data).with_legend("Group A");
    let plots = vec![Plot::Scatter(plot)];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("MyTitle")
        .with_x_label("MyLabel");

    let computed = ComputedLayout::from_layout(&layout);
    // Both title and x-label center on the plot area, not the full canvas.
    // With a legend present margin_right > margin_left, so plot-area centre ≠ canvas centre.
    let expected_x = computed.margin_left + computed.plot_width() / 2.0;

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/label_centering_legend.svg", &svg).unwrap();

    let title_x = extract_text_x(&svg, "MyTitle").expect("title element not found in SVG");
    let label_x = extract_text_x(&svg, "MyLabel").expect("x-label element not found in SVG");

    assert!(
        (title_x - expected_x).abs() < 1.0,
        "title x={title_x:.1} should equal plot-area centre={expected_x:.1}"
    );
    assert!(
        (label_x - expected_x).abs() < 1.0,
        "x-label x={label_x:.1} should equal plot-area centre={expected_x:.1}"
    );
    // With a legend, plot-area centre must differ from canvas centre (the regression).
    assert!(
        (expected_x - computed.width / 2.0).abs() > 1.0,
        "legend should make plot-area centre differ from canvas centre \
         (plot={expected_x:.1}, canvas={:.1})",
        computed.width / 2.0
    );
}

#[test]
fn test_title_centred_twin_y() {
    let primary = vec![Plot::Line(
        LinePlot::new()
            .with_data(vec![(1.0f64, 5.0f64), (2.0, 8.0), (3.0, 14.0)])
            .with_legend("Temp"),
    )];
    let secondary = vec![Plot::Line(
        LinePlot::new()
            .with_data(vec![(1.0f64, 80.0f64), (2.0, 60.0), (3.0, 45.0)])
            .with_legend("Rain"),
    )];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary)
        .with_title("TwinTitle")
        .with_x_label("X");

    let computed = ComputedLayout::from_layout(&layout);
    // Both title and x-label center on the plot area (twin-y has margin on both sides).
    let expected_x = computed.margin_left + computed.plot_width() / 2.0;

    let scene = render_twin_y(primary, secondary, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/label_centering_twin_y.svg", &svg).unwrap();

    let title_x = extract_text_x(&svg, "TwinTitle").expect("title element not found in SVG");
    let label_x = extract_text_x(&svg, "X").expect("x-label element not found in SVG");

    assert!(
        (title_x - expected_x).abs() < 1.0,
        "twin-y title x={title_x:.1} should equal plot-area centre={expected_x:.1}"
    );
    assert!(
        (label_x - expected_x).abs() < 1.0,
        "twin-y x-label x={label_x:.1} should equal plot-area centre={expected_x:.1}"
    );
}

#[test]
fn test_title_centred_pie_outside_labels() {
    let pie = PiePlot::new()
        .with_slice("Alpha", 30.0, "steelblue")
        .with_slice("Beta", 25.0, "tomato")
        .with_slice("Gamma", 20.0, "seagreen")
        .with_slice("Delta", 15.0, "orange")
        .with_slice("Epsilon", 10.0, "purple")
        .with_label_position(PieLabelPosition::Outside);

    let plots = vec![Plot::Pie(pie.clone())];
    let layout = Layout::auto_from_plots(&plots).with_title("PieTitle");

    // render_multiple may widen the canvas for outside labels; compute expected x
    // from the final canvas width, keeping the same margins (only width changes).
    let pre = ComputedLayout::from_layout(&layout);
    let margin_left = pre.margin_left;
    let margin_right = pre.margin_right;

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/label_centering_pie_outside.svg", &svg).unwrap();

    let canvas_width: f64 = {
        let w_pos = svg.find("width=\"").expect("width attr in SVG");
        let after = &svg[w_pos + 7..];
        let end = after.find('"').unwrap();
        after[..end].parse().unwrap()
    };

    // Both pie visual centre and title use margin_left + plot_width()/2 of widened canvas.
    let expected_x = margin_left + (canvas_width - margin_left - margin_right) / 2.0;
    let title_x = extract_text_x(&svg, "PieTitle").expect("title element not found in SVG");

    assert!(
        (title_x - expected_x).abs() < 1.0,
        "pie title x={title_x:.1} should equal plot-area centre={expected_x:.1} \
         (canvas={canvas_width:.1})"
    );
}

// ── Label offset API tests ────────────────────────────────────────────────────

fn make_scatter_plots() -> Vec<Plot> {
    let data = vec![(1.0f64, 2.0f64), (3.0, 4.0), (5.0, 6.0)];
    vec![Plot::Scatter(ScatterPlot::new().with_data(data))]
}

fn make_twin_y_plots() -> (Vec<Plot>, Vec<Plot>) {
    let primary = vec![Plot::Line(LinePlot::new().with_data(vec![
        (1.0f64, 5.0f64),
        (2.0, 8.0),
        (3.0, 14.0),
    ]))];
    let secondary = vec![Plot::Line(LinePlot::new().with_data(vec![
        (1.0f64, 80.0f64),
        (2.0, 60.0),
        (3.0, 45.0),
    ]))];
    (primary, secondary)
}

#[test]
fn test_x_label_offset() {
    let (dx, dy) = (20.0_f64, -5.0_f64);

    // Baseline — no offset
    let plots = make_scatter_plots();
    let layout = Layout::auto_from_plots(&plots).with_x_label("XLbl");
    let svg_base = SvgBackend.render_scene(&render_multiple(plots, layout));

    // With offset
    let plots = make_scatter_plots();
    let layout = Layout::auto_from_plots(&plots)
        .with_x_label("XLbl")
        .with_x_label_offset(dx, dy);
    let svg_off = SvgBackend.render_scene(&render_multiple(plots, layout));

    common::write_test_output("test_outputs/x_label_offset.svg", &svg_off).unwrap();

    let base_x = extract_text_x(&svg_base, "XLbl").expect("base x-label not found");
    let base_y = extract_text_y(&svg_base, "XLbl").expect("base y not found");
    let off_x = extract_text_x(&svg_off, "XLbl").expect("offset x-label not found");
    let off_y = extract_text_y(&svg_off, "XLbl").expect("offset y not found");

    assert!(
        (off_x - base_x - dx).abs() < 0.5,
        "x-label x: expected shift {dx}, got {:.1} → {:.1}",
        base_x,
        off_x
    );
    assert!(
        (off_y - base_y - dy).abs() < 0.5,
        "x-label y: expected shift {dy}, got {:.1} → {:.1}",
        base_y,
        off_y
    );
}

#[test]
fn test_y_label_offset() {
    let (dx, dy) = (8.0_f64, 15.0_f64);

    // Baseline
    let plots = make_scatter_plots();
    let layout = Layout::auto_from_plots(&plots).with_y_label("YLbl");
    let svg_base = SvgBackend.render_scene(&render_multiple(plots, layout));

    // With offset
    let plots = make_scatter_plots();
    let layout = Layout::auto_from_plots(&plots)
        .with_y_label("YLbl")
        .with_y_label_offset(dx, dy);
    let svg_off = SvgBackend.render_scene(&render_multiple(plots, layout));

    common::write_test_output("test_outputs/y_label_offset.svg", &svg_off).unwrap();

    let base_x = extract_text_x(&svg_base, "YLbl").expect("base y-label x not found");
    let base_y = extract_text_y(&svg_base, "YLbl").expect("base y-label y not found");
    let off_x = extract_text_x(&svg_off, "YLbl").expect("offset y-label x not found");
    let off_y = extract_text_y(&svg_off, "YLbl").expect("offset y-label y not found");

    assert!(
        (off_x - base_x - dx).abs() < 0.5,
        "y-label x: expected shift {dx}, got {:.1} → {:.1}",
        base_x,
        off_x
    );
    assert!(
        (off_y - base_y - dy).abs() < 0.5,
        "y-label y: expected shift {dy}, got {:.1} → {:.1}",
        base_y,
        off_y
    );
}

#[test]
fn test_y2_label_offset() {
    let (dx, dy) = (-10.0_f64, 20.0_f64);

    // Baseline
    let (primary, secondary) = make_twin_y_plots();
    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary).with_y2_label("Y2Lbl");
    let svg_base = SvgBackend.render_scene(&render_twin_y(primary, secondary, layout));

    // With offset
    let (primary, secondary) = make_twin_y_plots();
    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary)
        .with_y2_label("Y2Lbl")
        .with_y2_label_offset(dx, dy);
    let svg_off = SvgBackend.render_scene(&render_twin_y(primary, secondary, layout));

    common::write_test_output("test_outputs/y2_label_offset.svg", &svg_off).unwrap();

    let base_x = extract_text_x(&svg_base, "Y2Lbl").expect("base y2-label x not found");
    let base_y = extract_text_y(&svg_base, "Y2Lbl").expect("base y2-label y not found");
    let off_x = extract_text_x(&svg_off, "Y2Lbl").expect("offset y2-label x not found");
    let off_y = extract_text_y(&svg_off, "Y2Lbl").expect("offset y2-label y not found");

    assert!(
        (off_x - base_x - dx).abs() < 0.5,
        "y2-label x: expected shift {dx}, got {:.1} → {:.1}",
        base_x,
        off_x
    );
    assert!(
        (off_y - base_y - dy).abs() < 0.5,
        "y2-label y: expected shift {dy}, got {:.1} → {:.1}",
        base_y,
        off_y
    );
}

/// Regression #83: y-axis label must be centred on the plot area, not the full canvas.
/// When no title is present margin_top is smaller, so `height/2` places the label
/// at what would be the title-present midpoint — too low. The correct anchor is
/// `margin_top + plot_height() / 2`.
#[test]
fn test_y_label_centred_on_plot_area() {
    let data = vec![(1.0f64, 2.0f64), (3.0, 4.0), (5.0, 6.0)];

    // Without title: y-label must be centred on the plot area.
    let plots = vec![Plot::Scatter(ScatterPlot::new().with_data(data.clone()))];
    let layout = Layout::auto_from_plots(&plots).with_y_label("YAxisLbl");
    let computed = ComputedLayout::from_layout(&layout);
    let expected_no_title = computed.margin_top + computed.plot_height() / 2.0;
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/y_label_centred_no_title.svg", &svg).unwrap();
    let label_y = extract_text_y(&svg, "YAxisLbl").expect("y-label not found (no title)");
    assert!(
        (label_y - expected_no_title).abs() < 2.0,
        "y-label y={label_y:.1} should equal margin_top+plot_height/2={expected_no_title:.1} (no title)"
    );

    // With title: same formula must still hold.
    let plots = vec![Plot::Scatter(ScatterPlot::new().with_data(data))];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("SomeTitle")
        .with_y_label("YAxisLbl");
    let computed = ComputedLayout::from_layout(&layout);
    let expected_with_title = computed.margin_top + computed.plot_height() / 2.0;
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/y_label_centred_with_title.svg", &svg).unwrap();
    let label_y = extract_text_y(&svg, "YAxisLbl").expect("y-label not found (with title)");
    assert!(
        (label_y - expected_with_title).abs() < 2.0,
        "y-label y={label_y:.1} should equal margin_top+plot_height/2={expected_with_title:.1} (with title)"
    );

    // The two expected positions must differ — if they're equal the test isn't exercising the bug.
    assert!(
        (expected_no_title - expected_with_title).abs() > 2.0,
        "title should shift the plot-area centre: no_title={expected_no_title:.1} with_title={expected_with_title:.1}"
    );
}
