use clap::Args;
use kuva::plot::annotations::{BracketAnchor, BracketLabel, SignificanceBracket};
use kuva::plot::axis_break::AxisBreak;
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
    /// Output file. Extension must be .svg, .svgz (gzipped SVG), .html (a self-contained page
    /// that shows the compressed figure in any browser), .png, or .pdf (case-insensitive).
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
    /// alger, seafarer, luminaries, casa-natal; optional `ltc-` prefix).
    /// Run `kuva palettes` to preview them in the terminal.
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

    /// Font family for all text, as a CSS font-family list (e.g. "Inter, sans-serif").
    /// The first family is the one used: a font loaded with --font-file/--font-dir is
    /// measured, drawn and embedded exactly; any other family is named in SVG output
    /// and spaced as DejaVu Sans.
    #[arg(long, value_name = "FAMILY", help_heading = "Fonts")]
    pub font: Option<String>,

    /// Font family for axis tick labels only (default: --font).
    #[arg(long, value_name = "FAMILY", help_heading = "Fonts")]
    pub tick_font: Option<String>,

    /// Font family for `code` spans in text (default: DejaVu Sans Mono).
    #[arg(long, value_name = "FAMILY", help_heading = "Fonts")]
    pub mono_font: Option<String>,

    /// Load a font file (TTF, OTF, TTC) to use by family name. Repeat for each
    /// file, e.g. the regular and bold files of a family. Without --font, the
    /// family of the first file becomes the font for all text.
    #[cfg(feature = "custom_fonts")]
    #[arg(long, value_name = "PATH", help_heading = "Fonts")]
    pub font_file: Vec<std::path::PathBuf>,

    /// Load every font file in a directory (and its subdirectories). Repeatable.
    #[cfg(feature = "custom_fonts")]
    #[arg(long, value_name = "DIR", help_heading = "Fonts")]
    pub font_dir: Vec<std::path::PathBuf>,

    /// Embed the fonts directly in SVG output: DejaVu Sans, plus any font loaded
    /// with --font-file/--font-dir that the plot uses. Use this when rendering SVG
    /// in environments without those fonts (headless servers, containers, CI
    /// pipelines). Adds ~1 MB to the SVG, plus the loaded fonts' size.
    /// Has no effect on PNG/PDF output (those backends always have the fonts).
    #[arg(long, conflicts_with = "terminal", help_heading = "Fonts")]
    pub embed_font: bool,

    /// Write every rectangle as its own `<rect>` in SVG output. By default, runs of plain,
    /// non-overlapping rectangles (bricks, heatmap cells, bars) are written as one `<path>` per
    /// colour, which renders identically and roughly halves the file. Use this if something
    /// downstream needs to select or restyle individual cells.
    #[arg(long, conflicts_with = "terminal")]
    pub no_rect_batching: bool,

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

    /// Put the X axis major ticks exactly at these values, e.g. 20,50,100,200
    /// (values outside the axis range are skipped). Overrides --x-tick-step.
    #[arg(long, value_delimiter = ',', allow_hyphen_values = true)]
    pub x_tick_values: Vec<f64>,

    /// Put the Y axis major ticks exactly at these values. Overrides --y-tick-step.
    #[arg(long, value_delimiter = ',', allow_hyphen_values = true)]
    pub y_tick_values: Vec<f64>,

    /// Subdivisions between major ticks, e.g. 5 draws 4 minor marks per interval.
    #[arg(long)]
    pub minor_ticks: Option<u32>,

    /// Minor tick subdivisions on the X axis only (overrides --minor-ticks there).
    #[arg(long)]
    pub x_minor_ticks: Option<u32>,

    /// Minor tick subdivisions on the Y axis only (overrides --minor-ticks there).
    #[arg(long)]
    pub y_minor_ticks: Option<u32>,

    /// Draw faint gridlines at minor tick positions, on each axis with minor ticks.
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

    /// Break the X axis: skip the values between LO and HI and draw a short
    /// gap with slanted marks on the axis line (`--x-break 10,990`). An
    /// optional third number is the fraction of the axis given to the
    /// segment before the break (`--x-break 10,990,0.5`; default follows the
    /// data, clamped to 0.3..0.7). Linear numeric axes only; ignored on log
    /// or category axes.
    #[arg(long, value_name = "LO,HI[,SHARE]", value_parser = parse_break_pair, allow_hyphen_values = true)]
    pub x_break: Option<(f64, f64, Option<f64>)>,

    /// Break the Y axis between LO and HI (`--y-break 12,940`); see --x-break.
    #[arg(long, value_name = "LO,HI[,SHARE]", value_parser = parse_break_pair, allow_hyphen_values = true)]
    pub y_break: Option<(f64, f64, Option<f64>)>,
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

