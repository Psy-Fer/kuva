use clap::Args;
use std::collections::HashMap;
use std::path::PathBuf;

use kuva::plot::{SyntenyPlot, Strand};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{DataTable, InputArgs};
use crate::layout_args::{BaseArgs, apply_base_args};
use crate::output::write_output;

/// Genome synteny ribbons from sequence and block files.
#[derive(Args, Debug)]
pub struct SyntenyArgs {
    /// Blocks TSV file: seq1, start1, end1, seq2, start2, end2, strand.
    #[arg(long, required = true)]
    pub blocks_file: PathBuf,

    /// Bar height in pixels (default: 18.0).
    #[arg(long)]
    pub bar_height: Option<f64>,

    /// Block opacity 0.0–1.0 (default: 0.65).
    #[arg(long)]
    pub opacity: Option<f64>,

    /// Scale bar widths proportionally to sequence length
    /// (default: each bar fills the full width regardless of length).
    #[arg(long)]
    pub proportional: bool,

    /// Show a legend with this label.
    #[arg(long)]
    pub legend: Option<String>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
}

pub fn run(args: SyntenyArgs) -> Result<(), String> {
    // Primary input file: sequences TSV (name, length).
    let seqs_table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    // Blocks file.
    let blocks_table = DataTable::parse(
        Some(args.blocks_file.as_path()),
        false,
        args.input.delimiter,
    )?;

    // Parse sequences: (name, length as f64) and build name→index map.
    let mut name_to_idx: HashMap<String, usize> = HashMap::new();
    let sequences: Vec<(&str, f64)> = seqs_table.rows.iter().enumerate().map(|(r, row)| {
        if row.len() < 2 {
            return Err(format!("sequences row {r}: need at least 2 columns (name, length)"));
        }
        let len = row[1].trim().parse::<f64>().map_err(|_| {
            format!("sequences row {r}: '{}' is not a valid length", row[1])
        })?;
        Ok((row[0].as_str(), len))
    }).collect::<Result<Vec<_>, _>>()?;

    for (i, row) in seqs_table.rows.iter().enumerate() {
        name_to_idx.insert(row[0].clone(), i);
    }

    let mut plot = SyntenyPlot::new().with_sequences(sequences);

    // Parse blocks: seq1, start1, end1, seq2, start2, end2, strand.
    for (r, row) in blocks_table.rows.iter().enumerate() {
        if row.len() < 7 {
            return Err(format!("blocks row {r}: need 7 columns (seq1, start1, end1, seq2, start2, end2, strand)"));
        }
        let seq1_name = &row[0];
        let s1 = row[1].trim().parse::<f64>().map_err(|_| format!("blocks row {r}: start1 not a number"))?;
        let e1 = row[2].trim().parse::<f64>().map_err(|_| format!("blocks row {r}: end1 not a number"))?;
        let seq2_name = &row[3];
        let s2 = row[4].trim().parse::<f64>().map_err(|_| format!("blocks row {r}: start2 not a number"))?;
        let e2 = row[5].trim().parse::<f64>().map_err(|_| format!("blocks row {r}: end2 not a number"))?;
        let strand = if row[6].trim() == "-" { Strand::Reverse } else { Strand::Forward };

        let i1 = *name_to_idx.get(seq1_name).ok_or_else(|| {
            format!("blocks row {r}: unknown sequence '{seq1_name}'")
        })?;
        let i2 = *name_to_idx.get(seq2_name).ok_or_else(|| {
            format!("blocks row {r}: unknown sequence '{seq2_name}'")
        })?;

        plot = match strand {
            Strand::Forward => plot.with_block(i1, s1, e1, i2, s2, e2),
            Strand::Reverse => plot.with_inv_block(i1, s1, e1, i2, s2, e2),
        };
    }

    if let Some(h) = args.bar_height {
        plot = plot.with_bar_height(h);
    }
    if let Some(op) = args.opacity {
        plot = plot.with_opacity(op);
    }
    if args.proportional {
        plot = plot.with_shared_scale();
    }
    if let Some(ref label) = args.legend {
        plot = plot.with_legend(label.clone());
    }

    let plots = vec![Plot::Synteny(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
