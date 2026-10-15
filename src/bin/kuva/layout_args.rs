use clap::Args;
use kuva::render::datetime::DateTimeAxis;
use kuva::render::layout::{AxisLabelOverlap, AxisLine, Layout, TickAlign, TickFormat, TickPos};
use kuva::render::palette::Palette;
use kuva::render::theme::Theme;

// ── Composable arg structs ────────────────────────────────────────────────────
// Flatten only the relevant combination into each subcommand:
//   Pie                    →  BaseArgs
//   Bar / Box / Violin     →  BaseArgs + AxisArgs
//   Scatter / Line / Hist  →  BaseArgs + AxisArgs + LogArgs
//   Scatter / Line         →  ... + DateArgs (date/time X axis)

#[derive(Args, Debug)]
#[command(next_help_heading = "Output & appearance")]
pub struct BaseArgs {
    /// Output file. Extension must be .svg, .png, or .pdf (case-insensitive).
    /// Defaults to SVG on stdout.
    #[arg(
        short = 'o',
        long,
        value_parser = crate::output::output_path_parser()
    )]
    pub output: Option<std::path::PathBuf>,

    /// Plot title displayed above the chart.
    #[arg(long)]
    pub title: Option<String>,

    /// Subtitle displayed under the title at a smaller, muted size (e.g. a data summary).
    #[arg(long)]
    pub subtitle: Option<String>,

    /// Canvas width in pixels. Default is auto-computed from plot content.
    #[arg(long)]
    pub width: Option<f64>,

    /// Canvas height in pixels. Default is auto-computed from plot content.
    #[arg(long)]
    pub height: Option<f64>,

    /// Visual theme: light (default), dark, solarized, minimal
    #[arg(long)]
    pub theme: Option<String>,

    /// Named color palette: category10, wong, okabe-ito, pastel, bold,
    /// tol-bright, tol-muted, tol-light, ibm, or an LTC palette (paloma, maya,
    /// dora, ploen, olga, mterese, franscoise, fernande, sylvie, expevo, minou,
    /// alger, seafarer, luminaries, casa-natal; optional `ltc-` prefix)
    #[arg(long)]
    pub palette: Option<String>,

    /// Select a colour palette optimised for a colour vision deficiency (CVD):
    /// deuteranopia, protanopia, tritanopia. Overrides --palette.
    #[arg(long)]
    pub cvd_palette: Option<String>,

    /// Override the SVG background color (CSS color string).
    /// When omitted the theme's background is used.
    #[arg(long)]
    pub background: Option<String>,

    /// Render to the terminal using Unicode braille/block characters.
    #[arg(long, conflicts_with = "output", help_heading = "Terminal")]
    pub terminal: bool,

    /// Terminal background style used to auto-select a readable colour theme:
    /// dark (default) or light. Ignored when --theme is also provided.
    #[arg(long, requires = "terminal", help_heading = "Terminal")]
    pub term_bg: Option<String>,

    /// Override terminal width in columns (default: $COLUMNS or 80).
    #[arg(long, requires = "terminal", help_heading = "Terminal")]
    pub term_width: Option<u16>,

    /// Override terminal height in rows (default: $LINES or 24).
    #[arg(long, requires = "terminal", help_heading = "Terminal")]
    pub term_height: Option<u16>,

    /// Uniform scale factor for all plot chrome: fonts, margins, tick marks, legend geometry.
    /// Canvas size is unchanged. Values > 1.0 make everything larger. Default: 1.0.
    #[arg(long)]
    pub scale: Option<f64>,

    /// Enable SVG interactivity: hover highlight, click-to-pin, search, and coordinate readout.
    #[arg(long)]
    pub interactive: bool,

    /// Black-and-white accessibility mode. Replaces palette colours with grey
    /// shades and hatch patterns, cycles dash styles on lines, and cycles
    /// marker shapes on scatter plots. Produces output legible when printed
    /// in greyscale, meeting most journal accessibility requirements.
    #[arg(long)]
    pub bw: bool,

    /// Draw a semi-opaque background rect behind in-fill value labels (Bar,
    /// Treemap, Sunburst, Waffle, Mosaic, Funnel, Gantt) for readability over
    /// busy fills. On automatically with `--bw`; this flag forces it on for
    /// color output too.
    #[arg(long)]
    pub label_background: bool,

    /// Wrap all text (title, subtitle, axis labels, legend) at N characters.
    /// Per-element flags (--title-wrap, --subtitle-wrap, etc.) override this when set.
    #[arg(long, value_name = "CHARS")]
    pub wrap: Option<usize>,

    /// Wrap the plot title at N characters.
    #[arg(long, value_name = "CHARS")]
    pub title_wrap: Option<usize>,

    /// Wrap the subtitle at N characters (independent of --title-wrap).
    #[arg(long, value_name = "CHARS")]
    pub subtitle_wrap: Option<usize>,

    /// Wrap the x-axis label at N characters.
    #[arg(long, value_name = "CHARS")]
    pub x_label_wrap: Option<usize>,

    /// Wrap the y-axis label at N characters.
    #[arg(long, value_name = "CHARS")]
    pub y_label_wrap: Option<usize>,

    /// Wrap the secondary y-axis label at N characters.
    #[arg(long, value_name = "CHARS")]
    pub y2_label_wrap: Option<usize>,

    /// Wrap legend labels and titles at N characters.
    #[arg(long, value_name = "CHARS")]
    pub legend_wrap: Option<usize>,

    /// Embed DejaVu Sans font directly in SVG output.
    /// Use this when rendering SVG in environments without system fonts
    /// (headless servers, containers, CI pipelines). Adds ~1 MB to the SVG.
    /// Has no effect on PNG/PDF output (those backends always have the font).
    #[arg(long, conflicts_with = "terminal")]
    pub embed_font: bool,

    /// Print the equivalent Rust library code for this plot instead of rendering it.
    /// The emitted snippet bakes in the resolved data as literals (a copy-pasteable
    /// starting point, not a live re-parse of your input file). Ignores --output
    /// and all rendering flags; prints to stdout.
    ///
    /// Requires the `emit_code` build feature (cargo build --features cli,emit_code).
    #[cfg(feature = "emit_code")]
    #[arg(long, conflicts_with = "terminal")]
    pub emit_code: bool,
}

