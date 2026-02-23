use std::fs;
use visus::plot::CandlestickPlot;
use visus::render::plots::Plot;
use visus::render::layout::Layout;
use visus::render::render::render_multiple;
use visus::backend::svg::SvgBackend;

fn write_svg(name: &str, svg: &str) {
    fs::create_dir_all("test_outputs").unwrap();
    fs::write(format!("test_outputs/{}.svg", name), svg).unwrap();
}

#[test]
fn candlestick_basic() {
    let plot = CandlestickPlot::new()
        .with_candle("Jan 1", 100.0, 115.0,  95.0, 110.0)  // up
        .with_candle("Jan 2", 110.0, 120.0, 105.0, 108.0)  // down
        .with_candle("Jan 3", 108.0, 112.0, 106.0, 108.0)  // doji
        .with_candle("Jan 4",  95.0, 118.0,  90.0, 115.0)  // up
        .with_candle("Jan 5", 115.0, 116.0, 100.0, 102.0); // down

    let layout = Layout::auto_from_plots(&[Plot::Candlestick(plot)])
        .with_title("Candlestick Basic")
        .with_x_label("Date")
        .with_y_label("Price");

    // Rebuild plot (consumed above)
    let plot2 = CandlestickPlot::new()
        .with_candle("Jan 1", 100.0, 115.0,  95.0, 110.0)
        .with_candle("Jan 2", 110.0, 120.0, 105.0, 108.0)
        .with_candle("Jan 3", 108.0, 112.0, 106.0, 108.0)
        .with_candle("Jan 4",  95.0, 118.0,  90.0, 115.0)
        .with_candle("Jan 5", 115.0, 116.0, 100.0, 102.0);

    let scene = render_multiple(vec![Plot::Candlestick(plot2)], layout);
    let svg = SvgBackend.render_scene(&scene);

    write_svg("candlestick_basic", &svg);

    // Should have body rects (at least 5 bodies + 5 wicks as lines)
    assert!(svg.contains("<rect"), "Expected rect elements for candle bodies");
    assert!(svg.contains("<line"), "Expected line elements for candle wicks");
    // Category labels should appear
    assert!(svg.contains("Jan 1"), "Expected x-axis category label");
}

#[test]
fn candlestick_volume() {
    let make_plot = || {
        CandlestickPlot::new()
            .with_candle("Mon", 100.0, 115.0,  95.0, 110.0)
            .with_candle("Tue", 110.0, 120.0, 105.0, 108.0)
            .with_candle("Wed", 108.0, 112.0, 106.0, 108.0)
            .with_candle("Thu",  95.0, 118.0,  90.0, 115.0)
            .with_candle("Fri", 115.0, 116.0, 100.0, 102.0)
    };

    // Count rects without volume
    let plain = make_plot();
    let layout_plain = Layout::auto_from_plots(&[Plot::Candlestick(make_plot())])
        .with_title("No Volume");
    let scene_plain = render_multiple(vec![Plot::Candlestick(plain)], layout_plain);
    let svg_plain = SvgBackend.render_scene(&scene_plain);
    let rect_count_plain = svg_plain.matches("<rect").count();

    // Count rects with volume
    let with_vol = make_plot()
        .with_volume([120_000.0, 95_000.0, 60_000.0, 150_000.0, 80_000.0])
        .with_volume_panel();
    let layout_vol = Layout::auto_from_plots(&[Plot::Candlestick(make_plot()
        .with_volume([120_000.0, 95_000.0, 60_000.0, 150_000.0, 80_000.0])
        .with_volume_panel())])
        .with_title("With Volume");
    let scene_vol = render_multiple(vec![Plot::Candlestick(with_vol)], layout_vol);
    let svg_vol = SvgBackend.render_scene(&scene_vol);
    let rect_count_vol = svg_vol.matches("<rect").count();

    write_svg("candlestick_volume", &svg_vol);

    assert!(
        rect_count_vol > rect_count_plain,
        "Volume panel should add more rects: plain={rect_count_plain}, vol={rect_count_vol}"
    );
}

#[test]
fn candlestick_continuous() {
    let plot = CandlestickPlot::new()
        .with_candle_at(1.0, "Q1", 100.0, 115.0,  95.0, 110.0)
        .with_candle_at(2.0, "Q2", 110.0, 125.0, 105.0, 108.0)
        .with_candle_at(3.0, "Q3", 108.0, 118.0, 102.0, 116.0)
        .with_candle_at(4.0, "Q4",  95.0, 120.0,  90.0, 100.0)
        .with_candle_at(5.0, "Q5", 100.0, 130.0,  98.0, 128.0);

    let layout = Layout::auto_from_plots(&[Plot::Candlestick(CandlestickPlot::new()
        .with_candle_at(1.0, "Q1", 100.0, 115.0,  95.0, 110.0)
        .with_candle_at(2.0, "Q2", 110.0, 125.0, 105.0, 108.0)
        .with_candle_at(3.0, "Q3", 108.0, 118.0, 102.0, 116.0)
        .with_candle_at(4.0, "Q4",  95.0, 120.0,  90.0, 100.0)
        .with_candle_at(5.0, "Q5", 100.0, 130.0,  98.0, 128.0))])
        .with_title("Candlestick Continuous")
        .with_x_label("Quarter")
        .with_y_label("Price");

    let scene = render_multiple(vec![Plot::Candlestick(plot)], layout);
    let svg = SvgBackend.render_scene(&scene);

    write_svg("candlestick_continuous", &svg);

    assert!(svg.contains("<rect"), "Expected rect elements");
    assert!(svg.contains("<line"), "Expected line elements");
}

#[test]
fn candlestick_legend() {
    let plot = CandlestickPlot::new()
        .with_candle("Jan", 100.0, 115.0,  95.0, 110.0)
        .with_candle("Feb", 110.0, 120.0, 105.0, 108.0)
        .with_candle("Mar", 108.0, 112.0, 106.0, 112.0)
        .with_legend("AAPL");

    let layout = Layout::auto_from_plots(&[Plot::Candlestick(CandlestickPlot::new()
        .with_candle("Jan", 100.0, 115.0,  95.0, 110.0)
        .with_candle("Feb", 110.0, 120.0, 105.0, 108.0)
        .with_candle("Mar", 108.0, 112.0, 106.0, 112.0)
        .with_legend("AAPL"))])
        .with_title("Candlestick Legend");

    let scene = render_multiple(vec![Plot::Candlestick(plot)], layout);
    let svg = SvgBackend.render_scene(&scene);

    write_svg("candlestick_legend", &svg);

    assert!(svg.contains("AAPL"), "Legend label 'AAPL' should appear in SVG");
}
