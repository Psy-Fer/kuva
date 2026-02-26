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
mod heatmap;
mod hist2d;
mod contour;
mod dot;
mod upset;
mod chord;
mod sankey;
mod phylo;
mod synteny;

use clap::{CommandFactory, Parser, Subcommand};

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
    Heatmap(heatmap::HeatmapArgs),
    #[command(name = "hist2d")]
    Hist2d(hist2d::Hist2dArgs),
    Contour(contour::ContourArgs),
    Dot(dot::DotArgs),
    #[command(name = "upset")]
    UpSet(upset::UpSetArgs),
    Chord(chord::ChordArgs),
    Sankey(sankey::SankeyArgs),
    Phylo(phylo::PhyloArgs),
    Synteny(synteny::SyntenyArgs),
    #[command(hide = true, about = "Print the man page to stdout")]
    Man,
}

fn main() {
    let cli = Cli::parse();
    if let Commands::Man = cli.command {
        let mut stdout = std::io::stdout();
        clap_mangen::Man::new(Cli::command())
            .render(&mut stdout)
            .expect("man page generation failed");
        return;
    }
    let result = match cli.command {
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
        Commands::Heatmap(args) => heatmap::run(args),
        Commands::Hist2d(args) => hist2d::run(args),
        Commands::Contour(args) => contour::run(args),
        Commands::Dot(args) => dot::run(args),
        Commands::UpSet(args) => upset::run(args),
        Commands::Chord(args) => chord::run(args),
        Commands::Sankey(args) => sankey::run(args),
        Commands::Phylo(args) => phylo::run(args),
        Commands::Synteny(args) => synteny::run(args),
        Commands::Man => unreachable!(),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
