use clap::Args;

use kuva::plot::scatter::{MarkerShape, ScatterPlot, TrendLine};
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{apply_na, ColSpec, DataTable, InputArgs};
use crate::layout_args::{
    apply_axis_args, apply_base_args, apply_log_args, date_axis_from_args, AxisArgs, BaseArgs,
    DateArgs, LogArgs, NaArgs,
};
use crate::output::write_output;

/// Scatter plot from two numeric columns.
#[derive(Args, Debug)]
pub struct ScatterArgs {
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

    /// Point color (CSS string). Ignored when --color-by is used.
    #[arg(long)]
    pub color: Option<String>,

    /// Point radius in pixels (default: 3.0).
    #[arg(long)]
    pub size: Option<f64>,

    /// Marker shape: circle (default), square, triangle, triangle-down, diamond,
    /// cross, plus, star, pentagon, hexagon.
    #[arg(long)]
    pub marker: Option<String>,

    /// Label each point with the values of this column (single-series mode only).
    #[arg(long)]
    pub label_col: Option<ColSpec>,

    /// Point-label placement: nudge (default), exact, or repel (force-directed).
    #[arg(long)]
    pub label_style: Option<String>,

    /// Overlay a linear trend line.
    #[arg(long)]
    pub trend: bool,

    /// Overlay a LOESS smoother (local regression) instead of a linear trend.
    #[arg(long)]
    pub loess: bool,

    /// LOESS span: fraction of points per local fit, 0.05–1.0 (default 0.5).
    /// Smaller = wigglier. Implies --loess.
    #[arg(long)]
    pub loess_span: Option<f64>,

    /// Annotate with the regression equation (requires --trend).
    #[arg(long)]
    pub equation: bool,

    /// Annotate with the Pearson R² value (requires --trend).
    #[arg(long)]
    pub correlation: bool,

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
    #[command(flatten)]
    pub date: DateArgs,
    #[command(flatten)]
    pub na: NaArgs,
}

