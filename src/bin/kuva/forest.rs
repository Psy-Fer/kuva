use clap::Args;

use kuva::plot::ForestPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, apply_base_args, apply_axis_args};
use crate::output::write_output;

/// Forest plot — point estimates with confidence intervals on a categorical Y-axis.
#[derive(Args, Debug)]
pub struct ForestArgs {
    /// Label column (study name; 0-based index or header name; default: 0).
    #[arg(long)]
    pub label_col: Option<ColSpec>,

    /// Estimate column (point estimate; 0-based index or header name; default: 1).
    #[arg(long)]
    pub estimate_col: Option<ColSpec>,

    /// CI lower-bound column (0-based index or header name; default: 2).
    #[arg(long)]
    pub ci_lower_col: Option<ColSpec>,

    /// CI upper-bound column (0-based index or header name; default: 3).
    #[arg(long)]
    pub ci_upper_col: Option<ColSpec>,

    /// Optional weight column (scales marker radius).
    #[arg(long)]
    pub weight_col: Option<ColSpec>,

    /// Point and whisker color (CSS string; default: "steelblue").
    #[arg(long)]
    pub color: Option<String>,

    /// Marker half-width in pixels (default: 6.0).
    #[arg(long)]
    pub marker_size: Option<f64>,

    /// Whisker stroke width in pixels (default: 1.5).
    #[arg(long)]
    pub whisker_width: Option<f64>,

    /// Null-effect reference value (default: 0.0).
    #[arg(long)]
    pub null_value: Option<f64>,

    /// Disable the dashed null-effect reference line.
    #[arg(long)]
    pub no_null_line: bool,

    /// Whisker end-cap half-height in pixels (default: 0, no caps).
    #[arg(long)]
    pub cap_size: Option<f64>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: ForestArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let label_col = args.label_col.unwrap_or(ColSpec::Index(0));
    let estimate_col = args.estimate_col.unwrap_or(ColSpec::Index(1));
    let ci_lower_col = args.ci_lower_col.unwrap_or(ColSpec::Index(2));
    let ci_upper_col = args.ci_upper_col.unwrap_or(ColSpec::Index(3));

    let labels = table.col_str(&label_col)?;
    let estimates = table.col_f64(&estimate_col)?;
    let ci_lowers = table.col_f64(&ci_lower_col)?;
    let ci_uppers = table.col_f64(&ci_upper_col)?;

    let n = labels.len();
    if estimates.len() != n || ci_lowers.len() != n || ci_uppers.len() != n {
        return Err(format!(
            "column length mismatch: labels={}, estimates={}, ci_lower={}, ci_upper={}",
            n, estimates.len(), ci_lowers.len(), ci_uppers.len()
        ));
    }

    let weights = if let Some(ref wc) = args.weight_col {
        let w = table.col_f64(wc)?;
        if w.len() != n {
            return Err(format!(
                "weight column length ({}) does not match label column ({})", w.len(), n
            ));
        }
        Some(w)
    } else {
        None
    };

    let mut plot = ForestPlot::new();

    if let Some(color) = args.color {
        plot = plot.with_color(color);
    }
    if let Some(size) = args.marker_size {
        plot = plot.with_marker_size(size);
    }
    if let Some(ww) = args.whisker_width {
        plot = plot.with_whisker_width(ww);
    }
    if let Some(nv) = args.null_value {
        plot = plot.with_null_value(nv);
    }
    if args.no_null_line {
        plot = plot.with_show_null_line(false);
    }
    if let Some(cs) = args.cap_size {
        plot = plot.with_cap_size(cs);
    }

    let rows = labels.iter().zip(estimates).zip(ci_lowers).zip(ci_uppers);
    if let Some(w) = weights {
        for ((((label, est), lo), hi), weight) in rows.zip(w) {
            plot = plot.with_weighted_row(label.as_str(), est, lo, hi, weight);
        }
    } else {
        for (((label, est), lo), hi) in rows {
            plot = plot.with_row(label.as_str(), est, lo, hi);
        }
    }

    let plots = vec![Plot::Forest(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
