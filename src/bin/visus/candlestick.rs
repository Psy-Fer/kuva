use clap::Args;

use visus::plot::candlestick::CandlestickPlot;
use visus::render::layout::Layout;
use visus::render::plots::Plot;
use visus::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, apply_base_args, apply_axis_args};
use crate::output::write_output;

#[derive(Args, Debug)]
pub struct CandlestickArgs {
    /// Label column (0-based index or header name; default: 0).
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

    /// Candle body width as a fraction of the slot (default: 0.7).
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
    let open_col = args.open_col.unwrap_or(ColSpec::Index(1));
    let high_col = args.high_col.unwrap_or(ColSpec::Index(2));
    let low_col = args.low_col.unwrap_or(ColSpec::Index(3));
    let close_col = args.close_col.unwrap_or(ColSpec::Index(4));

    let labels = table.col_str(&label_col)?;
    let opens = table.col_f64(&open_col)?;
    let highs = table.col_f64(&high_col)?;
    let lows = table.col_f64(&low_col)?;
    let closes = table.col_f64(&close_col)?;

    let mut plot = CandlestickPlot::new();

    if let Some(w) = args.candle_width {
        plot = plot.with_candle_width(w);
    }
    if let Some(ref c) = args.color_up {
        plot = plot.with_color_up(c.clone());
    }
    if let Some(ref c) = args.color_down {
        plot = plot.with_color_down(c.clone());
    }
    if let Some(ref c) = args.color_doji {
        plot = plot.with_color_doji(c.clone());
    }

    for (((label, open), (high, low)), close) in labels.iter()
        .zip(opens.iter())
        .zip(highs.iter().zip(lows.iter()))
        .zip(closes.iter())
    {
        plot = plot.with_candle(label.clone(), *open, *high, *low, *close);
    }

    if let Some(ref vcol) = args.volume_col {
        let volumes = table.col_f64(vcol)?;
        plot = plot.with_volume(volumes);
    }

    if args.volume_panel {
        plot = plot.with_volume_panel();
    }

    let plots = vec![Plot::Candlestick(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
