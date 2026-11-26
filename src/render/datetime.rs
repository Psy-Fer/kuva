use chrono::{
    DateTime, Datelike, Days, Months, NaiveDate, NaiveDateTime, TimeDelta, Timelike, Utc,
};

#[derive(Clone, Debug)]
pub enum DateUnit {
    Year,
    Month,
    Week,
    Day,
    Hour,
    Minute,
    Second,
}

#[derive(Clone, Debug)]
pub struct DateTimeAxis {
    pub unit: DateUnit,
    pub step: usize,
    pub format: String,
}

impl DateTimeAxis {
    pub fn years(fmt: &str) -> Self {
        Self {
            unit: DateUnit::Year,
            step: 1,
            format: fmt.to_string(),
        }
    }

    pub fn months(fmt: &str) -> Self {
        Self {
            unit: DateUnit::Month,
            step: 1,
            format: fmt.to_string(),
        }
    }

    pub fn weeks(fmt: &str) -> Self {
        Self {
            unit: DateUnit::Week,
            step: 1,
            format: fmt.to_string(),
        }
    }

    pub fn days(fmt: &str) -> Self {
        Self {
            unit: DateUnit::Day,
            step: 1,
            format: fmt.to_string(),
        }
    }

    pub fn hours(fmt: &str) -> Self {
        Self {
            unit: DateUnit::Hour,
            step: 1,
            format: fmt.to_string(),
        }
    }

    pub fn minutes(fmt: &str) -> Self {
        Self {
            unit: DateUnit::Minute,
            step: 1,
            format: fmt.to_string(),
        }
    }

    pub fn with_step(mut self, step: usize) -> Self {
        self.step = step;
        self
    }

    /// Auto-select unit and format from range in seconds.
    pub fn auto(min: f64, max: f64) -> Self {
        let range = max - min;
        if range < 120.0 {
            Self {
                unit: DateUnit::Second,
                step: 1,
                format: "%H:%M:%S".to_string(),
            }
        } else if range < 7200.0 {
            Self {
                unit: DateUnit::Minute,
                step: 1,
                format: "%H:%M".to_string(),
            }
        } else if range < 259200.0 {
            Self {
                unit: DateUnit::Hour,
                step: 1,
                format: "%m-%d %H:00".to_string(),
            }
        } else if range < 7776000.0 {
            Self {
                unit: DateUnit::Day,
                step: 1,
                format: "%Y-%m-%d".to_string(),
            }
        } else if range < 94608000.0 {
            Self {
                unit: DateUnit::Month,
                step: 1,
                format: "%b %Y".to_string(),
            }
        } else {
            Self {
                unit: DateUnit::Year,
                step: 1,
                format: "%Y".to_string(),
            }
        }
    }

    /// Generate tick positions (as Unix timestamps) aligned to calendar boundaries.
    ///
    /// Stops at `max`, after 10 000 ticks, or when the next tick would fall
    /// outside the dates chrono can represent (about 262 000 years either
    /// side of year 0).
    pub fn generate_ticks(&self, min: f64, max: f64) -> Vec<f64> {
        let step = self.step.max(1);
        let mut ticks = Vec::new();

        let min_dt = to_naive(min);
        let Some(mut current) = snap_to_boundary(&min_dt, &self.unit, step) else {
            return ticks;
        };

        let max_ts = max as i64;
        let mut iters = 0;
        loop {
            let ts = current.and_utc().timestamp();
            if ts > max_ts {
                break;
            }
            if ts >= min as i64 {
                ticks.push(ts as f64);
            }
            match advance(&current, &self.unit, step) {
                Some(next) => current = next,
                None => break,
            }
            iters += 1;
            if iters > 10000 {
                break;
            }
        }

        ticks
    }

    /// Format a Unix timestamp using the configured strftime format string.
    ///
    /// If chrono can't render the format (an unknown `%` specifier, or a time
    /// zone specifier such as `%Z`, which has no value here), the label uses
    /// `%Y-%m-%d %H:%M:%S` instead. [`Self::check_format`] reports this up
    /// front.
    pub fn format_tick(&self, ts: f64) -> String {
        use std::fmt::Write;
        let dt = to_naive(ts);
        let mut out = String::new();
        if write!(out, "{}", dt.format(&self.format)).is_err() {
            out.clear();
            let _ = write!(out, "{}", dt.format(FALLBACK_FORMAT));
        }
        out
    }

