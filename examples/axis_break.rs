//! Axis break documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example axis_break
//! ```
//!
//! SVGs are written to `docs/src/assets/axis_break/`.

use kuva::backend::svg::SvgBackend;
use kuva::plot::axis_break::AxisBreak;
use kuva::plot::{BarPlot, BoxPlot, LinePlot, ScatterPlot};
use kuva::render::layout::{AxisLine, Layout};
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::render::theme::Theme;
use std::fs;

const OUT: &str = "docs/src/assets/axis_break";

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all(OUT).unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

fn main() {
    // ── Bar with one dominant category ──────────────────────────────────────
    let bar = BarPlot::new()
        .with_bar("Liver", 8.2)
        .with_bar("Kidney", 6.9)
        .with_bar("Heart", 5.1)
        .with_bar("Brain", 7.4)
        .with_bar("Testis", 940.0)
        .with_error(vec![0.6, 0.5, 0.4, 0.7, 31.0])
        .with_color("#4C72B0");
    let plots = vec![Plot::Bar(bar)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Expression (TPM)")
        .with_y_label("TPM")
        .with_y_break(12.0, 880.0);
    write("bar_outlier", plots, layout);

    // ── Line with a gap in time ─────────────────────────────────────────────
    let early: Vec<(f64, f64)> = (0..=10)
        .map(|i| {
            (
                i as f64,
                1.0 + 0.6 * i as f64 + (i as f64 * 0.9).sin() * 0.4,
            )
        })
        .collect();
    let late: Vec<(f64, f64)> = (0..=10)
        .map(|i| {
            (
                100.0 + i as f64,
                9.0 - 0.3 * i as f64 + (i as f64 * 0.7).cos() * 0.3,
            )
        })
        .collect();
    let mut pts = early;
    pts.extend(late);
    let line = LinePlot::new().with_data(pts).with_color("#C44E52");
    let plots = vec![Plot::Line(line)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Response before and after a 90-day washout")
        .with_x_label("Day")
        .with_y_label("Score")
        .with_x_break(12.0, 98.0);
    write("line_time_gap", plots, layout);

    // ── Scatter, boxed axes, wider gap, no marks ────────────────────────────
    let mut data: Vec<(f64, f64)> = (0..40)
        .map(|i| {
            let x = i as f64 * 0.25;
            (x, 2.0 + (x * 1.3).sin() * 1.5 + (i % 7) as f64 * 0.1)
        })
        .collect();
    data.extend([(2.0, 48.0), (5.5, 51.0), (8.0, 46.5)]);
    let scatter = ScatterPlot::new()
        .with_data(data)
        .with_color("#55A868")
        .with_size(5.0);
    let plots = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Boxed axes, wide gap, no marks")
        .with_x_label("x")
        .with_y_label("y")
        .with_axis_line(AxisLine::Box)
        .with_y_break_config(AxisBreak::new(6.0, 44.0).with_gap(0.08).with_no_marks());
    write("scatter_boxed", plots, layout);

    // ── Box plot: whole-group outlier ───────────────────────────────────────
    let boxes = BoxPlot::new()
        .with_group("Input", vec![3.1, 3.6, 4.0, 4.4, 4.9, 5.2])
        .with_group("Mock", vec![2.8, 3.2, 3.7, 4.1, 4.5, 4.8])
        .with_group("IP", vec![610.0, 640.0, 655.0, 670.0, 690.0, 720.0])
        .with_group_colors(["#4C72B0", "#8C8C8C", "#DD8452"]);
    let plots = vec![Plot::Box(boxes)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Box plot with a y break")
        .with_y_label("Reads (thousands)")
        .with_y_break(7.0, 580.0);
    write("box_group", plots, layout);

    // ── Data inside the gap: marked with a cross ────────────────────────────
    let control: Vec<(f64, f64)> = (0..=12)
        .map(|i| (i as f64, 3.0 + (i as f64 * 0.8).sin() * 1.2))
        .collect();
    // Two treated readings land between the break edges (25 and 90).
    let treated: Vec<(f64, f64)> = vec![
        (0.0, 4.0),
        (1.0, 6.0),
        (2.0, 9.0),
        (3.0, 15.0),
        (4.0, 40.0),
        (5.0, 104.0),
        (6.0, 112.0),
        (7.0, 118.0),
        (8.0, 60.0),
        (9.0, 115.0),
        (10.0, 121.0),
        (11.0, 117.0),
        (12.0, 124.0),
    ];
    let plots = vec![
        Plot::Line(
            LinePlot::new()
                .with_data(control)
                .with_color("#4C72B0")
                .with_legend("Control"),
        ),
        Plot::Line(
            LinePlot::new()
                .with_data(treated)
                .with_color("#DD8452")
                .with_legend("Treated"),
        ),
    ];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Values inside the gap are marked with a cross")
        .with_x_label("Hour")
        .with_y_label("Signal")
        .with_y_break(25.0, 90.0);
    write("points_in_gap", plots, layout);

    // ── Custom background: the gap band follows it ──────────────────────────
    let bar = BarPlot::new()
        .with_bar("Liver", 8.2)
        .with_bar("Kidney", 6.9)
        .with_bar("Heart", 5.1)
        .with_bar("Brain", 7.4)
        .with_bar("Testis", 940.0)
        .with_color("#4C72B0");
    let plots = vec![Plot::Bar(bar)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("with_background(\"#f3eee2\")")
        .with_y_label("TPM")
        .with_background("#f3eee2")
        .with_y_break(12.0, 880.0);
    write("background_custom", plots, layout);

    // ── Dark theme ──────────────────────────────────────────────────────────
    let bar = BarPlot::new()
        .with_bar("Liver", 8.2)
        .with_bar("Kidney", 6.9)
        .with_bar("Heart", 5.1)
        .with_bar("Brain", 7.4)
        .with_bar("Testis", 940.0)
        .with_color("#8FB8DE");
    let plots = vec![Plot::Bar(bar)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Theme::dark()")
        .with_y_label("TPM")
        .with_theme(Theme::dark())
        .with_y_break(12.0, 880.0);
    write("background_dark", plots, layout);

    println!("Axis break SVGs written to {OUT}/");
}
