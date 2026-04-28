//! Hexbin plot documentation examples.
use kuva::plot::{HexbinPlot, ColorMap};
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use std::fs;

const OUT: &str = "docs/src/assets/hexbin";

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all(OUT).unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

fn main() {
    // Generate clustered data with simple deterministic noise
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let centers: &[(f64, f64)] = &[(1.0, 1.0), (4.0, 3.0), (2.0, 5.0)];
    for (i, &(cx, cy)) in centers.iter().enumerate() {
        let n = 120 + i * 40;
        for j in 0..n {
            let t = j as f64 * 0.1;
            let jf = j as f64;
            xs.push(cx + t.sin() * 0.6 + (jf * 7919.0).sin() * 0.4);
            ys.push(cy + t.cos() * 0.6 + (jf * 6271.0).sin() * 0.4);
        }
    }

    // Basic — 20 bins, Viridis
    let plot = HexbinPlot::new()
        .with_data(xs.clone(), ys.clone())
        .with_n_bins(20)
        .with_color_map(ColorMap::Viridis)
        .with_colorbar(true)
        .with_colorbar_label("count");

    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Hexbin Density — three clusters")
        .with_x_label("X")
        .with_y_label("Y")
        .with_width(560.0)
        .with_height(400.0);
    write("basic", plots, layout);

    // Coarse bins
    let plot = HexbinPlot::new()
        .with_data(xs.clone(), ys.clone())
        .with_n_bins(10)
        .with_color_map(ColorMap::Plasma)
        .with_colorbar(true);

    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Hexbin — 10 bins (coarse)")
        .with_x_label("X")
        .with_y_label("Y")
        .with_width(560.0)
        .with_height(400.0);
    write("bins_coarse", plots, layout);

    // Fine bins
    let plot = HexbinPlot::new()
        .with_data(xs, ys)
        .with_n_bins(40)
        .with_color_map(ColorMap::Inferno)
        .with_colorbar(true);

    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Hexbin — 40 bins (fine)")
        .with_x_label("X")
        .with_y_label("Y")
        .with_width(560.0)
        .with_height(400.0);
    write("bins_fine", plots, layout);

    println!("Hexbin SVGs written to {OUT}/");
}
