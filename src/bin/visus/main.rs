mod data;
mod layout_args;
mod output;
mod scatter;
mod line;
mod bar;
mod histogram;
mod boxplot;
mod violin;
mod pie;
mod strip;
mod waterfall;
mod stacked_area;
mod volcano;
mod manhattan;
mod candlestick;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "visus", about = "Scientific plotting from the command line")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Scatter(scatter::ScatterArgs),
    Line(line::LineArgs),
    Bar(bar::BarArgs),
    Histogram(histogram::HistogramArgs),
    #[command(name = "box")]
    Boxplot(boxplot::BoxArgs),
    Violin(violin::ViolinArgs),
    Pie(pie::PieArgs),
    Strip(strip::StripArgs),
    Waterfall(waterfall::WaterfallArgs),
    #[command(name = "stacked-area")]
    StackedArea(stacked_area::StackedAreaArgs),
    Volcano(volcano::VolcanoArgs),
    Manhattan(manhattan::ManhattanArgs),
    Candlestick(candlestick::CandlestickArgs),
}

fn main() {
    let result = match Cli::parse().command {
        Commands::Scatter(args) => scatter::run(args),
        Commands::Line(args) => line::run(args),
        Commands::Bar(args) => bar::run(args),
        Commands::Histogram(args) => histogram::run(args),
        Commands::Boxplot(args) => boxplot::run(args),
        Commands::Violin(args) => violin::run(args),
        Commands::Pie(args) => pie::run(args),
        Commands::Strip(args) => strip::run(args),
        Commands::Waterfall(args) => waterfall::run(args),
        Commands::StackedArea(args) => stacked_area::run(args),
        Commands::Volcano(args) => volcano::run(args),
        Commands::Manhattan(args) => manhattan::run(args),
        Commands::Candlestick(args) => candlestick::run(args),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
