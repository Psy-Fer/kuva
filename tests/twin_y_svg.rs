use kuva::plot::line::LinePlot;
use kuva::plot::bar::BarPlot;
use kuva::plot::histogram::Histogram;
use kuva::plot::scatter::ScatterPlot;
use kuva::plot::LegendPosition;
use kuva::render::layout::{Layout, ComputedLayout};
use kuva::render::plots::Plot;
use kuva::render::render::render_twin_y;
use kuva::backend::svg::SvgBackend;
use kuva::Palette;

fn extract_text_x(svg: &str, text: &str) -> Option<f64> {
    let needle = format!(">{}<", text);
    let pos = svg.find(&needle)?;
    let before = &svg[..pos];
    let x_attr = before.rfind("x=\"")?;
    let after_quote = &before[x_attr + 3..];
    let end = after_quote.find('"')?;
    after_quote[..end].parse::<f64>().ok()
}

fn make_temperature_line() -> Plot {
    let points: Vec<(f64, f64)> = vec![
        (1.0, 5.0), (2.0, 8.0), (3.0, 14.0), (4.0, 20.0), (5.0, 24.0), (6.0, 22.0),
    ];
    Plot::Line(LinePlot::new().with_data(points).with_legend("Temperature (°C)"))
}

fn make_rainfall_line() -> Plot {
    let points: Vec<(f64, f64)> = vec![
        (1.0, 80.0), (2.0, 60.0), (3.0, 45.0), (4.0, 30.0), (5.0, 20.0), (6.0, 35.0),
    ];
    Plot::Line(LinePlot::new().with_data(points).with_legend("Rainfall (mm)"))
}

#[test]
fn test_twin_y_basic() {
    let primary = vec![make_temperature_line()];
    let secondary = vec![make_rainfall_line()];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary);
    let scene = render_twin_y(primary, secondary, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/twin_y_basic.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"), "SVG should start with <svg element");
    // Right-side axis line should exist (add_y2_axis draws a vertical line at the right edge)
    assert!(svg.contains("x1="), "SVG should contain line elements");
}

#[test]
fn test_twin_y_labels() {
    let primary = vec![make_temperature_line()];
    let secondary = vec![make_rainfall_line()];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary)
        .with_y_label("Temp (°C)")
        .with_y2_label("Rain");

    let scene = render_twin_y(primary, secondary, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/twin_y_labels.svg", svg.clone()).unwrap();

    assert!(svg.contains("Rain"), "SVG should contain the y2 label 'Rain'");
    assert!(svg.contains("Temp"), "SVG should contain the y label 'Temp'");
}

#[test]
fn test_twin_y_auto() {
    // Test that auto_from_twin_y_plots builds and renders without panic
    let primary = vec![make_temperature_line()];
    let secondary = vec![make_rainfall_line()];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary);
    let scene = render_twin_y(primary, secondary, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/twin_y_auto.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"), "SVG output should be valid");
}

#[test]
fn test_twin_y_palette() {
    let primary = vec![make_temperature_line()];
    let secondary = vec![make_rainfall_line()];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary)
        .with_palette(Palette::wong());

    let scene = render_twin_y(primary, secondary, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/twin_y_palette.svg", svg.clone()).unwrap();

    // Wong palette first two colors: #e69f00, #56b4e9 (Color outputs lowercase hex)
    assert!(svg.contains("#e69f00"), "SVG should contain wong palette color 1");
    assert!(svg.contains("#56b4e9"), "SVG should contain wong palette color 2");
}

#[test]
fn test_twin_y_log_y2() {
    let primary_points: Vec<(f64, f64)> = vec![
        (1.0, 10.0), (2.0, 20.0), (3.0, 30.0), (4.0, 40.0), (5.0, 50.0),
    ];
    let secondary_points: Vec<(f64, f64)> = vec![
        (1.0, 1.0), (2.0, 10.0), (3.0, 100.0), (4.0, 1000.0), (5.0, 10000.0),
    ];

    let primary = vec![Plot::Line(LinePlot::new().with_data(primary_points))];
    let secondary = vec![Plot::Line(LinePlot::new().with_data(secondary_points))];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary)
        .with_log_y2();

    let scene = render_twin_y(primary, secondary, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/twin_y_log_y2.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"), "SVG should be valid");
    // Log ticks like 1, 100 should appear as text elements in the right-side axis labels
    assert!(svg.contains(">1<"), "SVG should contain log tick '1'");
    assert!(svg.contains(">100<"), "SVG should contain log tick '100'");
}