pub fn run(args: ScatterArgs) -> Result<(), String> {
    let x_spec = args.x.clone().unwrap_or(ColSpec::Index(0));
    let y_specs: Vec<ColSpec> = if args.y.is_empty() {
        vec![ColSpec::Index(1)]
    } else {
        args.y.clone()
    };
    let mut proj: Vec<ColSpec> = std::iter::once(x_spec).chain(y_specs).collect();
    if let Some(ref c) = args.color_by {
        proj.push(c.clone());
    }
    if let Some(ref c) = args.label_col {
        proj.push(c.clone());
    }
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.header_mode(),
        args.input.delimiter,
        &proj,
    )?;

    let color = args.color.unwrap_or_else(|| "steelblue".to_string());
    let size = args.size.unwrap_or(3.0);
    let trend = args.trend;
    let equation = args.equation;
    let correlation = args.correlation;
    let legend = args.legend;
    let x_col = args.x.unwrap_or(ColSpec::Index(0));
    let y_cols: Vec<ColSpec> = if args.y.is_empty() {
        vec![ColSpec::Index(1)]
    } else {
        args.y
    };
    // Expand column ranges / globs against the parsed table (issue #109).
    let y_cols = table.expand_columns(&y_cols)?;
    let (na_set, na_strat, clamp) = args.na.resolve()?;
    // When --x-date-format is set, the X column holds date/time strings, not plain numbers.
    // Read X (and Y) as `Option<f64>` (NA-aware); date-parsed X has no missing handling here.
    let read_x_opt = |t: &DataTable, c: &ColSpec| -> Result<Vec<Option<f64>>, String> {
        match &args.date.x_date_format {
            Some(fmt) => Ok(t.col_date_f64(c, fmt)?.into_iter().map(Some).collect()),
            None => t.col_f64_opt(c, &na_set, clamp),
        }
    };
    // Build cleaned (x, y) pairs applying the missing-value strategy.
    let clean = |xs: Vec<Option<f64>>, ys: Vec<Option<f64>>| -> Result<Vec<(f64, f64)>, String> {
        let n = xs.len();
        let keep = apply_na(na_strat, &[&xs, &ys], n)?;
        Ok(keep
            .iter()
            .map(|&i| (xs[i].unwrap_or(0.0), ys[i].unwrap_or(0.0)))
            .collect())
    };

    if args.label_col.is_some() && (args.color_by.is_some() || y_cols.len() > 1) {
        return Err(
            "--label-col is only supported in single-series mode (no --color-by, a single --y)"
                .to_string(),
        );
    }

    let mut plots: Vec<ScatterPlot> = if let Some(color_by) = args.color_by {
        if y_cols.len() > 1 {
            return Err(
                "--color-by and multiple --y columns are mutually exclusive. \
                 Use one or the other to create multiple series."
                    .to_string(),
            );
        }
        let y_col = &y_cols[0];
        let groups = table.group_by(&color_by)?;
        let palette = Palette::category10();
        groups
            .into_iter()
            .enumerate()
            .map(|(i, (name, subtable))| {
                let data = clean(
                    read_x_opt(&subtable, &x_col)?,
                    subtable.col_f64_opt(y_col, &na_set, clamp)?,
                )?;
                let grp_color = palette[i].to_string();
                let mut plot = ScatterPlot::new()
                    .with_data(data)
                    .with_color(&grp_color)
                    .with_size(size)
                    .with_group_name(name.clone());
                if legend {
                    plot = plot.with_legend(name);
                }
                Ok(plot)
            })
            .collect::<Result<Vec<_>, String>>()?
    } else if y_cols.len() > 1 {
        // Multi-column mode: one series per y column, auto-colored by palette.
        let palette = Palette::category10();
        let xs = read_x_opt(&table, &x_col)?;
        y_cols
            .iter()
            .enumerate()
            .map(|(i, y_col)| {
                let series_name = col_display_name(&table, y_col);
                let data = clean(xs.clone(), table.col_f64_opt(y_col, &na_set, clamp)?)?;
                let grp_color = palette[i].to_string();
                let mut plot = ScatterPlot::new()
                    .with_data(data)
                    .with_color(&grp_color)
                    .with_size(size)
                    .with_group_name(series_name.clone());
                if legend {
                    plot = plot.with_legend(series_name);
                }
                Ok(plot)
            })
            .collect::<Result<Vec<_>, String>>()?
    } else {
        let y_col = &y_cols[0];
        // Compute the kept-row indices explicitly so point labels stay aligned to the
        // (x, y) pairs after missing-value rows are dropped.
        let xs = read_x_opt(&table, &x_col)?;
        let ys = table.col_f64_opt(y_col, &na_set, clamp)?;
        let keep = apply_na(na_strat, &[&xs, &ys], xs.len())?;
        let data: Vec<(f64, f64)> = keep
            .iter()
            .map(|&i| (xs[i].unwrap_or(0.0), ys[i].unwrap_or(0.0)))
            .collect();
        let mut plot = ScatterPlot::new()
            .with_data(data)
            .with_color(&color)
            .with_size(size);
        if let Some(ref lcol) = args.label_col {
            let all = table.col_str(lcol)?;
            let labels: Vec<String> = keep
                .iter()
                .map(|&i| all.get(i).cloned().unwrap_or_default())
                .collect();
            plot = plot.with_labels(labels);
        }
        vec![plot]
    };

    if let Some(ref m) = args.marker {
        let shape = MarkerShape::parse(m)
            .ok_or_else(|| format!("unknown --marker '{m}' (see --help for shapes)"))?;
        plots = plots.into_iter().map(|p| p.with_marker(shape)).collect();
    }

    if let Some(ref s) = args.label_style {
        let style = kuva::plot::LabelStyle::parse(s).ok_or_else(|| {
            format!("unknown --label-style '{s}' (expected nudge, exact, or repel)")
        })?;
        plots = plots
            .into_iter()
            .map(|p| p.with_label_style(style.clone()))
            .collect();
    }

    // LOESS takes precedence over --trend when both are given.
    if args.loess || args.loess_span.is_some() {
        let span = args.loess_span.unwrap_or(0.5);
        plots = plots.into_iter().map(|p| p.with_loess_span(span)).collect();
    } else if trend {
        plots = plots
            .into_iter()
            .map(|p| p.with_trend(TrendLine::Linear))
            .collect();
    }
    if equation {
        plots = plots.into_iter().map(|p| p.with_equation()).collect();
    }
    if correlation {
        plots = plots.into_iter().map(|p| p.with_correlation()).collect();
    }

    #[cfg(feature = "emit_code")]
    if args.base.emit_code {
        // Known fidelity gap: `--x-date-format` resolves to plain f64 timestamp
        // literals in the emitted point data (correct), but `assemble` has no
        // `DateArgs` parameter, so the emitted snippet won't include the
        // matching `.with_x_datetime(...)` call — the axis renders as a plain
        // number scale until the emitted code adds that manually.
        let exprs: Vec<String> = plots
            .iter()
            .map(crate::emit_code::emit_scatter_plot)
            .collect();
        print!(
            "{}",
            crate::emit_code::assemble(
                &["kuva::plot::ScatterPlot", "kuva::plot::scatter::TrendLine"],
                "Scatter",
                &exprs,
                &args.base,
                Some(&args.axis),
                Some(&args.log),
            )
        );
        return Ok(());
    }

    let plots: Vec<Plot> = plots.into_iter().map(Plot::Scatter).collect();
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let layout = apply_log_args(layout, &args.log);
    let layout = if let Some(ref fmt) = args.date.x_date_format {
        let xs = table.col_date_f64(&x_col, fmt)?;
        layout.with_x_datetime(date_axis_from_args(&args.date, &xs))
    } else {
        layout
    };
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}

/// Return a human-readable name for a column: the header name when available,
/// or "col_N" for index-based specs with no header.
fn col_display_name(table: &DataTable, col: &ColSpec) -> String {
    // Delegate to the shared resolver so an all-numeric column name (e.g. "2024")
    // shows its header label rather than "col_2024" (issue #109).
    table.col_display_name(col)
}
