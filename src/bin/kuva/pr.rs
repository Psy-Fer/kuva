use clap::Args;

use kuva::plot::pr::{PrGroup, PrPlot};
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_axis_args, apply_base_args, AxisArgs, BaseArgs};
use crate::output::write_output;

/// PR curve — Precision-Recall plot from score/label data.
#[derive(Args, Debug)]
pub struct PrArgs {
    /// Score column (classifier output; 0-based index or header name; default: 0).
    #[arg(long)]
    pub score_col: Option<ColSpec>,

    /// Label column (0/1 or false/true; 1 = positive; default: 1).
    #[arg(long)]
    pub label_col: Option<ColSpec>,

    /// Group by this column — one PR curve per unique value.
    #[arg(long)]
    pub color_by: Option<ColSpec>,

    /// Hide the no-skill prevalence baseline.
    #[arg(long)]
    pub no_baseline: bool,

    /// Annotate each curve with its AUC-PR value.
    #[arg(long)]
    pub auc_label: bool,

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

pub fn run(args: PrArgs) -> Result<(), String> {
    let mut proj: Vec<ColSpec> = vec![
        args.score_col.clone().unwrap_or(ColSpec::Index(0)),
        args.label_col.clone().unwrap_or(ColSpec::Index(1)),
    ];
    if let Some(ref c) = args.color_by { proj.push(c.clone()); }
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;

    let score_col = args.score_col.unwrap_or(ColSpec::Index(0));
    let label_col = args.label_col.unwrap_or(ColSpec::Index(1));

    let pal = Palette::category10();

    let mut plot = PrPlot::new();

    if args.no_baseline {
        plot = plot.with_baseline(false);
    }
    if let Some(legend) = args.legend {
        plot = plot.with_legend(legend);
    }

    if let Some(ref cb) = args.color_by {
        let groups = table.group_by(cb)?;
        for (i, (name, subtable)) in groups.into_iter().enumerate() {
            let scores = subtable.col_f64(&score_col)?;
            let labels = subtable.col_f64(&label_col)?;
            let predictions: Vec<(f64, bool)> = scores
                .into_iter()
                .zip(labels)
                .map(|(s, l)| (s, l > 0.5))
                .collect();
            let mut group = PrGroup::new(&name)
                .with_raw(predictions)
                .with_color(pal[i % pal.len()].to_string());
            if args.auc_label {
                group = group.with_auc_label(true);
            }
            plot = plot.with_group(group);
        }
    } else {
        let scores = table.col_f64(&score_col)?;
        let labels = table.col_f64(&label_col)?;
        let predictions: Vec<(f64, bool)> = scores
            .into_iter()
            .zip(labels)
            .map(|(s, l)| (s, l > 0.5))
            .collect();
        let mut group = PrGroup::new("Model")
            .with_raw(predictions)
            .with_color(pal[0].to_string());
        if args.auc_label {
            group = group.with_auc_label(true);
        }
        plot = plot.with_group(group);
    }

    let plots = vec![Plot::Pr(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
