mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::horizon::HorizonPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_horizon;

fn svg(hp: HorizonPlot) -> String {
    let layout = Layout::auto_from_plots(&[Plot::Horizon(hp.clone())]);
    SvgBackend.render_scene(&render_horizon(hp, layout))
}

fn svg_with_layout(hp: HorizonPlot, layout: Layout) -> String {
    SvgBackend.render_scene(&render_horizon(hp, layout))
}

fn save(name: &str, content: &str) {
    std::fs::create_dir_all("test_outputs").unwrap();
    common::write_test_output(format!("test_outputs/{name}.svg"), content).unwrap();
}

// --- structural tests ---

#[test]
fn test_horizon_empty() {
    let hp = HorizonPlot::new();
    let out = svg(hp);
    assert!(out.contains("<svg"));
}

#[test]
fn test_horizon_single_series_positive() {
    let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&t| (t * 0.5).sin() * 5.0 + 3.0).collect();
    let hp = HorizonPlot::new().with_series("CPU", x, y);
    let out = svg(hp);
    assert!(out.contains("<path"));
    assert!(out.contains("opacity"));
    save("horizon_single_positive", &out);
}

#[test]
fn test_horizon_single_series_with_negatives() {
    let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&t| (t * 0.4).sin() * 8.0).collect();
    let hp = HorizonPlot::new().with_series("Temp", x, y);
    let out = svg(hp);
    // Should have both positive (palette slot 0 = #1f77b4) and negative (#d62728) paths
    assert!(out.contains("#1f77b4")); // palette pos_color for series 0
    assert!(out.contains("#d62728")); // default neg_color
    save("horizon_single_negatives", &out);
}

#[test]
fn test_horizon_multi_series() {
    let x: Vec<f64> = (0..30).map(|i| i as f64).collect();
    let hp = HorizonPlot::new()
        .with_series(
            "Series A",
            x.clone(),
            x.iter()
                .map(|&t| (t * 0.3).sin() * 10.0)
                .collect::<Vec<_>>(),
        )
        .with_series(
            "Series B",
            x.clone(),
            x.iter()
                .map(|&t| (t * 0.5 + 1.0).cos() * 8.0)
                .collect::<Vec<_>>(),
        )
        .with_series(
            "Series C",
            x.clone(),
            x.iter().map(|&t| t * 0.3 - 4.5).collect::<Vec<_>>(),
        );
    let out = svg(hp);
    // Three rows of fills
    assert!(out.contains("Series A"));
    assert!(out.contains("Series B"));
    assert!(out.contains("Series C"));
    save("horizon_multi_series", &out);
}

#[test]
fn test_horizon_n_bands_1() {
    let x: Vec<f64> = (0..15).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&t| t * 0.5).collect();
    let hp = HorizonPlot::new().with_series("S", x, y).with_n_bands(1);
    let out = svg(hp);
    assert!(out.contains("<path"));
    save("horizon_1_band", &out);
}

#[test]
fn test_horizon_n_bands_5() {
    let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&t| (t * 0.4).sin() * 15.0).collect();
    let hp = HorizonPlot::new()
        .with_series("Pressure", x, y)
        .with_n_bands(5);
    let out = svg(hp);
    assert!(out.contains("opacity"));
    save("horizon_5_bands", &out);
}

#[test]
fn test_horizon_custom_colors() {
    let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&t| (t * 0.4).sin() * 8.0).collect();
    let hp = HorizonPlot::new().with_series_colored("Metric", x, y, "#2ca02c", "#ff7f0e");
    let out = svg(hp);
    // #2ca02c = green (pos), #ff7f0e = orange (neg)
    assert!(out.contains("#2ca02c")); // green positive
    save("horizon_custom_colors", &out);
}

#[test]
fn test_horizon_baseline_nonzero() {
    let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&t| t * 0.5 + 5.0).collect();
    let hp = HorizonPlot::new()
        .with_series("Metric", x, y)
        .with_baseline(10.0);
    let out = svg(hp);
    assert!(out.contains("<path"));
    save("horizon_nonzero_baseline", &out);
}

#[test]
fn test_horizon_value_max_override() {
    let x: Vec<f64> = (0..15).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&t| t * 0.3).collect();
    let hp = HorizonPlot::new()
        .with_series("S", x, y)
        .with_value_max(10.0);
    let out = svg(hp);
    assert!(out.contains("<path"));
    save("horizon_value_max", &out);
}

