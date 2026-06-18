use clap::Args;

use kuva::plot::DensityPlot;
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{
    apply_axis_args, apply_base_args, apply_log_args, AxisArgs, BaseArgs, LogArgs,
};
use crate::output::write_output;

/// Kernel density estimate curve from a numeric column.
#[derive(Args, Debug)]
pub struct DensityArgs {
    /// Column containing numeric values (0-based index or header name; default: 0).
    #[arg(long, default_value = "0")]
    pub value: ColSpec,

    /// Value column(s). Comma-separated for multi-column mode: `--y A,B,C` plots one density
    /// curve per column (column name = legend label). Mutually exclusive with --color-by.
    #[arg(long, value_delimiter = ',')]
    pub y: Vec<ColSpec>,

    /// Group by this column — one density curve per unique value.
    #[arg(long)]
    pub color_by: Option<ColSpec>,

    /// Fill the area under the density curve.
    #[arg(long)]
    pub filled: bool,

    /// KDE bandwidth (default: Silverman's rule-of-thumb).
    #[arg(long)]
    pub bandwidth: Option<f64>,

    /// Fit the y-axis to the data range instead of anchoring at zero.
    #[arg(long)]
    pub fit: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,

    #[command(flatten)]
    pub axis: AxisArgs,

    #[command(flatten)]
    pub log: LogArgs,
}

pub fn run(args: DensityArgs) -> Result<(), String> {
    // Multi-column --y mode: one density curve per column
    if args.y.len() > 1 {
        if args.color_by.is_some() {
            return Err(
                "--y with multiple columns is mutually exclusive with --color-by".to_string(),
            );
        }
        let table = DataTable::parse(
            args.input.input.as_deref(),
            args.input.no_header,
            args.input.delimiter,
            &args.y,
        )?;
        let pal = Palette::category10();
        let plots: Vec<Plot> = args
            .y
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let vals = table.col_f64(col)?;
                let name = table.col_display_name(col);
                let mut dp = DensityPlot::new()
                    .with_data(vals)
                    .with_color(pal[i].to_string())
                    .with_legend(name);
                if args.filled {
                    dp = dp.with_filled(true);
                }
                if let Some(bw) = args.bandwidth {
                    dp = dp.with_bandwidth(bw);
                }
                if let Some(lo) = args.axis.x_min {
                    dp = dp.with_x_lo(lo);
                }
                if let Some(hi) = args.axis.x_max {
                    dp = dp.with_x_hi(hi);
                }
                if args.fit {
                    dp = dp.with_fit();
                }
                Ok(Plot::Density(dp))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let layout = Layout::auto_from_plots(&plots);
        let layout = apply_base_args(layout, &args.base);
        let layout = apply_axis_args(layout, &args.axis);
        let layout = apply_log_args(layout, &args.log);
        let scene = render_multiple(plots, layout);
        return write_output(scene, &args.base);
    }

    let mut proj: Vec<ColSpec> = if args.y.len() == 1 {
        vec![args.y[0].clone()]
    } else {
        vec![args.value.clone()]
    };
    if let Some(ref c) = args.color_by {
        proj.push(c.clone());
    }
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;
    let value_col = if args.y.len() == 1 {
        args.y[0].clone()
    } else {
        args.value.clone()
    };

    let plots: Vec<Plot> = if let Some(ref cb) = args.color_by {
        let pal = Palette::category10();
        let groups = table.group_by(cb)?;
        groups
            .into_iter()
            .enumerate()
            .map(|(i, (name, subtable))| {
                let vals = subtable.col_f64(&value_col)?;
                let mut dp = DensityPlot::new()
                    .with_data(vals)
                    .with_color(pal[i].to_string())
                    .with_legend(name);
                if args.filled {
                    dp = dp.with_filled(true);
                }
                if let Some(bw) = args.bandwidth {
                    dp = dp.with_bandwidth(bw);
                }
                if let Some(lo) = args.axis.x_min {
                    dp = dp.with_x_lo(lo);
                }
                if let Some(hi) = args.axis.x_max {
                    dp = dp.with_x_hi(hi);
                }
                if args.fit {
                    dp = dp.with_fit();
                }
                Ok(Plot::Density(dp))
            })
            .collect::<Result<Vec<_>, String>>()?
    } else {
        let vals = table.col_f64(&value_col)?;
        let mut dp = DensityPlot::new().with_data(vals);
        if args.filled {
            dp = dp.with_filled(true);
        }
        if let Some(bw) = args.bandwidth {
            dp = dp.with_bandwidth(bw);
        }
        if let (Some(lo), Some(hi)) = (args.axis.x_min, args.axis.x_max) {
            dp = dp.with_x_range(lo, hi);
        }
        if args.fit {
            dp = dp.with_fit();
        }
        vec![Plot::Density(dp)]
    };

    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let layout = apply_log_args(layout, &args.log);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
