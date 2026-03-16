//! Scatter plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example scatter
//! ```
//!
//! SVGs are written to `docs/src/assets/scatter/`.

use kuva::plot::scatter::{MarkerShape, ScatterPlot, TrendLine};
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

const OUT: &str = "docs/src/assets/scatter";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/scatter");

    basic();
    trend();
    confidence_band();
    error_bars();
    markers();
    bubble();
    per_point_colors();
    multiple_series();
    marker_semi_transparent();
    marker_hollow();

    println!("Scatter SVGs written to {OUT}/");
}

/// Basic scatter plot — x/y data, color, point size.
fn basic() {
    let data = vec![
        (0.5_f64, 1.2_f64),
        (1.4, 3.1),
        (2.1, 2.4),
        (3.3, 5.0),
        (4.0, 4.3),
        (5.2, 6.8),
        (6.1, 6.0),
        (7.0, 8.5),
        (8.4, 7.9),
        (9.1, 9.8),
    ];

    let plot = ScatterPlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_size(5.0);

    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Scatter Plot")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Scatter with a linear trend line, regression equation, and R² annotation.
fn trend() {
    let data = vec![
        (1.0_f64, 2.1_f64),
        (2.0, 3.9),
        (3.0, 6.2),
        (4.0, 7.8),
        (5.0, 10.1),
        (6.0, 12.3),
        (7.0, 13.9),
        (8.0, 16.2),
        (9.0, 17.8),
        (10.0, 19.7),
    ];

    let plot = ScatterPlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_size(5.0)
        .with_trend(TrendLine::Linear)
        .with_trend_color("crimson")
        .with_equation()
        .with_correlation();

    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Linear Trend Line")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/trend.svg"), svg).unwrap();
}

/// Scatter with a shaded confidence band.
fn confidence_band() {
    let xs: Vec<f64> = (1..=10).map(|i| i as f64).collect();
    let ys: Vec<f64> = xs.iter().map(|&x| x * 1.8 + 0.5).collect();
    let lower: Vec<f64> = ys.iter().map(|&y| y - 1.2).collect();
    let upper: Vec<f64> = ys.iter().map(|&y| y + 1.2).collect();

    let data: Vec<(f64, f64)> = xs.into_iter().zip(ys).collect();

    let plot = ScatterPlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_size(5.0)
        .with_band(lower, upper);

    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Confidence Band")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/confidence_band.svg"), svg).unwrap();
}

/// Scatter with symmetric error bars on both axes.
fn error_bars() {
    let data = vec![
        (1.0_f64, 2.0_f64),
        (2.0, 4.5),
        (3.0, 5.8),
        (4.0, 8.2),
        (5.0, 10.1),
    ];
    let x_err = vec![0.2_f64, 0.15, 0.3, 0.1, 0.25];
    let y_err = vec![0.6_f64, 0.8, 0.4, 0.9, 0.5];

    let plot = ScatterPlot::new()
        .with_data(data)
        .with_x_err(x_err)
        .with_y_err(y_err)
        .with_color("steelblue")
        .with_size(5.0);

    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Error Bars")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/error_bars.svg"), svg).unwrap();
}

/// All six marker shapes shown together.
fn markers() {
    let y_offsets = [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0];
    let shapes = [
        (MarkerShape::Circle,   "Circle",   "steelblue"),
        (MarkerShape::Square,   "Square",   "crimson"),
        (MarkerShape::Triangle, "Triangle", "seagreen"),
        (MarkerShape::Diamond,  "Diamond",  "darkorange"),
        (MarkerShape::Cross,    "Cross",    "purple"),
        (MarkerShape::Plus,     "Plus",     "saddlebrown"),
    ];

    let plots: Vec<Plot> = shapes
        .iter()
        .zip(y_offsets.iter())
        .map(|((shape, label, color), y)| {
            let data = vec![(1.0_f64, *y), (2.0, *y), (3.0, *y)];
            Plot::Scatter(
                ScatterPlot::new()
                    .with_data(data)
                    .with_color(*color)
                    .with_size(7.0)
                    .with_marker(*shape)
                    .with_legend(*label),
            )
        })
        .collect();

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Marker Shapes")
        .with_x_label("X")
        .with_y_label("");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/markers.svg"), svg).unwrap();
}

