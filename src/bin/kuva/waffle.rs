use clap::Args;

use kuva::plot::waffle::CellShape;
use kuva::plot::WafflePlot;
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_base_args, BaseArgs};
use crate::output::write_output;

/// Waffle plot — grid of cells colored by categorical proportions.
#[derive(Args, Debug)]
pub struct WaffleArgs {
    /// Label column (0-based index or header name; default: 0).
    #[arg(long)]
    pub label_col: Option<ColSpec>,

    /// Value column (0-based index or header name; default: 1).
    #[arg(long)]
    pub value_col: Option<ColSpec>,

    /// Optional per-row color column (CSS color strings).
    #[arg(long)]
    pub color_col: Option<ColSpec>,

    /// Number of rows in the waffle grid.
    #[arg(long)]
    pub rows: Option<u32>,

    /// Number of columns in the waffle grid.
    #[arg(long)]
    pub cols: Option<u32>,

    /// Gap between cells in pixels.
    #[arg(long)]
    pub gap: Option<f64>,

    /// Cell shape: "square" (default) or "circle".
    #[arg(long)]
    pub shape: Option<String>,

    /// Show percentage labels on cells.
    #[arg(long)]
    pub show_percents: bool,

    /// Show count labels on cells.
    #[arg(long)]
    pub show_counts: bool,

    /// Legend title (enables legend display).
    #[arg(long)]
    pub legend: Option<String>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
}

pub fn run(args: WaffleArgs) -> Result<(), String> {
    let mut proj: Vec<ColSpec> = vec![
        args.label_col.clone().unwrap_or(ColSpec::Index(0)),
        args.value_col.clone().unwrap_or(ColSpec::Index(1)),
    ];
    if let Some(ref c) = args.color_col { proj.push(c.clone()); }
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;

    let label_col = args.label_col.unwrap_or(ColSpec::Index(0));
    let value_col = args.value_col.unwrap_or(ColSpec::Index(1));

    let labels = table.col_str(&label_col)?;
    let values = table.col_f64(&value_col)?;

    let n = labels.len();
    if values.len() != n {
        return Err(format!(
            "column length mismatch: label={}, value={}",
            n,
            values.len()
        ));
    }

    let colors = if let Some(ref cc) = args.color_col {
        table.col_str(cc)?
    } else {
        let palette = Palette::category10();
        (0..n)
            .map(|i| palette[i % palette.len()].to_string())
            .collect()
    };

    let mut plot = WafflePlot::new();

    if let Some(v) = args.rows {
        plot = plot.with_rows(v as usize);
    }
    if let Some(v) = args.cols {
        plot = plot.with_cols(v as usize);
    }
    if let Some(v) = args.gap {
        plot = plot.with_gap(v);
    }
    if let Some(ref shape_str) = args.shape {
        let shape = if shape_str.eq_ignore_ascii_case("circle") {
            CellShape::Circle
        } else {
            CellShape::Square
        };
        plot = plot.with_shape(shape);
    }
    if args.show_percents {
        plot = plot.with_show_percents();
    }
    if args.show_counts {
        plot = plot.with_show_counts();
    }
    if let Some(s) = args.legend {
        plot = plot.with_legend(s);
    }

    for ((label, value), color) in labels.iter().zip(values.iter()).zip(colors.iter()) {
        plot = plot.with_category(label.as_str(), *value, color.as_str());
    }

    let plots = vec![Plot::Waffle(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