/// Parse `LO,HI[,SHARE]` for `--x-break` / `--y-break`: two finite numbers
/// with LO < HI, and an optional fraction of the axis for the segment before
/// the break.
pub fn parse_break_pair(s: &str) -> Result<(f64, f64, Option<f64>), String> {
    let parts: Vec<&str> = s.split(',').map(str::trim).collect();
    let (a, b, share) = match parts[..] {
        [a, b] => (a, b, None),
        [a, b, c] => (a, b, Some(c)),
        _ => {
            return Err(format!(
                "'{s}': expected LO,HI or LO,HI,SHARE (comma-separated numbers)"
            ))
        }
    };
    let num = |label: &str, v: &str| -> Result<f64, String> {
        let x: f64 = v
            .parse()
            .map_err(|_| format!("'{s}': {label} '{v}' is not a number"))?;
        if !x.is_finite() {
            return Err(format!("'{s}': {label} must be finite"));
        }
        Ok(x)
    };
    let (lo, hi) = (num("LO", a)?, num("HI", b)?);
    if lo >= hi {
        return Err(format!("'{s}': LO must be less than HI"));
    }
    let share = share.map(|c| num("SHARE", c)).transpose()?;
    if let Some(f) = share {
        if !(0.05..=0.95).contains(&f) {
            return Err(format!("'{s}': SHARE must be between 0.05 and 0.95"));
        }
    }
    Ok((lo, hi, share))
}

/// Significance-bracket flags shared by the categorical subcommands
/// (`bar`, `box`, `violin`, `strip`, `raincloud`).
#[derive(Args, Debug, Clone, Default)]
pub struct SignifArgs {
    /// Significance bracket between two groups: `A,B,P`. A and B are group
    /// labels as they appear in the data (or `#i` for the zero-based group
    /// index), P is the p-value. A non-numeric P is drawn verbatim
    /// (`WT,KO,n.s.`). Labels that contain commas work as long as they
    /// match the data's group names (`"Day 1, fed","Day 1, fasted",0.01`). Repeat the flag for several brackets; they stack
    /// above the data and the value axis grows to fit. Brackets follow
    /// `--horizontal` automatically.
    #[arg(long = "signif", value_name = "A,B,P", action = clap::ArgAction::Append)]
    pub signif: Vec<String>,

    /// Label style for --signif brackets: `stars` (default; `*` to `****`,
    /// `ns`), `p` (`p = 0.023`), or `both` (`** (p = 0.0031)`).
    #[arg(long = "signif-format", value_name = "STYLE", default_value = "stars")]
    pub signif_format: String,
}

/// Parse `--signif-format`.
pub fn parse_signif_format(style: &str) -> Result<BracketLabel, String> {
    match style {
        "stars" => Ok(BracketLabel::Stars),
        "p" => Ok(BracketLabel::PValue),
        "both" => Ok(BracketLabel::Both),
        other => Err(format!(
            "unknown --signif-format '{other}': expected stars, p, or both"
        )),
    }
}