#[test]
fn test_horizon_row_height_auto_sizing() {
    let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let hp = HorizonPlot::new()
        .with_series(
            "A",
            x.clone(),
            x.iter().map(|&t| t * 0.5).collect::<Vec<_>>(),
        )
        .with_series(
            "B",
            x.clone(),
            x.iter().map(|&t| -t * 0.3).collect::<Vec<_>>(),
        )
        .with_series(
            "C",
            x.clone(),
            x.iter()
                .map(|&t| (t - 10.0).abs() - 5.0)
                .collect::<Vec<_>>(),
        )
        .with_row_height(60.0);

    let plots = vec![Plot::Horizon(hp.clone())];
    let layout = Layout::auto_from_plots(&plots);
    // With 3 series at 60px each, canvas should be sized appropriately
    // Just verify it builds
    let out = svg_with_layout(hp, layout);
    assert!(out.contains("<path"));
    save("horizon_row_height", &out);
}

#[test]
fn test_horizon_legend() {
    let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let hp = HorizonPlot::new()
        .with_series(
            "CPU",
            x.clone(),
            x.iter()
                .map(|&t| (t * 0.4).sin() * 10.0)
                .collect::<Vec<_>>(),
        )
        .with_series(
            "Memory",
            x.clone(),
            x.iter().map(|&t| t * 0.4).collect::<Vec<_>>(),
        )
        .with_legend(true);
    let out = svg(hp);
    assert!(out.contains("CPU"));
    assert!(out.contains("Memory"));
    // Series 0 gets palette slot 0 (#1f77b4), series 1 gets slot 1 (#ff7f0e) — distinct swatches
    assert!(out.contains("#1f77b4"));
    assert!(out.contains("#ff7f0e"));
    save("horizon_legend", &out);
}

#[test]
fn test_horizon_with_x_label() {
    let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let hp = HorizonPlot::new().with_series(
        "Series",
        x.clone(),
        x.iter().map(|&t| (t * 0.5).sin() * 5.0).collect::<Vec<_>>(),
    );
    let plots = vec![Plot::Horizon(hp.clone())];
    let layout = Layout::auto_from_plots(&plots)
        .with_x_label("Time (hours)")
        .with_title("Horizon Chart");
    let out = svg_with_layout(hp, layout);
    assert!(out.contains("Time (hours)"));
    assert!(out.contains("Horizon Chart"));
    save("horizon_with_labels", &out);
}

#[test]
fn test_horizon_pos_band_width() {
    // pos_band_width = max_pos_deviation / n_bands
    let x = vec![0.0, 1.0, 2.0];
    let y = vec![3.0, 6.0, 9.0]; // max deviation from 0 = 9
    let hp = HorizonPlot::new().with_series("S", x, y).with_n_bands(3);
    assert!((hp.pos_band_width() - 3.0).abs() < 1e-9);
}

#[test]
fn test_horizon_neg_band_width() {
    let x = vec![0.0, 1.0, 2.0];
    let y = vec![-3.0, -6.0, -9.0]; // max neg deviation = 9
    let hp = HorizonPlot::new().with_series("S", x, y).with_n_bands(3);
    assert!((hp.neg_band_width() - 3.0).abs() < 1e-9);
}

#[test]
fn test_horizon_value_max_overrides_band_width() {
    let x = vec![0.0, 1.0];
    let y = vec![1.0, 2.0]; // max deviation = 2 normally
    let hp = HorizonPlot::new()
        .with_series("S", x, y)
        .with_n_bands(2)
        .with_value_max(10.0); // override: bw = 10/2 = 5
    assert!((hp.pos_band_width() - 5.0).abs() < 1e-9);
}

#[test]
fn test_horizon_x_range() {
    let hp = HorizonPlot::new()
        .with_series("S1", vec![1.0, 2.0, 3.0], vec![0.0, 1.0, 2.0])
        .with_series("S2", vec![-1.0, 0.0, 5.0], vec![0.0, 0.0, 0.0]);
    let (xmin, xmax) = hp.x_range().unwrap();
    assert!((xmin - (-1.0)).abs() < 1e-9);
    assert!((xmax - 5.0).abs() < 1e-9);
}

