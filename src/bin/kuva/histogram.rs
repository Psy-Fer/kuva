use clap::Args;

use kuva::plot::histogram::{BinMethod, Histogram};
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

    /// Number of bins (default: 10). Ignored when --bin-method is set.
    #[arg(long)]
    pub bins: Option<usize>,

    /// Automatic bin-count rule, overriding --bins: sturges, scott, or fd
    /// (Freedman-Diaconis).
    #[arg(long)]
    pub bin_method: Option<String>,

    /// Normalize counts to a probability density (area = 1).
    #[arg(long)]
    pub normalize: bool,

    /// Draw outline-only staircases instead of filled bars (histtype='step').
    #[arg(long)]
    pub step: bool,

    /// Accumulate counts left-to-right into a cumulative histogram.
    #[arg(long)]
    pub cumulative: bool,

    /// Stack multiple --y columns on top of each other instead of overlaying
    /// them (only applies when --y lists more than one column).
    #[arg(long)]
    pub stacked: bool,

    /// Per-sample weight column (index or header name). Applies in single-column
    /// mode; each row contributes its weight to its bin instead of 1.
    #[arg(long)]
    pub weight_col: Option<ColSpec>,

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
    // Expand column ranges / globs against the parsed table (issue #109).
    let y_specs = table.expand_columns(&y_specs)?;

    let bins = args.bins.unwrap_or(10);
    let bin_method = match &args.bin_method {
        Some(s) => Some(BinMethod::parse(s).ok_or_else(|| {
            format!("unknown --bin-method '{s}' (expected sturges, scott, or fd)")
        })?),
        None => None,
    };

    let (na_set, na_strat, clamp) = args.na.resolve()?;
    // Drop/zero/error missing values before binning.
    let clean_values = |vals: Vec<Option<f64>>| -> Result<Vec<f64>, String> {
        let n = vals.len();
        let keep = apply_na(na_strat, &[&vals], n)?;
        Ok(keep.iter().map(|&i| vals[i].unwrap_or(0.0)).collect())
    };

    // Apply the single-series modes shared by every code path.
    let apply_modes = |mut h: Histogram| -> Histogram {
        if let Some(m) = bin_method {
            h = h.with_bin_method(m);
        }
        if args.step {
            h = h.with_step(true);
        }
        if args.cumulative {
            h = h.with_cumulative(true);
        }
        if args.normalize {
            h = h.with_normalize();
        }
        h
    };

    // Multi-column mode: stacked (one Histogram with groups) or overlaid (separate plots).
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

        let hists: Vec<Histogram> = if args.stacked {
            // One histogram: first column is the primary series, the rest are stacked groups.
            // Full-opacity palette colours (no alpha) since stacked bars do not overlap.
            let mut iter = y_specs.iter().enumerate().zip(all_values);
            let ((_, first_col), first_vals) = iter.next().unwrap();
            let mut h = Histogram::new()
                .with_data(first_vals)
                .with_bins(bins)
                .with_range((min, max))
                .with_color(pal[0].to_string())
                .with_stacked(true);
            if args.legend {
                h = h.with_legend(table.col_display_name(first_col));
            }
            for ((i, col), values) in iter {
                let label = args.legend.then(|| table.col_display_name(col));
                h = h.with_group(values, pal[i].to_string(), label);
            }
            vec![apply_modes(h)]
        } else {
            y_specs
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
                    h = apply_modes(h);
                    if args.legend {
                        h = h.with_legend(table.col_display_name(col));
                    }
                    h
                })
                .collect()
        };

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

    // Read values (and, if requested, a parallel weight column) with a joint NA pass
    // so weights stay aligned to their samples after row drops.
    let (values, weights): (Vec<f64>, Option<Vec<f64>>) = if let Some(wcol) = &args.weight_col {
        let vals = table.col_f64_opt(&value_col, &na_set, clamp)?;
        let wts = table.col_f64_opt(wcol, &na_set, clamp)?;
        let keep = apply_na(na_strat, &[&vals, &wts], vals.len())?;
        let values: Vec<f64> = keep.iter().map(|&i| vals[i].unwrap_or(0.0)).collect();
        let weights: Vec<f64> = keep.iter().map(|&i| wts[i].unwrap_or(0.0)).collect();
        (values, Some(weights))
    } else {
        (
            clean_values(table.col_f64_opt(&value_col, &na_set, clamp)?)?,
            None,
        )
    };
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
    if let Some(w) = weights {
        plot = plot.with_weights(w);
    }
    plot = apply_modes(plot);

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
