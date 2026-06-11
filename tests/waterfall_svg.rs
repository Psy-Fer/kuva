mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::WaterfallPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

#[test]
fn test_waterfall_basic() {
    let wf = WaterfallPlot::new()
        .with_delta("Start", 100.0)
        .with_delta("Gain A", 25.0)
        .with_delta("Loss B", -10.0)
        .with_delta("Gain C", 15.0)
        .with_delta("Loss D", -30.0);

    let plots = vec![Plot::Waterfall(wf)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Basic Waterfall")
        .with_y_label("Value");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/waterfall_basic.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("#44aa44"));
    assert!(svg.contains("#cc4444"));
}

#[test]
fn test_waterfall_with_totals() {
    let wf = WaterfallPlot::new()
        .with_delta("Revenue", 500.0)
        .with_delta("Cost", -200.0)
        .with_total("Gross Profit")
        .with_delta("OpEx", -80.0)
        .with_delta("Tax", -30.0)
        .with_total("Net Profit");

    let plots = vec![Plot::Waterfall(wf)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Waterfall with Totals")
        .with_y_label("USD");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/waterfall_with_totals.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("#4682b4"));
}

#[test]
fn test_waterfall_connectors_and_values() {
    // Alpha=40, Beta=-15, Gamma=+20 → Subtotal=45.
    // The Difference bar shows the net change from Alpha (40) to Subtotal (45):
    // a green +5 bar anchored at y=40..45, independent of the running total.
    let wf = WaterfallPlot::new()
        .with_delta("Alpha", 40.0)
        .with_delta("Beta", -15.0)
        .with_delta("Gamma", 20.0)
        .with_total("Subtotal")
        .with_difference("Net change", 40.0, 45.0)
        .with_connectors()
        .with_values();

    let plots = vec![Plot::Waterfall(wf)];
    let layout = Layout::auto_from_plots(&plots).with_title("Connectors and Values");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/waterfall_connectors_values.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("4,3")); // dasharray from connectors
}

#[test]
fn test_waterfall_difference() {
    // Positive difference (green): 40 → 45
    // Negative difference (red):   50 → 40
    let wf_pos = WaterfallPlot::new()
        .with_delta("Start", 40.0)
        .with_difference("Overall change", 40.0, 45.0)
        .with_values();
    let plots = vec![Plot::Waterfall(wf_pos)];
    let layout = Layout::auto_from_plots(&plots).with_title("Difference +5");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/waterfall_difference_pos.svg", svg.clone()).unwrap();
    assert!(svg.contains("#44aa44")); // green

    let wf_neg = WaterfallPlot::new()
        .with_delta("Start", 50.0)
        .with_difference("Overall change", 50.0, 40.0)
        .with_values();
    let plots2 = vec![Plot::Waterfall(wf_neg)];
    let layout2 = Layout::auto_from_plots(&plots2).with_title("Difference -10");
    let svg2 = SvgBackend.render_scene(&render_multiple(plots2, layout2));
    common::write_test_output("test_outputs/waterfall_difference_neg.svg", svg2.clone()).unwrap();
    assert!(svg2.contains("#cc4444")); // red
}

#[test]
fn test_waterfall_custom_colors() {
    let wf = WaterfallPlot::new()
        .with_delta("Step 1", 50.0)
        .with_delta("Step 2", -20.0)
        .with_total("Total")
        .with_color_positive("darkgreen")
        .with_color_negative("crimson")
        .with_color_total("navy");

    let plots = vec![Plot::Waterfall(wf)];
    let layout = Layout::auto_from_plots(&plots).with_title("Custom Colors");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/waterfall_custom_colors.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("#006400"));
    assert!(svg.contains("#dc143c"));
    assert!(svg.contains("#000080"));
}

#[test]
fn test_waterfall_all_negative() {
    let wf = WaterfallPlot::new()
        .with_delta("Loss 1", -30.0)
        .with_delta("Loss 2", -20.0)
        .with_delta("Loss 3", -10.0);

    let plots = vec![Plot::Waterfall(wf)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("All Negative Waterfall")
        .with_y_label("Value");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/waterfall_all_negative.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("#cc4444"));
}

#[test]
fn test_waterfall_with_gap() {
    // with_gap(g) is the complement of with_bar_width(1-g).
    // Wider bars (small gap) should produce wider rect widths in the SVG.
    fn render_gap(gap: f64) -> String {
        let wf = WaterfallPlot::new()
            .with_delta("A", 100.0)
            .with_delta("B", -40.0)
            .with_total("Total")
            .with_gap(gap);
        let plots = vec![Plot::Waterfall(wf)];
        let layout = Layout::auto_from_plots(&plots)
            .with_width(600.0)
            .with_height(400.0);
        let scene = render_multiple(plots, layout);
        SvgBackend.render_scene(&scene)
    }

    let svg_narrow = render_gap(0.6); // bar_width = 0.4  → narrow bars
    let svg_wide = render_gap(0.1); // bar_width = 0.9  → wide bars

    common::write_test_output("test_outputs/waterfall_bar_narrow.svg", &svg_narrow).unwrap();
    common::write_test_output("test_outputs/waterfall_bar_wide.svg", &svg_wide).unwrap();

    // Extract the width of the first <rect> element that looks like a bar:
    // > 10px (not a tick artifact) and < 300px (not the clip/background rect).
    fn first_rect_width(svg: &str) -> f64 {
        for rect_chunk in svg.split("<rect").skip(1) {
            if let Some(after) = rect_chunk.split("width=\"").nth(1) {
                let s = after.split('"').next().unwrap_or("");
                if let Ok(v) = s.parse::<f64>() {
                    if v > 10.0 && v < 300.0 {
                        return v;
                    }
                }
            }
        }
        panic!("no bar rect found in SVG");
    }

    let w_narrow = first_rect_width(&svg_narrow);
    let w_wide = first_rect_width(&svg_wide);

    assert!(
        w_wide > w_narrow,
        "wide bars ({w_wide:.1}px) should be wider than narrow bars ({w_narrow:.1}px)"
    );
}
