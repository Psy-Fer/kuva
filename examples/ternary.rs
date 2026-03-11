use kuva::plot::ternary::TernaryPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::backend::svg::SvgBackend;

fn main() {
    // Three groups clustering near each vertex
    let mut plot = TernaryPlot::new()
        .with_corner_labels("A", "B", "C")
        .with_grid_lines(5)
        .with_legend(true);

    // Group A: high A component
    for (a, b, c) in [
        (0.75, 0.15, 0.10),
        (0.80, 0.12, 0.08),
        (0.70, 0.18, 0.12),
        (0.82, 0.10, 0.08),
        (0.68, 0.20, 0.12),
    ] {
        plot = plot.with_point_group(a, b, c, "A-rich");
    }

    // Group B: high B component
    for (a, b, c) in [
        (0.12, 0.75, 0.13),
        (0.10, 0.80, 0.10),
        (0.15, 0.72, 0.13),
        (0.08, 0.82, 0.10),
        (0.13, 0.70, 0.17),
    ] {
        plot = plot.with_point_group(a, b, c, "B-rich");
    }

    // Group C: high C component
    for (a, b, c) in [
        (0.10, 0.12, 0.78),
        (0.08, 0.10, 0.82),
        (0.13, 0.15, 0.72),
        (0.09, 0.09, 0.82),
        (0.12, 0.18, 0.70),
    ] {
        plot = plot.with_point_group(a, b, c, "C-rich");
    }

    let plots = vec![Plot::Ternary(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Ternary Plot");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::create_dir_all("docs/src/assets/ternary").unwrap();
    std::fs::write("docs/src/assets/ternary/basic.svg", svg).unwrap();
    println!("Written docs/src/assets/ternary/basic.svg");
}
