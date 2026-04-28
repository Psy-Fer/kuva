//! Joint plot documentation examples.
use kuva::plot::{JointPlot, MarginalType};
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use std::fs;

const OUT: &str = "docs/src/assets/jointplot";

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all(OUT).unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

fn lcg(seed: &mut u64) -> f64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (*seed >> 33) as f64 / u32::MAX as f64
}

fn normal(seed: &mut u64) -> f64 {
    let u1 = lcg(seed).max(1e-10);
    let u2 = lcg(seed);
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

fn main() {
    let mut seed = 42u64;

    // Basic — single group with histogram marginals
    let n = 200;
    let xs: Vec<f64> = (0..n).map(|_| normal(&mut seed) * 1.2 + 2.0).collect();
    let ys: Vec<f64> = xs.iter().map(|&x| 0.8 * x + normal(&mut seed) * 0.8).collect();

    let plot = JointPlot::new()
        .with_xy(xs, ys)
        .with_marginal_type(MarginalType::Histogram);

    let plots = vec![Plot::Joint(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Joint Plot — histogram marginals")
        .with_x_label("X")
        .with_y_label("Y")
        .with_width(520.0)
        .with_height(460.0);
    write("basic", plots, layout);

    // KDE marginals
    let xs2: Vec<f64> = (0..n).map(|_| normal(&mut seed) * 1.0 + 1.5).collect();
    let ys2: Vec<f64> = xs2.iter().map(|&x| 0.7 * x + normal(&mut seed) * 1.0 + 0.5).collect();

    let plot = JointPlot::new()
        .with_group("samples", xs2, ys2, "coral")
        .with_marginal_type(MarginalType::Density);

    let plots = vec![Plot::Joint(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Joint Plot — KDE marginals")
        .with_x_label("X")
        .with_y_label("Y")
        .with_width(520.0)
        .with_height(460.0);
    write("density", plots, layout);

    // Two groups
    let xs_a: Vec<f64> = (0..120).map(|_| normal(&mut seed) * 0.9 + 1.0).collect();
    let ys_a: Vec<f64> = xs_a.iter().map(|&x| x * 0.8 + normal(&mut seed) * 0.5).collect();
    let xs_b: Vec<f64> = (0..120).map(|_| normal(&mut seed) * 0.9 + 4.0).collect();
    let ys_b: Vec<f64> = xs_b.iter().map(|&x| x * 0.6 + normal(&mut seed) * 0.7 + 0.5).collect();

    let plot = JointPlot::new()
        .with_group("Group A", xs_a, ys_a, "steelblue")
        .with_group("Group B", xs_b, ys_b, "tomato")
        .with_marginal_type(MarginalType::Density);

    let plots = vec![Plot::Joint(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Joint Plot — two groups")
        .with_x_label("X")
        .with_y_label("Y")
        .with_width(560.0)
        .with_height(480.0);
    write("two_groups", plots, layout);

    println!("Joint plot SVGs written to {OUT}/");
}
