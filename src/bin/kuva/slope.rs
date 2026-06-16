use clap::Args;

use kuva::plot::SlopePlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_axis_args, apply_base_args, AxisArgs, BaseArgs};
use crate::output::write_output;

/// Slope plot — compare paired before/after values per labeled entity.
#[derive(Args, Debug)]
pub struct SlopeArgs {
    /// Label column (0-based index or header name; default: 0).
    #[arg(long)]
    pub label_col: Option<ColSpec>,

    /// Before-value column (0-based index or header name; default: 1).
    #[arg(long)]
    pub before_col: Option<ColSpec>,

    /// After-value column (0-based index or header name; default: 2).
    #[arg(long)]
    pub after_col: Option<ColSpec>,

    /// Label for the "before" axis (e.g. "2020").
    #[arg(long)]
    pub before_label: Option<String>,

    /// Label for the "after" axis (e.g. "2024").
    #[arg(long)]
    pub after_label: Option<String>,

    /// Color for upward-trending lines (CSS color string).
    #[arg(long)]
    pub color_up: Option<String>,

    /// Color for downward-trending lines (CSS color string).
    #[arg(long)]
    pub color_down: Option<String>,

    /// Disable coloring lines by direction (all lines use a single color).
    #[arg(long)]
    pub no_direction_colors: bool,

    /// Show value labels at each endpoint.
    #[arg(long)]
    pub show_values: bool,

    /// Stroke width of each slope line in pixels.
    #[arg(long)]
    pub line_width: Option<f64>,

    /// Radius of endpoint dots in pixels.
    #[arg(long)]
    pub dot_radius: Option<f64>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,

    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: SlopeArgs) -> Result<(), String> {
    let proj: Vec<ColSpec> = vec![
        args.label_col.clone().unwrap_or(ColSpec::Index(0)),
        args.before_col.clone().unwrap_or(ColSpec::Index(1)),
        args.after_col.clone().unwrap_or(ColSpec::Index(2)),
    ];
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;

    let label_col = args.label_col.unwrap_or(ColSpec::Index(0));
    let before_col = args.before_col.unwrap_or(ColSpec::Index(1));
    let after_col = args.after_col.unwrap_or(ColSpec::Index(2));

    let labels = table.col_str(&label_col)?;
    let befores = table.col_f64(&before_col)?;
    let afters = table.col_f64(&after_col)?;

    let n = labels.len();
    if befores.len() != n || afters.len() != n {
        return Err(format!(
            "column length mismatch: labels={}, before={}, after={}",
            n,
            befores.len(),
            afters.len()
        ));
    }

    let mut plot = SlopePlot::new();

    if let Some(s) = args.before_label {
        plot = plot.with_before_label(s);
    }
    if let Some(s) = args.after_label {
        plot = plot.with_after_label(s);
    }
    if let Some(s) = args.color_up {
        plot = plot.with_color_up(s);
    }
    if let Some(s) = args.color_down {
        plot = plot.with_color_down(s);
    }
    if args.no_direction_colors {
        plot = plot.with_direction_colors(false);
    }
    if args.show_values {
        plot = plot.with_values(true);
    }
    if let Some(v) = args.line_width {
        plot = plot.with_line_width(v);
    }
    if let Some(v) = args.dot_radius {
        plot = plot.with_dot_radius(v);
    }

    for ((label, before), after) in labels.iter().zip(befores).zip(afters) {
        plot = plot.with_point(label.as_str(), before, after);
    }

    let plots = vec![Plot::Slope(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