#[test]
fn test_horizon_empty_x_range() {
    let hp = HorizonPlot::new();
    assert!(hp.x_range().is_none());
}

#[test]
fn test_horizon_n_series() {
    let x = vec![0.0, 1.0];
    let hp = HorizonPlot::new()
        .with_series("A", x.clone(), vec![1.0, 2.0])
        .with_series("B", x.clone(), vec![2.0, 3.0]);
    assert_eq!(hp.n_series(), 2);
}

// --- showcase tests ---

#[test]
fn test_horizon_server_metrics_showcase() {
    // Simulate server monitoring: CPU, Memory, Network I/O, Disk I/O over 48h
    let hours: Vec<f64> = (0..48).map(|i| i as f64).collect();

    let cpu: Vec<f64> = hours
        .iter()
        .map(|&h| {
            let base = 30.0 + 20.0 * (h * 0.2).sin();
            let spike = if (h as usize % 12) == 6 { 40.0 } else { 0.0 };
            base + spike - 10.0
        })
        .collect();

    let mem: Vec<f64> = hours
        .iter()
        .map(|&h| 50.0 + 30.0 * (h / 24.0) - 5.0 * (h * 0.5).cos())
        .collect();

    let net: Vec<f64> = hours
        .iter()
        .map(|&h| {
            let traffic = (h * 0.8).sin() * 50.0 + 20.0;
            traffic - 30.0 // can go negative (below baseline)
        })
        .collect();

    let disk: Vec<f64> = hours
        .iter()
        .map(|&h| {
            if h > 24.0 {
                (h - 24.0) * 2.0 - 10.0
            } else {
                (h * 0.5).sin() * 5.0
            }
        })
        .collect();

    let hp = HorizonPlot::new()
        .with_series_colored("CPU %", hours.clone(), cpu, "#4292c6", "#d73027")
        .with_series_colored("Memory %", hours.clone(), mem, "#2ca02c", "#d73027")
        .with_series_colored("Net MB/s", hours.clone(), net, "#9467bd", "#e6550d")
        .with_series_colored("Disk MB/s", hours.clone(), disk, "#8c564b", "#e6550d")
        .with_n_bands(3)
        .with_baseline(0.0)
        .with_legend(true);

    let layout = Layout::auto_from_plots(&[Plot::Horizon(hp.clone())])
        .with_title("Server Metrics — 48h Window")
        .with_x_label("Hour");

    let out = svg_with_layout(hp, layout);
    assert!(out.contains("CPU %"));
    assert!(out.contains("Memory %"));
    assert!(out.contains("Net MB/s"));
    assert!(out.contains("Disk MB/s"));
    save("horizon_server_metrics", &out);
}

#[test]
fn test_horizon_temperature_anomaly_showcase() {
    // Temperature anomaly chart: deviation from 1990-2020 baseline
    // Values range from -3°C to +3°C
    let months: Vec<f64> = (0..120).map(|i| i as f64).collect();
    let anomaly: Vec<f64> = months
        .iter()
        .map(|&m| {
            let trend = m * 0.025 - 1.5; // warming trend
            let seasonal = (m * std::f64::consts::TAU / 12.0).sin() * 0.5;
            let noise = ((m * 7.3).sin() + (m * 3.1).cos()) * 0.3;
            trend + seasonal + noise
        })
        .collect();

    let hp = HorizonPlot::new()
        .with_series_colored(
            "Global Temp Anomaly (°C)",
            months,
            anomaly,
            "#d73027", // warm = red
            "#4575b4", // cool = blue
        )
        .with_n_bands(3)
        .with_baseline(0.0)
        .with_row_height(80.0); // triggers auto canvas sizing

    let layout = Layout::auto_from_plots(&[Plot::Horizon(hp.clone())])
        .with_title("Temperature Anomaly 1990–2020")
        .with_x_label("Month");

    let out = svg_with_layout(hp, layout);
    assert!(out.contains("Global Temp Anomaly"));
    save("horizon_temperature_anomaly", &out);
}

