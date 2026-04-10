use clap::Args;
use kuva::plot::QQPlot;
use kuva::render::plots::Plot;
use kuva::render::layout::Layout;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, apply_base_args, apply_axis_args};
use crate::output::write_output;

#[derive(Args)]
#[command(about = "Q-Q plot: normal or genomic (GWAS) quantile-quantile")]
pub struct QQArgs {
    /// Column of numeric values (normal mode) or p-values (genomic mode)
    #[arg(long, default_value = "0")]
    value: ColSpec,

    /// Group by this column; one set of points per unique value
    #[arg(long)]
    color_by: Option<ColSpec>,

    /// Genomic Q-Q mode: input values are p-values in (0, 1]
    #[arg(long)]
    genomic: bool,

    /// Draw 95% pointwise CI band around the reference diagonal
    #[arg(long)]
    ci_band: bool,

    /// Show genomic inflation factor λ (genomic mode only)
    #[arg(long)]
    lambda: bool,

    /// Hide the reference line
    #[arg(long)]
    no_reference_line: bool,

    /// Marker size in pixels
    #[arg(long, default_value_t = 3.0)]
    marker_size: f64,

    /// Marker fill opacity (0–1)
    #[arg(long)]
    fill_opacity: Option<f64>,

    #[command(flatten)]
    base: BaseArgs,
    #[command(flatten)]
    axis: AxisArgs,
    #[command(flatten)]
    input: InputArgs,
}

pub fn run(args: QQArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let mut plot = QQPlot::new();

    if args.genomic {
        plot = plot.with_genomic();
    }
    if args.ci_band {
        plot = plot.with_ci_band();
    }
    if args.lambda {
        plot = plot.with_lambda();
    }
    if args.no_reference_line {
        plot = plot.without_reference_line();
    }
    plot = plot.with_marker_size(args.marker_size);
    if let Some(op) = args.fill_opacity {
        plot = plot.with_fill_opacity(op);
    }

    if let Some(ref color_col) = args.color_by {
        let groups = table.group_by(color_col)?;
        let show_legend = groups.len() > 1;
        for (label, sub) in groups {
            let values: Vec<f64> = sub.col_f64(&args.value)?;
            if args.genomic {
                plot = plot.with_pvalues(label.clone(), values);
            } else {
                plot = plot.with_data(label.clone(), values);
            }
        }
        if show_legend {
            plot = plot.with_legend("");
        }
    } else {
        let values: Vec<f64> = table.col_f64(&args.value)?;
        if args.genomic {
            plot = plot.with_pvalues("", values);
        } else {
            plot = plot.with_data("", values);
        }
    }

    let plots = vec![Plot::QQ(plot)];
    let mut layout = Layout::auto_from_plots(&plots);

    // Set sensible default axis labels if not provided
    if layout.x_label.is_none() {
        layout.x_label = Some(if args.genomic {
            "Expected −log₁₀(p)".into()
        } else {
            "Theoretical Quantiles".into()
        });
    }
    if layout.y_label.is_none() {
        layout.y_label = Some(if args.genomic {
            "Observed −log₁₀(p)".into()
        } else {
            "Sample Quantiles".into()
        });
    }

    let layout = apply_axis_args(layout, &args.axis);
    let layout = apply_base_args(layout, &args.base);

    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
