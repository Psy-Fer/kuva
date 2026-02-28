use kuva::plot::scatter::ScatterPlot;
use kuva::plot::line::LinePlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::{DateTimeAxis, ymd, ymd_hms};

fn ensure_output_dir() {
    std::fs::create_dir_all("test_outputs").unwrap();
}

#[test]
fn test_datetime_days() {
    ensure_output_dir();
    let t0 = ymd(2024, 1, 1);
    let t1 = ymd(2024, 1, 10);
    let data: Vec<(f64, f64)> = (0..10)
        .map(|i| (t0 + i as f64 * 86400.0, i as f64 * 1.5))
        .collect();

    let plot = ScatterPlot::new().with_data(data).with_color("steelblue");
    let layout = Layout::new((t0, t1), (0.0, 15.0))
        .with_title("Daily ticks")
        .with_x_datetime(DateTimeAxis::days("%Y-%m-%d"));

    let scene = render_multiple(vec![Plot::Scatter(plot)], layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/datetime_days.svg", svg.clone()).unwrap();

    assert!(svg.contains("2024-01"), "SVG should contain date labels like 2024-01-*");
}

#[test]
fn test_datetime_months() {
    ensure_output_dir();
    let t0 = ymd(2023, 1, 1);
    let t1 = ymd(2023, 12, 31);
    let data: Vec<(f64, f64)> = (0..12)
        .map(|i| (t0 + i as f64 * 30.0 * 86400.0, (i as f64).sin().abs() * 10.0))
        .collect();

    let plot = LinePlot::new().with_data(data).with_color("tomato");
    let layout = Layout::new((t0, t1), (0.0, 12.0))
        .with_title("Monthly ticks")
        .with_x_datetime(DateTimeAxis::months("%b %Y"));

    let scene = render_multiple(vec![Plot::Line(plot)], layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/datetime_months.svg", svg.clone()).unwrap();

    // Should contain an abbreviated month name
    assert!(
        svg.contains("Jan") || svg.contains("Feb") || svg.contains("Mar"),
        "SVG should contain abbreviated month names"
    );
}

#[test]
fn test_datetime_years() {
    ensure_output_dir();
    let t0 = ymd(2020, 1, 1);
    let t1 = ymd(2025, 1, 1);
    let data: Vec<(f64, f64)> = (0..5)
        .map(|i| (t0 + i as f64 * 365.25 * 86400.0, i as f64 * 3.0))
        .collect();

    let plot = ScatterPlot::new().with_data(data).with_color("mediumseagreen");
    let layout = Layout::new((t0, t1), (0.0, 20.0))
        .with_title("Yearly ticks")
        .with_x_datetime(DateTimeAxis::years("%Y"));

    let scene = render_multiple(vec![Plot::Scatter(plot)], layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/datetime_years.svg", svg.clone()).unwrap();

    assert!(svg.contains("2022") || svg.contains("2021"), "SVG should contain year labels");
    assert!(svg.contains("2023"), "SVG should contain 2023");
}

#[test]
fn test_datetime_auto() {
    ensure_output_dir();
    // 180-day range → should auto-select Month unit
    let t0 = ymd(2024, 3, 1);
    let t1 = ymd(2024, 9, 1);
    let data: Vec<(f64, f64)> = (0..6)
        .map(|i| (t0 + i as f64 * 30.0 * 86400.0, i as f64))
        .collect();

    let plot = ScatterPlot::new().with_data(data).with_color("orchid");
    let layout = Layout::new((t0, t1), (0.0, 7.0))
        .with_title("Auto datetime")
        .with_x_datetime(DateTimeAxis::auto(t0, t1));

    let scene = render_multiple(vec![Plot::Scatter(plot)], layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/datetime_auto.svg", svg.clone()).unwrap();

    // With a 180-day range auto picks Month / "%b %Y" → abbreviated months
    assert!(
        svg.contains("2024") || svg.contains("Mar") || svg.contains("Apr"),
        "SVG should contain month or year labels from auto datetime"
    );
}

#[test]
fn test_datetime_rotated() {
    ensure_output_dir();
    let t0 = ymd(2024, 1, 1);
    let t1 = ymd(2024, 1, 15);
    let data: Vec<(f64, f64)> = (0..15)
        .map(|i| (t0 + i as f64 * 86400.0, i as f64))
        .collect();

    let plot = ScatterPlot::new().with_data(data).with_color("slateblue");
    let layout = Layout::new((t0, t1), (0.0, 16.0))
        .with_title("Rotated ticks")
        .with_x_datetime(DateTimeAxis::days("%Y-%m-%d"))
        .with_x_tick_rotate(-45.0);

    let scene = render_multiple(vec![Plot::Scatter(plot)], layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/datetime_rotated.svg", svg.clone()).unwrap();

    assert!(svg.contains("rotate"), "SVG should contain rotate attribute on tick labels");
}

#[test]
fn test_datetime_helpers() {
    // ymd round-trip
    let ts = ymd(2024, 3, 15);
    let dt = chrono::DateTime::from_timestamp(ts as i64, 0).unwrap().naive_utc();
    assert_eq!(chrono::Datelike::year(&dt), 2024);
    assert_eq!(chrono::Datelike::month(&dt), 3);
    assert_eq!(chrono::Datelike::day(&dt), 15);
    assert_eq!(chrono::Timelike::hour(&dt), 0);

    // ymd_hms round-trip
    let ts2 = ymd_hms(2024, 6, 21, 12, 30, 0);
    let dt2 = chrono::DateTime::from_timestamp(ts2 as i64, 0).unwrap().naive_utc();
    assert_eq!(chrono::Datelike::year(&dt2), 2024);
    assert_eq!(chrono::Datelike::month(&dt2), 6);
    assert_eq!(chrono::Datelike::day(&dt2), 21);
    assert_eq!(chrono::Timelike::hour(&dt2), 12);
    assert_eq!(chrono::Timelike::minute(&dt2), 30);
}

#[test]
fn test_datetime_y_axis() {
    ensure_output_dir();
    // y_datetime: timestamps on y-axis, plain x
    let t0 = ymd(2024, 1, 1);
    let t1 = ymd(2024, 6, 1);
    let data: Vec<(f64, f64)> = (0..5)
        .map(|i| (i as f64, t0 + i as f64 * 30.0 * 86400.0))
        .collect();

    let plot = ScatterPlot::new().with_data(data).with_color("coral");
    let layout = Layout::new((0.0, 5.0), (t0, t1))
        .with_title("Y datetime axis")
        .with_y_datetime(DateTimeAxis::months("%b %Y"));

    let scene = render_multiple(vec![Plot::Scatter(plot)], layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/datetime_y_axis.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"), "Should render valid SVG without panic");
    assert!(
        svg.contains("2024") || svg.contains("Jan") || svg.contains("Feb"),
        "Y axis should show datetime labels"
    );
}