#[derive(Args, Debug)]
#[command(next_help_heading = "Axes")]
pub struct AxisArgs {
    /// Label for the X axis.
    #[arg(long)]
    pub x_label: Option<String>,

    /// Label for the Y axis.
    #[arg(long)]
    pub y_label: Option<String>,

    /// Target number of axis tick marks (default: 5).
    ///
    /// This is a hint, not a guarantee. The renderer snaps the step size to a
    /// clean value (1, 2, 2.5, 5, or 10 × a power of 10), so the actual count
    /// is usually N ± 1 or 2. Changing N also widens or narrows the axis range,
    /// since the range is expanded to the nearest clean multiple of the step.
    /// Ignored on log-scale axes and category axes (bar, box, violin).
    #[arg(long)]
    pub ticks: Option<usize>,

    /// Disable the background grid.
    #[arg(long)]
    pub no_grid: bool,

    /// Axis line style: left or box.
    #[arg(long, value_name = "FRAME")]
    pub axis_line: Option<String>,

    /// Tick alignment relative to the axis line: outside, inside, or center.
    #[arg(long, value_name = "ALIGN")]
    pub tick_align: Option<String>,

    /// Tick position: primary (bottom/left) or both.
    #[arg(long, value_name = "POS")]
    pub tick_pos: Option<String>,

    /// Fix the X axis lower bound; overrides auto-range.
    #[arg(long, allow_hyphen_values = true)]
    pub x_min: Option<f64>,

    /// Fix the X axis upper bound; overrides auto-range.
    #[arg(long, allow_hyphen_values = true)]
    pub x_max: Option<f64>,