#[test]
fn test_twin_y_multiplot() {
    let primary = vec![make_temperature_line()];
    let secondary = vec![make_rainfall_line()];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary)
        .with_title("rainfall and temperature twin y multiplot")
        .with_legend_position(kuva::plot::LegendPosition::OutsideRightTop);
    let scene = render_twin_y(primary, secondary, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/twin_y_multiplot.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"), "SVG should contain an <svg element");
    assert!(svg.contains("rainfall and temperature twin y multiplot"), "SVG should contain the title");
    assert!(svg.contains("Temperature"), "SVG should contain the primary series legend label");
    assert!(svg.contains("Rainfall"), "SVG should contain the secondary series legend label");
    // RightTop legend is placed in the right margin — it should appear after the plot area elements
    assert!(svg.contains("x1="), "SVG should contain axis line elements");
}

#[test]
fn test_twin_y_y_label_position() {
    let primary = vec![make_temperature_line()];
    let secondary = vec![make_rainfall_line()];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary)
        .with_y_label("Temp");
    let computed = ComputedLayout::from_layout(&layout);
    let scene = render_twin_y(primary, secondary, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/twin_y_y_label_pos.svg", svg.clone()).unwrap();

    let label_x = extract_text_x(&svg, "Temp").expect("y-label 'Temp' not found in SVG");
    // Y label is placed just left of tick labels: margin_left - 8 - y_tick_label_px - 5 - label_size/2.
    let expected_x = (computed.margin_left - 8.0 - computed.y_tick_label_px - 5.0
        - computed.label_size as f64 * 0.5)
        .max(computed.label_size as f64 * 0.5 + 3.0);
    assert!(
        (label_x - expected_x).abs() < 0.5,
        "y-label x ({label_x}) should be ~{expected_x:.1} (margin_left - tick_label - gaps)"
    );
}

#[test]
fn test_twin_y_bar_primary() {
    let bar = Plot::Bar(
        BarPlot::new()
            .with_bar("A", 10.0)
            .with_bar("B", 20.0)
            .with_legend(vec!["Counts"]),
    );
    let secondary = vec![make_rainfall_line()];
    let primary = vec![bar];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary);
    let scene = render_twin_y(primary, secondary, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/twin_y_bar_primary.svg", svg.clone()).unwrap();

    assert!(svg.contains("<rect"), "SVG should contain <rect elements (bars rendered)");
}

#[test]
fn test_twin_y_bar_secondary() {
    let bar = Plot::Bar(
        BarPlot::new()
            .with_bar("A", 100.0)
            .with_bar("B", 200.0)
            .with_legend(vec!["Secondary"]),
    );
    let primary = vec![make_temperature_line()];
    let secondary = vec![bar];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary);
    let scene = render_twin_y(primary, secondary, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/twin_y_bar_secondary.svg", svg.clone()).unwrap();

    assert!(svg.contains("<rect"), "SVG should contain <rect elements (bars rendered)");
}

#[test]
fn test_twin_y_histogram_primary() {
    let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let hist = Plot::Histogram(
        Histogram::new()
            .with_data(data)
            .with_range((0.0, 10.0)),
    );
    let primary = vec![hist];
    let secondary = vec![make_rainfall_line()];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary);
    let scene = render_twin_y(primary, secondary, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/twin_y_histogram_primary.svg", svg.clone()).unwrap();

    assert!(svg.contains("<rect"), "SVG should contain <rect elements (histogram bars rendered)");
}

