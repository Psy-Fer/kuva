use clap::Args;

use kuva::plot::LollipopPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_axis_args, apply_base_args, AxisArgs, BaseArgs};
use crate::output::write_output;

/// Lollipop plot — points on stems rising from a baseline.
#[derive(Args, Debug)]
pub struct LollipopArgs {
    /// X-value column (0-based index or header name; default: 0).
    #[arg(long)]
    pub x_col: Option<ColSpec>,

    /// Y-value column (0-based index or header name; default: 1).
    #[arg(long)]
    pub y_col: Option<ColSpec>,

    /// Optional label column (0-based index or header name).
    #[arg(long)]
    pub label_col: Option<ColSpec>,

    /// Dot and stem color (CSS color string).
    #[arg(long)]
    pub color: Option<String>,

    /// Baseline value (default: 0.0).
    #[arg(long)]
    pub baseline: Option<f64>,

    /// Stem stroke width in pixels.
    #[arg(long)]
    pub stem_width: Option<f64>,

    /// Dot radius in pixels.
    #[arg(long)]
    pub dot_radius: Option<f64>,

    /// Hide the horizontal baseline rule.
    #[arg(long)]
    pub no_baseline_line: bool,

    /// Legend label for the series.
    #[arg(long)]
    pub legend: Option<String>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,

    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: LollipopArgs) -> Result<(), String> {
    let mut proj: Vec<ColSpec> = vec![
        args.x_col.clone().unwrap_or(ColSpec::Index(0)),
        args.y_col.clone().unwrap_or(ColSpec::Index(1)),
    ];
    if let Some(ref c) = args.label_col { proj.push(c.clone()); }
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;

    let x_col = args.x_col.unwrap_or(ColSpec::Index(0));
    let y_col = args.y_col.unwrap_or(ColSpec::Index(1));

    let ys = table.col_f64(&y_col)?;

    // x can be numeric or categorical (string labels → use row index as x)
    let (xs, cat_labels): (Vec<f64>, Option<Vec<String>>) = match table.col_f64(&x_col) {
        Ok(nums) => (nums, None),
        Err(_) => {
            let strs = table.col_str(&x_col)?;
            let idxs = (0..strs.len()).map(|i| i as f64).collect();
            (idxs, Some(strs))
        }
    };

    let n = xs.len();
    if ys.len() != n {
        return Err(format!("column length mismatch: x={}, y={}", n, ys.len()));
    }

    // Explicit label_col overrides categorical x labels
    let labels: Option<Vec<String>> = if let Some(ref lc) = args.label_col {
        Some(table.col_str(lc)?)
    } else {
        cat_labels
    };

    let mut plot = LollipopPlot::new();

    if let Some(s) = args.color {
        plot = plot.with_color(s);
    }
    if let Some(v) = args.baseline {
        plot = plot.with_baseline(v);
    }
    if let Some(v) = args.stem_width {
        plot = plot.with_stem_width(v);
    }
    if let Some(v) = args.dot_radius {
        plot = plot.with_dot_radius(v);
    }
    if args.no_baseline_line {
        plot = plot.with_show_baseline(false);
    }
    if let Some(s) = args.legend {
        plot = plot.with_legend(s);
    }

    if let Some(ref lbls) = labels {
        for ((x, y), label) in xs.iter().zip(ys.iter()).zip(lbls.iter()) {
            plot = plot.with_labeled_point(*x, *y, label.as_str());
        }
    } else {
        for (x, y) in xs.iter().zip(ys.iter()) {
            plot = plot.with_point(*x, *y);
        }
    }

    let plots = vec![Plot::Lollipop(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
