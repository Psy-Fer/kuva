use clap::Args;

use kuva::plot::line::{LinePlot, LineStyle};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::render::palette::Palette;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, LogArgs, apply_base_args, apply_axis_args, apply_log_args};
use crate::output::write_output;

/// Line plot from two numeric columns.
#[derive(Args, Debug)]
pub struct LineArgs {
    /// X-axis column (0-based index or header name; default: 0).
    #[arg(long)]
    pub x: Option<ColSpec>,

    /// Y-axis column(s). A single name/index (default: 1) or a comma-separated list
    /// for multiple series: --y A,B,C plots each column as a separate colour-coded series.
    /// Mutually exclusive with --color-by when more than one column is given.
    #[arg(long, value_delimiter = ',')]
    pub y: Vec<ColSpec>,

    /// Colour-code data by group. Provide a column of categorical labels; each unique value
    /// becomes a separate colour-coded series using the active palette. Overrides --color.
    #[arg(long)]
    pub color_by: Option<ColSpec>,

    /// Line color (CSS string). Ignored when --color-by is used.
    #[arg(long)]
    pub color: Option<String>,

    /// Stroke width in pixels (default: 2.0).
    #[arg(long)]
    pub stroke_width: Option<f64>,

    /// Use a dashed line style.
    #[arg(long)]
    pub dashed: bool,

    /// Use a dotted line style.
    #[arg(long)]
    pub dotted: bool,

    /// Fill the area under the line.
    #[arg(long)]
    pub fill: bool,

    /// Show a legend for each series.
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
}

pub fn run(args: LineArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let x_col = args.x.unwrap_or(ColSpec::Index(0));
    let y_cols: Vec<ColSpec> = if args.y.is_empty() { vec![ColSpec::Index(1)] } else { args.y };
    let color = args.color.unwrap_or_else(|| "steelblue".to_string());
    let stroke_width = args.stroke_width.unwrap_or(2.0);
    let line_style = if args.dashed {
        LineStyle::Dashed
    } else if args.dotted {
        LineStyle::Dotted
    } else {
        LineStyle::Solid
    };
    let fill = args.fill;
    let legend = args.legend;

    let plots: Vec<Plot> = if let Some(color_by) = args.color_by {
        if y_cols.len() > 1 {
            return Err("--color-by and multiple --y columns are mutually exclusive. \
                        Use one or the other to create multiple series.".to_string());
        }
        let y_col = &y_cols[0];
        let groups = table.group_by(&color_by)?;
        let palette = Palette::category10();
        let colors: Vec<String> = (0..groups.len()).map(|i| palette[i].to_string()).collect();

        groups
            .into_iter()
            .zip(colors)
            .map(|((name, subtable), grp_color)| {
                let xs = subtable.col_f64(&x_col)?;
                let ys = subtable.col_f64(y_col)?;
                let mut data: Vec<(f64, f64)> = xs.into_iter().zip(ys).collect();
                data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

                let mut plot = LinePlot::new()
                    .with_data(data)
                    .with_color(&grp_color)
                    .with_stroke_width(stroke_width)
                    .with_line_style(line_style.clone());

                if fill { plot = plot.with_fill(); }
                if legend { plot = plot.with_legend(name); }

                Ok(Plot::Line(plot))
            })
            .collect::<Result<Vec<_>, String>>()?
    } else if y_cols.len() > 1 {
        // Multi-column mode: one series per y column, auto-colored by palette.
        let palette = Palette::category10();
        let xs = table.col_f64(&x_col)?;

        y_cols
            .iter()
            .enumerate()
            .map(|(i, y_col)| {
                let series_name = col_display_name(&table, y_col);
                let ys = table.col_f64(y_col)?;
                let mut data: Vec<(f64, f64)> = xs.iter().copied().zip(ys).collect();
                data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
                let grp_color = palette[i].to_string();

                let mut plot = LinePlot::new()
                    .with_data(data)
                    .with_color(&grp_color)
                    .with_stroke_width(stroke_width)
                    .with_line_style(line_style.clone());

                if fill { plot = plot.with_fill(); }
                if legend { plot = plot.with_legend(series_name); }

                Ok(Plot::Line(plot))
            })
            .collect::<Result<Vec<_>, String>>()?
    } else {
        let y_col = &y_cols[0];
        let xs = table.col_f64(&x_col)?;
        let ys = table.col_f64(y_col)?;
        let mut data: Vec<(f64, f64)> = xs.into_iter().zip(ys).collect();
        data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let mut plot = LinePlot::new()
            .with_data(data)
            .with_color(&color)
            .with_stroke_width(stroke_width)
            .with_line_style(line_style);

        if fill { plot = plot.with_fill(); }

        vec![Plot::Line(plot)]
    };

    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let layout = apply_log_args(layout, &args.log);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}

/// Return a human-readable name for a column: the header name when available,
/// or "col_N" for index-based specs with no header.
fn col_display_name(table: &DataTable, col: &ColSpec) -> String {
    match col {
        ColSpec::Name(n) => n.clone(),
        ColSpec::Index(i) => table
            .header
            .as_ref()
            .and_then(|h| h.get(*i))
            .cloned()
            .unwrap_or_else(|| format!("col_{i}")),
    }
}
