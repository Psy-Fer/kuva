use std::sync::Arc;
use crate::plot::colormap::ColorMap;
use crate::plot::legend::ColorBarInfo;

/// Aggregation function for multiple values on the same day.
#[derive(Debug, Clone, Default)]
pub enum CalendarAgg {
    /// Count the number of data points on each day (default).
    #[default]
    Count,
    /// Sum all values for each day.
    Sum,
    /// Average all values for each day.
    Mean,
    /// Maximum value for each day.
    Max,
}

/// Which day starts the week row in the calendar grid.
#[derive(Debug, Clone, Default)]
pub enum WeekStart {
    /// ISO week — Monday is the top row (default).
    #[default]
    Monday,
    /// US/GitHub convention — Sunday is the top row.
    Sunday,
}

/// A named display period for [`CalendarPlot`].
///
/// Each period is shown as one calendar row.  Periods can span any date
/// range — a single month, a quarter, a financial year, or multiple years.
#[derive(Debug, Clone)]
pub struct CalendarPeriod {
    /// Label shown to the left of the period's grid row (e.g. `"2024"`, `"FY2023/24"`).
    pub label: String,
    /// First day of the period in `"YYYY-MM-DD"` format.
    pub start: String,
    /// Last day of the period (inclusive) in `"YYYY-MM-DD"` format.
    pub end: String,
}

impl CalendarPeriod {
    pub fn new(label: impl Into<String>, start: impl Into<String>, end: impl Into<String>) -> Self {
        Self { label: label.into(), start: start.into(), end: end.into() }
    }
}

/// GitHub-style calendar heatmap.
///
/// Displays daily data values in a grid of week columns × 7 day rows,
/// optionally spanning multiple years or arbitrary date ranges stacked
/// vertically.  Each cell shows an interactive tooltip on hover.
///
/// # Basic usage — event counting
///
/// ```rust,no_run
/// use kuva::plot::calendar::CalendarPlot;
///
/// let plot = CalendarPlot::new()
///     .with_events(vec!["2024-03-15", "2024-03-15", "2024-03-16"])
///     .with_year(2024)
///     .with_legend_label("commits");
/// ```
///
/// # Custom date range (e.g. Australian financial year)
///
/// ```rust,no_run
/// use kuva::plot::calendar::CalendarPlot;
///
/// let plot = CalendarPlot::new()
///     .with_data(vec![("2023-07-15", 3.0), ("2024-02-01", 7.0)])
///     .with_period("FY2023/24", "2023-07-01", "2024-06-30");
/// ```
#[derive(Debug, Clone)]
pub struct CalendarPlot {
    /// `(date_string, value)` pairs.  Date format: `"YYYY-MM-DD"`.
    pub data: Vec<(String, f64)>,
    /// Aggregation to apply when multiple values land on the same day.
    pub aggregation: CalendarAgg,
    /// Color map for the value → color mapping.  Default: `Viridis`.
    pub color_map: ColorMap,
    /// Color for days with no data.  Default: `"#ebedf0"`.
    pub missing_color: String,
    /// Distinct color for days with zero value.  `None` → uses `missing_color`.
    pub zero_color: Option<String>,
    /// Which day starts the week row.  Default: `Monday`.
    pub week_start: WeekStart,
    /// Show abbreviated month labels above each period's grid.  Default: `true`.
    pub show_month_labels: bool,
    /// Show Mon/Wed/Fri labels to the left.  Default: `true`.
    pub show_day_labels: bool,
    /// Cell size in pixels.  Default: `13.0`.
    pub cell_size: f64,
    /// Gap between cells in pixels.  Default: `2.0`.
    pub cell_gap: f64,
    /// Calendar years to display as full Jan–Dec periods.
    /// Auto-detected from data if both `years` and `periods` are `None`.
    pub years: Option<Vec<i32>>,
    /// Explicit named display periods.  When set, overrides `years`.
    /// Each period defines a label, a start date, and an end date.
    pub periods: Option<Vec<CalendarPeriod>>,
    /// Show a color legend (colorbar) below the grids.  Default: `true`.
    pub show_legend: bool,
    /// Label for the color legend.
    pub legend_label: Option<String>,
    /// Explicit `[min, max]` range for color scaling.  Auto-computed if `None`.
    pub value_range: Option<(f64, f64)>,
}

