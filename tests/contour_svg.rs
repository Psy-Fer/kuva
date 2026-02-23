use visus::plot::{ContourPlot, ColorMap};
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

use std::fs;

fn ensure_output_dir() {
    fs::create_dir_all("test_outputs").unwrap();
}

/// Simple 4x4 Gaussian-ish grid centred at (0,0)
fn gaussian_grid() -> (Vec<Vec<f64>>, Vec<f64>, Vec<f64>) {
    let coords: Vec<f64> = (-2..=2).map(|i| i as f64).collect(); // 5 values
    let n = coords.len();
    let mut z = vec![vec![0.0f64; n]; n];
    for (r, &y) in coords.iter().enumerate() {
        for (c, &x) in coords.iter().enumerate() {
            z[r][c] = (-(x * x + y * y) / 2.0).exp();
        }
    }
    (z, coords.clone(), coords)
}

#[test]
fn contour_grid_basic() {
    ensure_output_dir();

    let (z, xs, ys) = gaussian_grid();
    let cp = ContourPlot::new()
        .with_grid(z, xs, ys)
        .with_n_levels(6);

    let plots = vec![Plot::Contour(cp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Contour – grid, lines only");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    fs::write("test_outputs/contour_grid_basic.svg", &svg).unwrap();

    assert!(svg.contains("<path"), "Expected <path elements for iso-lines");
}

#[test]
fn contour_filled() {
    ensure_output_dir();

    let (z, xs, ys) = gaussian_grid();
    let cp = ContourPlot::new()
        .with_grid(z, xs, ys)
        .with_n_levels(6)
        .with_filled();

    let plots = vec![Plot::Contour(cp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Contour – filled bands");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    fs::write("test_outputs/contour_filled.svg", &svg).unwrap();

    // Filled mode produces at least as many paths as lines-only (bands + iso-lines)
    let path_count = svg.matches("<path").count();
    assert!(path_count >= 6, "Expected at least 6 <path elements, got {}", path_count);
}

#[test]
fn contour_scatter() {
    ensure_output_dir();

    // Build scattered (x, y, z) from a simple cone function
    let mut pts = Vec::new();
    for i in -5i32..=5 {
        for j in -5i32..=5 {
            let x = i as f64;
            let y = j as f64;
            let z = 1.0 - (x * x + y * y).sqrt() / 7.0;
            pts.push((x, y, z));
        }
    }

    let cp = ContourPlot::new()
        .with_points(pts)
        .with_n_levels(5);

    let plots = vec![Plot::Contour(cp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Contour – scattered IDW input");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    fs::write("test_outputs/contour_scatter.svg", &svg).unwrap();

    assert!(svg.contains("<path"), "Expected <path elements for iso-lines");
}

#[test]
fn contour_explicit_levels() {
    ensure_output_dir();

    let (z, xs, ys) = gaussian_grid();
    let cp = ContourPlot::new()
        .with_grid(z, xs, ys)
        .with_levels(&[0.1, 0.3, 0.5, 0.7, 0.9]);

    let plots = vec![Plot::Contour(cp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Contour – explicit iso-levels");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    fs::write("test_outputs/contour_explicit_levels.svg", &svg).unwrap();

    assert!(svg.contains("<path"), "Expected <path elements");
}

#[test]
fn contour_colormap_legend() {
    ensure_output_dir();

    let (z, xs, ys) = gaussian_grid();
    let cp = ContourPlot::new()
        .with_grid(z, xs, ys)
        .with_n_levels(7)
        .with_filled()
        .with_colormap(ColorMap::Inferno)
        .with_legend("Density");

    let plots = vec![Plot::Contour(cp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Contour – Inferno colormap + colorbar");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    fs::write("test_outputs/contour_colormap_legend.svg", &svg).unwrap();

    assert!(svg.contains("<path"), "Expected <path elements");
    // Colorbar is rendered as rects
    assert!(svg.contains("<rect"), "Expected <rect elements for colorbar");
}
