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
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
