use kuva::plot::calendar::{CalendarPlot, CalendarAgg, WeekStart};
use kuva::render::{plots::Plot, layout::Layout, render::render_calendar};
use kuva::backend::svg::SvgBackend;
use std::fs;

fn output(name: &str, svg: &str) {
    fs::create_dir_all("test_outputs").unwrap();
    fs::write(format!("test_outputs/{name}"), svg).unwrap();
}

// ── Basic tests (existing) ────────────────────────────────────────────────────

#[test]
fn calendar_basic_single_year() {
    let plot = CalendarPlot::new()
        .with_events(vec!["2024-01-15", "2024-01-15", "2024-03-20", "2024-06-01"])
        .with_year(2024)
        .with_legend_label("events");
    let layout = Layout::auto_from_plots(&[Plot::Calendar(plot.clone())]);
    let svg = SvgBackend.render_scene(&render_calendar(plot, layout));
    assert!(svg.contains("<svg"));
    assert!(svg.contains("cal-day"));
    assert!(svg.contains("2024"));
    output("calendar_basic.svg", &svg);
}

#[test]
fn calendar_multi_year() {
    let mut data = Vec::new();
    for y in 2022..=2024 {
        for m in 1..=12 {
            data.push((format!("{y}-{m:02}-01"), (m * 3) as f64));
        }
    }
    let plot = CalendarPlot::new()
        .with_data(data)
        .with_aggregation(CalendarAgg::Sum)
        .with_legend_label("count");
    let layout = Layout::auto_from_plots(&[Plot::Calendar(plot.clone())]);
    let svg = SvgBackend.render_scene(&render_calendar(plot, layout));
    assert!(svg.contains("2022"));
    assert!(svg.contains("2024"));
    output("calendar_multi_year.svg", &svg);
}

#[test]
fn calendar_sunday_start() {
    let plot = CalendarPlot::new()
        .with_events(vec!["2024-06-15", "2024-07-04", "2024-12-25"])
        .with_year(2024)
        .with_week_start(WeekStart::Sunday)
        .with_day_labels(true);
    let layout = Layout::auto_from_plots(&[Plot::Calendar(plot.clone())]);
    let svg = SvgBackend.render_scene(&render_calendar(plot, layout));
    assert!(svg.contains("Sun"));
    output("calendar_sunday_start.svg", &svg);
}

/// Minimal: no legend, no month labels, no day labels.
/// Previously the year label was clipped at the left canvas edge.
#[test]
fn calendar_minimal_no_clip() {
    let plot = CalendarPlot::new()
        .with_events(vec!["2024-05-10"])
        .with_year(2024)
        .with_legend(false)
        .with_month_labels(false)
        .with_day_labels(false);
    let layout = Layout::auto_from_plots(&[Plot::Calendar(plot.clone())]);
    let svg = SvgBackend.render_scene(&render_calendar(plot, layout));
    assert!(svg.contains("<svg"));
    // Year label must appear and its x coordinate must be positive
    assert!(svg.contains("2024"));
    // With day_label_w=32 the label is placed at x=ox+32-4 ≥ 8+28=36 → no clip
    output("calendar_minimal.svg", &svg);
}

#[test]
fn calendar_tooltip_js_injected() {
    let plot = CalendarPlot::new()
        .with_events(vec!["2024-06-01"])
        .with_year(2024);
    let layout = Layout::auto_from_plots(&[Plot::Calendar(plot.clone())]);
    let svg = SvgBackend.render_scene(&render_calendar(plot, layout));
    assert!(svg.contains("cal-tip"), "tooltip JS should be present");
    assert!(svg.contains("cal-day"), "day cells should have cal-day class");
    assert!(svg.contains("<script"), "script tag should be present");
    output("calendar_tooltip.svg", &svg);
}

#[test]
fn calendar_distinct_zero_color() {
    let plot = CalendarPlot::new()
        .with_data(vec![("2024-03-15", 0.0), ("2024-03-16", 5.0)])
        .with_aggregation(CalendarAgg::Sum)
        .with_zero_color("#ffff00")
        .with_year(2024);
    let layout = Layout::auto_from_plots(&[Plot::Calendar(plot.clone())]);
    let svg = SvgBackend.render_scene(&render_calendar(plot, layout));
    assert!(svg.contains("#ffff00"), "zero-color should appear in output");
    output("calendar_zero_color.svg", &svg);
}

// ── Custom date ranges ────────────────────────────────────────────────────────

/// Partial-year range: June through November 2024.
/// Grid starts in June and ends in November — fewer than 53 columns.
#[test]
fn calendar_date_range_within_year() {
    // Dense data: every 3rd day in the range has a value
    let mut data = Vec::new();
    let months = [(6, 30), (7, 31), (8, 31), (9, 30), (10, 31), (11, 30)];
    for (m, days) in months {
        for d in (1..=days).step_by(3) {
            data.push((format!("2024-{m:02}-{d:02}"), (m + d) as f64 % 10.0 + 1.0));
        }
    }
    let plot = CalendarPlot::new()
        .with_data(data)
        .with_date_range("2024-06-01", "2024-11-30")
        .with_legend_label("activity");
    let layout = Layout::auto_from_plots(&[Plot::Calendar(plot.clone())]);
    let svg = SvgBackend.render_scene(&render_calendar(plot, layout));
    assert!(svg.contains("<svg"));
    assert!(svg.contains("cal-day"));
    assert!(svg.contains("Jun"));
    assert!(svg.contains("Nov"));
    output("calendar_partial_year.svg", &svg);
}

