mod bar;
mod boxplot;
mod bump;
mod calendar;
mod candlestick;
mod chord;
mod contour;
mod data;
mod density;
#[cfg(feature = "doom")]
mod doom;
mod dot;
mod ecdf;
mod forest;
mod funnel;
mod gantt;
mod heatmap;
mod hexbin;
mod hist2d;
mod histogram;
mod horizon;
mod layout_args;
mod line;
mod lollipop;
mod manhattan;
mod mosaic;
mod network;
mod output;
mod parallel;
mod phylo;
mod pie;
mod polar;
mod pr;
mod pyramid;
mod qq;
mod quiver;
mod radar;
mod raincloud;
mod ridgeline;
mod roc;
mod rose;
mod sankey;
mod scatter;
mod scatter3d;
mod slope;
mod stacked_area;
mod streamgraph;
mod strip;
mod sunburst;
mod surface3d;
mod survival;
mod synteny;
mod ternary;
mod treemap;
mod upset;
mod venn;
mod violin;
mod volcano;
mod waffle;
mod waterfall;

use clap::{CommandFactory, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "kuva",
    about = "Scientific plotting from the command line",
    version
)]
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
    #[command(name = "density")]
    Density(density::DensityArgs),
    #[command(name = "ecdf")]
    Ecdf(ecdf::EcdfArgs),
    #[command(name = "qq")]
    QQ(qq::QQArgs),
    #[command(name = "streamgraph")]
    Streamgraph(streamgraph::StreamgraphArgs),
    #[command(name = "ridgeline")]
    Ridgeline(ridgeline::RidgelineArgs),
    Polar(polar::PolarArgs),
    Ternary(ternary::TernaryArgs),
    Forest(forest::ForestArgs),
    #[command(name = "scatter3d")]
    Scatter3D(scatter3d::Scatter3DArgs),
    #[command(name = "surface3d")]
    Surface3D(surface3d::Surface3DArgs),
    Network(network::NetworkArgs),
    Radar(radar::RadarArgs),
    Hexbin(hexbin::HexbinArgs),
    Treemap(treemap::TreemapArgs),
    /// Sunburst chart — radial hierarchy diagram.
    Sunburst(sunburst::SunburstArgs),
    /// Bump chart — rank of series across discrete time points.
    Bump(bump::BumpArgs),
    /// Funnel chart — attrition / conversion through ordered stages.
    Funnel(funnel::FunnelArgs),
    /// Nightingale rose / coxcomb chart — polar bar chart.
    Rose(rose::RoseArgs),
    /// Slope chart — paired before/after comparisons on two axes.
    Slope(slope::SlopeArgs),
    /// Lollipop chart — dot-and-stem alternative to bar charts.
    Lollipop(lollipop::LollipopArgs),
    /// Raincloud plot — half-violin + box + jittered raw points.
    Raincloud(raincloud::RaincloudArgs),
    /// Mosaic / Marimekko chart — two-way contingency table.
    Mosaic(mosaic::MosaicArgs),
    /// Waffle chart — proportional grid of filled squares.
    Waffle(waffle::WaffleArgs),
    /// Population pyramid — back-to-back horizontal bar chart.
    Pyramid(pyramid::PyramidArgs),
    /// ROC curve — receiver operating characteristic.
    #[command(name = "roc")]
    Roc(roc::RocArgs),
    /// Precision-recall curve.
    #[command(name = "pr")]
    Pr(pr::PrArgs),
    /// Kaplan-Meier survival curve.
    Survival(survival::SurvivalArgs),
    /// Horizon chart — stacked folded time-series.
    Horizon(horizon::HorizonArgs),
    /// Parallel coordinates plot — multivariate comparison.
    Parallel(parallel::ParallelArgs),
    /// Venn diagram — 2–4 set overlaps.
    Venn(venn::VennArgs),
    /// Calendar heatmap — GitHub-style contribution grid.
    Calendar(calendar::CalendarArgs),
    /// Gantt chart — horizontal task bars with groups, progress, and milestones.
    Gantt(gantt::GanttArgs),
    /// Quiver plot — 2-D vector field rendered as arrows.
    Quiver(quiver::QuiverArgs),
    #[cfg(feature = "doom")]
    /// Generate a self-contained DOOM SVG playable in any browser.
    Doom(doom::DoomArgs),
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
        Commands::Density(args) => density::run(args),
        Commands::Ecdf(args) => ecdf::run(args),
        Commands::QQ(args) => qq::run(args),
        Commands::Streamgraph(args) => streamgraph::run(args),
        Commands::Ridgeline(args) => ridgeline::run(args),
        Commands::Polar(args) => polar::run(args),
        Commands::Ternary(args) => ternary::run(args),
        Commands::Forest(args) => forest::run(args),
        Commands::Scatter3D(args) => scatter3d::run(args),
        Commands::Surface3D(args) => surface3d::run(args),
        Commands::Network(args) => network::run(args),
        Commands::Radar(args) => radar::run(args),
        Commands::Hexbin(args) => hexbin::run(args),
        Commands::Treemap(args) => treemap::run(args),
        Commands::Sunburst(args) => sunburst::run(args),
        Commands::Bump(args) => bump::run(args),
        Commands::Funnel(args) => funnel::run(args),
        Commands::Rose(args) => rose::run(args),
        Commands::Slope(args) => slope::run(args),
        Commands::Lollipop(args) => lollipop::run(args),
        Commands::Raincloud(args) => raincloud::run(args),
        Commands::Mosaic(args) => mosaic::run(args),
        Commands::Waffle(args) => waffle::run(args),
        Commands::Pyramid(args) => pyramid::run(args),
        Commands::Roc(args) => roc::run(args),
        Commands::Pr(args) => pr::run(args),
        Commands::Survival(args) => survival::run(args),
        Commands::Horizon(args) => horizon::run(args),
        Commands::Parallel(args) => parallel::run(args),
        Commands::Venn(args) => venn::run(args),
        Commands::Calendar(args) => calendar::run(args),
        Commands::Gantt(args) => gantt::run(args),
        Commands::Quiver(args) => quiver::run(args),
        #[cfg(feature = "doom")]
        Commands::Doom(args) => doom::run(args),
        Commands::Man => unreachable!(),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
