use clap::Args;

use kuva::plot::calendar::{CalendarAgg, CalendarPlot};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_base_args, BaseArgs};
use crate::output::write_output;

/// GitHub-style calendar heatmap from date/value data.
#[derive(Args, Debug)]
pub struct CalendarArgs {
    /// Date column in YYYY-MM-DD format (0-based index or header name; default: 0).
    #[arg(long)]
    pub date_col: Option<ColSpec>,

    /// Value column (0-based index or header name; default: 1).
    #[arg(long)]
    pub value_col: Option<ColSpec>,

    /// Aggregation function for multiple values on the same day: count, sum, mean, max.
    #[arg(long)]
    pub agg: Option<String>,

    /// Display a single calendar year (auto-detected from data if not set).
    #[arg(long)]
    pub year: Option<i32>,

    /// Start date for a custom date range (YYYY-MM-DD); use with --end.
    #[arg(long)]
    pub start: Option<String>,

    /// End date for a custom date range (YYYY-MM-DD); use with --start.
    #[arg(long)]
    pub end: Option<String>,

    /// Hide the color legend.
    #[arg(long)]
    pub no_legend: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
}

pub fn run(args: CalendarArgs) -> Result<(), String> {
    let proj: Vec<ColSpec> = vec![
        args.date_col.clone().unwrap_or(ColSpec::Index(0)),
        args.value_col.clone().unwrap_or(ColSpec::Index(1)),
    ];
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;

    let date_col = args.date_col.unwrap_or(ColSpec::Index(0));
    let value_col = args.value_col.unwrap_or(ColSpec::Index(1));

    let dates = table.col_str(&date_col)?;
    let values = table.col_f64(&value_col)?;

    let data: Vec<(String, f64)> = dates.into_iter().zip(values).collect();

    let mut plot = CalendarPlot::new().with_data(data);

    if let Some(agg_str) = args.agg {
        let agg = match agg_str.to_ascii_lowercase().as_str() {
            "count" => CalendarAgg::Count,
            "sum" => CalendarAgg::Sum,
            "mean" => CalendarAgg::Mean,
            "max" => CalendarAgg::Max,
            other => {
                return Err(format!(
                    "unknown aggregation '{other}'; accepted: count, sum, mean, max"
                ))
            }
        };
        plot = plot.with_aggregation(agg);
    }

    if let Some(year) = args.year {
        plot = plot.with_year(year);
    }

    if let (Some(start), Some(end)) = (args.start, args.end) {
        plot = plot.with_date_range(start, end);
    }

    if args.no_legend {
        plot = plot.with_legend(false);
    }

    let plots = vec![Plot::Calendar(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