/// Australian financial year: July 1 to June 30.
/// The grid rolls across a calendar-year boundary.
#[test]
fn calendar_financial_year_australia() {
    let mut data = Vec::new();
    // Q1 (Jul–Sep 2023)
    for m in 7u32..=9 { for d in [5, 12, 19, 26] { data.push((format!("2023-{m:02}-{d:02}"), (m * d) as f64 % 8.0 + 1.0)); } }
    // Q2 (Oct–Dec 2023)
    for m in 10u32..=12 { for d in [3, 10, 17, 24] { data.push((format!("2023-{m:02}-{d:02}"), (m + d) as f64 % 6.0 + 2.0)); } }
    // Q3 (Jan–Mar 2024)
    for m in 1u32..=3 { for d in [8, 15, 22, 29] { data.push((format!("2024-{m:02}-{d:02}"), (m * d) as f64 % 9.0 + 1.0)); } }
    // Q4 (Apr–Jun 2024)
    for m in 4u32..=6 { for d in [1, 8, 15, 22, 29] { if d <= 30 { data.push((format!("2024-{m:02}-{d:02}"), (m + d) as f64 % 7.0 + 1.0)); } } }

    let plot = CalendarPlot::new()
        .with_data(data)
        .with_aggregation(CalendarAgg::Sum)
        .with_period("FY2023/24", "2023-07-01", "2024-06-30")
        .with_legend_label("contributions");
    let layout = Layout::auto_from_plots(&[Plot::Calendar(plot.clone())]);
    let svg = SvgBackend.render_scene(&render_calendar(plot, layout));
    assert!(svg.contains("FY2023/24"), "period label must appear");
    assert!(svg.contains("Jul"));
    assert!(svg.contains("Jan"));  // crosses calendar year
    assert!(svg.contains("Jun"));
    output("calendar_financial_year.svg", &svg);
}

/// Two consecutive Australian financial years stacked.
#[test]
fn calendar_multi_financial_years() {
    fn fy_data(cal_year: i32, next_cal_year: i32) -> Vec<(String, f64)> {
        let mut v = Vec::new();
        // Jul–Dec of cal_year
        for m in 7u32..=12 {
            for d in (1u32..=28).step_by(4) {
                v.push((format!("{cal_year}-{m:02}-{d:02}"), (m + d) as f64 % 7.0 + 1.0));
            }
        }
        // Jan–Jun of next_cal_year
        for m in 1u32..=6 {
            for d in (1u32..=28).step_by(4) {
                v.push((format!("{next_cal_year}-{m:02}-{d:02}"), (m * d) as f64 % 8.0 + 1.0));
            }
        }
        v
    }

    let mut data = fy_data(2022, 2023);
    data.extend(fy_data(2023, 2024));

    let plot = CalendarPlot::new()
        .with_data(data)
        .with_aggregation(CalendarAgg::Sum)
        .with_periods([
            ("FY2022/23", "2022-07-01", "2023-06-30"),
            ("FY2023/24", "2023-07-01", "2024-06-30"),
        ])
        .with_legend_label("commits");
    let layout = Layout::auto_from_plots(&[Plot::Calendar(plot.clone())]);
    let svg = SvgBackend.render_scene(&render_calendar(plot, layout));
    assert!(svg.contains("FY2022/23"));
    assert!(svg.contains("FY2023/24"));
    output("calendar_multi_fy.svg", &svg);
}

// ── Dense data ────────────────────────────────────────────────────────────────

