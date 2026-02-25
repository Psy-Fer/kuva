use clap::Args;
use visus::render::layout::Layout;
use visus::render::palette::Palette;
use visus::render::theme::Theme;

// ── Composable arg structs ────────────────────────────────────────────────────
// Flatten only the relevant combination into each subcommand:
//   Pie                    →  BaseArgs
//   Bar / Box / Violin     →  BaseArgs + AxisArgs
//   Scatter / Line / Hist  →  BaseArgs + AxisArgs + LogArgs

/// Output & appearance
#[derive(Args, Debug)]
pub struct BaseArgs {
    /// Output file. SVG/PNG/PDF inferred from extension. Defaults to SVG on stdout.
    #[arg(short = 'o', long)]
    pub output: Option<std::path::PathBuf>,

    #[arg(long)]
    pub title: Option<String>,

    #[arg(long, default_value_t = 800.0)]
    pub width: f64,

    #[arg(long, default_value_t = 500.0)]
    pub height: f64,

    /// Visual theme: light (default), dark, solarized, minimal
    #[arg(long)]
    pub theme: Option<String>,

    /// Named color palette: category10, wong, okabe-ito, pastel, bold,
    /// tol-bright, tol-muted, tol-light, ibm
    #[arg(long)]
    pub palette: Option<String>,

    /// Colourblind-safe palette by condition: deuteranopia, protanopia, tritanopia.
    /// Overrides --palette when both are provided.
    #[arg(long)]
    pub colourblind: Option<String>,

    /// Override the SVG background color (CSS color string).
    /// When omitted the theme's background is used.
    #[arg(long)]
    pub background: Option<String>,
}

/// Axis options for plots that have numeric axes.
/// Not applicable to pie charts.
#[derive(Args, Debug)]
pub struct AxisArgs {
    #[arg(long)]
    pub x_label: Option<String>,

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
}

/// Log-scale options for continuous-axis plots (scatter, line, histogram).
/// Not applicable to category-axis plots (bar, box, violin) or pie.
#[derive(Args, Debug)]
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
    layout = layout.with_width(args.width).with_height(args.height);
    if let Some(ref t) = args.title {
        layout = layout.with_title(t.clone());
    }
    // Theme first so subsequent flags can override individual settings.
    if let Some(ref name) = args.theme {
        layout = layout.with_theme(theme_from_name(name));
    }
    if let Some(ref name) = args.palette {
        if let Some(pal) = palette_from_name(name) {
            layout = layout.with_palette(pal);
        }
    }
    // --colourblind overrides --palette when both are provided.
    if let Some(ref condition) = args.colourblind {
        if let Some(pal) = colourblind_palette(condition) {
            layout = layout.with_palette(pal);
        }
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