    /// Checks that `fmt` is a strftime format chrono can render for a date
    /// and time without a time zone. Returns an error message naming the
    /// format if not.
    pub fn check_format(fmt: &str) -> Result<(), String> {
        use std::fmt::Write;
        let mut out = String::new();
        write!(out, "{}", NaiveDateTime::default().format(fmt))
            .map_err(|_| format!("'{fmt}' is not a strftime format kuva can render"))
    }

    /// The tick one calendar step after `ts` — calendar-correct (e.g. a
    /// month step lands on the 1st of the next month, not 30/31 days later).
    /// Used to extend a real tick list with the tick that would exist just
    /// past the last major, when the axis range doesn't end on one.
    /// Returns `ts` itself when the next step is outside chrono's range.
    pub(crate) fn tick_after(&self, ts: f64) -> f64 {
        advance(&to_naive(ts), &self.unit, self.step.max(1))
            .map_or(ts, |dt| dt.and_utc().timestamp() as f64)
    }

    /// The tick one calendar step before `ts` — see [`Self::tick_after`].
    pub(crate) fn tick_before(&self, ts: f64) -> f64 {
        retreat(&to_naive(ts), &self.unit, self.step.max(1))
            .map_or(ts, |dt| dt.and_utc().timestamp() as f64)
    }
}

/// Tick label format used when the configured one can't be rendered.
const FALLBACK_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// The UTC date and time of a Unix timestamp. Timestamps outside chrono's
/// range are moved to its first or last representable second, and NaN to the
/// epoch.
fn to_naive(ts: f64) -> NaiveDateTime {
    let lo = DateTime::<Utc>::MIN_UTC.timestamp();
    let hi = DateTime::<Utc>::MAX_UTC.timestamp();
    // `as` saturates, and turns NaN into 0.
    DateTime::from_timestamp((ts as i64).clamp(lo, hi), 0)
        .map(|dt: DateTime<Utc>| dt.naive_utc())
        .unwrap_or_default()
}

/// Snap a datetime to the first calendar boundary >= dt for the given unit.
/// `None` when that boundary is outside chrono's range.
fn snap_to_boundary(dt: &NaiveDateTime, unit: &DateUnit, step: usize) -> Option<NaiveDateTime> {
    let base = match unit {
        DateUnit::Second => {
            // Round down to a multiple of the step
            let s = dt.second() as usize;
            let snapped = (s / step) * step;
            dt.with_second(snapped as u32)
                .unwrap_or(*dt)
                .with_nanosecond(0)
                .unwrap_or(*dt)
        }
        DateUnit::Minute => {
            let m = dt.minute() as usize;
            let snapped = (m / step) * step;
            dt.with_minute(snapped as u32)
                .unwrap_or(*dt)
                .with_second(0)
                .unwrap_or(*dt)
                .with_nanosecond(0)
                .unwrap_or(*dt)
        }
        DateUnit::Hour => {
            let h = dt.hour() as usize;
            let snapped = (h / step) * step;
            dt.with_hour(snapped as u32)
                .unwrap_or(*dt)
                .with_minute(0)
                .unwrap_or(*dt)
                .with_second(0)
                .unwrap_or(*dt)
                .with_nanosecond(0)
                .unwrap_or(*dt)
        }
        // Snap to midnight
        DateUnit::Day => dt.date().and_hms_opt(0, 0, 0).unwrap_or(*dt),
        DateUnit::Week => {
            // Snap to the nearest preceding Monday midnight
            let days_since_monday = dt.weekday().num_days_from_monday() as u64;
            let monday = dt.date().checked_sub_days(Days::new(days_since_monday))?;
            monday.and_hms_opt(0, 0, 0).unwrap_or(*dt)
        }
        // Snap to first of the month
        DateUnit::Month => NaiveDate::from_ymd_opt(dt.year(), dt.month(), 1)
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .unwrap_or(*dt),
        // Snap to Jan 1
        DateUnit::Year => NaiveDate::from_ymd_opt(dt.year(), 1, 1)
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .unwrap_or(*dt),
    };
    if base < *dt {
        advance(&base, unit, step)
    } else {
        Some(base)
    }
}

/// The length of `step` units as a fixed duration, or as a number of months
/// for calendar units. `None` when the step is too large to represent.
enum Step {
    Fixed(TimeDelta),
    Months(Months),
}