#[test]
fn test_horizon_financial_showcase() {
    // Multi-stock daily returns: percentage change from previous day
    let days: Vec<f64> = (0..252).map(|i| i as f64).collect();

    let stocks = vec![
        ("AAPL", 0.3_f64, 0.7_f64),
        ("GOOG", 0.25, 0.9),
        ("MSFT", 0.2, 0.5),
        ("AMZN", 0.35, 1.1),
        ("META", 0.4, 0.8),
    ];

    let mut hp = HorizonPlot::new().with_n_bands(3);
    for (name, freq, amp) in stocks {
        let returns: Vec<f64> = days
            .iter()
            .map(|&d| {
                let trend = (d * freq * 0.01).sin() * amp;
                let noise = (d * freq * 7.3).sin() * 0.5;
                (trend + noise) * 2.0
            })
            .collect();
        hp = hp.with_series_colored(name, days.clone(), returns, "#4292c6", "#d73027");
    }

    let layout = Layout::auto_from_plots(&[Plot::Horizon(hp.clone())])
        .with_title("Daily Stock Returns (%)")
        .with_x_label("Trading Day");

    let out = svg_with_layout(hp, layout);
    assert!(out.contains("AAPL"));
    assert!(out.contains("META"));
    save("horizon_financial", &out);
}

#[test]
fn test_horizon_many_rows_dense_timeseries() {
    // 32 series × 365 time points: simulates a full year of daily sensor readings
    // across a panel of instruments, with each instrument having its own frequency,
    // amplitude, drift, and occasional spike pattern.
    let pi = std::f64::consts::PI;
    let days: Vec<f64> = (0..365).map(|i| i as f64).collect();

    // Instrument names — a mix of long and short labels to exercise margin sizing
    let instruments: &[(&str, f64, f64, f64, f64)] = &[
        ("Temp sensor A1", 1.0, 8.0, 0.01, 2.5),
        ("Temp sensor A2", 1.0, 7.5, -0.008, 1.8),
        ("Temp sensor B1", 2.0, 6.0, 0.005, 3.1),
        ("Temp sensor B2", 2.0, 5.5, -0.012, 2.0),
        ("Humidity East", 0.5, 12.0, 0.003, 4.0),
        ("Humidity West", 0.5, 11.0, -0.006, 3.5),
        ("Pressure roof", 3.0, 4.0, 0.002, 1.2),
        ("Pressure basement", 3.0, 3.5, -0.004, 1.0),
        ("CO2 lab 1", 0.8, 9.0, 0.015, 5.0),
        ("CO2 lab 2", 0.8, 8.5, -0.010, 4.2),
        ("CO2 office", 1.2, 7.0, 0.007, 3.8),
        ("VOC corridor", 1.5, 6.5, 0.009, 2.9),
        ("Wind speed N", 2.5, 10.0, 0.000, 6.0),
        ("Wind speed S", 2.5, 9.5, 0.001, 5.5),
        ("Solar irradiance", 1.0, 15.0, 0.020, 7.0),
        ("UV index", 1.0, 5.0, 0.018, 2.0),
        ("Soil moisture 10cm", 0.3, 11.0, -0.005, 4.5),
        ("Soil moisture 30cm", 0.3, 10.0, -0.003, 3.9),
        ("Soil moisture 60cm", 0.3, 8.0, -0.002, 3.2),
        ("Groundwater depth", 0.2, 6.0, 0.004, 2.1),
        ("River flow rate", 0.7, 13.0, 0.006, 5.8),
        ("Reservoir level", 0.4, 9.0, -0.007, 4.1),
        ("Grid voltage", 4.0, 2.0, 0.000, 0.8),
        ("Grid frequency", 4.0, 1.5, 0.000, 0.5),
        ("Panel output kW", 1.0, 11.0, 0.022, 4.8),
        ("Battery SoC %", 0.6, 7.0, -0.015, 3.0),
        ("Load avg 1min", 1.8, 8.0, 0.008, 3.6),
        ("Load avg 5min", 1.8, 7.5, 0.007, 3.4),
        ("Network rx MB/s", 2.2, 9.5, 0.000, 4.7),
        ("Network tx MB/s", 2.2, 6.0, 0.000, 2.8),
        ("Disk read MB/s", 3.5, 5.0, 0.000, 2.2),
        ("Disk write MB/s", 3.5, 4.5, 0.000, 1.9),
    ];

    let mut hp = HorizonPlot::new()
        .with_n_bands(3)
        .with_baseline(0.0)
        .with_row_height(24.0);

    // Two color pairs — alternate between blue/red and teal/orange
    let color_pairs = [("#4292c6", "#d73027"), ("#2ca02c", "#e6550d")];

    for (idx, &(name, freq, amp, drift, noise_amp)) in instruments.iter().enumerate() {
        let (pos_col, neg_col) = color_pairs[idx % 2];
        let y: Vec<f64> = days
            .iter()
            .map(|&d| {
                let seasonal = (d * freq * pi / 182.5).sin() * amp;
                let diurnal = (d * 2.0 * pi / 365.0).cos() * amp * 0.3;
                let drift_val = d * drift;
                let noise = ((d * freq * 13.7).sin() + (d * freq * 5.3).cos()) * noise_amp * 0.4;
                // Occasional spike on a different schedule per instrument
                let spike = if (d as usize + idx * 17) % 30 == 0 {
                    amp * 1.5
                } else {
                    0.0
                };
                seasonal + diurnal + drift_val + noise + spike
            })
            .collect();

        hp = hp.with_series_colored(name, days.clone(), y, pos_col, neg_col);
    }

    let layout = Layout::auto_from_plots(&[Plot::Horizon(hp.clone())])
        .with_title("Environmental & Infrastructure Monitoring — 365 Days")
        .with_x_label("Day of year");

    let out = svg_with_layout(hp, layout);

    // All 32 series names appear in the y-axis labels
    assert!(out.contains("Temp sensor A1"));
    assert!(out.contains("Disk write MB/s"));
    assert!(out.contains("Solar irradiance"));
    assert!(out.contains("Groundwater depth"));

    // Both color ramps are used
    assert!(out.contains("#4292c6") || out.contains("#2ca02c")); // positive
    assert!(out.contains("#d73027") || out.contains("#e6550d")); // negative

    // Multiple paths drawn
    let path_count = out.matches("<path").count();
    assert!(path_count > 32, "expected > 32 paths, got {path_count}");

    save("horizon_many_rows_dense", &out);
}

