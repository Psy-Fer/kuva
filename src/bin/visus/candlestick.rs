use clap::Args;
use chrono::NaiveDate;

use visus::plot::candlestick::CandlestickPlot;
use visus::render::layout::Layout;
use visus::render::plots::Plot;
use visus::render::render::render_multiple;
use visus::DateTimeAxis;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, apply_base_args, apply_axis_args};
use crate::output::write_output;

/// Candlestick / OHLC chart from open, high, low, close columns.
#[derive(Args, Debug)]
pub struct CandlestickArgs {
    /// Label/date column (0-based index or header name; default: 0).
    /// Values parsed as YYYY-MM-DD; if all values parse successfully the chart
    /// uses a continuous date-range x-axis (ticks at month/week boundaries).
    #[arg(long)]
    pub label_col: Option<ColSpec>,

    /// Open price column (0-based index or header name; default: 1).
    #[arg(long)]
    pub open_col: Option<ColSpec>,

    /// High price column (0-based index or header name; default: 2).
    #[arg(long)]
    pub high_col: Option<ColSpec>,

    /// Low price column (0-based index or header name; default: 3).
    #[arg(long)]
    pub low_col: Option<ColSpec>,

    /// Close price column (0-based index or header name; default: 4).
    #[arg(long)]
    pub close_col: Option<ColSpec>,

    /// Volume column (optional).
    #[arg(long)]
    pub volume_col: Option<ColSpec>,

    /// Show a volume bar panel below the price chart.
    #[arg(long)]
    pub volume_panel: bool,

    /// Candle body width as a fraction of the slot between candles (default: 0.7).
    #[arg(long)]
    pub candle_width: Option<f64>,

    /// Color for bullish candles (close > open; default: "rgb(68,170,68)").
    #[arg(long)]
    pub color_up: Option<String>,

    /// Color for bearish candles (close < open; default: "rgb(204,68,68)").
    #[arg(long)]
    pub color_down: Option<String>,

    /// Color for doji candles (close == open; default: "#888888").
    #[arg(long)]
    pub color_doji: Option<String>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: CandlestickArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let label_col = args.label_col.unwrap_or(ColSpec::Index(0));
    let open_col  = args.open_col.unwrap_or(ColSpec::Index(1));
    let high_col  = args.high_col.unwrap_or(ColSpec::Index(2));
    let low_col   = args.low_col.unwrap_or(ColSpec::Index(3));
    let close_col = args.close_col.unwrap_or(ColSpec::Index(4));

    let labels = table.col_str(&label_col)?;
    let opens  = table.col_f64(&open_col)?;
    let highs  = table.col_f64(&high_col)?;
    let lows   = table.col_f64(&low_col)?;
    let closes = table.col_f64(&close_col)?;

    let volumes: Vec<Option<f64>> = if let Some(ref vcol) = args.volume_col {
        table.col_f64(vcol)?.into_iter().map(Some).collect()
    } else {
        vec![None; labels.len()]
    };

    // Try parsing every label as YYYY-MM-DD.
    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    let timestamps: Vec<Option<f64>> = labels.iter().map(|s| {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").ok().map(|d| {
            d.signed_duration_since(epoch).num_seconds() as f64
        })
    }).collect();
    let all_dates = timestamps.iter().all(|t| t.is_some());

    let mut plot = CandlestickPlot::new();
    if let Some(ref c) = args.color_up    { plot = plot.with_color_up(c.clone()); }
    if let Some(ref c) = args.color_down  { plot = plot.with_color_down(c.clone()); }
    if let Some(ref c) = args.color_doji  { plot = plot.with_color_doji(c.clone()); }

    let datetime_axis: Option<DateTimeAxis>;

    if all_dates {
        // Datetime mode: position candles at epoch-second x values and use a
        // date-range axis so only month/week boundaries are labelled.
        let mut rows: Vec<(f64, f64, f64, f64, f64, Option<f64>)> = timestamps.iter()
            .zip(opens.iter().zip(highs.iter().zip(lows.iter().zip(closes.iter()))))
            .zip(volumes.iter())
            .map(|((ts_opt, (o, (h, (l, c)))), vol)| {
                (ts_opt.unwrap(), *o, *h, *l, *c, *vol)
            })
            .collect();
        rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // candle_width is a fraction (0–1) of the slot pixel width; the renderer
        // converts it from data-unit average spacing to pixels internally.
        plot = plot.with_candle_width(args.candle_width.unwrap_or(0.7));

        for (ts, o, h, l, c, _) in &rows {
            plot = plot.with_candle_at(*ts, "", *o, *h, *l, *c);
        }

        if args.volume_col.is_some() {
            let vols: Vec<f64> = rows.iter().filter_map(|r| r.5).collect();
            if !vols.is_empty() { plot = plot.with_volume(vols); }
        }

        let min_ts = rows.first().map(|r| r.0).unwrap_or(0.0);
        let max_ts = rows.last().map(|r| r.0).unwrap_or(0.0);
        datetime_axis = Some(DateTimeAxis::auto(min_ts, max_ts));
    } else {
        // Categorical mode: labels used as-is, ticks rotated.
        if let Some(w) = args.candle_width { plot = plot.with_candle_width(w); }
        for (((label, open), (high, low)), close) in labels.iter()
            .zip(opens.iter())
            .zip(highs.iter().zip(lows.iter()))
            .zip(closes.iter())
        {
            plot = plot.with_candle(label.clone(), *open, *high, *low, *close);
        }

        if args.volume_col.is_some() {
            let vols: Vec<f64> = volumes.iter().filter_map(|v| *v).collect();
            if !vols.is_empty() { plot = plot.with_volume(vols); }
        }

        datetime_axis = None;
    }

    if args.volume_panel { plot = plot.with_volume_panel(); }

    let plots = vec![Plot::Candlestick(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let layout = if let Some(dt) = datetime_axis {
        layout.with_x_datetime(dt)
    } else {
        layout.with_x_tick_rotate(-45.0)
    };
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