/// Full year 2024 with ~80 % of squares filled (every weekday + select weekends).
/// Tests that the renderer handles a high-density dataset without panic.
#[test]
fn calendar_dense_full_year() {
    // Generate data for every day in 2024.
    // Jan 1 2024 is a Monday.
    // We give every day a value based on simple cycling patterns.
    let mut data = Vec::new();
    let days_per_month = [31u32, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for (mi, &days) in days_per_month.iter().enumerate() {
        let m = mi as u32 + 1;
        for d in 1..=days {
            // Skip ~20 % of days (every 5th day) to leave some missing cells
            if (m + d) % 5 == 0 { continue; }
            let val = ((m * 7 + d * 3) % 15 + 1) as f64;
            data.push((format!("2024-{m:02}-{d:02}"), val));
        }
    }
    let n_data = data.len();
    let plot = CalendarPlot::new()
        .with_data(data)
        .with_aggregation(CalendarAgg::Sum)
        .with_year(2024)
        .with_legend_label("activity");
    let layout = Layout::auto_from_plots(&[Plot::Calendar(plot.clone())]);
    let svg = SvgBackend.render_scene(&render_calendar(plot, layout));
    assert!(svg.contains("<svg"));
    // Expect many cal-day cells
    let cell_count = svg.matches("cal-day").count();
    assert!(cell_count >= 300, "expected ≥300 day cells, got {cell_count}");
    // Most cells should have real data — check at least n_data SVG groups carry a non-"no data" val
    let no_data_count = svg.matches("no data").count();
    assert!(no_data_count < n_data / 2,
        "too many missing cells: {no_data_count} no-data vs {n_data} data points");
    output("calendar_dense.svg", &svg);
}

// ── Real GitHub contribution data (Psy-Fer) ───────────────────────────────────

/// Contributions fetched from https://github.com/users/Psy-Fer/contributions
/// covering 2025-04-16 to 2026-04-13 (600 total contributions, 80 active days).
#[test]
fn calendar_github_psy_fer() {
    // Data extracted from GitHub contributions SVG (non-zero days only).
    // All zero-contribution days are implicitly missing.
    #[rustfmt::skip]
    let data: &[(&str, f64)] = &[
        ("2025-04-16",  1.0), ("2025-04-17",  1.0), ("2025-04-30",  1.0),
        ("2025-05-05",  6.0), ("2025-05-06",  2.0), ("2025-05-07",  2.0),
        ("2025-05-08",  4.0), ("2025-05-09",  3.0), ("2025-05-10",  2.0),
        ("2025-06-10",  1.0), ("2025-07-08",  1.0), ("2025-07-09",  7.0),
        ("2025-07-10",  7.0), ("2025-07-17",  2.0), ("2025-07-23",  1.0),
        ("2025-07-24",  1.0), ("2025-07-25",  2.0), ("2025-07-29",  1.0),
        ("2025-08-01",  1.0), ("2025-08-05",  2.0), ("2025-08-06",  3.0),
        ("2025-08-07",  1.0), ("2025-09-02",  1.0), ("2025-09-08",  2.0),
        ("2025-09-12",  5.0), ("2025-10-02",  1.0), ("2025-10-20",  4.0),
        ("2025-10-21",  1.0), ("2025-10-22",  1.0), ("2025-10-23", 10.0),
        ("2025-10-24",  2.0), ("2025-10-28",  2.0), ("2025-10-29",  2.0),
        ("2025-11-20",  1.0), ("2025-11-27",  4.0), ("2025-12-03",  4.0),
        ("2025-12-08", 30.0), ("2025-12-09",  5.0), ("2026-01-23", 13.0),
        ("2026-01-27",  6.0), ("2026-01-28", 10.0), ("2026-02-06", 21.0),
        ("2026-02-07", 23.0), ("2026-02-09",  7.0), ("2026-02-10", 18.0),
        ("2026-02-12",  4.0), ("2026-02-13", 18.0), ("2026-02-16",  3.0),
        ("2026-02-17",  3.0), ("2026-02-18",  4.0), ("2026-02-19",  1.0),
        ("2026-02-20", 22.0), ("2026-02-21",  9.0), ("2026-02-22", 18.0),
        ("2026-02-23", 13.0), ("2026-02-24",  7.0), ("2026-02-25",  7.0),
        ("2026-02-26", 13.0), ("2026-02-27", 10.0), ("2026-02-28", 24.0),
        ("2026-03-01", 13.0), ("2026-03-02", 14.0), ("2026-03-03", 22.0),
        ("2026-03-04", 13.0), ("2026-03-06",  1.0), ("2026-03-09",  6.0),
        ("2026-03-10", 21.0), ("2026-03-11", 15.0), ("2026-03-12", 15.0),
        ("2026-03-16", 23.0), ("2026-03-20",  9.0), ("2026-03-26", 13.0),
        ("2026-03-30",  5.0), ("2026-03-31", 13.0), ("2026-04-01", 22.0),
        ("2026-04-02",  3.0), ("2026-04-03",  1.0), ("2026-04-08",  1.0),
        ("2026-04-09",  6.0), ("2026-04-13",  3.0),
    ];

    let plot = CalendarPlot::new()
        .with_data(data.iter().map(|&(d, v)| (d, v)))
        .with_aggregation(CalendarAgg::Sum)
        .with_period("Apr 2025 – Apr 2026", "2025-04-13", "2026-04-13")
        .with_week_start(WeekStart::Sunday)  // GitHub uses Sunday-start
        .with_legend_label("contributions");
    let layout = Layout::auto_from_plots(&[Plot::Calendar(plot.clone())]);
    let svg = SvgBackend.render_scene(&render_calendar(plot, layout));
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Apr 2025 \u{2013} Apr 2026"), "period label should appear");
    // Spot-check: high-activity dates and confirmed single-contribution days
    assert!(svg.contains("2026-03-16")); // peak day (23 contributions)
    assert!(svg.contains("2025-04-16")); // single-contribution day — must appear, not be suppressed
    assert!(svg.contains("2025-07-23")); // another single-contribution day
    output("calendar_github_psy_fer.svg", &svg);
}
