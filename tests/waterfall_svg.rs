use visus::plot::WaterfallPlot;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

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
    std::fs::write("test_outputs/waterfall_basic.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("rgb(68,170,68)"));
    assert!(svg.contains("rgb(204,68,68)"));
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
    std::fs::write("test_outputs/waterfall_with_totals.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("steelblue"));
}

#[test]
fn test_waterfall_connectors_and_values() {
    let wf = WaterfallPlot::new()
        .with_delta("Alpha", 40.0)
        .with_delta("Beta", -15.0)
        .with_delta("Gamma", 20.0)
        .with_total("Subtotal")
        .with_delta("Delta", -5.0)
        .with_connectors()
        .with_values();

    let plots = vec![Plot::Waterfall(wf)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Connectors and Values");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/waterfall_connectors_values.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("4,3"));  // dasharray from connectors
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
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Custom Colors");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/waterfall_custom_colors.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("darkgreen"));
    assert!(svg.contains("crimson"));
    assert!(svg.contains("navy"));
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
    std::fs::write("test_outputs/waterfall_all_negative.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("rgb(204,68,68)"));
}
