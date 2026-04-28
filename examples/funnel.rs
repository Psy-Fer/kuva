//! Funnel chart documentation examples.
use kuva::plot::{FunnelPlot, FunnelStage};
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use std::fs;

const OUT: &str = "docs/src/assets/funnel";

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all(OUT).unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

fn main() {
    // Basic sales funnel
    let plot = FunnelPlot::new()
        .with_stage("Impressions",    50_000.0)
        .with_stage("Website visits", 12_400.0)
        .with_stage("Sign-ups",        3_200.0)
        .with_stage("Free trials",     1_100.0)
        .with_stage("Paid customers",    420.0)
        .with_show_values(true)
        .with_show_percents(true);

    let plots = vec![Plot::Funnel(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Sales Funnel")
        .with_width(480.0)
        .with_height(380.0);
    write("basic", plots, layout);

    // Clinical trial funnel (mirror / diverging)
    let plot = FunnelPlot::new()
        .with_stage("Screened",  800.0)
        .with_stage("Eligible",  540.0)
        .with_stage("Enrolled",  400.0)
        .with_stage("Completed", 360.0)
        .with_mirror(vec![
            FunnelStage::new("Screened",  760.0),
            FunnelStage::new("Eligible",  490.0),
            FunnelStage::new("Enrolled",  380.0),
            FunnelStage::new("Completed", 350.0),
        ])
        .with_mirror_labels("Treatment", "Control")
        .with_show_values(true);

    let plots = vec![Plot::Funnel(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Clinical Trial Enrollment")
        .with_width(560.0)
        .with_height(360.0);
    write("mirror", plots, layout);

    println!("Funnel chart SVGs written to {OUT}/");
}
