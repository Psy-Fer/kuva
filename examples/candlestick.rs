//! Candlestick plot documentation examples.
//!
//! Generates canonical SVG outputs used in the visus documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example candlestick
//! ```
//!
//! SVGs are written to `docs/src/assets/candlestick/`.

use visus::plot::CandlestickPlot;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

const OUT: &str = "docs/src/assets/candlestick";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/candlestick");

    basic();
    volume();
    custom_colors();
    continuous();

    println!("Candlestick SVGs written to {OUT}/");
}

/// Twenty daily OHLC candles — the core chart.
///
/// A mix of bullish (close > open), bearish (close < open), and doji
/// (close == open) candles across a month of trading. Labels serve as
/// categorical x-axis ticks.
fn basic() {
    // 20 trading days — (label, open, high, low, close)
    let data: &[(&str, f64, f64, f64, f64)] = &[
        ("Nov 01", 142.50, 146.20, 141.80, 145.30),
        ("Nov 02", 145.40, 147.80, 143.50, 144.10),
        ("Nov 03", 144.10, 144.90, 142.20, 144.10), // doji
        ("Nov 04", 143.80, 148.50, 143.20, 147.90),
        ("Nov 05", 147.90, 150.20, 146.30, 149.80),
        ("Nov 06", 149.80, 151.00, 146.50, 147.20),
        ("Nov 07", 147.20, 148.10, 143.80, 144.50),
        ("Nov 08", 144.50, 146.80, 143.20, 145.90),
        ("Nov 11", 145.90, 149.30, 145.50, 148.70),
        ("Nov 12", 148.70, 152.00, 148.10, 151.50),
        ("Nov 13", 151.50, 154.80, 150.90, 153.80),
        ("Nov 14", 153.80, 154.20, 150.30, 150.60),
        ("Nov 15", 150.60, 152.10, 148.80, 151.20),
        ("Nov 18", 151.20, 153.40, 150.80, 153.40),
        ("Nov 19", 153.40, 157.20, 152.80, 156.50),
        ("Nov 20", 156.50, 158.10, 154.20, 154.80),
        ("Nov 21", 154.80, 155.20, 152.10, 154.80), // doji
        ("Nov 22", 154.80, 158.50, 154.30, 157.90),
        ("Nov 25", 157.90, 160.10, 156.80, 159.50),
        ("Nov 26", 159.50, 161.20, 157.30, 158.40),
    ];

    let mut plot = CandlestickPlot::new();
    for &(label, open, high, low, close) in data {
        plot = plot.with_candle(label, open, high, low, close);
    }

    let layout = Layout::auto_from_plots(&[Plot::Candlestick(plot)])
        .with_title("Daily OHLC — November")
        .with_x_label("Date")
        .with_y_label("Price (USD)");

    // Rebuild — Layout consumed the first clone
    let mut plot2 = CandlestickPlot::new();
    for &(label, open, high, low, close) in data {
        plot2 = plot2.with_candle(label, open, high, low, close);
    }

    let svg = SvgBackend.render_scene(&render_multiple(vec![Plot::Candlestick(plot2)], layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Volume bars stacked below the price chart.
///
/// `.with_volume()` attaches a volume value to each candle.
/// `.with_volume_panel()` enables the panel. The panel occupies the bottom
/// 22 % of the chart area by default.
fn volume() {
    let data: &[(&str, f64, f64, f64, f64, f64)] = &[
        ("Nov 01", 142.50, 146.20, 141.80, 145.30, 1_250_000.0),
        ("Nov 02", 145.40, 147.80, 143.50, 144.10,   980_000.0),
        ("Nov 03", 144.10, 144.90, 142.20, 144.10,   720_000.0),
        ("Nov 04", 143.80, 148.50, 143.20, 147.90, 1_580_000.0),
        ("Nov 05", 147.90, 150.20, 146.30, 149.80, 1_420_000.0),
        ("Nov 06", 149.80, 151.00, 146.50, 147.20, 1_100_000.0),
        ("Nov 07", 147.20, 148.10, 143.80, 144.50, 1_350_000.0),
        ("Nov 08", 144.50, 146.80, 143.20, 145.90,   890_000.0),
        ("Nov 11", 145.90, 149.30, 145.50, 148.70, 1_180_000.0),
        ("Nov 12", 148.70, 152.00, 148.10, 151.50, 1_620_000.0),
        ("Nov 13", 151.50, 154.80, 150.90, 153.80, 1_780_000.0),
        ("Nov 14", 153.80, 154.20, 150.30, 150.60, 1_290_000.0),
        ("Nov 15", 150.60, 152.10, 148.80, 151.20, 1_050_000.0),
        ("Nov 18", 151.20, 153.40, 150.80, 153.40, 1_340_000.0),
        ("Nov 19", 153.40, 157.20, 152.80, 156.50, 2_100_000.0),
        ("Nov 20", 156.50, 158.10, 154.20, 154.80, 1_650_000.0),
        ("Nov 21", 154.80, 155.20, 152.10, 154.80,   980_000.0),
        ("Nov 22", 154.80, 158.50, 154.30, 157.90, 1_420_000.0),
        ("Nov 25", 157.90, 160.10, 156.80, 159.50, 1_890_000.0),
        ("Nov 26", 159.50, 161.20, 157.30, 158.40, 1_560_000.0),
    ];

    let volumes: Vec<f64> = data.iter().map(|r| r.5).collect();

    let make_plot = || {
        let mut p = CandlestickPlot::new();
        for &(label, open, high, low, close, _) in data {
            p = p.with_candle(label, open, high, low, close);
        }
        p.with_volume(volumes.clone()).with_volume_panel()
    };

    let layout = Layout::auto_from_plots(&[Plot::Candlestick(make_plot())])
        .with_title("Daily OHLC with Volume Panel")
        .with_x_label("Date")
        .with_y_label("Price (USD)");

    let svg = SvgBackend.render_scene(&render_multiple(vec![Plot::Candlestick(make_plot())], layout));
    std::fs::write(format!("{OUT}/volume.svg"), svg).unwrap();
}

/// Custom up/down/doji colors.
///
/// Replace the default green/red with theme-matched colors using
/// `.with_color_up()`, `.with_color_down()`, and `.with_color_doji()`.
fn custom_colors() {
    let data: &[(&str, f64, f64, f64, f64)] = &[
        ("Mon", 100.00, 105.80, 99.20, 104.50),
        ("Tue", 104.50, 106.20, 103.10, 103.40),
        ("Wed", 103.40, 107.50, 102.80, 106.90),
        ("Thu", 106.90, 109.10, 106.30, 108.60),
        ("Fri", 108.60, 109.00, 105.70, 108.60), // doji
        ("Mon", 108.60, 112.30, 108.10, 111.50),
        ("Tue", 111.50, 113.80, 109.90, 110.20),
        ("Wed", 110.20, 111.00, 107.40, 108.80),
        ("Thu", 108.80, 113.50, 108.50, 112.70),
        ("Fri", 112.70, 115.20, 111.80, 114.90),
    ];

    let mut plot = CandlestickPlot::new()
        .with_color_up("#00c896")
        .with_color_down("#ff4560")
        .with_color_doji("#aaaaaa");
    for &(label, open, high, low, close) in data {
        plot = plot.with_candle(label, open, high, low, close);
    }

    let layout = Layout::auto_from_plots(&[Plot::Candlestick({
        let mut p = CandlestickPlot::new()
            .with_color_up("#00c896")
            .with_color_down("#ff4560")
            .with_color_doji("#aaaaaa");
        for &(label, open, high, low, close) in data {
            p = p.with_candle(label, open, high, low, close);
        }
        p
    })])
    .with_title("Custom Colors")
    .with_x_label("Day")
    .with_y_label("Price");

    let svg = SvgBackend.render_scene(&render_multiple(vec![Plot::Candlestick(plot)], layout));
    std::fs::write(format!("{OUT}/custom_colors.svg"), svg).unwrap();
}

/// Continuous numeric x-axis using `with_candle_at`.
///
/// Each candle is placed at an explicit numeric x position, enabling uneven
/// spacing and a true numeric axis. Useful for quarterly data, time-indexed
/// series, or any scenario where candles are not evenly spaced.
fn continuous() {
    // Quarterly data: x = fractional year (e.g. 2022.0 = Q1 2022, 2022.25 = Q2)
    let data: &[(f64, &str, f64, f64, f64, f64)] = &[
        (2022.00, "Q1'22",  98.0, 108.0,  95.0, 105.0),
        (2022.25, "Q2'22", 105.0, 112.0, 101.0, 108.5),
        (2022.50, "Q3'22", 108.5, 115.0, 107.0, 113.0),
        (2022.75, "Q4'22", 113.0, 116.0, 109.0, 110.5),
        (2023.00, "Q1'23", 110.5, 118.0, 110.0, 116.8),
        (2023.25, "Q2'23", 116.8, 122.0, 115.5, 121.0),
        (2023.50, "Q3'23", 121.0, 125.5, 119.0, 120.2),
        (2023.75, "Q4'23", 120.2, 128.0, 119.8, 127.0),
        (2024.00, "Q1'24", 127.0, 135.0, 126.0, 133.5),
        (2024.25, "Q2'24", 133.5, 138.0, 131.0, 134.0),
        (2024.50, "Q3'24", 134.0, 141.0, 133.5, 140.0),
        (2024.75, "Q4'24", 140.0, 144.5, 137.0, 142.8),
    ];

    let make_plot = || {
        let mut p = CandlestickPlot::new();
        for &(x, label, open, high, low, close) in data {
            p = p.with_candle_at(x, label, open, high, low, close);
        }
        p.with_candle_width(0.15)
    };

    let layout = Layout::auto_from_plots(&[Plot::Candlestick(make_plot())])
        .with_title("Quarterly OHLC — Numeric x-axis")
        .with_x_label("Quarter")
        .with_y_label("Price");

    let svg = SvgBackend.render_scene(&render_multiple(vec![Plot::Candlestick(make_plot())], layout));
    std::fs::write(format!("{OUT}/continuous.svg"), svg).unwrap();
}