/// Parse one `--signif A,B,P` spec into a bracket. A numeric P in `0..=1`
/// takes `format`; anything else (including `nan`) is drawn verbatim.
/// Orientation is inferred from the layout at render time.
///
/// `groups` are the plot's group labels. The spec is split on commas, and
/// when there are more than three fields the split is chosen so that A and B
/// both name a group (or are `#i` indices), which lets labels contain
/// commas. Without a match it falls back to `A,B,P` with P taking the rest.
pub fn parse_signif_spec(
    spec: &str,
    format: &BracketLabel,
    groups: &[String],
) -> Result<SignificanceBracket, String> {
    let fields: Vec<&str> = spec.split(',').collect();
    if fields.len() < 3 {
        return Err(format!(
            "--signif '{spec}': expected three comma-separated fields A,B,P"
        ));
    }
    let known = |s: &str| {
        let s = s.trim();
        s.strip_prefix('#').map_or_else(
            || groups.iter().any(|g| g == s),
            |i| i.parse::<usize>().is_ok_and(|i| i < groups.len()),
        )
    };
    // First split (shortest A, then shortest B) whose A and B both resolve;
    // otherwise the plain three-way split.
    let n = fields.len();
    let (i, j) = (1..n - 1)
        .flat_map(|i| (i + 1..n).map(move |j| (i, j)))
        .find(|&(i, j)| known(&fields[..i].join(",")) && known(&fields[i..j].join(",")))
        .unwrap_or((1, 2));
    let a = fields[..i].join(",");
    let b = fields[i..j].join(",");
    let p = fields[j..].join(",");
    let (a, b, p) = (a.trim(), b.trim(), p.trim());
    if a.is_empty() || b.is_empty() || p.is_empty() {
        return Err(format!("--signif '{spec}': empty field"));
    }
    let anchor = |s: &str| -> Result<BracketAnchor, String> {
        if let Some(idx) = s.strip_prefix('#') {
            idx.parse::<usize>()
                .map(BracketAnchor::Index)
                .map_err(|_| format!("--signif '{spec}': '{s}' is not a group index"))
        } else {
            Ok(BracketAnchor::Label(s.to_string()))
        }
    };
    // An unknown group is not an error (the bracket is just not drawn), but
    // it is almost always a typo, so say so.
    for s in [a, b] {
        if !known(s) && !groups.is_empty() {
            let what = if s.starts_with('#') {
                format!("index {s} is out of range ({} groups)", groups.len())
            } else {
                format!("no group named '{s}'")
            };
            eprintln_safe!(
                "warning: --signif '{spec}': {what}; the bracket is skipped (groups: {})",
                groups.join(", ")
            );
        }
    }
    let (a, b) = (anchor(a)?, anchor(b)?);
    match p.parse::<f64>() {
        Ok(pv) if pv.is_finite() && (0.0..=1.0).contains(&pv) => {
            let mut bracket = SignificanceBracket::new(a, b, pv);
            bracket.format = format.clone();
            Ok(bracket)
        }
        Ok(pv) if pv.is_finite() => {
            Err(format!("--signif '{spec}': p-value {pv} is outside 0..=1"))
        }
        _ => Ok(SignificanceBracket::text(a, b, p)),
    }
}

/// Apply `--signif` / `--signif-format` to the layout. Call it after the
/// layout knows its categories, so labels can be checked against them.
pub fn apply_signif_args(mut layout: Layout, args: &SignifArgs) -> Result<Layout, String> {
    let format = parse_signif_format(&args.signif_format)?;
    let groups = layout
        .x_categories
        .clone()
        .or_else(|| layout.y_categories.clone())
        .unwrap_or_default();
    for spec in &args.signif {
        layout = layout.with_bracket(parse_signif_spec(spec, &format, &groups)?);
    }
    Ok(layout)
}

/// Apply base output/appearance args to a layout.
/// The font families a run asked for, after loading any font files.
#[derive(Debug, Default)]
pub struct FontChoice {
    pub body: Option<String>,
    pub tick: Option<String>,
    pub mono: Option<String>,
}

