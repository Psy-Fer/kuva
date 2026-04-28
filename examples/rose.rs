//! Nightingale rose chart documentation examples.
use kuva::plot::RosePlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use std::fs;

const OUT: &str = "docs/src/assets/rose";

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all(OUT).unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

fn main() {
    // Basic rose — monthly incident counts
    let months = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
    let values = [42.0_f64, 38.0, 55.0, 71.0, 88.0, 95.0, 90.0, 82.0, 65.0, 50.0, 44.0, 48.0];

    let mut plot = RosePlot::new().with_x_labels(months);
    for (label, &value) in months.iter().zip(values.iter()) {
        plot = plot.with_slice(*label, value);
    }
    let plot = plot.with_show_values(true);

    let plots = vec![Plot::Rose(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Monthly Incidents")
        .with_width(520.0)
        .with_height(480.0);
    write("basic", plots, layout);

    // Wind rose — bearing data with compass labels
    let bearings: Vec<f64> = (0..360_usize).step_by(10)
        .flat_map(|deg| {
            let d = deg as f64;
            let weight = (1.0 + (d.to_radians() * 2.0).cos()) * 3.0 + 1.0;
            vec![d; weight as usize]
        })
        .collect();

    let plot = RosePlot::new()
        .with_bearing_data(bearings, 16)
        .with_compass_labels();

    let plots = vec![Plot::Rose(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Wind Rose")
        .with_width(520.0)
        .with_height(480.0);
    write("wind", plots, layout);

    // Stacked rose — cause of death (Nightingale's original style)
    let labels = ["Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec","Jan","Feb","Mar"];
    let plot = RosePlot::new()
        .with_x_labels(labels)
        .with_stack("Preventable disease",
            [1.0_f64, 12.0, 11.0, 35.0, 31.0, 28.0, 26.0, 17.0, 7.0, 5.0, 5.0, 13.0])
        .with_stack("Wounds",
            [5.0_f64, 9.0, 6.0, 23.0, 25.0, 20.0, 14.0, 7.0, 4.0, 7.0, 9.0, 6.0])
        .with_stack("Other causes",
            [2.0_f64, 2.0, 2.0, 3.0, 3.0, 3.0, 3.0, 2.0, 2.0, 2.0, 2.0, 2.0])
        .with_show_labels(true);

    let plots = vec![Plot::Rose(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Causes of Mortality — Crimean War (1854–55)")
        .with_width(560.0)
        .with_height(520.0);
    write("nightingale", plots, layout);

    println!("Rose chart SVGs written to {OUT}/");
}