    /// Fix the Y axis lower bound; overrides auto-range.
    #[arg(long, allow_hyphen_values = true)]
    pub y_min: Option<f64>,

    /// Fix the Y axis upper bound; overrides auto-range.
    #[arg(long, allow_hyphen_values = true)]
    pub y_max: Option<f64>,

    /// Exact major tick step for the X axis. Overrides auto-calculation.
    #[arg(long, allow_hyphen_values = true)]
    pub x_tick_step: Option<f64>,

    /// Exact major tick step for the Y axis. Overrides auto-calculation.
    #[arg(long, allow_hyphen_values = true)]
    pub y_tick_step: Option<f64>,

    /// Subdivisions between major ticks, e.g. 5 draws 4 minor marks per interval.
    #[arg(long)]
    pub minor_ticks: Option<u32>,

    /// Draw faint gridlines at minor tick positions (requires --minor-ticks).
    #[arg(long)]
    pub minor_grid: bool,

    /// Tick label format for the X axis.
    /// auto (default), int, sci, percent, or fixed:N (e.g. fixed:2 → "3.14").
    #[arg(long, value_name = "FORMAT")]
    pub x_tick_format: Option<String>,

    /// Tick label format for the Y axis.
    /// auto (default), int, sci, percent, or fixed:N (e.g. fixed:2 → "3.14").
    #[arg(long, value_name = "FORMAT")]
    pub y_tick_format: Option<String>,

    /// How to handle overlapping x-axis tick labels: allow (default), thin, stagger.
    /// allow: draw every label even if they overlap.
    /// thin: skip labels that would overlap the previous one.
    /// stagger: place colliding labels in an alternating second row.
    /// On a Manhattan plot this controls chromosome label placement.
    #[arg(long, value_name = "STRATEGY")]
    pub x_label_overlap: Option<String>,
}

#[derive(Args, Debug)]
#[command(next_help_heading = "Log scale")]
pub struct LogArgs {
    /// Log-scale X axis.
    #[arg(long)]
    pub log_x: bool,

    /// Log-scale Y axis.
    #[arg(long)]
    pub log_y: bool,
}

#[derive(Args, Debug)]
#[command(next_help_heading = "Date/time axis")]
pub struct DateArgs {
    /// Parse the X column as a date/time using this strftime-style format
    /// (e.g. "%Y-%m-%d" or "%m/%d/%Y %H:%M") instead of a plain number.
    /// Formats with no time component parse as midnight UTC.
    #[arg(long, value_name = "FMT")]
    pub x_date_format: Option<String>,

    /// Tick spacing unit for the date axis: years, months, weeks, days, hours,
    /// or minutes. Default: auto (inspects the data range and picks one).
    /// Ignored unless --x-date-format is set.
    #[arg(long, value_name = "UNIT")]
    pub x_date_unit: Option<String>,

    /// Tick label format (strftime-style), overriding the unit's default
    /// format. Ignored in auto mode (i.e. when --x-date-unit is not set).
    #[arg(long, value_name = "FMT")]
    pub x_date_tick_format: Option<String>,

    /// Draw one date-axis tick every N units instead of every 1.
    /// Ignored unless --x-date-format is set.
    #[arg(long, value_name = "N")]
    pub x_date_tick_step: Option<usize>,
}

/// Missing-value handling for numeric columns (issue #108). Flattened into the subcommands that
/// read continuous numeric data (scatter, line, bar, histogram).
#[derive(Args, Debug)]
#[command(next_help_heading = "Missing values")]
pub struct NaArgs {
    /// Comma-separated tokens to treat as missing, replacing the default set
    /// (NA, NaN, null, N/A, .). Empty cells are always missing. Case-insensitive.
    /// Accepts negative sentinels like -999 (e.g. `--na-values -999,NA`).
    #[arg(
        long,
        value_name = "TOKENS",
        value_delimiter = ',',
        allow_hyphen_values = true
    )]
    pub na_values: Vec<String>,

    /// What to do with a missing value in a selected numeric column:
    /// drop (remove the row, default), zero (replace with 0), or error (fail).
    #[arg(long, value_name = "STRATEGY", default_value = "drop")]
    pub na_strategy: String,

    /// Clip numeric values to a lower bound. Values below it (and -inf) become this number.
    #[arg(long, value_name = "N", allow_hyphen_values = true)]
    pub clamp_min: Option<f64>,

    /// Clip numeric values to an upper bound. Values above it (and +inf) become this number.
    /// e.g. cap infinite -log10(p) at a ceiling: `--clamp-max 300`.
    #[arg(long, value_name = "N", allow_hyphen_values = true)]
    pub clamp_max: Option<f64>,
}

