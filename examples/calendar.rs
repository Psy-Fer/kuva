//! Calendar heatmap documentation examples.
use kuva::plot::{CalendarPlot, ColorMap};
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use std::fs;

const OUT: &str = "docs/src/assets/calendar";

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all(OUT).unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

fn make_date(year: i32, month: u32, day: u32) -> String {
    format!("{year}-{month:02}-{day:02}")
}

fn gen_year(year: i32) -> Vec<(String, f64)> {
    let months = [31u32, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let months = if leap {
        let mut m = months;
        m[1] = 29;
        m
    } else {
        months
    };
    let mut data = Vec::new();
    for (mi, &days) in months.iter().enumerate() {
        let month = mi as u32 + 1;
        for day in 1..=days {
            if (month + day) % 5 == 0 { continue; }
            let val = ((month * 7 + day * 3) % 15 + 1) as f64;
            data.push((make_date(year, month, day), val));
        }
    }
    data
}

fn main() {
    // Basic — GitHub-style contribution graph for 2024
    let data = gen_year(2024);

    let plot = CalendarPlot::new()
        .with_data(data.clone())
        .with_year(2024)
        .with_color_map(ColorMap::Greens);

    let plots = vec![Plot::Calendar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("2024 Contributions")
        .with_width(860.0)
        .with_height(160.0);
    write("basic", plots, layout);

    // Two years stacked
    let mut data2 = data;
    data2.extend(gen_year(2023));

    let plot = CalendarPlot::new()
        .with_data(data2)
        .with_years([2023, 2024])
        .with_color_map(ColorMap::Blues);

    let plots = vec![Plot::Calendar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Commit Activity 2023 – 2024")
        .with_width(860.0)
        .with_height(280.0);
    write("two_years", plots, layout);

    println!("Calendar heatmap SVGs written to {OUT}/");
}
