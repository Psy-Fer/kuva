use std::collections::HashMap;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_base_args, BaseArgs};
use crate::output::write_output;
use clap::Args;
use kuva::plot::rose::{RoseEncoding, RoseMode, RosePlot};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

/// Nightingale rose / coxcomb chart from a tabular file.
#[derive(Args, Debug)]
pub struct RoseArgs {
    /// Sector label column (name or 0-based index; default: 0).
    #[arg(long)]
    pub label: Option<ColSpec>,

    /// Value column (name or 0-based index; default: 1).
    #[arg(long)]
    pub value: Option<ColSpec>,

    /// Group / series column — enables multi-series stacked or grouped mode.
    #[arg(long)]
    pub group_by: Option<ColSpec>,

    /// Multi-series mode: `stacked` (default) or `grouped`.
    #[arg(long, default_value = "stacked")]
    pub mode: String,

    /// Radius encoding: `area` (default, perceptually accurate) or `radius`.
    #[arg(long, default_value = "area")]
    pub encoding: String,

    /// Inner radius fraction 0-1 for donut style (default: 0).
    #[arg(long)]
    pub inner_radius: Option<f64>,

    /// Angular gap between sectors in degrees (default: 1).
    #[arg(long)]
    pub gap: Option<f64>,

    /// Start angle in degrees clockwise from north (default: 0).
    #[arg(long)]
    pub start_angle: Option<f64>,

    /// Go counterclockwise instead of clockwise.
    #[arg(long)]
    pub no_clockwise: bool,

    /// Hide concentric grid rings.
    #[arg(long)]
    pub no_grid: bool,

    /// Number of concentric grid rings (default: 4).
    #[arg(long)]
    pub grid_lines: Option<usize>,

    /// Hide sector labels.
    #[arg(long)]
    pub no_labels: bool,

    /// Show value labels at tip of each sector.
    #[arg(long)]
    pub show_values: bool,

    /// Replace sector labels with compass directions (N/NE/E/…).
    #[arg(long)]
    pub compass: bool,

    /// Show a legend with this label (for multi-series plots).
    #[arg(long)]
    pub legend: Option<String>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
}

pub fn run(args: RoseArgs) -> Result<(), String> {
    let mut proj: Vec<ColSpec> = vec![
        args.label.clone().unwrap_or(ColSpec::Index(0)),
        args.value.clone().unwrap_or(ColSpec::Index(1)),
    ];
    if let Some(ref c) = args.group_by {
        proj.push(c.clone());
    }
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.header_mode(),
        args.input.delimiter,
        &proj,
    )?;

    let label_col = args.label.unwrap_or(ColSpec::Index(0));
    let value_col = args.value.unwrap_or(ColSpec::Index(1));

    let encoding = match args.encoding.as_str() {
        "radius" | "r" => RoseEncoding::Radius,
        _ => RoseEncoding::Area,
    };
    let mode = match args.mode.as_str() {
        "grouped" | "group" => RoseMode::Grouped,
        _ => RoseMode::Stacked,
    };

    let mut plot = RosePlot::new()
        .with_encoding(encoding)
        .with_mode(mode.clone())
        .with_clockwise(!args.no_clockwise)
        .with_grid(!args.no_grid)
        .with_show_labels(!args.no_labels)
        .with_show_values(args.show_values);

    if let Some(ir) = args.inner_radius {
        plot = plot.with_inner_radius(ir);
    }
    if let Some(g) = args.gap {
        plot = plot.with_gap(g);
    }
    if let Some(sa) = args.start_angle {
        plot = plot.with_start_angle(sa);
    }
    if let Some(gl) = args.grid_lines {
        plot = plot.with_grid_lines(gl);
    }
    if let Some(ref lbl) = args.legend {
        plot = plot.with_legend(lbl.clone());
    }

    if let Some(ref gcol) = args.group_by {
        // Multi-series: group by (label, group) columns
        let labels_col = table.col_str(&label_col)?;
        let values_col = table.col_f64(&value_col)?;
        let groups_col = table.col_str(gcol)?;

        // Ordered unique labels and groups, then the sum for each (group,
        // label) in one pass over the rows. Starting from -0.0 and adding in
        // row order gives the same result as `Iterator::sum`.
        let mut label_idx: HashMap<&str, usize> = HashMap::new();
        let mut ordered_labels: Vec<String> = vec![];
        for l in &labels_col {
            label_idx.entry(l).or_insert_with(|| {
                ordered_labels.push(l.clone());
                ordered_labels.len() - 1
            });
        }
        let mut group_idx: HashMap<&str, usize> = HashMap::new();
        let mut ordered_groups: Vec<String> = vec![];
        for g in &groups_col {
            group_idx.entry(g).or_insert_with(|| {
                ordered_groups.push(g.clone());
                ordered_groups.len() - 1
            });
        }
        let mut sums = vec![vec![-0.0_f64; ordered_labels.len()]; ordered_groups.len()];
        for ((l, g), v) in labels_col.iter().zip(&groups_col).zip(&values_col) {
            sums[group_idx[g.as_str()]][label_idx[l.as_str()]] += *v;
        }

        plot = plot.with_x_labels(ordered_labels.iter().cloned());

        for (grp, vals) in ordered_groups.iter().zip(sums) {
            plot = match mode {
                RoseMode::Grouped => plot.with_group(grp.clone(), vals),
                RoseMode::Stacked => plot.with_stack(grp.clone(), vals),
            };
        }
    } else {
        // Simple single-series
        let labels_col = table.col_str(&label_col)?;
        let values_col = table.col_f64(&value_col)?;
        for (label, value) in labels_col.into_iter().zip(values_col) {
            plot = plot.with_slice(label, value);
        }
    }

    if args.compass {
        plot = plot.with_compass_labels();
    }

    #[cfg(feature = "emit_code")]
    if args.base.emit_code {
        print!(
            "{}",
            crate::emit_code::assemble(
                &[
                    "kuva::plot::RosePlot",
                    "kuva::plot::RoseMode",
                    "kuva::plot::RoseEncoding",
                ],
                "Rose",
                &[crate::emit_code::emit_rose_plot(&plot)],
                &args.base,
                None,
                None,
            )
        );
        return Ok(());
    }

    let plots = vec![Plot::Rose(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