/// Missing-value config resolved from [`NaArgs`]: token set, strategy, and clamp bounds.
pub type NaConfig = (
    crate::data::NaSet,
    crate::data::NaStrategy,
    (Option<f64>, Option<f64>),
);

impl NaArgs {
    /// Resolve to a token set, a strategy, and clamp bounds `(min, max)`.
    pub fn resolve(&self) -> Result<NaConfig, String> {
        let na_set = if self.na_values.is_empty() {
            crate::data::NaSet::default_tokens()
        } else {
            crate::data::NaSet::from_tokens(self.na_values.iter().cloned())
        };
        let strategy = match self.na_strategy.to_ascii_lowercase().as_str() {
            "drop" => crate::data::NaStrategy::Drop,
            "zero" => crate::data::NaStrategy::Zero,
            "error" => crate::data::NaStrategy::Error,
            other => {
                return Err(format!(
                    "unknown --na-strategy '{other}': expected drop, zero, or error"
                ))
            }
        };
        if let (Some(lo), Some(hi)) = (self.clamp_min, self.clamp_max) {
            if lo > hi {
                return Err(format!(
                    "--clamp-min ({lo}) must not exceed --clamp-max ({hi})"
                ));
            }
        }
        Ok((na_set, strategy, (self.clamp_min, self.clamp_max)))
    }
}

/// Controls for a secondary (right-hand) Y axis — currently only meaningful
/// on `kuva twin-y`, which is the only subcommand with a real secondary Y axis
/// exposed to CLI configuration (`ParetoPlot`'s secondary axis is fixed 0-100%
/// and needs no flags).
#[derive(Args, Debug)]
#[command(next_help_heading = "Secondary Y axis")]
pub struct Y2AxisArgs {
    /// Label for the secondary (right) Y axis.
    #[arg(long)]
    pub y2_label: Option<String>,

    /// Fix the secondary Y axis lower bound; overrides auto-range.
    #[arg(long, allow_hyphen_values = true)]
    pub y2_min: Option<f64>,

    /// Fix the secondary Y axis upper bound; overrides auto-range.
    #[arg(long, allow_hyphen_values = true)]
    pub y2_max: Option<f64>,

    /// Log-scale the secondary Y axis.
    #[arg(long)]
    pub log_y2: bool,

    /// Tick label format for the secondary Y axis.
    /// auto (default), int, sci, percent, or fixed:N (e.g. fixed:2 → "3.14").
    #[arg(long, value_name = "FORMAT")]
    pub y2_tick_format: Option<String>,
}

// ── Apply functions ───────────────────────────────────────────────────────────

