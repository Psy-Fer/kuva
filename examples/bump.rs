//! Bump chart documentation examples.
use kuva::plot::BumpPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use std::fs;

const OUT: &str = "docs/src/assets/bump";

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all(OUT).unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

fn main() {
    // Basic bump chart — programming language rankings
    // show_series_labels labels each line at both ends, making a separate legend redundant
    let plot = BumpPlot::new()
        .with_series("Python",     [1.0, 1.0, 1.0, 1.0, 1.0])
        .with_series("JavaScript", [3.0, 2.0, 2.0, 2.0, 2.0])
        .with_series("Java",       [2.0, 3.0, 3.0, 4.0, 3.0])
        .with_series("C/C++",      [4.0, 4.0, 4.0, 3.0, 4.0])
        .with_series("Rust",       [8.0, 7.0, 6.0, 5.0, 5.0])
        .with_series("Go",         [6.0, 6.0, 5.0, 6.0, 6.0])
        .with_x_labels(["2020", "2021", "2022", "2023", "2024"])
        .with_show_rank_labels(true)
        .with_show_series_labels(true)
        .with_legend(true);

    let plots = vec![Plot::Bump(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Programming Language Rankings")
        .with_x_label("Year")
        .with_y_label("Rank")
        .with_width(600.0)
        .with_height(380.0);
    write("basic", plots, layout);

    // Raw values (auto-ranked)
    let plot = BumpPlot::new()
        .with_raw_series("Team A", [120.0, 145.0, 138.0, 162.0, 180.0])
        .with_raw_series("Team B", [110.0, 130.0, 155.0, 148.0, 140.0])
        .with_raw_series("Team C", [90.0,  95.0,  112.0, 130.0, 175.0])
        .with_raw_series("Team D", [140.0, 135.0, 125.0, 115.0, 100.0])
        .with_x_labels(["Q1", "Q2", "Q3", "Q4", "Q5"])
        .with_show_rank_labels(true)
        .with_show_series_labels(true)
        .with_legend(false);

    let plots = vec![Plot::Bump(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Team Performance Rankings")
        .with_x_label("Quarter")
        .with_y_label("Rank")
        .with_width(600.0)
        .with_height(320.0);
    write("raw_values", plots, layout);

    println!("Bump chart SVGs written to {OUT}/");
}