// --- value label / sign color annotation tests ---

#[test]
fn test_horizon_value_labels_text_emitted() {
    // show_value_labels=true should produce "+" text annotations in the SVG
    let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&t| (t - 10.0).abs() - 5.0).collect(); // pos + neg
    let hp = HorizonPlot::new()
        .with_series("Signal", x, y)
        .with_n_bands(3)
        .with_value_labels(true);
    let out = svg(hp);
    // A "+" annotation should appear in the SVG text elements
    assert!(
        out.contains(">+"),
        "expected '+' annotation text, not found in SVG"
    );
    assert!(
        out.contains(">-"),
        "expected '-' annotation text, not found in SVG"
    );
    save("horizon_value_labels", &out);
}

#[test]
fn test_horizon_value_labels_pos_only() {
    // All-positive series: only "+" annotation, no "-"
    let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&t| t * 0.5 + 1.0).collect();
    let hp = HorizonPlot::new()
        .with_series("Pos", x, y)
        .with_value_labels(true);
    let out = svg(hp);
    assert!(
        out.contains(">+"),
        "expected '+' label for positive-only series"
    );
    // No negative annotation
    let minus_count = out.matches(">-").count();
    assert_eq!(
        minus_count, 0,
        "no '-' label expected for all-positive series, got {minus_count}"
    );
    save("horizon_value_labels_pos_only", &out);
}

#[test]
fn test_horizon_sign_colors_colorize_sign_chars() {
    // show_sign_colors=true should embed the pos_color into the "+" text element
    let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&t| (t - 10.0).abs() - 5.0).collect();
    let hp = HorizonPlot::new()
        .with_series_colored("S", x.clone(), y.clone(), "#1f77b4", "#d62728")
        .with_value_labels(true)
        .with_sign_colors(true);
    let out = svg(hp);
    // The pos_color should appear inside a <text fill="..."> or similar attribute
    assert!(
        out.contains("#1f77b4"),
        "pos_color should appear in SVG when sign_colors=true"
    );
    // neg_color on the "-" sign
    assert!(
        out.contains("#d62728"),
        "neg_color should appear in SVG when sign_colors=true"
    );
    save("horizon_sign_colors", &out);
}