/// Apply base output/appearance args to a layout.
pub fn apply_base_args(mut layout: Layout, args: &BaseArgs) -> Layout {
    if let Some(w) = args.width {
        layout = layout.with_width(w);
    }
    if let Some(h) = args.height {
        layout = layout.with_height(h);
    }
    if let Some(ref t) = args.title {
        layout = layout.with_title(t.clone());
    }
    if let Some(ref s) = args.subtitle {
        layout = layout.with_subtitle(s.clone());
    }
    // When rendering to the terminal, auto-select a theme matched to the
    // terminal background unless the user has already chosen one via --theme.
    if args.terminal && args.theme.is_none() {
        let theme = if args.term_bg.as_deref() == Some("light") {
            Theme::light()
        } else {
            Theme::dark() // dark background is the sensible default for terminals
        };
        layout = layout.with_theme(theme);
    }
    // Explicit --theme overrides the auto-selected terminal theme.
    if let Some(ref name) = args.theme {
        layout = layout.with_theme(theme_from_name(name));
    }
    // Suppress grid AFTER theme application (with_theme resets show_grid from
    // the theme's value, so this must come last).
    if args.terminal {
        layout = layout.with_show_grid(false);
        let rows = args
            .term_height
            .map(|h| h as u32)
            .or_else(|| std::env::var("LINES").ok().and_then(|s| s.parse().ok()))
            .unwrap_or(24u32);
        layout = layout.with_term_rows(rows);
    }
    if let Some(ref name) = args.palette {
        if let Some(pal) = palette_from_name(name) {
            layout = layout.with_palette(pal);
        }
    }
    // --cvd-palette overrides --palette when both are provided.
    if let Some(ref condition) = args.cvd_palette {
        if let Some(pal) = colourblind_palette(condition) {
            layout = layout.with_palette(pal);
        }
    }
    if let Some(f) = args.scale {
        layout = layout.with_scale(f);
    }
    if args.interactive {
        layout = layout.with_interactive();
    }
    if args.bw {
        layout = layout.with_bw_mode();
    }
    if args.label_background {
        layout = layout.with_label_background(true);
    }
    // Global wrap first, then per-element overrides.
    if let Some(n) = args.wrap {
        layout = layout.with_wrap(n);
    }
    if let Some(n) = args.title_wrap {
        layout = layout.with_title_wrap(n);
    }
    if let Some(n) = args.subtitle_wrap {
        layout = layout.with_subtitle_wrap(n);
    }
    if let Some(n) = args.x_label_wrap {
        layout = layout.with_x_label_wrap(n);
    }
    if let Some(n) = args.y_label_wrap {
        layout = layout.with_y_label_wrap(n);
    }
    if let Some(n) = args.y2_label_wrap {
        layout = layout.with_y2_label_wrap(n);
    }
    if let Some(n) = args.legend_wrap {
        layout = layout.with_legend_wrap(n);
    }
    layout
}

/// Apply axis label / tick / grid args to a layout.
pub fn apply_axis_args(mut layout: Layout, args: &AxisArgs) -> Layout {
    if let Some(ref l) = args.x_label {
        layout = layout.with_x_label(l.clone());
    }
    if let Some(ref l) = args.y_label {
        layout = layout.with_y_label(l.clone());
    }
    if let Some(t) = args.ticks {
        layout = layout.with_ticks(t);
    }
    if args.no_grid {
        layout = layout.with_show_grid(false);
    }
    if let Some(ref line) = args.axis_line {
        if let Some(line) = parse_axis_line(line) {
            layout = layout.with_axis_line(line);
        }
    }
    if let Some(ref align) = args.tick_align {
        if let Some(align) = parse_tick_align(align) {
            layout = layout.with_tick_align(align);
        }
    }
    if let Some(ref pos) = args.tick_pos {
        if let Some(pos) = parse_tick_pos(pos) {
            layout = layout.with_tick_pos(pos);
        }
    }
    if let Some(v) = args.x_min {
        layout = layout.with_x_axis_min(v);
    }
    if let Some(v) = args.x_max {
        layout = layout.with_x_axis_max(v);
    }
    if let Some(v) = args.y_min {
        layout = layout.with_y_axis_min(v);
    }
    if let Some(v) = args.y_max {
        layout = layout.with_y_axis_max(v);
    }
    if let Some(s) = args.x_tick_step {
        layout = layout.with_x_tick_step(s);
    }
    if let Some(s) = args.y_tick_step {
        layout = layout.with_y_tick_step(s);
    }
    if let Some(n) = args.minor_ticks {
        layout = layout.with_minor_ticks(n);
    }
    if args.minor_grid {
        layout = layout.with_show_minor_grid(true);
    }
    if let Some(ref fmt) = args.x_tick_format {
        if let Some(tf) = parse_tick_format(fmt) {
            layout = layout.with_x_tick_format(tf);
        }
    }
    if let Some(ref fmt) = args.y_tick_format {
        if let Some(tf) = parse_tick_format(fmt) {
            layout = layout.with_y_tick_format(tf);
        }
    }
    if let Some(ref s) = args.x_label_overlap {
        if let Some(strategy) = parse_label_overlap(s) {
            layout = layout.with_x_label_overlap(strategy);
        }
    }
    layout
}