/// Per-point colors — three clusters colored independently via `.with_colors()`.
fn per_point_colors() {
    let data = vec![
        (1.0_f64, 1.5_f64), (1.5, 2.0), (2.0, 1.8),
        (4.0, 4.5),         (4.5, 5.0), (5.0, 4.8),
        (7.0, 2.0),         (7.5, 2.5), (8.0, 2.2),
    ];
    let colors = vec![
        "steelblue", "steelblue", "steelblue",
        "crimson",   "crimson",   "crimson",
        "seagreen",  "seagreen",  "seagreen",
    ];

    let plot = ScatterPlot::new()
        .with_data(data)
        .with_colors(colors)
        .with_size(6.0);

    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Per-Point Colors")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/per_point_colors.svg"), svg).unwrap();
}

/// Multiple series — two labeled scatter plots on the same axes.
fn multiple_series() {
    let series_a = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0_f64), (3.0, 4.0), (5.0, 3.5)])
        .with_color("steelblue")
        .with_size(5.0)
        .with_legend("Series A");

    let series_b = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 5.0_f64), (3.0, 6.5), (5.0, 7.0)])
        .with_color("crimson")
        .with_size(5.0)
        .with_legend("Series B");

    let plots = vec![Plot::Scatter(series_a), Plot::Scatter(series_b)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Two Series")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/multiple_series.svg"), svg).unwrap();
}

/// Three overlapping Gaussian clusters (200 pts each) — semi-transparent fill
/// + stroke. Solid markers at this density merge into an opaque blob; reducing
/// opacity lets the darker overlap region reveal where clusters share space.
fn marker_semi_transparent() {
    // Simple LCG so no external crate is needed in this example.
    let mut seed: u64 = 9_123_456_789;
    let mut lcg = || -> f64 {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        (seed >> 33) as f64 / ((1u64 << 31) as f64)
    };
    let gauss = |lcg: &mut dyn FnMut() -> f64| -> (f64, f64) {
        let u1 = lcg().max(1e-10);
        let u2 = lcg();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        let w = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).sin();
        (z, w)
    };

    let centers = [(3.0_f64, 4.0_f64), (5.0, 5.5), (4.0, 3.0)];
    let colors  = ["steelblue", "tomato", "seagreen"];
    let labels  = ["Cluster A", "Cluster B", "Cluster C"];

    let plots: Vec<Plot> = centers
        .iter()
        .zip(colors.iter())
        .zip(labels.iter())
        .map(|((&(cx, cy), &color), &label)| {
            let mut data = Vec::new();
            for _ in 0..200 {
                let (z0, z1) = gauss(&mut lcg);
                data.push((cx + z0 * 0.7, cy + z1 * 0.7));
            }
            Plot::Scatter(
                ScatterPlot::new()
                    .with_data(data)
                    .with_color(color)
                    .with_size(5.0)
                    .with_marker_opacity(0.25)
                    .with_marker_stroke_width(0.7)
                    .with_legend(label),
            )
        })
        .collect();

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Overlapping Clusters — semi-transparent markers")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/marker_semi_transparent.svg"), svg).unwrap();
}

/// 800 uniformly-sampled points in a noisy annulus — hollow open circles.
///
/// With solid fill, the ring interior becomes one dark mass. Hollow circles
/// let you see exactly where the density is concentrated along the arc.
fn marker_hollow() {
    let mut seed: u64 = 5_555_555_555;
    let mut lcg = || -> f64 {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        (seed >> 33) as f64 / ((1u64 << 31) as f64)
    };

    let data: Vec<(f64, f64)> = (0..800)
        .map(|_| {
            let angle = lcg() * 2.0 * std::f64::consts::PI;
            let r = 3.0 + (lcg() - 0.5) * 1.2;
            (r * angle.cos(), r * angle.sin())
        })
        .collect();

    let plot = ScatterPlot::new()
        .with_data(data)
        .with_color("steelblue")
        .with_size(4.0)
        .with_marker_opacity(0.0)
        .with_marker_stroke_width(1.0);

    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Hollow open circles — 800 pts in a noisy annulus")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/marker_hollow.svg"), svg).unwrap();
}

/// Bubble plot — per-point sizes via `.with_sizes()`.
fn bubble() {
    let data = vec![
        (1.0_f64, 3.0_f64),
        (2.5, 6.5),
        (4.0, 4.0),
        (5.5, 8.0),
        (7.0, 5.5),
        (8.5, 9.0),
    ];
    let sizes = vec![5.0_f64, 14.0, 9.0, 18.0, 11.0, 7.0];

    let plot = ScatterPlot::new()
        .with_data(data)
        .with_sizes(sizes)
        .with_color("steelblue");

    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Bubble Plot")
        .with_x_label("X")
        .with_y_label("Y");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/bubble.svg"), svg).unwrap();
}
