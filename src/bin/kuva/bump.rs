use clap::Args;

use kuva::plot::bump::{BumpPlot, BumpTieBreak, CurveStyle};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_base_args, BaseArgs, AxisArgs};
use crate::output::write_output;

/// Bump chart — rank of named series across discrete time points or conditions.
///
/// Input TSV/CSV columns: series name, time/condition label, rank or raw value.
///
/// Example (pre-ranked):
///   series  time   rank
///   Alpha   2021   1
///   Alpha   2022   3
///   Beta    2021   2
///   Beta    2022   1
///
/// Example (auto-ranked from raw values, use --raw-value):
///   series  time   score
///   Alpha   2021   95.0
///   Beta    2021   80.0
#[derive(Args, Debug)]
pub struct BumpArgs {
    /// Series name column (name or 0-based index; default: 0).
    #[arg(long)]
    pub series: Option<ColSpec>,

    /// Time / condition label column (name or 0-based index; default: 1).
    #[arg(long)]
    pub time: Option<ColSpec>,

    /// Rank column (name or 0-based index; default: 2).
    /// Use with pre-ranked data.  Ignored when --raw-value is set.
    #[arg(long)]
    pub rank: Option<ColSpec>,

    /// Treat the third column as a raw value and auto-compute ranks per time point.
    /// Lower value = rank 1 when --rank-ascending is also passed.
    #[arg(long)]
    pub raw_value: bool,

    /// Rank ascending: lower raw value → better (lower) rank number.
    /// By default, higher raw value = rank 1.
    #[arg(long)]
    pub rank_ascending: bool,

    /// Tie-breaking strategy when auto-ranking: average (default), min, max, stable.
    #[arg(long, default_value = "average", value_enum)]
    pub tie_break: CliTieBreak,

    /// Curve style: sigmoid (default), straight.
    #[arg(long, default_value = "sigmoid", value_enum)]
    pub curve: CliCurveStyle,

    /// Show rank numbers inside each dot.
    #[arg(long)]
    pub rank_labels: bool,

    /// Hide series name labels at the left/right edges.
    #[arg(long)]
    pub no_series_labels: bool,

    /// Dot radius in pixels (default: 6.0).
    #[arg(long)]
    pub dot_radius: Option<f64>,

    /// Line stroke width in pixels (default: 2.5).
    #[arg(long)]
    pub stroke_width: Option<f64>,

    /// Highlight this series by name; all others are muted.
    #[arg(long)]
    pub highlight: Option<String>,

    /// Hide the legend.
    #[arg(long)]
    pub no_legend: bool,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
    #[command(flatten)]
    pub input: InputArgs,
}

#[derive(clap::ValueEnum, Clone, Default, Debug)]
pub enum CliTieBreak {
    #[default]
    Average,
    Min,
    Max,
    Stable,
}

#[derive(clap::ValueEnum, Clone, Default, Debug)]
pub enum CliCurveStyle {
    #[default]
    Sigmoid,
    Straight,
}

pub fn run(args: BumpArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let series_col = args.series.unwrap_or(ColSpec::Index(0));
    let time_col   = args.time.unwrap_or(ColSpec::Index(1));
    let rank_col   = args.rank.unwrap_or(ColSpec::Index(2));

    let series_vals = table.col_str(&series_col)?;
    let time_vals   = table.col_str(&time_col)?;
    let rank_vals   = table.col_f64(&rank_col)?;

    if series_vals.is_empty() {
        return Err("bump: input has no data".into());
    }

    // ── Collect ordered time labels (preserving first-seen order) ─────────────
    let mut time_order: Vec<String> = Vec::new();
    for t in &time_vals {
        if !time_order.contains(t) {
            time_order.push(t.clone());
        }
    }

    // ── Group rows by series name ─────────────────────────────────────────────
    // Each series gets a vec of Option<f64> in time_order position.
    use std::collections::BTreeMap;
    let mut series_map: BTreeMap<String, Vec<Option<f64>>> = BTreeMap::new();
    let n_time = time_order.len();

    for ((series_name, time_label), val) in series_vals.iter()
        .zip(time_vals.iter())
        .zip(rank_vals.iter())
    {
        let entry = series_map.entry(series_name.clone())
            .or_insert_with(|| vec![None; n_time]);
        if let Some(pos) = time_order.iter().position(|t| t == time_label) {
            entry[pos] = Some(*val);
        }
    }

    // ── Build BumpPlot ────────────────────────────────────────────────────────
    let mut bp = BumpPlot::new();

    bp = bp.with_x_labels(time_order);

    let curve = match args.curve {
        CliCurveStyle::Sigmoid  => CurveStyle::Sigmoid,
        CliCurveStyle::Straight => CurveStyle::Straight,
    };
    bp = bp.with_curve_style(curve);

    let tie_break = match args.tie_break {
        CliTieBreak::Average => BumpTieBreak::Average,
        CliTieBreak::Min     => BumpTieBreak::Min,
        CliTieBreak::Max     => BumpTieBreak::Max,
        CliTieBreak::Stable  => BumpTieBreak::Stable,
    };
    bp = bp.with_tie_break(tie_break);

    if args.raw_value {
        bp = bp.with_rank_ascending(args.rank_ascending);
        for (name, vals) in series_map {
            bp = bp.with_raw_series_opt(name, vals);
        }
    } else {
        for (name, vals) in series_map {
            bp = bp.with_ranked_series(name, vals);
        }
    }

    if args.rank_labels   { bp = bp.with_show_rank_labels(true); }
    if args.no_series_labels { bp = bp.with_show_series_labels(false); }
    if args.no_legend     { bp = bp.with_legend(false); }
    if let Some(r) = args.dot_radius  { bp = bp.with_dot_radius(r); }
    if let Some(w) = args.stroke_width { bp = bp.with_stroke_width(w); }
    if let Some(hl) = args.highlight   { bp = bp.with_highlight(hl); }

    // ── Layout and render ─────────────────────────────────────────────────────
    let plots = vec![Plot::Bump(bp)];
    let mut layout = Layout::auto_from_plots(&plots);
    layout = apply_base_args(layout, &args.base);
    if let Some(xl) = args.axis.x_label { layout = layout.with_x_label(xl); }
    if let Some(yl) = args.axis.y_label { layout = layout.with_y_label(yl); }
    if args.axis.no_grid { layout = layout.with_show_grid(false); }

    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