/// Apply secondary-Y-axis args to a layout.
pub fn apply_y2_axis_args(mut layout: Layout, args: &Y2AxisArgs) -> Layout {
    if let Some(ref l) = args.y2_label {
        layout = layout.with_y2_label(l.clone());
    }
    if let Some(v) = args.y2_min {
        layout = layout.with_y2_axis_min(v);
    }
    if let Some(v) = args.y2_max {
        layout = layout.with_y2_axis_max(v);
    }
    if args.log_y2 {
        layout = layout.with_log_y2();
    }
    if let Some(ref fmt) = args.y2_tick_format {
        if let Some(tf) = parse_tick_format(fmt) {
            layout = layout.with_y2_tick_format(tf);
        }
    }
    layout
}

/// Apply log-scale args to a layout.
pub fn apply_log_args(mut layout: Layout, args: &LogArgs) -> Layout {
    if args.log_x {
        layout = layout.with_log_x();
    }
    if args.log_y {
        layout = layout.with_log_y();
    }
    layout
}

/// Build a `DateTimeAxis` from `--x-date-*` flags and the already-parsed x
/// values (needed for `DateTimeAxis::auto`'s data-range inspection). Callers
/// only invoke this when `args.x_date_format` is set — that flag is what
/// actually switches the x column from a numeric to a date/time parse; this
/// function only decides how the resulting timestamps are ticked.
pub fn date_axis_from_args(args: &DateArgs, xs: &[f64]) -> DateTimeAxis {
    let axis = match args.x_date_unit.as_deref() {
        Some(unit) => {
            let fmt = args
                .x_date_tick_format
                .clone()
                .unwrap_or_else(|| default_date_tick_format(unit).to_string());
            match unit.to_ascii_lowercase().as_str() {
                "years" | "year" => DateTimeAxis::years(&fmt),
                "months" | "month" => DateTimeAxis::months(&fmt),
                "weeks" | "week" => DateTimeAxis::weeks(&fmt),
                "days" | "day" => DateTimeAxis::days(&fmt),
                "hours" | "hour" => DateTimeAxis::hours(&fmt),
                "minutes" | "minute" => DateTimeAxis::minutes(&fmt),
                _ => auto_date_axis(xs),
            }
        }
        None => auto_date_axis(xs),
    };
    match args.x_date_tick_step {
        Some(step) => axis.with_step(step),
        None => axis,
    }
}

fn auto_date_axis(xs: &[f64]) -> DateTimeAxis {
    let min = xs.iter().copied().fold(f64::MAX, f64::min);
    let max = xs.iter().copied().fold(f64::MIN, f64::max);
    DateTimeAxis::auto(min, max)
}

