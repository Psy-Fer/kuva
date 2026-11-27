use clap::Args;

use kuva::plot::ChordPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{DataTable, InputArgs};
use crate::layout_args::{apply_base_args, BaseArgs};
use crate::output::write_output;

/// Chord diagram from an N×N flow matrix.
#[derive(Args, Debug)]
pub struct ChordArgs {
    /// Gap between arcs in degrees (default: 2.0).
    #[arg(long)]
    pub gap: Option<f64>,

    /// Ribbon opacity 0.0–1.0 (default: 0.7).
    #[arg(long)]
    pub opacity: Option<f64>,

    /// Show a legend with this label.
    #[arg(long)]
    pub legend: Option<String>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
}

pub fn run(args: ChordArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.header_mode(),
        args.input.delimiter,
        &[],
    )?;

    if table.rows.is_empty() {
        return Err("chord input has no data rows".into());
    }

    // Node labels: from header (skipping the first "row-label" column) or row-first-column values.
    // The file format is: first column = row label, header = [ignored, col1, col2, ...].
    let labels: Vec<String> = if let Some(ref hdr) = table.header {
        hdr[1..].to_vec()
    } else {
        table.rows.iter().map(|r| r[0].clone()).collect()
    };

    // Parse N×N matrix from columns [1..].
    let matrix: Vec<Vec<f64>> = table
        .rows
        .iter()
        .enumerate()
        .map(|(r, row)| {
            row[1..]
                .iter()
                .enumerate()
                .map(|(c, cell)| {
                    cell.trim()
                        .parse::<f64>()
                        .map_err(|_| format!("row {r}, col {}: '{}' is not a number", c + 1, cell))
                })
                .collect::<Result<Vec<f64>, String>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut plot = ChordPlot::new().with_matrix(matrix).with_labels(labels);
    // One value column per data row, checked before anything is laid out.
    plot.check_square()
        .map_err(|e| format!("{e} (one value column per data row)"))?;

    if let Some(g) = args.gap {
        plot = plot.with_gap(g);
    }
    if let Some(op) = args.opacity {
        plot = plot.with_opacity(op);
    }
    if let Some(ref label) = args.legend {
        plot = plot.with_legend(label.clone());
    }

    #[cfg(feature = "emit_code")]
    if args.base.emit_code {
        print!(
            "{}",
            crate::emit_code::assemble(
                &["kuva::plot::ChordPlot"],
                "Chord",
                &[crate::emit_code::emit_chord_plot(&plot)],
                &args.base,
                None,
                None,
            )
        );
        return Ok(());
    }

    let plots = vec![Plot::Chord(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
