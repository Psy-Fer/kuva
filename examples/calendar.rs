//! Calendar heatmap documentation examples.
use kuva::plot::calendar::{CalendarPlot, CalendarAgg, WeekStart};
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_calendar;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use std::fs;

const OUT: &str = "docs/src/assets/calendar";

fn write_calendar(name: &str, plot: CalendarPlot) {
    fs::create_dir_all(OUT).unwrap();
    let layout = Layout::auto_from_plots(&[Plot::Calendar(plot.clone())]);
    let svg = SvgBackend.render_scene(&render_calendar(plot, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

fn main() {
    // basic — GitHub-style contribution graph (Apr 2025 – Apr 2026)
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
        .with_period("Apr 2025 \u{2013} Apr 2026", "2025-04-13", "2026-04-13")
        .with_week_start(WeekStart::Sunday)
        .with_legend_label("contributions");
    write_calendar("basic", plot);

    // two_years — full year 2024 with varied numeric values
    let mut data2 = Vec::new();
    let days_per_month = [31u32, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for (mi, &days) in days_per_month.iter().enumerate() {
        let m = mi as u32 + 1;
        for d in 1..=days {
            if (m + d) % 5 == 0 { continue; }
            let val = ((m * 7 + d * 3) % 15 + 1) as f64;
            data2.push((format!("2024-{m:02}-{d:02}"), val));
        }
    }

    let plot = CalendarPlot::new()
        .with_data(data2)
        .with_aggregation(CalendarAgg::Sum)
        .with_year(2024)
        .with_legend_label("activity");
    write_calendar("two_years", plot);

    // financial_year — Australian FY Jul 2023 – Jun 2024
    let mut fy_data_vec = Vec::new();
    for m in 7u32..=9  { for d in [5, 12, 19, 26] { fy_data_vec.push((format!("2023-{m:02}-{d:02}"), (m * d) as f64 % 8.0 + 1.0)); } }
    for m in 10u32..=12 { for d in [3, 10, 17, 24] { fy_data_vec.push((format!("2023-{m:02}-{d:02}"), (m + d) as f64 % 6.0 + 2.0)); } }
    for m in 1u32..=3  { for d in [8, 15, 22, 29] { fy_data_vec.push((format!("2024-{m:02}-{d:02}"), (m * d) as f64 % 9.0 + 1.0)); } }
    for m in 4u32..=6  { for d in [1u32, 8, 15, 22, 29] { if d <= 30 { fy_data_vec.push((format!("2024-{m:02}-{d:02}"), (m + d) as f64 % 7.0 + 1.0)); } } }

    let plot = CalendarPlot::new()
        .with_data(fy_data_vec)
        .with_aggregation(CalendarAgg::Sum)
        .with_period("FY2023/24", "2023-07-01", "2024-06-30")
        .with_legend_label("contributions");
    write_calendar("financial_year", plot);

    // multi_fy — two consecutive Australian financial years stacked
    fn fy_data(cal_year: i32, next_cal_year: i32) -> Vec<(String, f64)> {
        let mut v = Vec::new();
        for m in 7u32..=12 {
            for d in (1u32..=28).step_by(4) {
                v.push((format!("{cal_year}-{m:02}-{d:02}"), (m + d) as f64 % 7.0 + 1.0));
            }
        }
        for m in 1u32..=6 {
            for d in (1u32..=28).step_by(4) {
                v.push((format!("{next_cal_year}-{m:02}-{d:02}"), (m * d) as f64 % 8.0 + 1.0));
            }
        }
        v
    }

    let mut mfy_data = fy_data(2022, 2023);
    mfy_data.extend(fy_data(2023, 2024));

    let plot = CalendarPlot::new()
        .with_data(mfy_data)
        .with_aggregation(CalendarAgg::Sum)
        .with_periods([
            ("FY2022/23", "2022-07-01", "2023-06-30"),
            ("FY2023/24", "2023-07-01", "2024-06-30"),
        ])
        .with_legend_label("commits");
    write_calendar("multi_fy", plot);

    println!("Calendar heatmap SVGs written to {OUT}/");
}
