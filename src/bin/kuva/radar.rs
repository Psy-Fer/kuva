use clap::Args;

use kuva::plot::radar::RadarPlot;
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_base_args, BaseArgs};
use crate::output::write_output;

/// Radar / spider chart — multivariate data on radial axes.
#[derive(Args, Debug)]
pub struct RadarArgs {
    /// Columns to use as axes (names or 0-based indices); at least 3 required.
    #[arg(long, required = true, num_args = 1..)]
    pub axes: Vec<ColSpec>,

    /// Optional column for series labels / legend (one series per row).
    #[arg(long)]
    pub label_col: Option<ColSpec>,

    /// Group rows by this column; one polygon per unique value.
    #[arg(long)]
    pub color_by: Option<ColSpec>,

    /// Fill each polygon.
    #[arg(long)]
    pub filled: bool,

    /// Fill opacity when --filled is set (default: 0.25).
    #[arg(long, default_value_t = 0.25)]
    pub opacity: f64,

    /// Shared minimum value (default: 0 or data min).
    #[arg(long)]
    pub min: Option<f64>,

    /// Shared maximum value (default: data max).
    #[arg(long)]
    pub max: Option<f64>,

    /// Number of concentric grid rings (default: 5).
    #[arg(long, default_value_t = 5)]
    pub grid_lines: usize,

    /// Normalise each axis independently to [0, 1].
    #[arg(long)]
    pub normalize: bool,

    /// Draw dots at polygon vertices (radius in px).
    #[arg(long)]
    pub dot_size: Option<f64>,

    /// Show a legend.
    #[arg(long)]
    pub legend: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
}

pub fn run(args: RadarArgs) -> Result<(), String> {
    let mut proj: Vec<ColSpec> = args.axes.to_vec();
    if let Some(ref c) = args.label_col { proj.push(c.clone()); }
    if let Some(ref c) = args.color_by { proj.push(c.clone()); }
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;

    // Axis names from column headers (or indices as strings).
    let axis_names: Vec<String> = args
        .axes
        .iter()
        .map(|cs| match cs {
            crate::data::ColSpec::Name(n) => n.clone(),
            crate::data::ColSpec::Index(i) => format!("axis{}", i),
        })
        .collect();

    if axis_names.len() < 3 {
        return Err("--axes requires at least 3 columns".to_string());
    }

    let pal = Palette::category10();
    let mut plot = RadarPlot::new(axis_names.clone())
        .with_filled(args.filled)
        .with_opacity(args.opacity)
        .with_grid_lines(args.grid_lines)
        .with_normalize(args.normalize);

    if let (Some(lo), Some(hi)) = (args.min, args.max) {
        plot = plot.with_range(lo, hi);
    } else if let Some(hi) = args.max {
        plot = plot.with_range(0.0, hi);
    }

    if let Some(r) = args.dot_size {
        plot = plot.with_dot_size(r);
    }

    if args.legend {
        plot = plot.with_legend(true);
    }

    // Group by column → one series per unique value (mean of numeric cols per group).
    if let Some(ref grp_col) = args.color_by {
        let groups = table.group_by(grp_col)?;
        for (gi, (group_name, rows)) in groups.iter().enumerate() {
            let color = pal[gi % pal.len()].to_string();
            let mut vals: Vec<f64> = Vec::with_capacity(args.axes.len());
            for cs in &args.axes {
                let col_vals = rows.col_f64(cs)?;
                let mean = col_vals.iter().sum::<f64>() / col_vals.len().max(1) as f64;
                vals.push(mean);
            }
            plot = plot.with_series_color(vals, group_name.as_str(), color.as_str());
        }
    } else {
        // Each row is one series.
        let n_rows = table.rows.len();
        for row in 0..n_rows {
            let mut vals: Vec<f64> = Vec::with_capacity(args.axes.len());
            for cs in &args.axes {
                let col = table.col_f64(cs)?;
                vals.push(*col.get(row).unwrap_or(&0.0));
            }
            let label = if let Some(ref lc) = args.label_col {
                let lcol = table.col_str(lc)?;
                lcol.get(row).cloned()
            } else {
                None
            };
            let color = pal[row % pal.len()].to_string();
            if let Some(lbl) = label {
                plot = plot.with_series_color(vals, lbl.as_str(), color.as_str());
            } else {
                plot = plot.with_series(vals);
            }
        }
    }

    let plots = vec![Plot::Radar(plot)];
    let mut layout = Layout::auto_from_plots(&plots);
    layout = apply_base_args(layout, &args.base);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
