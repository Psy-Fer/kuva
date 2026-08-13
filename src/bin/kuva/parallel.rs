use clap::Args;

use kuva::plot::ParallelPlot;
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{parse_cell_opt, ColSpec, DataTable, InputArgs, NaStrategy};
use crate::layout_args::{apply_axis_args, apply_base_args, AxisArgs, BaseArgs, NaArgs};
use crate::output::write_output;

/// Parallel coordinates plot — multi-dimensional comparison.
#[derive(Args, Debug)]
pub struct ParallelArgs {
    /// Numeric axis columns (required; 0-based index or header name).
    #[arg(long, required = true, num_args = 1..)]
    pub value_cols: Vec<ColSpec>,

    /// Group (color) column.
    #[arg(long)]
    pub group_col: Option<ColSpec>,

    /// Axis names; if absent, column headers or "Axis 0", "Axis 1", … are used.
    #[arg(long, num_args = 1..)]
    pub axis_names: Option<Vec<String>>,

    /// Disable per-axis normalisation (default: normalise each axis independently).
    #[arg(long)]
    pub no_normalize: bool,

    /// Draw smooth S-shaped bezier curves instead of straight polylines.
    #[arg(long)]
    pub curved: bool,

    /// Global polyline opacity.
    #[arg(long)]
    pub opacity: Option<f64>,

    /// Draw a bold group-mean line over individual polylines.
    #[arg(long)]
    pub show_mean: bool,

    /// Legend title.
    #[arg(long)]
    pub legend: Option<String>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
    #[command(flatten)]
    pub na: NaArgs,
}

pub fn run(args: ParallelArgs) -> Result<(), String> {
    let mut proj: Vec<ColSpec> = args.value_cols.to_vec();
    if let Some(ref c) = args.group_col {
        proj.push(c.clone());
    }
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.header_mode(),
        args.input.delimiter,
        &proj,
    )?;
    // Expand column ranges / globs against the parsed table (issue #109).
    let value_cols = table.expand_columns(&args.value_cols)?;

    // Resolve axis names: explicit > header > "Axis N"
    let axis_names: Vec<String> = if let Some(names) = args.axis_names {
        names
    } else if let Some(ref header) = table.header {
        value_cols
            .iter()
            .enumerate()
            .map(|(fallback_i, col)| match col {
                // Resolve first so an all-numeric column name (e.g. "2024") maps to its
                // header label instead of an out-of-range index miss (issue #109).
                ColSpec::Index(_) | ColSpec::ForcedIndex(_) => table
                    .resolve(col)
                    .ok()
                    .and_then(|idx| header.get(idx))
                    .cloned()
                    .unwrap_or_else(|| format!("Axis {fallback_i}")),
                ColSpec::Name(n) => n.clone(),
                // Ranges/globs are expanded to indices before this point.
                ColSpec::Range { .. } | ColSpec::Glob(_) => format!("Axis {fallback_i}"),
            })
            .collect()
    } else {
        (0..value_cols.len()).map(|i| format!("Axis {i}")).collect()
    };

    let pal = Palette::category10();

    let mut plot = ParallelPlot::new().with_axis_names(axis_names);

    if args.no_normalize {
        plot = plot.with_normalize(false);
    }
    if args.curved {
        plot = plot.with_curved(true);
    }
    if let Some(op) = args.opacity {
        plot = plot.with_opacity(op);
    }
    if args.show_mean {
        plot = plot.with_mean(true);
    }
    if let Some(legend) = args.legend {
        plot = plot.with_legend(legend);
    }

    let (na_set, na_strat, clamp) = args.na.resolve()?;
    // Build one row's axis values NA-aware. A parallel-coords line needs every axis, so a row with
    // any missing value is dropped as a whole (or zero-filled / errored, per --na-strategy).
    // Returns Ok(None) to skip the row (drop).
    let row_values = |t: &DataTable, row: &[String]| -> Result<Option<Vec<f64>>, String> {
        let mut vals = Vec::with_capacity(value_cols.len());
        let mut any_missing = false;
        for col in &value_cols {
            let idx = t.resolve(col)?;
            let s = row
                .get(idx)
                .ok_or_else(|| format!("no column at index {idx}"))?;
            match parse_cell_opt(s, &na_set, clamp)? {
                Some(v) => vals.push(v),
                None => {
                    any_missing = true;
                    vals.push(0.0);
                }
            }
        }
        if any_missing {
            match na_strat {
                NaStrategy::Drop => return Ok(None),
                NaStrategy::Zero => {} // missing already filled with 0.0
                NaStrategy::Error => {
                    return Err("missing value (use --na-strategy drop or zero)".to_string())
                }
            }
        }
        Ok(Some(vals))
    };

    let mut dropped = 0usize;
    if let Some(ref gc) = args.group_col {
        let groups = table.group_by(gc)?;
        let colors: Vec<String> = groups
            .iter()
            .enumerate()
            .map(|(i, _)| pal[i % pal.len()].to_string())
            .collect();
        plot = plot.with_group_colors(colors);
        for (name, subtable) in groups {
            for row in &subtable.rows {
                match row_values(&subtable, row)? {
                    Some(values) => plot = plot.with_row_group(name.clone(), values),
                    None => dropped += 1,
                }
            }
        }
    } else {
        for row in &table.rows {
            match row_values(&table, row)? {
                Some(values) => plot = plot.with_row(values),
                None => dropped += 1,
            }
        }
    }
    if dropped > 0 {
        eprintln!(
            "note: dropped {dropped} row(s) with missing or non-finite values \
             (use --na-strategy zero to keep them as 0, or error to fail)"
        );
    }

    #[cfg(feature = "emit_code")]
    if args.base.emit_code {
        print!(
            "{}",
            crate::emit_code::assemble(
                &["kuva::plot::ParallelPlot"],
                "Parallel",
                &[crate::emit_code::emit_parallel_plot(&plot)],
                &args.base,
                Some(&args.axis),
                None,
            )
        );
        return Ok(());
    }

    let plots = vec![Plot::Parallel(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