fn step_of(unit: &DateUnit, step: usize) -> Option<Step> {
    let n = i64::try_from(step).ok()?;
    let months = |per: u32| -> Option<Step> {
        let m = u32::try_from(step).ok()?.checked_mul(per)?;
        Some(Step::Months(Months::new(m)))
    };
    match unit {
        DateUnit::Second => TimeDelta::try_seconds(n).map(Step::Fixed),
        DateUnit::Minute => TimeDelta::try_minutes(n).map(Step::Fixed),
        DateUnit::Hour => TimeDelta::try_hours(n).map(Step::Fixed),
        DateUnit::Day => TimeDelta::try_days(n).map(Step::Fixed),
        DateUnit::Week => TimeDelta::try_weeks(n).map(Step::Fixed),
        DateUnit::Month => months(1),
        DateUnit::Year => months(12),
    }
}

/// Advance a datetime by step units. `None` when the step is too large or the
/// result is outside chrono's range.
fn advance(dt: &NaiveDateTime, unit: &DateUnit, step: usize) -> Option<NaiveDateTime> {
    match step_of(unit, step)? {
        Step::Fixed(d) => dt.checked_add_signed(d),
        Step::Months(m) => dt.checked_add_months(m),
    }
}

/// Retreat a datetime by step units — the inverse of [`advance`].
fn retreat(dt: &NaiveDateTime, unit: &DateUnit, step: usize) -> Option<NaiveDateTime> {
    match step_of(unit, step)? {
        Step::Fixed(d) => dt.checked_sub_signed(d),
        Step::Months(m) => dt.checked_sub_months(m),
    }
}

/// Unix timestamp (seconds) from year/month/day UTC.
pub fn ymd(year: i32, month: u32, day: u32) -> f64 {
    NaiveDate::from_ymd_opt(year, month, day)
        .expect("invalid date passed to ymd()")
        .and_hms_opt(0, 0, 0)
        .expect("midnight is always a valid time")
        .and_utc()
        .timestamp() as f64
}

/// Unix timestamp from year/month/day/hour/minute/second UTC.
pub fn ymd_hms(year: i32, month: u32, day: u32, h: u32, m: u32, s: u32) -> f64 {
    NaiveDate::from_ymd_opt(year, month, day)
        .expect("invalid date passed to ymd_hms()")
        .and_hms_opt(h, m, s)
        .expect("invalid time passed to ymd_hms()")
        .and_utc()
        .timestamp() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    // Regression (PR #101 follow-up): the phantom-tick extrapolation used to
    // guess the next/previous major from a fixed timestamp delta between the
    // outermost real tick pair, which is wrong for calendar units whose
    // length varies (a month is 28-31 days). `tick_after`/`tick_before` must
    // instead step by the actual calendar unit.

    #[test]
    fn tick_after_month_crosses_into_march_not_31_days_later() {
        let dt = DateTimeAxis::months("%Y-%m-%d");
        let feb_1 = ymd(2025, 2, 1);
        assert_eq!(
            dt.tick_after(feb_1),
            ymd(2025, 3, 1),
            "one month after Feb 1 is Mar 1, not Feb 1 + 31 days"
        );
    }

    #[test]
    fn tick_before_month_crosses_out_of_a_short_month() {
        let dt = DateTimeAxis::months("%Y-%m-%d");
        let mar_1 = ymd(2025, 3, 1);
        assert_eq!(
            dt.tick_before(mar_1),
            ymd(2025, 2, 1),
            "one month before Mar 1 is Feb 1, not Mar 1 - 31 days"
        );
    }

    #[test]
    fn tick_after_year_handles_leap_years() {
        let dt = DateTimeAxis::years("%Y");
        assert_eq!(dt.tick_after(ymd(2024, 1, 1)), ymd(2025, 1, 1));
        assert_eq!(
            dt.tick_after(ymd(2023, 1, 1)),
            ymd(2024, 1, 1),
            "one year after Jan 1 2023 is Jan 1 2024, crossing into a leap year"
        );
    }

    #[test]
    fn tick_after_before_day_step_matches_fixed_delta() {
        // Day/week/hour/etc. units DO have a fixed-length step, so this
        // should behave identically to simple arithmetic for them.
        let dt = DateTimeAxis::days("%Y-%m-%d");
        let d = ymd(2025, 6, 15);
        assert_eq!(dt.tick_after(d), d + 86_400.0);
        assert_eq!(dt.tick_before(d), d - 86_400.0);
    }
}
