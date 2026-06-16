use clap::Args;

use kuva::plot::SurvivalPlot;
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_axis_args, apply_base_args, AxisArgs, BaseArgs};
use crate::output::write_output;

/// Kaplan-Meier survival curve from time-to-event data.
#[derive(Args, Debug)]
pub struct SurvivalArgs {
    /// Time column (time to event or censoring; default: 0).
    #[arg(long)]
    pub time_col: Option<ColSpec>,

    /// Event column (1 = event occurred, 0 = censored; default: 1).
    #[arg(long)]
    pub event_col: Option<ColSpec>,

    /// Group by this column — one curve per unique value.
    #[arg(long)]
    pub group_col: Option<ColSpec>,

    /// Hide 95% confidence interval bands (CI is on by default).
    #[arg(long)]
    pub no_ci: bool,

    /// Hide censoring tick marks on curves.
    #[arg(long)]
    pub no_censoring: bool,

    /// Line stroke width in pixels.
    #[arg(long)]
    pub line_width: Option<f64>,

    /// Legend title.
    #[arg(long)]
    pub legend: Option<String>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: SurvivalArgs) -> Result<(), String> {
    let mut proj: Vec<ColSpec> = vec![
        args.time_col.clone().unwrap_or(ColSpec::Index(0)),
        args.event_col.clone().unwrap_or(ColSpec::Index(1)),
    ];
    if let Some(ref c) = args.group_col {
        proj.push(c.clone());
    }
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;

    let time_col = args.time_col.unwrap_or(ColSpec::Index(0));
    let event_col = args.event_col.unwrap_or(ColSpec::Index(1));

    let pal = Palette::category10();

    let mut plot = SurvivalPlot::new();

    // CI defaults to false in SurvivalPlot; the flag enables it unless --no-ci is passed.
    // Spec says CI is on by default for CLI — so enable by default, disable with --no-ci.
    if !args.no_ci {
        plot = plot.with_ci(true);
    }
    if args.no_censoring {
        plot = plot.with_censoring(false);
    }
    if let Some(lw) = args.line_width {
        plot = plot.with_line_width(lw);
    }
    if let Some(legend) = args.legend {
        plot = plot.with_legend(legend);
    }

    if let Some(ref gc) = args.group_col {
        let groups = table.group_by(gc)?;
        for (i, (name, subtable)) in groups.into_iter().enumerate() {
            let times = subtable.col_f64(&time_col)?;
            let event_vals = subtable.col_f64(&event_col)?;
            let events: Vec<bool> = event_vals.into_iter().map(|v| v > 0.5).collect();
            let color = pal[i % pal.len()].to_string();
            plot = plot.with_colored_group(name, times, events, color);
        }
    } else {
        let times = table.col_f64(&time_col)?;
        let event_vals = table.col_f64(&event_col)?;
        let events: Vec<bool> = event_vals.into_iter().map(|v| v > 0.5).collect();
        plot = plot.with_colored_group("All", times, events, pal[0].to_string());
    }

    let plots = vec![Plot::Survival(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
