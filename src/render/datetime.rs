use chrono::{NaiveDate, NaiveDateTime, DateTime, Utc, Datelike, Timelike, TimeDelta, Months};

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
        Self { unit: DateUnit::Year, step: 1, format: fmt.to_string() }
    }

    pub fn months(fmt: &str) -> Self {
        Self { unit: DateUnit::Month, step: 1, format: fmt.to_string() }
    }

    pub fn weeks(fmt: &str) -> Self {
        Self { unit: DateUnit::Week, step: 1, format: fmt.to_string() }
    }

    pub fn days(fmt: &str) -> Self {
        Self { unit: DateUnit::Day, step: 1, format: fmt.to_string() }
    }

    pub fn hours(fmt: &str) -> Self {
        Self { unit: DateUnit::Hour, step: 1, format: fmt.to_string() }
    }

    pub fn minutes(fmt: &str) -> Self {
        Self { unit: DateUnit::Minute, step: 1, format: fmt.to_string() }
    }

    pub fn with_step(mut self, step: usize) -> Self {
        self.step = step;
        self
    }

    /// Auto-select unit and format from range in seconds.
    pub fn auto(min: f64, max: f64) -> Self {
        let range = max - min;
        if range < 120.0 {
            Self { unit: DateUnit::Second, step: 1, format: "%H:%M:%S".to_string() }
        } else if range < 7200.0 {
            Self { unit: DateUnit::Minute, step: 1, format: "%H:%M".to_string() }
        } else if range < 259200.0 {
            Self { unit: DateUnit::Hour, step: 1, format: "%m-%d %H:00".to_string() }
        } else if range < 7776000.0 {
            Self { unit: DateUnit::Day, step: 1, format: "%Y-%m-%d".to_string() }
        } else if range < 94608000.0 {
            Self { unit: DateUnit::Month, step: 1, format: "%b %Y".to_string() }
        } else {
            Self { unit: DateUnit::Year, step: 1, format: "%Y".to_string() }
        }
    }

    /// Generate tick positions (as Unix timestamps) aligned to calendar boundaries.
    pub fn generate_ticks(&self, min: f64, max: f64) -> Vec<f64> {
        let step = self.step.max(1);
        let mut ticks = Vec::new();

        let min_dt = DateTime::from_timestamp(min as i64, 0)
            .map(|dt: DateTime<Utc>| dt.naive_utc())
            .unwrap_or_default();

        let mut current = snap_to_boundary(&min_dt, &self.unit, step);

        let max_ts = max as i64;
        let mut iters = 0;
        loop {
            let ts = current.and_utc().timestamp();
            if ts > max_ts { break; }
            if ts >= min as i64 {
                ticks.push(ts as f64);
            }
            current = advance(&current, &self.unit, step);
            iters += 1;
            if iters > 10000 { break; }
        }

        ticks
    }

    /// Format a Unix timestamp using the configured strftime format string.
    pub fn format_tick(&self, ts: f64) -> String {
        let dt = DateTime::from_timestamp(ts as i64, 0)
            .map(|dt: DateTime<Utc>| dt.naive_utc())
            .unwrap_or_default();
        dt.format(&self.format).to_string()
    }
}

/// Snap a datetime to the first calendar boundary >= dt for the given unit.
fn snap_to_boundary(dt: &NaiveDateTime, unit: &DateUnit, step: usize) -> NaiveDateTime {
    match unit {
        DateUnit::Second => {
            // Round up to nearest step
            let s = dt.second() as usize;
            let snapped = (s / step) * step;
            let base = dt.with_second(snapped as u32).unwrap_or(*dt)
                .with_nanosecond(0).unwrap_or(*dt);
            if base < *dt { advance(&base, unit, step) } else { base }
        }
        DateUnit::Minute => {
            let m = dt.minute() as usize;
            let snapped = (m / step) * step;
            let base = dt.with_minute(snapped as u32).unwrap_or(*dt)
                .with_second(0).unwrap_or(*dt)
                .with_nanosecond(0).unwrap_or(*dt);
            if base < *dt { advance(&base, unit, step) } else { base }
        }
        DateUnit::Hour => {
            let h = dt.hour() as usize;
            let snapped = (h / step) * step;
            let base = dt.with_hour(snapped as u32).unwrap_or(*dt)
                .with_minute(0).unwrap_or(*dt)
                .with_second(0).unwrap_or(*dt)
                .with_nanosecond(0).unwrap_or(*dt);
            if base < *dt { advance(&base, unit, step) } else { base }
        }
        DateUnit::Day => {
            // Snap to midnight
            let base = dt.date().and_hms_opt(0, 0, 0).unwrap_or(*dt);
            if base < *dt { advance(&base, unit, step) } else { base }
        }
        DateUnit::Week => {
            // Snap to the nearest preceding Monday midnight
            let days_since_monday = dt.weekday().num_days_from_monday() as i64;
            let monday = dt.date() - TimeDelta::days(days_since_monday);
            let base = monday.and_hms_opt(0, 0, 0).unwrap_or(*dt);
            if base < *dt { advance(&base, unit, step) } else { base }
        }
        DateUnit::Month => {
            // Snap to first of the month
            let base = NaiveDate::from_ymd_opt(dt.year(), dt.month(), 1)
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .unwrap_or(*dt);
            if base < *dt { advance(&base, unit, step) } else { base }
        }
        DateUnit::Year => {
            // Snap to Jan 1
            let base = NaiveDate::from_ymd_opt(dt.year(), 1, 1)
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .unwrap_or(*dt);
            if base < *dt { advance(&base, unit, step) } else { base }
        }
    }
}

/// Advance a datetime by step units.
fn advance(dt: &NaiveDateTime, unit: &DateUnit, step: usize) -> NaiveDateTime {
    let step = step as i64;
    match unit {
        DateUnit::Second => *dt + TimeDelta::seconds(step),
        DateUnit::Minute => *dt + TimeDelta::minutes(step),
        DateUnit::Hour   => *dt + TimeDelta::hours(step),
        DateUnit::Day    => *dt + TimeDelta::days(step),
        DateUnit::Week   => *dt + TimeDelta::weeks(step),
        DateUnit::Month  => *dt + Months::new(step as u32),
        DateUnit::Year   => *dt + Months::new(step as u32 * 12),
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
