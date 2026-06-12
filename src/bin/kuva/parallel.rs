use clap::Args;

use kuva::plot::ParallelPlot;
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_axis_args, apply_base_args, AxisArgs, BaseArgs};
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
}

pub fn run(args: ParallelArgs) -> Result<(), String> {
    let mut proj: Vec<ColSpec> = args.value_cols.to_vec();
    if let Some(ref c) = args.group_col {
        proj.push(c.clone());
    }
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;

    // Resolve axis names: explicit > header > "Axis N"
    let axis_names: Vec<String> = if let Some(names) = args.axis_names {
        names
    } else if let Some(ref header) = table.header {
        args.value_cols
            .iter()
            .enumerate()
            .map(|(fallback_i, col)| match col {
                ColSpec::Index(i) => header
                    .get(*i)
                    .cloned()
                    .unwrap_or_else(|| format!("Axis {fallback_i}")),
                ColSpec::Name(n) => n.clone(),
            })
            .collect()
    } else {
        (0..args.value_cols.len())
            .map(|i| format!("Axis {i}"))
            .collect()
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
                let values: Result<Vec<f64>, String> = args
                    .value_cols
                    .iter()
                    .map(|col| {
                        let idx = subtable.resolve(col)?;
                        row.get(idx)
                            .ok_or_else(|| format!("no column at index {idx}"))
                            .and_then(|s| {
                                s.parse::<f64>()
                                    .map_err(|_| format!("cannot parse '{s}' as a number"))
                            })
                    })
                    .collect();
                plot = plot.with_row_group(name.clone(), values?);
            }
        }
    } else {
        for row in &table.rows {
            let values: Result<Vec<f64>, String> = args
                .value_cols
                .iter()
                .map(|col| {
                    let idx = table.resolve(col)?;
                    row.get(idx)
                        .ok_or_else(|| format!("no column at index {idx}"))
                        .and_then(|s| {
                            s.parse::<f64>()
                                .map_err(|_| format!("cannot parse '{s}' as a number"))
                        })
                })
                .collect();
            plot = plot.with_row(values?);
        }
    }

    let plots = vec![Plot::Parallel(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
