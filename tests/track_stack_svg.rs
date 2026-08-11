//! SVG-output tests for the `TrackStack` shared-x stacked-panel primitive (the library path).
//! Writes rendered figures to `test_outputs/` for local visual inspection (skipped under CI via
//! `common::write_test_output`); the CLI path is covered separately by `scripts/smoke_tests.sh`.
mod common;

use kuva::backend::svg::SvgBackend;
use kuva::plot::LinePlot;
use kuva::render::plots::Plot;
use kuva::render::track_stack::{
    AxisSpec, Interval, IntervalTrack, PlotTrack, RegionHighlight, TrackStack, VariantTrack,
};

/// Two line tracks with very different y-magnitudes sharing one axis — the x-alignment case
/// (`Figure`'s per-cell margins get this wrong; `TrackStack` forces a shared gutter).
#[test]
fn track_stack_two_tracks_x_aligned() {
    let small: Vec<(f64, f64)> = (0..=100)
        .map(|i| (i as f64, (i as f64 / 9.0).sin() * 0.5 + 0.5))
        .collect();
    let big: Vec<(f64, f64)> = (0..=100)
        .map(|i| (i as f64, (i as f64 / 13.0).cos() * 400_000.0 + 600_000.0))
        .collect();

    let scene = TrackStack::new()
        .x_range(0.0, 100.0)
        .track(
            PlotTrack::new(vec![Plot::Line(LinePlot::new().with_data(small))]).with_y_label("unit"),
        )
        .track(
            PlotTrack::new(vec![Plot::Line(LinePlot::new().with_data(big))]).with_y_label("counts"),
        )
        .x_axis()
        .render_sized(900.0, 340.0);

    let svg = SvgBackend::new().render_scene(&scene);
    common::write_test_output(
        "test_outputs/track_stack_two_tracks_x_aligned.svg",
        svg.clone(),
    )
    .unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("unit") && svg.contains("counts"));
}

/// Full genome-browser layout: a title, a coverage depth track, a variant lane, an amplicon lane
/// (overlapping amplicons tile onto two pool-coloured rows) above the axis, and a region-highlight
/// underlay spanning every track.
#[test]
fn track_stack_genome_browser() {
    let cov: Vec<(f64, f64)> = (0..=120)
        .map(|i| {
            let x = 1000.0 + i as f64 / 120.0 * 4000.0;
            (
                x,
                ((x / 250.0).sin().abs() + 0.3 * (x / 90.0).cos().abs()) * 400.0 + 30.0,
            )
        })
        .collect();

    let scene = TrackStack::new()
        .with_title("Amplicon coverage — chr1")
        .x_range(1000.0, 5000.0)
        .underlay(
            RegionHighlight::new(2820.0, 2980.0)
                .with_fill("#ffd166")
                .with_opacity(0.30),
        )
        .track(
            PlotTrack::new(vec![Plot::Line(
                LinePlot::new().with_data(cov).with_color("#2a9d8f"),
            )])
            .with_y_label("depth"),
        )
        .track(
            VariantTrack::new()
                .with_group("SNV", "#d1495b", vec![1420.0, 2380.0, 3910.0])
                .with_group("InDel", "#e9c46a", vec![2900.0, 4550.0])
                .with_name("variants")
                .with_height(14.0),
        )
        // Overlapping amplicons tile onto two rows, coloured by pool. Above the axis, with the data.
        .track(
            IntervalTrack::new(vec![
                Interval::new(1050.0, 2050.0).with_label("amp1"),
                Interval::new(1950.0, 2950.0).with_label("amp2"),
                Interval::new(2850.0, 3850.0).with_label("amp3"),
            ])
            .with_name("amplicons")
            .with_row_height(18.0)
            .with_row_colors(vec!["#8ecae6", "#ffb703"]),
        )
        .x_axis_with(AxisSpec::genomic("chr1 position (bp)"))
        .render_sized(940.0, 380.0);

    let svg = SvgBackend::new().render_scene(&scene);
    common::write_test_output("test_outputs/track_stack_genome_browser.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Amplicon coverage"));
    assert!(svg.contains("SNV") && svg.contains("amp1"));
    assert!(svg.contains(" bp") || svg.contains(" kb"));
}

/// Non-genomic use case: a financial-style stack over a DATE axis (x = Unix seconds), with a
/// *different* continuous plot type in the second track (scatter, not line). Exercises
/// `XAxisFormat::DateTime` and confirms `PlotTrack` works for any continuous-x plot.
#[test]
fn track_stack_datetime_scatter_and_line() {
    use kuva::plot::ScatterPlot;

    // 2024-01-01 00:00:00 UTC, weekly for ~30 weeks.
    let start = 1_704_067_200.0_f64;
    let week = 604_800.0_f64;
    let price: Vec<(f64, f64)> = (0..30)
        .map(|i| {
            let t = start + i as f64 * week;
            (t, 100.0 + (i as f64 / 4.0).sin() * 20.0 + i as f64 * 0.8)
        })
        .collect();
    let volume: Vec<(f64, f64)> = (0..30)
        .map(|i| {
            let t = start + i as f64 * week;
            (t, (i as f64 * 1.3).cos().abs() * 500.0 + 100.0)
        })
        .collect();

    let scene = TrackStack::new()
        .with_title("ACME Corp — weekly")
        .track(
            PlotTrack::new(vec![Plot::Line(
                LinePlot::new().with_data(price).with_color("#2a9d8f"),
            )])
            .with_y_label("price ($)"),
        )
        .track(
            PlotTrack::new(vec![Plot::Scatter(
                ScatterPlot::new().with_data(volume).with_color("#e76f51"),
            )])
            .with_y_label("volume")
            .with_height(kuva::render::track_stack::TrackHeight::Flex(0.6)),
        )
        .x_axis_with(AxisSpec::datetime("week"))
        .render_sized(900.0, 320.0);

    let svg = SvgBackend::new().render_scene(&scene);
    common::write_test_output(
        "test_outputs/track_stack_datetime_scatter_and_line.svg",
        svg.clone(),
    )
    .unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("price ($)") && svg.contains("volume"));
    // A date-formatted tick (year 2024 appears in the auto date format for a ~7-month span).
    assert!(svg.contains("2024"));
}
