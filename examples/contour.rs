//! Contour plot documentation examples.
//!
//! Generates canonical SVG outputs used in the visus documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example contour
//! ```
//!
//! SVGs are written to `docs/src/assets/contour/`.

use visus::plot::{ContourPlot, ColorMap};
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

const OUT: &str = "docs/src/assets/contour";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/contour");

    lines();
    filled();
    scatter();
    explicit_levels();

    println!("Contour SVGs written to {OUT}/");
}

/// Build a uniform grid z[row][col] over [xmin,xmax] x [ymin,ymax].
fn gaussian_grid(
    nx: usize,
    ny: usize,
    x_range: (f64, f64),
    y_range: (f64, f64),
    f: impl Fn(f64, f64) -> f64,
) -> (Vec<Vec<f64>>, Vec<f64>, Vec<f64>) {
    let xs: Vec<f64> = (0..nx)
        .map(|i| x_range.0 + i as f64 / (nx - 1) as f64 * (x_range.1 - x_range.0))
        .collect();
    let ys: Vec<f64> = (0..ny)
        .map(|j| y_range.0 + j as f64 / (ny - 1) as f64 * (y_range.1 - y_range.0))
        .collect();
    let z: Vec<Vec<f64>> = ys
        .iter()
        .map(|&y| xs.iter().map(|&x| f(x, y)).collect())
        .collect();
    (z, xs, ys)
}

/// Iso-line contours (no fill) from a regular grid.
///
/// The default mode: `n_levels` evenly spaced iso-lines are drawn using the
/// colormap. Here a custom line color overrides the map to produce clean navy
/// lines on a white background.
fn lines() {
    // Single Gaussian peak centred at origin
    let (z, xs, ys) = gaussian_grid(60, 60, (-3.0, 3.0), (-3.0, 3.0), |x, y| {
        (-(x * x + y * y) / 2.0).exp()
    });

    let cp = ContourPlot::new()
        .with_grid(z, xs, ys)
        .with_n_levels(10)
        .with_line_color("steelblue")
        .with_line_width(1.2);

    let plots = vec![Plot::Contour(cp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Iso-line Contours — Gaussian Peak")
        .with_x_label("x")
        .with_y_label("y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/lines.svg"), svg).unwrap();
}

/// Filled color bands with a colorbar.
///
/// `.with_filled()` fills each band between adjacent iso-levels using the
/// colormap. `.with_legend(label)` enables a colorbar in the right margin.
fn filled() {
    // Bimodal surface: two overlapping Gaussian peaks
    let (z, xs, ys) = gaussian_grid(60, 60, (-4.0, 4.0), (-4.0, 4.0), |x, y| {
        let peak1 = (- ((x - 1.2) * (x - 1.2) + (y - 1.0) * (y - 1.0)) / 1.5).exp();
        let peak2 = 0.6 * (- ((x + 1.5) * (x + 1.5) + (y + 1.2) * (y + 1.2)) / 1.2).exp();
        peak1 + peak2
    });

    let cp = ContourPlot::new()
        .with_grid(z, xs, ys)
        .with_n_levels(9)
        .with_filled()
        .with_legend("Density");

    let plots = vec![Plot::Contour(cp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Filled Contours — Bimodal Surface")
        .with_x_label("x")
        .with_y_label("y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/filled.svg"), svg).unwrap();
}

/// Scattered (x, y, z) input — IDW interpolation to a regular grid.
///
/// `with_points` accepts an iterator of `(x, y, z)` triples at arbitrary
/// positions. The values are interpolated onto an internal 50×50 grid via
/// inverse-distance weighting (IDW) before iso-lines are computed.
/// Useful for spatial data that does not come pre-gridded.
fn scatter() {
    // 11 x 11 = 121 scattered sample points from a saddle-like function
    let pts: Vec<(f64, f64, f64)> = (-5..=5)
        .flat_map(|i| {
            (-5..=5).map(move |j| {
                let x = i as f64;
                let y = j as f64;
                // Two peaks at different heights
                let v = (-((x - 1.5) * (x - 1.5) + (y - 1.5) * (y - 1.5)) / 4.0).exp()
                    + 0.7 * (-((x + 2.0) * (x + 2.0) + (y + 1.5) * (y + 1.5)) / 3.0).exp();
                (x, y, v)
            })
        })
        .collect();

    let cp = ContourPlot::new()
        .with_points(pts)
        .with_n_levels(8)
        .with_filled()
        .with_colormap(ColorMap::Inferno)
        .with_legend("Value");

    let plots = vec![Plot::Contour(cp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Scattered Input — IDW Interpolation")
        .with_x_label("x")
        .with_y_label("y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/scatter.svg"), svg).unwrap();
}

/// Explicit iso-level values via `with_levels`.
///
/// Instead of auto-spacing from `n_levels`, specific z values are passed.
/// Useful when iso-lines should correspond to meaningful thresholds
/// (e.g. specific expression cutoffs, contour intervals on a topographic map).
fn explicit_levels() {
    // Gaussian: z ranges 0 → 1; choose levels at meaningful fractions
    let (z, xs, ys) = gaussian_grid(60, 60, (-3.0, 3.0), (-3.0, 3.0), |x, y| {
        (-(x * x + y * y) / 2.0).exp()
    });

    let cp = ContourPlot::new()
        .with_grid(z, xs, ys)
        .with_levels(&[0.1, 0.25, 0.5, 0.75, 0.9])
        .with_line_color("darkgreen")
        .with_line_width(1.5);

    let plots = vec![Plot::Contour(cp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Explicit Iso-levels: 0.1, 0.25, 0.5, 0.75, 0.9")
        .with_x_label("x")
        .with_y_label("y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/explicit_levels.svg"), svg).unwrap();
}