impl Default for CalendarPlot {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            aggregation: CalendarAgg::default(),
            // Light green (#9be9a8) → dark green (#216e39) with sqrt gamma.
            // The gamma spreads low values apart perceptually: v=1 of 30 maps to
            // t_adj≈0.18 (clearly visible medium-light green) rather than t=0.033
            // (near-white, indistinguishable from the missing-color background).
            color_map: ColorMap::Custom(Arc::new(|t: f64| {
                let t = t.sqrt();  // sqrt gamma: low contributions visually distinct
                let r = (155.0 + t * (33.0  - 155.0)).round() as u8;
                let g = (233.0 + t * (110.0 - 233.0)).round() as u8;
                let b = (168.0 + t * (57.0  - 168.0)).round() as u8;
                format!("#{r:02x}{g:02x}{b:02x}")
            })),
            missing_color: "#ebedf0".to_string(),
            zero_color: None,
            week_start: WeekStart::default(),
            show_month_labels: true,
            show_day_labels: true,
            cell_size: 13.0,
            cell_gap: 2.0,
            years: None,
            periods: None,
            show_legend: true,
            legend_label: None,
            value_range: None,
        }
    }
}

impl CalendarPlot {
    pub fn new() -> Self { Self::default() }

    /// Add `(date, value)` pairs.  Date format: `"YYYY-MM-DD"`.
    pub fn with_data(mut self, data: impl IntoIterator<Item = (impl Into<String>, impl Into<f64>)>) -> Self {
        for (d, v) in data { self.data.push((d.into(), v.into())); }
        self
    }

    /// Add bare date strings for event counting.  Each occurrence adds 1.0.
    pub fn with_events(mut self, dates: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for d in dates { self.data.push((d.into(), 1.0)); }
        self
    }

    /// Set the aggregation function applied when multiple values share a date.
    pub fn with_aggregation(mut self, agg: CalendarAgg) -> Self { self.aggregation = agg; self }

    /// Set the color map.
    pub fn with_color_map(mut self, cmap: ColorMap) -> Self { self.color_map = cmap; self }

    /// Set the color for days with no data.
    pub fn with_missing_color(mut self, color: impl Into<String>) -> Self {
        self.missing_color = color.into(); self
    }

    /// Set a distinct color for days whose aggregated value is exactly zero.
    pub fn with_zero_color(mut self, color: impl Into<String>) -> Self {
        self.zero_color = Some(color.into()); self
    }

    /// Set which day starts each week row.
    pub fn with_week_start(mut self, ws: WeekStart) -> Self { self.week_start = ws; self }

    /// Show or hide month labels.
    pub fn with_month_labels(mut self, show: bool) -> Self { self.show_month_labels = show; self }

    /// Show or hide day-of-week labels (Mon/Wed/Fri).
    pub fn with_day_labels(mut self, show: bool) -> Self { self.show_day_labels = show; self }

    /// Set the cell size in pixels.
    pub fn with_cell_size(mut self, size: f64) -> Self { self.cell_size = size; self }

    /// Set the gap between cells in pixels.
    pub fn with_cell_gap(mut self, gap: f64) -> Self { self.cell_gap = gap; self }

    /// Set the years to display as full Jan–Dec rows.
    /// If not called, years are auto-detected from data.
    pub fn with_years(mut self, years: impl IntoIterator<Item = i32>) -> Self {
        self.years = Some(years.into_iter().collect()); self
    }

    /// Convenience: display a single calendar year.
    pub fn with_year(mut self, year: i32) -> Self { self.years = Some(vec![year]); self }

    /// Display a single custom date range as one calendar row.
    ///
    /// The label is taken from the start date's year if not otherwise specified.
    /// Use [`with_period`] to set a custom label.
    pub fn with_date_range(mut self, start: impl Into<String>, end: impl Into<String>) -> Self {
        let start = start.into();
        let label = start.get(..4).unwrap_or("").to_string();
        self.periods = Some(vec![CalendarPeriod { label, start, end: end.into() }]);
        self
    }

    /// Display a single named custom period as one calendar row.
    ///
    /// ```rust,no_run
    /// use kuva::plot::calendar::CalendarPlot;
    /// let plot = CalendarPlot::new()
    ///     .with_period("FY2023/24", "2023-07-01", "2024-06-30");
    /// ```
    pub fn with_period(
        mut self,
        label: impl Into<String>,
        start: impl Into<String>,
        end: impl Into<String>,
    ) -> Self {
        self.periods = Some(vec![CalendarPeriod {
            label: label.into(), start: start.into(), end: end.into(),
        }]);
        self
    }

    /// Display multiple named periods, each as a separate calendar row.
    ///
    /// ```rust,no_run
    /// use kuva::plot::calendar::CalendarPlot;
    /// let plot = CalendarPlot::new()
    ///     .with_periods([
    ///         ("FY2022/23", "2022-07-01", "2023-06-30"),
    ///         ("FY2023/24", "2023-07-01", "2024-06-30"),
    ///     ]);
    /// ```
    pub fn with_periods<L, S, E>(
        mut self,
        periods: impl IntoIterator<Item = (L, S, E)>,
    ) -> Self
    where
        L: Into<String>, S: Into<String>, E: Into<String>,
    {
        self.periods = Some(periods.into_iter().map(|(l, s, e)| CalendarPeriod {
            label: l.into(), start: s.into(), end: e.into(),
        }).collect());
        self
    }