#[test]
fn test_horizon_sign_colors_without_value_labels_is_noop() {
    // show_sign_colors=true has no effect when show_value_labels=false
    let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&t| (t - 10.0).abs() - 5.0).collect();
    let hp_no_labels = HorizonPlot::new()
        .with_series("S", x.clone(), y.clone())
        .with_sign_colors(true); // no value labels
    let hp_with_labels = HorizonPlot::new()
        .with_series("S", x, y)
        .with_value_labels(true)
        .with_sign_colors(true);
    let out_no = svg(hp_no_labels);
    let out_yes = svg(hp_with_labels);
    // The annotated version should have more text nodes
    let text_no = out_no.matches("<text").count();
    let text_yes = out_yes.matches("<text").count();
    assert!(
        text_yes > text_no,
        "value_labels+sign_colors should add extra text nodes"
    );
}

#[test]
fn test_horizon_annotations_expand_right_margin() {
    // With value labels the SVG should be wider (or the plot area narrower) than without.
    // We check that the SVG has more total width allocated for annotations by
    // verifying the width attribute differs, or that the margin is respected via
    // the fact that the SVG is otherwise valid and large enough.
    let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&t| (t - 10.0).abs() - 5.0).collect();

    let hp_plain = HorizonPlot::new().with_series("S", x.clone(), y.clone());
    let hp_annot = HorizonPlot::new()
        .with_series("S", x, y)
        .with_value_labels(true);

    let layout_plain = Layout::auto_from_plots(&[Plot::Horizon(hp_plain.clone())]);
    let layout_annot = Layout::auto_from_plots(&[Plot::Horizon(hp_annot.clone())]);

    // The annotated layout should have a larger horizon_right_annot_px (non-zero)
    assert!(
        layout_annot.horizon_right_annot_px > 0.0,
        "annotated layout should have non-zero horizon_right_annot_px"
    );
    assert_eq!(
        layout_plain.horizon_right_annot_px, 0.0,
        "plain layout should have zero horizon_right_annot_px"
    );

    save(
        "horizon_annotations_margin",
        &svg_with_layout(hp_annot, layout_annot),
    );
}

#[test]
fn test_horizon_multi_series_value_labels_showcase() {
    // Multi-series with value labels and sign colors — realistic showcase
    let months: Vec<f64> = (0..60).map(|i| i as f64).collect();
    let sensors = [
        ("CPU temp", 0.8_f64, 15.0_f64),
        ("GPU temp", 1.2, 20.0),
        ("Disk I/O", 0.3, 8.0),
        ("Net latency", 2.0, 5.0),
        ("Mem pressure", 0.5, 12.0),
    ];
    let mut hp = HorizonPlot::new()
        .with_n_bands(3)
        .with_row_height(40.0)
        .with_value_labels(true)
        .with_sign_colors(true);
    for (name, freq, amp) in &sensors {
        let y: Vec<f64> = months
            .iter()
            .map(|&t| {
                (t * freq * std::f64::consts::TAU / 60.0).sin() * amp
                    + ((t * freq * 5.1).cos()) * amp * 0.25
            })
            .collect();
        hp = hp.with_series(*name, months.clone(), y);
    }
    let layout = Layout::auto_from_plots(&[Plot::Horizon(hp.clone())])
        .with_title("Server Metrics — 5 Year Window")
        .with_x_label("Month");
    let out = svg_with_layout(hp, layout);
    assert!(out.contains("CPU temp"));
    assert!(out.contains("Mem pressure"));
    assert!(
        out.contains(">+"),
        "expected '+' annotation in multi-series showcase"
    );
    save("horizon_multi_value_labels", &out);
}

#[test]
fn test_horizon_all_positive_no_neg_paths() {
    // All-positive data: negative band paths should not be emitted
    let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&t| t * 0.5 + 1.0).collect(); // all > 0
    let hp = HorizonPlot::new().with_series("S", x, y);
    let out = svg(hp);
    // negative color (#d62728) should NOT appear since all values are positive
    assert!(!out.contains("#d62728"));
    save("horizon_all_positive", &out);
}

#[test]
fn test_horizon_all_negative_no_pos_paths() {
    // All-negative data: positive band paths should not be drawn
    let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&t| -t * 0.5 - 1.0).collect(); // all < 0
    let hp = HorizonPlot::new().with_series("S", x, y);
    let out = svg(hp);
    // negative color (#d62728) SHOULD appear
    assert!(out.contains("#d62728"));
    save("horizon_all_negative", &out);
}
