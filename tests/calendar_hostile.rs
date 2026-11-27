//! Calendar periods whose end is before their start, or whose dates do not
//! parse. A reversed period made the day count wrap to about 4.29 billion,
//! which overflowed in debug builds and gave a canvas hundreds of millions of
//! pixels wide in release builds.

mod hostile;

use kuva::plot::calendar::CalendarPlot;
use kuva::render::render::{render_calendar, Scene};
use kuva::render::{layout::Layout, plots::Plot};

fn render(plot: CalendarPlot) -> Scene {
    hostile::bounded(64 << 20, move || {
        let layout = Layout::auto_from_plots(&[Plot::Calendar(plot.clone())]);
        render_calendar(plot, layout)
    })
}

fn skipped(scene: &Scene) -> Vec<&str> {
    scene
        .warnings
        .iter()
        .filter(|w| w.code == "calendar.period_skipped")
        .map(|w| w.message.as_str())
        .collect()
}

#[test]
fn a_reversed_date_range_is_skipped_with_a_warning() {
    let plot = CalendarPlot::new()
        .with_events(["2024-03-01"])
        .with_date_range("2024-06-01", "2024-01-01");
    let err = plot.check_periods().unwrap_err();
    assert!(
        err.contains("end date 2024-01-01 is before start date 2024-06-01"),
        "{err}"
    );

    let scene = render(plot);
    // An ordinary calendar size, not one sized from a wrapped day count.
    assert!(scene.width < 2_000.0, "width {}", scene.width);
    assert!(scene.height < 2_000.0, "height {}", scene.height);
    let msgs = skipped(&scene);
    assert_eq!(msgs.len(), 1, "{msgs:?}");
    assert!(msgs[0].contains("'2024'"), "{}", msgs[0]);
    assert!(msgs[0].contains("not drawn"), "{}", msgs[0]);
}

#[test]
fn bad_periods_are_left_out_and_good_ones_drawn() {
    let good = ("Q1", "2024-01-01", "2024-03-31");
    let mixed = CalendarPlot::new()
        .with_events(["2024-02-01"])
        .with_periods([
            ("backwards", "2025-01-01", "2024-01-01"),
            good,
            ("typo", "2024-13-01", "2024-12-31"),
        ]);
    let only_good = CalendarPlot::new()
        .with_events(["2024-02-01"])
        .with_periods([good]);

    let err = mixed.check_periods().unwrap_err();
    assert!(err.contains("'backwards'"), "{err}");
    assert!(only_good.check_periods().is_ok());

    let scene = render(mixed);
    let reference = render(only_good);
    assert_eq!(
        (scene.width, scene.height),
        (reference.width, reference.height)
    );
    assert_eq!(scene.elements.len(), reference.elements.len());
    let msgs = skipped(&scene);
    assert_eq!(msgs.len(), 2, "{msgs:?}");
    assert!(msgs[0].contains("'backwards'") && msgs[0].contains("before start date"));
    assert!(msgs[1].contains("'typo'") && msgs[1].contains("start date '2024-13-01'"));
    assert!(reference.warnings.is_empty());
}

#[test]
fn a_one_day_period_is_valid() {
    let plot = CalendarPlot::new().with_events(["2024-05-05"]).with_period(
        "day",
        "2024-05-05",
        "2024-05-05",
    );
    assert!(plot.check_periods().is_ok());
    let scene = render(plot);
    assert!(scene.warnings.is_empty());
    assert!(scene.width < 2_000.0);
}

#[cfg(feature = "cli")]
#[test]
fn cli_rejects_a_reversed_range() {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new(env!("CARGO_BIN_EXE_kuva"))
        .args(["calendar", "--start", "2024-06-01", "--end", "2024-01-01"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn kuva");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"date\tvalue\n2024-03-01\t1\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "stderr: {stderr}");
    assert!(stderr.contains("before start date"), "stderr: {stderr}");
    assert!(out.stdout.is_empty());
}
