use clap::Args;

use kuva::plot::histogram::Histogram;
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{apply_na, ColSpec, DataTable, InputArgs};
use crate::layout_args::{
    apply_axis_args, apply_base_args, apply_log_args, AxisArgs, BaseArgs, LogArgs, NaArgs,
};
use crate::output::write_output;

/// Histogram from a numeric column.
#[derive(Args, Debug)]
pub struct HistogramArgs {
    /// Value column (0-based index or header name; default: 0). Use --y for multiple columns.
    #[arg(long)]
    pub value_col: Option<ColSpec>,

    /// Value column(s). A single name/index or comma-separated list for multiple overlapping
    /// histograms: `--y A,B,C` plots each as a separate colour-coded histogram over a shared
    /// x-range. Overrides --value-col when provided.
    #[arg(long, value_delimiter = ',')]
    pub y: Vec<ColSpec>,

    /// Bar fill color (CSS string; default: "steelblue"). Ignored when --y has multiple columns.
    #[arg(long)]
    pub color: Option<String>,

    /// Number of bins (default: 10).
    #[arg(long)]
    pub bins: Option<usize>,

    /// Normalize counts to a probability density (area = 1).
    #[arg(long)]
    pub normalize: bool,

    /// Show a legend entry for each series (applies when --y has multiple columns).
    #[arg(long)]
    pub legend: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
    #[command(flatten)]
    pub log: LogArgs,
    #[command(flatten)]
    pub na: NaArgs,
}

pub fn run(args: HistogramArgs) -> Result<(), String> {
    let y_specs: Vec<ColSpec> = if !args.y.is_empty() {
        args.y.clone()
    } else {
        vec![args.value_col.clone().unwrap_or(ColSpec::Index(0))]
    };

    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.header_mode(),
        args.input.delimiter,
        &y_specs,
    )?;

    let bins = args.bins.unwrap_or(10);

    let (na_set, na_strat, clamp) = args.na.resolve()?;
    // Drop/zero/error missing values before binning.
    let clean_values = |vals: Vec<Option<f64>>| -> Result<Vec<f64>, String> {
        let n = vals.len();
        let keep = apply_na(na_strat, &[&vals], n)?;
        Ok(keep.iter().map(|&i| vals[i].unwrap_or(0.0)).collect())
    };

    // Multi-column overlay mode
    if y_specs.len() > 1 {
        let all_values: Vec<Vec<f64>> = y_specs
            .iter()
            .map(|c| clean_values(table.col_f64_opt(c, &na_set, clamp)?))
            .collect::<Result<_, _>>()?;
        if all_values.iter().any(|v| v.is_empty()) {
            return Err("No data values found".to_string());
        }
        let min = all_values
            .iter()
            .flatten()
            .cloned()
            .fold(f64::INFINITY, f64::min);
        let max = all_values
            .iter()
            .flatten()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        let pal = Palette::category10();
        let hists: Vec<Histogram> = y_specs
            .iter()
            .enumerate()
            .zip(all_values)
            .map(|((i, col), values)| {
                // 8-digit hex: palette color + "b3" (≈70% alpha) for overlay legibility
                let color = format!("{}b3", &pal[i]);
                let mut h = Histogram::new()
                    .with_data(values)
                    .with_bins(bins)
                    .with_range((min, max))
                    .with_color(color);
                if args.normalize {
                    h = h.with_normalize();
                }
                if args.legend {
                    h = h.with_legend(table.col_display_name(col));
                }
                h
            })
            .collect();

        #[cfg(feature = "emit_code")]
        if args.base.emit_code {
            let exprs: Vec<String> = hists
                .iter()
                .map(crate::emit_code::emit_histogram_plot)
                .collect();
            print!(
                "{}",
                crate::emit_code::assemble(
                    &["kuva::plot::Histogram"],
                    "Histogram",
                    &exprs,
                    &args.base,
                    Some(&args.axis),
                    Some(&args.log),
                )
            );
            return Ok(());
        }

        let plots: Vec<Plot> = hists.into_iter().map(Plot::Histogram).collect();
        let layout = Layout::auto_from_plots(&plots);
        let layout = apply_base_args(layout, &args.base);
        let layout = apply_axis_args(layout, &args.axis);
        let layout = apply_log_args(layout, &args.log);
        let scene = render_multiple(plots, layout);
        return write_output(scene, &args.base);
    }

    // Single column mode
    let value_col = y_specs.into_iter().next().unwrap();
    let color = args.color.unwrap_or_else(|| "steelblue".to_string());

    let values = clean_values(table.col_f64_opt(&value_col, &na_set, clamp)?)?;
    if values.is_empty() {
        return Err("No data values found".to_string());
    }

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let mut plot = Histogram::new()
        .with_data(values)
        .with_bins(bins)
        .with_range((min, max))
        .with_color(&color);

    if args.normalize {
        plot = plot.with_normalize();
    }

    #[cfg(feature = "emit_code")]
    if args.base.emit_code {
        print!(
            "{}",
            crate::emit_code::assemble(
                &["kuva::plot::Histogram"],
                "Histogram",
                &[crate::emit_code::emit_histogram_plot(&plot)],
                &args.base,
                Some(&args.axis),
                Some(&args.log),
            )
        );
        return Ok(());
    }

    let plots = vec![Plot::Histogram(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let layout = apply_log_args(layout, &args.log);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
