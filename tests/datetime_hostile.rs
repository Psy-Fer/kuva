//! Datetime axes with times at the edge of chrono's range, huge tick steps or
//! format strings chrono can't render must not panic. Tick stepping used
//! chrono's `+` and `-`, which panic when the result is out of range, and tick
//! labels used `to_string()`, which panics when the format fails.

mod hostile;

use chrono::{DateTime, Utc};
use kuva::backend::svg::SvgBackend;
use kuva::plot::LinePlot;
use kuva::render::datetime::{DateTimeAxis, DateUnit};
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

fn chrono_min() -> f64 {
    DateTime::<Utc>::MIN_UTC.timestamp() as f64
}

fn chrono_max() -> f64 {
    DateTime::<Utc>::MAX_UTC.timestamp() as f64
}

fn axes() -> Vec<DateTimeAxis> {
    let second = DateTimeAxis {
        unit: DateUnit::Second,
        step: 1,
        format: "%H:%M:%S".into(),
    };
    vec![
        DateTimeAxis::years("%Y"),
        DateTimeAxis::months("%b %Y"),
        DateTimeAxis::weeks("%Y-%m-%d"),
        DateTimeAxis::days("%Y-%m-%d"),
        DateTimeAxis::hours("%H:00"),
        DateTimeAxis::minutes("%H:%M"),
        second,
    ]
}

/// Ranges that start inside chrono's range and end beyond it, the reverse,
/// and non-finite ones.
fn ranges() -> Vec<(f64, f64)> {
    let (lo, hi) = (chrono_min(), chrono_max());
    vec![
        (hi - 1e3, hi + 1e9),
        (hi - 4e8, hi),
        (hi - 4e8, f64::MAX),
        (hi, hi),
        (f64::MIN, lo + 4e8),
        (lo - 1e9, lo + 1e3),
        (lo, lo),
        (f64::NEG_INFINITY, f64::INFINITY),
        (f64::NAN, f64::NAN),
        (0.0, 4e8),
    ]
}

#[test]
fn ticks_near_the_chrono_limits_stop_cleanly() {
    hostile::bounded(8 << 20, || {
        for axis in axes() {
            for step in [1, 7, u32::MAX as usize, usize::MAX] {
                let axis = axis.clone().with_step(step);
                for (min, max) in ranges() {
                    let what = format!("{:?} step {step} range {min}..{max}", axis.unit);
                    let ticks = axis.generate_ticks(min, max);
                    assert!(ticks.len() <= 10_001, "{what}");
                    assert!(ticks.windows(2).all(|w| w[0] < w[1]), "{what}");
                    for &t in &ticks {
                        assert!((chrono_min()..=chrono_max()).contains(&t), "{what}");
                        let _ = axis.format_tick(t);
                    }
                }
            }
        }
    });
    // A normal range still ticks every year.
    let years = DateTimeAxis::years("%Y").generate_ticks(0.0, 4e8);
    assert_eq!(years.len(), 13);
}

#[test]
fn ticks_stop_before_the_last_representable_year() {
    // The last Jan 1 chrono can hold is a tick; the one after it is not.
    let hi = chrono_max();
    let ticks = DateTimeAxis::years("%Y").generate_ticks(hi - 4e8, hi + 1e9);
    assert!(!ticks.is_empty());
    assert!(*ticks.last().unwrap() > hi - 366.0 * 86_400.0);
}

#[test]
fn line_plots_near_the_chrono_limits_render_with_minor_ticks() {
    hostile::bounded(64 << 20, || {
        for axis in axes() {
            for (a, b) in [
                (chrono_max() - 4e8, chrono_max()),
                (chrono_min(), chrono_min() + 4e8),
            ] {
                let line = LinePlot::new().with_data([(a, 1.0), (b, 2.0)]);
                let plots = vec![Plot::Line(line)];
                let layout = Layout::auto_from_plots(&plots)
                    .with_x_datetime(axis.clone())
                    .with_minor_ticks(4);
                let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
                assert!(svg.contains("<svg"), "{:?}", axis.unit);
            }
        }
    });
}

#[test]
fn unrenderable_tick_formats_fall_back_instead_of_panicking() {
    let ts = 1_700_000_000.0; // 2023-11-14 22:13:20 UTC
    for fmt in ["%Q", "%Y %", "%Z", "%:z", "%Y %Ez"] {
        assert!(DateTimeAxis::check_format(fmt).is_err(), "{fmt}");
        assert_eq!(
            DateTimeAxis::days(fmt).format_tick(ts),
            "2023-11-14 22:13:20",
            "{fmt}"
        );
        // The axis still renders.
        let line = LinePlot::new().with_data([(ts, 1.0), (ts + 864_000.0, 2.0)]);
        let plots = vec![Plot::Line(line)];
        let layout = Layout::auto_from_plots(&plots).with_x_datetime(DateTimeAxis::days(fmt));
        let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
        assert!(svg.contains(">2023-11-15 00:00:00<"), "{fmt}");
    }
    for fmt in ["%Y-%m-%d", "%b %y", "%H:%M:%S", "plain text", ""] {
        assert!(DateTimeAxis::check_format(fmt).is_ok(), "{fmt}");
    }
    assert_eq!(DateTimeAxis::days("%Y-%m-%d").format_tick(ts), "2023-11-14");
}