/// GC bias-style showcase: Genome GC histogram + Coverage scatter on primary (Norm. Coverage);
/// Reported BQ + Empirical BQ lines on secondary (Base Quality 0–40). Mirrors the layout of
/// a typical WGS GC bias QC chart.
#[test]
fn test_twin_y_showcase() {
    // --- Genome GC histogram (primary axis, y 0–0.5) ---
    // Precomputed bell-curve fraction per 2 % GC bin, peak at ~38 % GC
    let edges: Vec<f64> = (0..=100).step_by(2).map(|x| x as f64).collect(); // 51 edges → 50 bins
    let counts: Vec<f64> = (0..50_usize).map(|i| {
        let gc_center = i as f64 * 2.0 + 1.0; // bin midpoint: 1, 3, 5, … 99
        0.50 * (-0.5 * ((gc_center - 38.0) / 14.0).powi(2)).exp()
    }).collect();

    let genome_gc = Plot::Histogram(
        Histogram::from_bins(edges, counts)
            .with_color("#a8d8f0")
            .with_legend("Genome GC"),
    );

    // --- Coverage scatter (primary axis, y ~0.9–2.0, clamped at 2.0) ---
    // U-shaped: near 1.0 at mid GC, saturates to 2.0 at extreme GC
    let coverage_pts: Vec<(f64, f64)> = {
        let mut v = Vec::new();
        // Extreme low GC — clamped
        for gc in [2.0, 4.0, 6.0, 8.0, 10.0] { v.push((gc, 2.0)); }
        // Transition low
        for (gc, cov) in [(12.0,1.70),(14.0,1.45),(16.0,1.35),(18.0,1.25),(20.0,1.15)] {
            v.push((gc, cov));
        }
        // Mid range — dense U-shaped minimum
        for i in 0..=24_u32 {
            let gc = 22.0 + i as f64 * 2.0;
            let cov = 0.90 + 0.35 * ((gc - 50.0) / 35.0).powi(2);
            v.push((gc, cov.min(2.0)));
        }
        // Transition high
        for (gc, cov) in [(72.0,1.05),(74.0,1.10),(76.0,1.20),(78.0,1.30),(80.0,1.45)] {
            v.push((gc, cov));
        }
        // Extreme high GC — clamped
        for (gc, cov) in [(82.0,1.60),(84.0,1.75),(86.0,1.80)] { v.push((gc, cov)); }
        for gc in [88.0, 90.0, 92.0, 94.0, 96.0, 98.0] { v.push((gc, 2.0)); }
        v
    };

    let coverage = Plot::Scatter(
        ScatterPlot::new()
            .with_data(coverage_pts)
            .with_color("#4e90d9")
            .with_size(5.0)
            .with_legend("Coverage"),
    );

    // --- Reported BQ line (secondary axis, Base Quality 0–40) ---
    // Broadly flat ~28–30 across mid GC, falling off at extremes
    let reported_bq: Vec<(f64, f64)> = (1..=20_u32).map(|i| {
        let gc = i as f64 * 5.0;
        let bq = if gc < 15.0 || gc > 85.0 {
            22.0 - (gc - 50.0).abs() * 0.3
        } else {
            29.5 - (gc - 50.0).abs() * 0.025
        };
        (gc, bq.clamp(8.0, 40.0))
    }).collect();

    // --- Empirical BQ line (secondary axis, lower trace ~14–16) ---
    let empirical_bq: Vec<(f64, f64)> = (1..=20_u32).map(|i| {
        let gc = i as f64 * 5.0;
        let bq = if gc < 15.0 || gc > 85.0 {
            10.0 - (gc - 50.0).abs() * 0.1
        } else {
            15.0 - (gc - 50.0).abs() * 0.01
        };
        (gc, bq.clamp(4.0, 40.0))
    }).collect();

    let reported = Plot::Line(
        LinePlot::new()
            .with_data(reported_bq)
            .with_color("#2ca02c")
            .with_legend("Reported BQ"),
    );
    let empirical = Plot::Line(
        LinePlot::new()
            .with_data(empirical_bq)
            .with_color("#17becf")
            .with_legend("Empirical BQ"),
    );

    let primary   = vec![genome_gc, coverage];
    let secondary = vec![reported, empirical];

    let layout = Layout::auto_from_twin_y_plots(&primary, &secondary)
        .with_title("GC Bias — Twin Y Showcase")
        .with_x_label("GC%")
        .with_y_label("Normalized Coverage")
        .with_y2_label("Base Quality")
        .with_legend_position(LegendPosition::OutsideRightTop);

    let scene = render_twin_y(primary, secondary, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/twin_y_showcase.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"), "SVG should be valid");
    assert!(svg.contains("GC Bias"), "SVG should contain the title");
    assert!(svg.contains("Genome GC"), "SVG should contain Genome GC legend entry");
    assert!(svg.contains("Coverage"), "SVG should contain Coverage legend entry");
    assert!(svg.contains("Reported BQ"), "SVG should contain Reported BQ legend entry");
    assert!(svg.contains("Empirical BQ"), "SVG should contain Empirical BQ legend entry");
    assert!(svg.contains("<rect"), "SVG should contain histogram bars for Genome GC");
}