    /// Show or hide the color legend.
    pub fn with_legend(mut self, show: bool) -> Self { self.show_legend = show; self }

    /// Set the color legend label.
    pub fn with_legend_label(mut self, label: impl Into<String>) -> Self {
        self.legend_label = Some(label.into()); self
    }

    /// Set an explicit `[min, max]` range for the color scale.
    pub fn with_value_range(mut self, min: f64, max: f64) -> Self {
        self.value_range = Some((min, max)); self
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    /// Aggregate data into a map of `"YYYY-MM-DD"` → aggregated value.
    pub(crate) fn aggregate(&self) -> std::collections::HashMap<String, f64> {
        let mut map: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();
        for (date, val) in &self.data {
            map.entry(date.clone()).or_default().push(*val);
        }
        map.into_iter().map(|(date, vals)| {
            let agg = match self.aggregation {
                CalendarAgg::Count => vals.len() as f64,
                CalendarAgg::Sum   => vals.iter().sum(),
                CalendarAgg::Mean  => vals.iter().sum::<f64>() / vals.len() as f64,
                CalendarAgg::Max   => vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            };
            (date, agg)
        }).collect()
    }

    /// Detect calendar years present in data (sorted).
    pub(crate) fn detect_years(&self) -> Vec<i32> {
        if let Some(ref yrs) = self.years { return yrs.clone(); }
        let mut ys: Vec<i32> = self.data.iter()
            .filter_map(|(d, _)| parse_date(d).map(|(y, _, _)| y))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        ys.sort();
        ys
    }

    /// Resolve display periods: `(label, start_triple, end_triple)`.
    ///
    /// Priority: explicit `periods` > explicit `years` > auto-detected years from data.
    pub(crate) fn detect_periods(&self) -> Vec<PeriodTriple> {
        if let Some(ref ps) = self.periods {
            return ps.iter().filter_map(|p| {
                let start = parse_date(&p.start)?;
                let end   = parse_date(&p.end)?;
                Some((p.label.clone(), start, end))
            }).collect();
        }
        self.detect_years().iter().map(|&y| {
            (y.to_string(), (y, 1u32, 1u32), (y, 12u32, 31u32))
        }).collect()
    }

    /// Natural canvas `(width, height)` for the given periods.
    pub(crate) fn natural_size_for_periods(&self, periods: &[PeriodTriple]) -> (f64, f64) {
        let sunday_start = matches!(self.week_start, WeekStart::Sunday);
        let pitch = self.cell_size + self.cell_gap;
        // Always reserve enough left margin for the period label even when day labels are off
        let max_label_len = periods.iter().map(|(l, _, _)| l.chars().count()).max().unwrap_or(4);
        let day_label_w: f64 = if self.show_day_labels {
            (max_label_len as f64 * 7.5).ceil().max(28.0)
        } else {
            (max_label_len as f64 * 7.5).ceil().max(32.0)
        };
        let month_label_h: f64 = if self.show_month_labels { 16.0 } else { 0.0 };
        let year_label_h: f64 = 16.0;
        let grid_h = 7.0 * pitch;
        let legend_h = if self.show_legend { 50.0 } else { 0.0 };
        let year_gap = 14.0;
        let pad = 20.0;
        let n = periods.len().max(1) as f64;

        let max_cols: u32 = periods.iter().map(|(_, start, end)| {
            let start_dow = if sunday_start {
                dow_sun0(start.0, start.1, start.2)
            } else {
                dow_mon0(start.0, start.1, start.2)
            };
            period_max_cols(*start, *end, start_dow)
        }).max().unwrap_or(53);

        let grid_w = max_cols as f64 * pitch;
        let w = pad * 2.0 + day_label_w + grid_w;
        let h = pad * 2.0 + n * (year_label_h + month_label_h + grid_h)
              + (n - 1.0) * year_gap + legend_h;
        (w.ceil(), h.ceil())
    }

    /// Build `ColorBarInfo` for the standard colorbar mechanism.
    #[allow(dead_code)]
    pub(crate) fn colorbar_info_inner(&self) -> Option<ColorBarInfo> {
        if !self.show_legend { return None; }
        let agg = self.aggregate();
        if agg.is_empty() { return None; }
        let (v_min, v_max) = if let Some(r) = self.value_range {
            r
        } else {
            let mut mx = f64::NEG_INFINITY;
            for &v in agg.values() { mx = mx.max(v); }
            (0.0, mx)
        };
        if !v_max.is_finite() { return None; }
        let cmap = self.color_map.clone();
        let range = v_max - v_min;
        let label = self.legend_label.clone();
        Some(ColorBarInfo {
            map_fn: Arc::new(move |t| {
                let norm = if range.abs() < f64::EPSILON { 0.5 } else { (t - v_min) / range };
                cmap.map(norm.clamp(0.0, 1.0))
            }),
            min_value: v_min,
            max_value: v_max,
            label,
            tick_labels: None,
        })
    }
}

/// Compact alias for the resolved period tuple used internally.
pub(crate) type PeriodTriple = (String, (i32, u32, u32), (i32, u32, u32));

// ── Date math ────────────────────────────────────────────────────────────────

/// Parse `"YYYY-MM-DD"` → `(year, month, day)`.
pub(crate) fn parse_date(s: &str) -> Option<(i32, u32, u32)> {
    let s = s.trim();
    if s.len() < 10 { return None; }
    let year: i32  = s[0..4].parse().ok()?;
    let month: u32 = s[5..7].parse().ok()?;
    let day: u32   = s[8..10].parse().ok()?;
    if !(1..=12).contains(&month) { return None; }
    if !(1..=days_in_month(year, month)).contains(&day) { return None; }
    Some((year, month, day))
}

pub(crate) fn is_leap_year(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

pub(crate) fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 0,
    }
}