/// Load the fonts named by --font-file/--font-dir, pick the families, and warn
/// about anything that will not render as asked: a family missing bold or italic
/// faces (kuva never fakes them), a variable font (only its default style is
/// used), a font whose licence forbids embedding when it will be embedded, and a
/// --font family that is not loaded. Exits with an error when a font cannot be
/// loaded. Call before rendering.
pub fn apply_font_args(args: &BaseArgs) -> FontChoice {
    #[allow(unused_mut)]
    let mut body = args.font.clone();
    #[cfg(feature = "custom_fonts")]
    {
        let loaded = load_fonts(args);
        if body.is_none() {
            body = loaded.first().map(|info| info.family.clone());
        }
        warn_font_issues(args, &loaded, body.as_deref());
    }
    FontChoice {
        body,
        tick: args.tick_font.clone(),
        mono: args.mono_font.clone(),
    }
}

/// Register the --font-file/--font-dir fonts, in order. Exits on an unreadable
/// file or a directory with no usable font; a bad file inside a directory is a
/// warning.
#[cfg(feature = "custom_fonts")]
fn load_fonts(args: &BaseArgs) -> Vec<kuva::fonts::FontInfo> {
    let mut loaded = Vec::new();
    for path in &args.font_file {
        match kuva::fonts::register_font_file(path) {
            Ok(infos) => loaded.extend(infos),
            Err(e) => {
                eprintln_safe!("error: {e}");
                std::process::exit(1);
            }
        }
    }
    for dir in &args.font_dir {
        match kuva::fonts::register_fonts_dir(dir) {
            Ok(report) => {
                for (file, e) in &report.skipped {
                    eprintln_safe!("warning: skipped {}: {e}", file.display());
                }
                loaded.extend(report.registered);
            }
            Err(e) => {
                eprintln_safe!("error: {e}");
                std::process::exit(1);
            }
        }
    }
    loaded
}

/// The first family of a CSS font-family list, unquoted.
#[cfg(feature = "custom_fonts")]
fn first_family(list: &str) -> &str {
    list.split(',')
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .trim()
}

/// "a", "a or b", "a, b or c".
#[cfg(feature = "custom_fonts")]
fn join_or(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [one] => one.to_string(),
        [rest @ .., last] => format!("{} or {last}", rest.join(", ")),
    }
}

#[cfg(feature = "custom_fonts")]
fn warn_font_issues(args: &BaseArgs, loaded: &[kuva::fonts::FontInfo], body: Option<&str>) {
    for info in loaded {
        if info.variable {
            eprintln_safe!(
                "warning: {} is a variable font; only its default style (weight {}) is used. \
                 Load the static font files for other weights.",
                info.family,
                info.weight
            );
        }
    }
    let embedding = args.embed_font
        || args
            .output
            .as_deref()
            .and_then(|p| p.extension())
            .is_some_and(|e| e.eq_ignore_ascii_case("pdf"));
    if embedding {
        let mut warned: Vec<&str> = Vec::new();
        for info in loaded.iter().filter(|i| i.restricted_embedding) {
            if !warned.contains(&info.family.as_str()) {
                warned.push(&info.family);
                eprintln_safe!(
                    "warning: the licence of {} does not allow embedding it in documents; \
                     check its licence before sharing this file.",
                    info.family
                );
            }
        }
    }
    let Some(body) = body else { return };
    let family = first_family(body);
    match kuva::fonts::family_styles(family) {
        Some(styles) => {
            let missing = styles.missing();
            if !missing.is_empty() {
                eprintln_safe!(
                    "warning: {family} has no {} face; that text uses the nearest face \
                     instead (kuva never fakes bold or italic). Load the font files for \
                     those styles with --font-file.",
                    join_or(&missing)
                );
            }
        }
        None => {
            const GENERIC: &[&str] = &[
                "serif",
                "sans-serif",
                "monospace",
                "cursive",
                "fantasy",
                "system-ui",
                "ui-sans-serif",
                "ui-serif",
                "ui-monospace",
            ];
            let builtin = family.to_ascii_lowercase().starts_with("dejavu sans");
            if !builtin && !GENERIC.contains(&family.to_ascii_lowercase().as_str()) {
                eprintln_safe!(
                    "warning: font '{family}' is not loaded, so text is spaced for DejaVu \
                     Sans and PNG output draws DejaVu Sans (SVG and PDF use '{family}' where \
                     it is installed). Load it with --font-file to use it exactly."
                );
            }
        }
    }
}

