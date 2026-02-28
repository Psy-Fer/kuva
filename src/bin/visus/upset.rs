use clap::Args;
use std::collections::HashMap;

use visus::plot::{UpSetPlot, UpSetSort};
use visus::render::layout::Layout;
use visus::render::plots::Plot;
use visus::render::render::render_multiple;

use crate::data::{DataTable, InputArgs};
use crate::layout_args::{BaseArgs, apply_base_args};
use crate::output::write_output;

/// UpSet intersection plot from binary set-membership columns.
#[derive(Args, Debug)]
pub struct UpSetArgs {
    /// Sort intersections: frequency (default), degree, natural.
    #[arg(long, default_value = "frequency")]
    pub sort: String,

    /// Show only this many intersections (largest first).
    #[arg(long)]
    pub max_visible: Option<usize>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
}

pub fn run(args: UpSetArgs) -> Result<(), String> {
    if args.base.terminal {
        eprintln!("UpSet plots are not yet supported in terminal mode.");
        return Ok(());
    }

    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    if table.rows.is_empty() {
        return Err("upset input has no data rows".into());
    }
    let ncols = table.rows[0].len();

    // Set names from header or generate defaults.
    let set_names: Vec<String> = if let Some(ref hdr) = table.header {
        hdr.clone()
    } else {
        (0..ncols).map(|i| format!("Set_{i}")).collect()
    };

    // Per-set sizes: count of 1s in each column.
    let set_sizes: Vec<usize> = (0..ncols).map(|col| {
        table.rows.iter().filter(|row| {
            row.get(col)
                .and_then(|v| v.trim().parse::<f64>().ok())
                .map(|x| x > 0.5)
                .unwrap_or(false)
        }).count()
    }).collect();

    // Group rows by bitmask to compute intersection sizes.
    let mut mask_counts: HashMap<u64, usize> = HashMap::new();
    for row in &table.rows {
        let mut mask: u64 = 0;
        for (i, cell) in row.iter().enumerate().take(ncols) {
            if cell.trim().parse::<f64>().map(|x| x > 0.5).unwrap_or(false) {
                mask |= 1u64 << i;
            }
        }
        if mask > 0 {
            *mask_counts.entry(mask).or_default() += 1;
        }
    }

    let intersections: Vec<(u64, usize)> = mask_counts.into_iter().collect();

    let sort = match args.sort.as_str() {
        "degree" => UpSetSort::ByDegree,
        "natural" => UpSetSort::Natural,
        _ => UpSetSort::ByFrequency,
    };

    let mut plot = UpSetPlot::new()
        .with_data(set_names, set_sizes, intersections)
        .with_sort(sort);

    if let Some(n) = args.max_visible {
        plot = plot.with_max_visible(n);
    }

    let plots = vec![Plot::UpSet(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