#[allow(dead_code)]
pub(crate) fn days_in_year(y: i32) -> u32 { if is_leap_year(y) { 366 } else { 365 } }

/// Julian Day Number — the foundation for date arithmetic.
pub(crate) fn to_jd(y: i32, m: u32, d: u32) -> i64 {
    let a = (14i64 - m as i64) / 12;
    let yr = y as i64 + 4800 - a;
    let mo = m as i64 + 12 * a - 3;
    d as i64 + (153 * mo + 2) / 5 + 365 * yr + yr / 4 - yr / 100 + yr / 400 - 32045
}

/// Convert Julian Day Number back to `(year, month, day)`.
pub(crate) fn from_jd(jd: i64) -> (i32, u32, u32) {
    let l = jd + 68569;
    let n = 4 * l / 146097;
    let l = l - (146097 * n + 3) / 4;
    let i = 4000 * (l + 1) / 1461001;
    let l = l - 1461 * i / 4 + 31;
    let j = 80 * l / 2447;
    let d = (l - 2447 * j / 80) as u32;
    let l = j / 11;
    let m = (j + 2 - 12 * l) as u32;
    let y = (100 * (n - 49) + i + l) as i32;
    (y, m, d)
}

/// Number of days from `from` to `to` (both inclusive of `to - from`).
/// `to` must be >= `from`.
pub(crate) fn days_between(from: (i32, u32, u32), to: (i32, u32, u32)) -> u32 {
    (to_jd(to.0, to.1, to.2) - to_jd(from.0, from.1, from.2)) as u32
}

/// Tomohiko Sakamoto — returns 0=Sun, 1=Mon, …, 6=Sat.
fn dow_sun0(y: i32, m: u32, d: u32) -> u32 {
    let t: [i64; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let yy = if m < 3 { (y - 1) as i64 } else { y as i64 };
    ((yy + yy/4 - yy/100 + yy/400 + t[m as usize - 1] + d as i64).rem_euclid(7)) as u32
}

/// Returns 0=Mon, 1=Tue, …, 6=Sun (ISO / Monday-start).
pub(crate) fn dow_mon0(y: i32, m: u32, d: u32) -> u32 { (dow_sun0(y, m, d) + 6) % 7 }

/// Day-of-year, 0-based.
#[allow(dead_code)]
pub(crate) fn day_of_year(y: i32, m: u32, d: u32) -> u32 {
    let mut doy = d - 1;
    for mo in 1..m { doy += days_in_month(y, mo); }
    doy
}

/// `(col, row)` of `date` within a period that starts on `period_start`.
///
/// `start_dow` is the day-of-week index (in Mon-start or Sun-start coordinates)
/// of `period_start`, which determines which row the first day falls in.
pub(crate) fn period_grid_pos(
    date: (i32, u32, u32),
    period_start: (i32, u32, u32),
    start_dow: u32,
) -> (u32, u32) {
    let offset = days_between(period_start, date) + start_dow;
    (offset / 7, offset % 7)
}

/// Maximum column index (exclusive) needed to display a period.
pub(crate) fn period_max_cols(
    start: (i32, u32, u32),
    end: (i32, u32, u32),
    start_dow: u32,
) -> u32 {
    let n_days = days_between(start, end) + 1;
    (n_days + start_dow).div_ceil(7)
}

/// Convenience — `grid_pos` relative to Jan 1 of the same year.
#[allow(dead_code)]
pub(crate) fn grid_pos(y: i32, m: u32, d: u32, sunday_start: bool) -> (u32, u32) {
    let jan1_dow = if sunday_start { dow_sun0(y, 1, 1) } else { dow_mon0(y, 1, 1) };
    period_grid_pos((y, m, d), (y, 1, 1), jan1_dow)
}