pub fn apply_base_args(mut layout: Layout, args: &BaseArgs) -> Layout {
    // Set on the layout (not on the finished scene) so anything drawn in the
    // background colour during rendering, such as an axis-break cover, matches.
    if let Some(ref bg) = args.background {
        layout = layout.with_background(bg.clone());
    }
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
        match palette_from_name(name) {
            Some(pal) => layout = layout.with_palette(pal),
            None => eprintln_safe!(
                "warning: unknown palette '{name}', using the default. \
                Run `kuva palettes` to see the available names."
            ),
        }
    }
    // --cvd-palette overrides --palette when both are provided.
    if let Some(ref condition) = args.cvd_palette {
        match colourblind_palette(condition) {
            Some(pal) => layout = layout.with_palette(pal),
            None => eprintln_safe!(
                "warning: unknown --cvd-palette '{condition}'. \
                Expected deuteranopia, protanopia or tritanopia."
            ),
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
    // Fonts after the theme, so --font wins over a theme's font family.
    let fonts = apply_font_args(args);
    if let Some(family) = fonts.body {
        layout = layout.with_font_family(family);
    }
    if let Some(family) = fonts.tick {
        layout = layout.with_tick_font_family(family);
    }
    if let Some(family) = fonts.mono {
        layout = layout.with_mono_font_family(family);
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
    if !args.x_tick_values.is_empty() {
        layout = layout.with_x_tick_values(args.x_tick_values.iter().copied());
    }
    if !args.y_tick_values.is_empty() {
        layout = layout.with_y_tick_values(args.y_tick_values.iter().copied());
    }
    if let Some(n) = args.x_minor_ticks {
        layout = layout.with_x_minor_ticks(n);
    }
    if let Some(n) = args.y_minor_ticks {
        layout = layout.with_y_minor_ticks(n);
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
    if let Some((lo, hi, share)) = args.x_break {
        let mut brk = AxisBreak::new(lo, hi);
        if let Some(f) = share {
            brk = brk.with_share(f);
        }
        layout = layout.with_x_break_config(brk);
    }
    if let Some((lo, hi, share)) = args.y_break {
        let mut brk = AxisBreak::new(lo, hi);
        if let Some(f) = share {
            brk = brk.with_share(f);
        }
        layout = layout.with_y_break_config(brk);
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
pub fn date_axis_from_args(args: &DateArgs, xs: &[f64]) -> Result<DateTimeAxis, String> {
    if let Some(fmt) = &args.x_date_tick_format {
        DateTimeAxis::check_format(fmt).map_err(|e| format!("--x-date-tick-format: {e}"))?;
    }
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
    Ok(match args.x_date_tick_step {
        Some(step) => axis.with_step(step),
        None => axis,
    })
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

/// One named palette the CLI accepts.
pub struct NamedPalette {
    /// Canonical name shown in help and `kuva palettes`.
    pub name: &'static str,
    /// Heading `kuva palettes` groups it under.
    pub group: &'static str,
    pub build: fn() -> Palette,
}

/// Every `--palette` name, in display order. `palette_from_name` and
/// `kuva palettes` both read this table, so the list can't drift.
pub const NAMED_PALETTES: &[NamedPalette] = &[
    NamedPalette {
        name: "category10",
        group: "General",
        build: Palette::category10,
    },
    NamedPalette {
        name: "pastel",
        group: "General",
        build: Palette::pastel,
    },
    NamedPalette {
        name: "bold",
        group: "General",
        build: Palette::bold,
    },
    NamedPalette {
        name: "wong",
        group: "Colourblind-safe",
        build: Palette::wong,
    },
    NamedPalette {
        name: "okabe-ito",
        group: "Colourblind-safe",
        build: Palette::okabe_ito,
    },
    NamedPalette {
        name: "tol-bright",
        group: "Colourblind-safe",
        build: Palette::tol_bright,
    },
    NamedPalette {
        name: "tol-muted",
        group: "Colourblind-safe",
        build: Palette::tol_muted,
    },
    NamedPalette {
        name: "tol-light",
        group: "Colourblind-safe",
        build: Palette::tol_light,
    },
    NamedPalette {
        name: "ibm",
        group: "Colourblind-safe",
        build: Palette::ibm,
    },
    // LTC palettes (loukesio/ltc-color-palettes, MIT). Accept an optional `ltc-` prefix.
    NamedPalette {
        name: "paloma",
        group: "LTC",
        build: Palette::paloma,
    },
    NamedPalette {
        name: "maya",
        group: "LTC",
        build: Palette::maya,
    },
    NamedPalette {
        name: "dora",
        group: "LTC",
        build: Palette::dora,
    },
    NamedPalette {
        name: "ploen",
        group: "LTC",
        build: Palette::ploen,
    },
    NamedPalette {
        name: "olga",
        group: "LTC",
        build: Palette::olga,
    },
    NamedPalette {
        name: "mterese",
        group: "LTC",
        build: Palette::mterese,
    },
    NamedPalette {
        name: "franscoise",
        group: "LTC",
        build: Palette::franscoise,
    },
    NamedPalette {
        name: "fernande",
        group: "LTC",
        build: Palette::fernande,
    },
    NamedPalette {
        name: "sylvie",
        group: "LTC",
        build: Palette::sylvie,
    },
    NamedPalette {
        name: "expevo",
        group: "LTC",
        build: Palette::expevo,
    },
    NamedPalette {
        name: "minou",
        group: "LTC",
        build: Palette::minou,
    },
    NamedPalette {
        name: "alger",
        group: "LTC",
        build: Palette::alger,
    },
    NamedPalette {
        name: "seafarer",
        group: "LTC",
        build: Palette::seafarer,
    },
    NamedPalette {
        name: "luminaries",
        group: "LTC",
        build: Palette::luminaries,
    },
    NamedPalette {
        name: "casa-natal",
        group: "LTC",
        build: Palette::casa_natal,
    },
];

/// One `--cvd-palette` condition.
pub struct CvdPalette {
    pub name: &'static str,
    /// Short alias (`deuter`, `protan`, `tritan`).
    pub alias: &'static str,
    pub build: fn() -> Palette,
}

/// Every `--cvd-palette` condition, in display order.
pub const CVD_PALETTES: &[CvdPalette] = &[
    CvdPalette {
        name: "deuteranopia",
        alias: "deuter",
        build: Palette::deuteranopia,
    },
    CvdPalette {
        name: "protanopia",
        alias: "protan",
        build: Palette::protanopia,
    },
    CvdPalette {
        name: "tritanopia",
        alias: "tritan",
        build: Palette::tritanopia,
    },
];

/// Look up a `--palette` name. `_` and `-` are interchangeable, and LTC
/// palettes take an optional `ltc-` prefix.
pub fn find_named_palette(name: &str) -> Option<&'static NamedPalette> {
    let name = name.replace('_', "-");
    let (key, ltc_only) = match name.strip_prefix("ltc-") {
        Some(rest) => (rest, true),
        None => (name.as_str(), false),
    };
    NAMED_PALETTES
        .iter()
        .find(|p| p.name == key && (!ltc_only || p.group == "LTC"))
}

pub fn palette_from_name(name: &str) -> Option<Palette> {
    find_named_palette(name).map(|p| (p.build)())
}

/// `--cvd-palette` lookup by canonical name or short alias.
pub fn find_cvd_palette(condition: &str) -> Option<&'static CvdPalette> {
    CVD_PALETTES
        .iter()
        .find(|c| condition == c.name || condition == c.alias)
}

fn colourblind_palette(condition: &str) -> Option<Palette> {
    find_cvd_palette(condition).map(|c| (c.build)())
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
