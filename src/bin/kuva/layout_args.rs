use clap::Args;
use kuva::render::layout::{AxisLabelOverlap, AxisLine, Layout, TickAlign, TickFormat, TickPos};
use kuva::render::palette::Palette;
use kuva::render::theme::Theme;

// ── Composable arg structs ────────────────────────────────────────────────────
// Flatten only the relevant combination into each subcommand:
//   Pie                    →  BaseArgs
//   Bar / Box / Violin     →  BaseArgs + AxisArgs
//   Scatter / Line / Hist  →  BaseArgs + AxisArgs + LogArgs

#[derive(Args, Debug)]
#[command(next_help_heading = "Output & appearance")]
pub struct BaseArgs {
    /// Output file. SVG/PNG/PDF inferred from extension. Defaults to SVG on stdout.
    #[arg(short = 'o', long)]
    pub output: Option<std::path::PathBuf>,

    /// Plot title displayed above the chart.
    #[arg(long)]
    pub title: Option<String>,

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
    /// tol-bright, tol-muted, tol-light, ibm
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

    /// Wrap all text (title, axis labels, legend) at N characters.
    /// Per-element flags (--title-wrap, etc.) override this when set.
    #[arg(long, value_name = "CHARS")]
    pub wrap: Option<usize>,

    /// Wrap the plot title at N characters.
    #[arg(long, value_name = "CHARS")]
    pub title_wrap: Option<usize>,

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
    // Global wrap first, then per-element overrides.
    if let Some(n) = args.wrap {
        layout = layout.with_wrap(n);
    }
    if let Some(n) = args.title_wrap {
        layout = layout.with_title_wrap(n);
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

// ── Helpers ───────────────────────────────────────────────────────────────────

fn theme_from_name(name: &str) -> Theme {
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
        _ => None,
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
