use clap::Args;

use kuva::plot::GanttPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_axis_args, apply_base_args, AxisArgs, BaseArgs};
use crate::output::write_output;

/// Gantt chart — horizontal task bars with optional groups, progress, and milestones.
#[derive(Args, Debug)]
pub struct GanttArgs {
    /// Task label column (default: 0).
    #[arg(long)]
    pub label_col: Option<ColSpec>,

    /// Start value column (default: 1).
    #[arg(long)]
    pub start_col: Option<ColSpec>,

    /// End value column (default: 2). Set equal to start for milestones.
    #[arg(long)]
    pub end_col: Option<ColSpec>,

    /// Optional group/phase column.
    #[arg(long)]
    pub group_col: Option<ColSpec>,

    /// Optional progress column (values 0–1 or 0–100; values >1 divided by 100).
    #[arg(long)]
    pub progress_col: Option<ColSpec>,

    /// Optional milestone column (non-empty / "1" / "true" marks a task as a milestone).
    #[arg(long)]
    pub milestone_col: Option<ColSpec>,

    /// Draw a vertical dashed "now" line at this x value.
    #[arg(long)]
    pub now: Option<f64>,

    /// Bar height as fraction of row height (default 0.6).
    #[arg(long)]
    pub bar_height: Option<f64>,

    /// Default bar color when no groups are present (CSS color string).
    #[arg(long)]
    pub color: Option<String>,

    /// Hide task labels inside/beside bars.
    #[arg(long)]
    pub no_labels: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: GanttArgs) -> Result<(), String> {
    let mut proj: Vec<ColSpec> = vec![
        args.label_col.clone().unwrap_or(ColSpec::Index(0)),
        args.start_col.clone().unwrap_or(ColSpec::Index(1)),
        args.end_col.clone().unwrap_or(ColSpec::Index(2)),
    ];
    if let Some(ref c) = args.group_col {
        proj.push(c.clone());
    }
    if let Some(ref c) = args.progress_col {
        proj.push(c.clone());
    }
    if let Some(ref c) = args.milestone_col {
        proj.push(c.clone());
    }
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;

    let label_col = args.label_col.unwrap_or(ColSpec::Index(0));
    let start_col = args.start_col.unwrap_or(ColSpec::Index(1));
    let end_col = args.end_col.unwrap_or(ColSpec::Index(2));

    let labels = table.col_str(&label_col)?;
    let starts = table.col_f64(&start_col)?;
    let ends = table.col_f64(&end_col)?;

    let n = labels.len();
    if starts.len() != n || ends.len() != n {
        return Err(format!(
            "column length mismatch: label={n}, start={}, end={}",
            starts.len(),
            ends.len()
        ));
    }

    let groups = if let Some(ref gc) = args.group_col {
        Some(table.col_str(gc)?)
    } else {
        None
    };

    let progresses = if let Some(ref pc) = args.progress_col {
        Some(table.col_f64(pc)?)
    } else {
        None
    };

    let milestones = if let Some(ref mc) = args.milestone_col {
        Some(table.col_str(mc)?)
    } else {
        None
    };

    let mut plot = GanttPlot::new();

    if let Some(c) = args.color {
        plot = plot.with_color(c);
    }
    if let Some(bh) = args.bar_height {
        plot = plot.with_bar_height(bh);
    }
    if args.no_labels {
        plot = plot.with_show_labels(false);
    }
    if let Some(now) = args.now {
        plot = plot.with_now_line(now);
    }

    for i in 0..n {
        let label = labels[i].as_str();
        let start = starts[i];
        let end = ends[i];
        let group = groups
            .as_ref()
            .map(|g| g[i].clone())
            .filter(|g| !g.is_empty());
        let progress = progresses.as_ref().map(|p| {
            let v = p[i];
            if v > 1.0 {
                v / 100.0
            } else {
                v
            }
        });
        let is_milestone = milestones
            .as_ref()
            .map(|m| {
                let s = m[i].to_lowercase();
                s == "1"
                    || s == "true"
                    || s == "yes"
                    || !s.is_empty() && s != "0" && s != "false" && s != "no"
            })
            .unwrap_or(false);

        if is_milestone {
            let at = start;
            match &group {
                Some(g) => plot = plot.with_milestone_group(g.clone(), label, at),
                None => plot = plot.with_milestone(label, at),
            }
        } else {
            match (&group, progress) {
                (Some(g), Some(p)) => {
                    plot = plot.with_task_group_progress(g.clone(), label, start, end, p)
                }
                (Some(g), None) => plot = plot.with_task_group(g.clone(), label, start, end),
                (None, Some(p)) => plot = plot.with_task_progress(label, start, end, p),
                (None, None) => plot = plot.with_task(label, start, end),
            }
        }
    }

    let plots = vec![Plot::Gantt(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