/// Default tick-label format per unit — matches the "Typical format" column
/// in the Date & Time Axes reference docs.
fn default_date_tick_format(unit: &str) -> &'static str {
    match unit.to_ascii_lowercase().as_str() {
        "years" | "year" => "%Y",
        "months" | "month" => "%b %Y",
        "weeks" | "week" => "%b %d",
        "days" | "day" => "%Y-%m-%d",
        "hours" | "hour" => "%H:%M",
        "minutes" | "minute" => "%H:%M",
        _ => "%Y-%m-%d",
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub(crate) fn theme_from_name(name: &str) -> Theme {
    match name {
        "dark" => Theme::dark(),
        "solarized" | "solar" => Theme::solarized(),
        "minimal" => Theme::minimal(),
        _ => Theme::light(),
    }
}

pub fn palette_from_name(name: &str) -> Option<Palette> {
    match name {
        "category10" => Some(Palette::category10()),
        "wong" => Some(Palette::wong()),
        "okabe-ito" | "okabe_ito" => Some(Palette::okabe_ito()),
        "pastel" => Some(Palette::pastel()),
        "bold" => Some(Palette::bold()),
        "tol-bright" | "tol_bright" => Some(Palette::tol_bright()),
        "tol-muted" | "tol_muted" => Some(Palette::tol_muted()),
        "tol-light" | "tol_light" => Some(Palette::tol_light()),
        "ibm" => Some(Palette::ibm()),
        // LTC palettes (loukesio/ltc-color-palettes, MIT). Accept an optional `ltc-` prefix.
        other => match other.strip_prefix("ltc-").unwrap_or(other) {
            "paloma" => Some(Palette::paloma()),
            "maya" => Some(Palette::maya()),
            "dora" => Some(Palette::dora()),
            "ploen" => Some(Palette::ploen()),
            "olga" => Some(Palette::olga()),
            "mterese" => Some(Palette::mterese()),
            "franscoise" => Some(Palette::franscoise()),
            "fernande" => Some(Palette::fernande()),
            "sylvie" => Some(Palette::sylvie()),
            "expevo" => Some(Palette::expevo()),
            "minou" => Some(Palette::minou()),
            "alger" => Some(Palette::alger()),
            "seafarer" => Some(Palette::seafarer()),
            "luminaries" => Some(Palette::luminaries()),
            "casa-natal" | "casa_natal" => Some(Palette::casa_natal()),
            _ => None,
        },
    }
}

fn colourblind_palette(condition: &str) -> Option<Palette> {
    match condition {
        "deuteranopia" | "deuter" => Some(Palette::deuteranopia()),
        "protanopia" | "protan" => Some(Palette::protanopia()),
        "tritanopia" | "tritan" => Some(Palette::tritanopia()),
        _ => None,
    }
}

fn parse_axis_line(s: &str) -> Option<AxisLine> {
    match s.to_ascii_lowercase().replace('_', "-").as_str() {
        "open" | "left" | "primary" => Some(AxisLine::Open),
        "box" | "frame" | "enclosed" => Some(AxisLine::Box),
        _ => None,
    }
}

fn parse_tick_align(s: &str) -> Option<TickAlign> {
    match s.to_ascii_lowercase().replace('_', "-").as_str() {
        "inside" | "in" => Some(TickAlign::Inside),
        "outside" | "out" => Some(TickAlign::Outside),
        "center" | "centre" | "middle" => Some(TickAlign::Center),
        _ => None,
    }
}

fn parse_tick_pos(s: &str) -> Option<TickPos> {
    match s.to_ascii_lowercase().replace('_', "-").as_str() {
        "primary" | "left" | "bottom" | "lower" => Some(TickPos::Primary),
        "both" | "mirror" | "mirrored" => Some(TickPos::Both),
        _ => None,
    }
}

fn parse_label_overlap(s: &str) -> Option<AxisLabelOverlap> {
    match s {
        "allow" => Some(AxisLabelOverlap::Allow),
        "thin" => Some(AxisLabelOverlap::Thin),
        "stagger" => Some(AxisLabelOverlap::Stagger),
        _ => None,
    }
}

/// Parse a tick format string from the CLI.
/// Accepted values: auto, int, sci, percent, fixed:N
fn parse_tick_format(s: &str) -> Option<TickFormat> {
    match s {
        "auto" => Some(TickFormat::Auto),
        "int" => Some(TickFormat::Integer),
        "sci" => Some(TickFormat::Sci),
        "percent" => Some(TickFormat::Percent),
        _ if s.starts_with("fixed:") => s["fixed:".len()..]
            .parse::<usize>()
            .ok()
            .map(TickFormat::Fixed),
        _ => None,
    }
}
