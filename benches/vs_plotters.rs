//! Head-to-head benchmark: kuva vs plotters
//!
//! Compares the full pipeline (data → rendered SVG string) for both crates
//! on identical datasets. Each benchmark produces an SVG scatter plot, line
//! chart, or heatmap-style grid so the comparison is apples-to-apples.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn make_scatter_data(n: usize) -> Vec<(f64, f64)> {
    (0..n)
        .map(|i| (i as f64, (i as f64 * 0.001).sin()))
        .collect()
}

fn make_grid_data(n: usize) -> Vec<Vec<f64>> {
    (0..n)
        .map(|i| (0..n).map(|j| (i * n + j) as f64).collect())
        .collect()
}

// ── Scatter: kuva ───────────────────────────────────────────────────────────

fn kuva_scatter_svg(data: &[(f64, f64)]) -> String {
    use kuva::backend::svg::SvgBackend;
    use kuva::plot::scatter::ScatterPlot;
    use kuva::render::layout::Layout;
    use kuva::render::plots::Plot;
    use kuva::render::render::render_multiple;

    let n = data.len() as f64;
    let plot = ScatterPlot::new().with_data(data.to_vec());
    let plots = vec![Plot::Scatter(plot)];
    let layout = Layout::new((0.0, n), (-1.0, 1.0));
    let scene = render_multiple(plots, layout);
    SvgBackend.render_scene(&scene)
}

// ── Scatter: plotters ───────────────────────────────────────────────────────

fn plotters_scatter_svg(data: &[(f64, f64)]) -> String {
    use plotters::prelude::*;

    let n = data.len() as f64;
    let mut buf = String::new();
    {
        let root = SVGBackend::with_string(&mut buf, (800, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(0.0..n, -1.0..1.0_f64)
            .unwrap();
        chart.configure_mesh().draw().unwrap();
        chart
            .draw_series(data.iter().map(|&(x, y)| Circle::new((x, y), 3, BLUE.filled())))
            .unwrap();
        root.present().unwrap();
    }
    buf
}

// ── Line: kuva ──────────────────────────────────────────────────────────────

fn kuva_line_svg(data: &[(f64, f64)]) -> String {
    use kuva::backend::svg::SvgBackend;
    use kuva::plot::line::LinePlot;
    use kuva::render::layout::Layout;
    use kuva::render::plots::Plot;
    use kuva::render::render::render_multiple;

    let n = data.len() as f64;
    let plot = LinePlot::new().with_data(data.to_vec());
    let plots = vec![Plot::Line(plot)];
    let layout = Layout::new((0.0, n), (-1.0, 1.0));
    let scene = render_multiple(plots, layout);
    SvgBackend.render_scene(&scene)
}

// ── Line: plotters ──────────────────────────────────────────────────────────

fn plotters_line_svg(data: &[(f64, f64)]) -> String {
    use plotters::prelude::*;

    let n = data.len() as f64;
    let mut buf = String::new();
    {
        let root = SVGBackend::with_string(&mut buf, (800, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(0.0..n, -1.0..1.0_f64)
            .unwrap();
        chart.configure_mesh().draw().unwrap();
        chart
            .draw_series(LineSeries::new(data.iter().copied(), &BLUE))
            .unwrap();
        root.present().unwrap();
    }
    buf
}

// ── Heatmap (grid of rects): kuva ───────────────────────────────────────────

fn kuva_heatmap_svg(data: &[Vec<f64>]) -> String {
    use kuva::backend::svg::SvgBackend;
    use kuva::plot::Heatmap;
    use kuva::render::layout::Layout;
    use kuva::render::plots::Plot;
    use kuva::render::render::render_multiple;

    let n = data.len();
    let plot = Heatmap::new().with_data(data.to_vec());
    let plots = vec![Plot::Heatmap(plot)];
    let layout = Layout::new((0.5, n as f64 + 0.5), (0.5, n as f64 + 0.5));
    let scene = render_multiple(plots, layout);
    SvgBackend.render_scene(&scene)
}

// ── Heatmap (grid of rects): plotters ───────────────────────────────────────

fn plotters_heatmap_svg(data: &[Vec<f64>]) -> String {
    use plotters::prelude::*;

    let n = data.len();
    let flat_max = data.iter().flatten().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mut buf = String::new();
    {
        let root = SVGBackend::with_string(&mut buf, (800, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(0..n, 0..n)
            .unwrap();
        chart.configure_mesh().draw().unwrap();

        chart
            .draw_series(data.iter().enumerate().flat_map(|(row, cols)| {
                cols.iter().enumerate().map(move |(col, &val)| {
                    let norm = (val / flat_max).clamp(0.0, 1.0);
                    let g = (norm * 255.0) as u8;
                    let color = RGBColor(0, g, (255 - g) as u8);
                    Rectangle::new([(col, row), (col + 1, row + 1)], color.filled())
                })
            }))
            .unwrap();
        root.present().unwrap();
    }
    buf
}

// ── Criterion groups ────────────────────────────────────────────────────────

fn bench_scatter(c: &mut Criterion) {
    let mut group = c.benchmark_group("scatter_svg");
    for &n in &[1_000usize, 10_000, 100_000] {
        let data = make_scatter_data(n);
        group.bench_with_input(BenchmarkId::new("kuva", n), &data, |b, d| {
            b.iter(|| criterion::black_box(kuva_scatter_svg(d)))
        });
        group.bench_with_input(BenchmarkId::new("plotters", n), &data, |b, d| {
            b.iter(|| criterion::black_box(plotters_scatter_svg(d)))
        });
    }
    group.finish();
}

fn bench_line(c: &mut Criterion) {
    let mut group = c.benchmark_group("line_svg");
    for &n in &[1_000usize, 10_000, 100_000] {
        let data = make_scatter_data(n);
        group.bench_with_input(BenchmarkId::new("kuva", n), &data, |b, d| {
            b.iter(|| criterion::black_box(kuva_line_svg(d)))
        });
        group.bench_with_input(BenchmarkId::new("plotters", n), &data, |b, d| {
            b.iter(|| criterion::black_box(plotters_line_svg(d)))
        });
    }
    group.finish();
}

fn bench_heatmap(c: &mut Criterion) {
    let mut group = c.benchmark_group("heatmap_svg");
    for &n in &[50usize, 100, 200] {
        let data = make_grid_data(n);
        group.bench_with_input(BenchmarkId::new("kuva", n), &data, |b, d| {
            b.iter(|| criterion::black_box(kuva_heatmap_svg(d)))
        });
        group.bench_with_input(BenchmarkId::new("plotters", n), &data, |b, d| {
            b.iter(|| criterion::black_box(plotters_heatmap_svg(d)))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_scatter, bench_line, bench_heatmap);
criterion_main!(benches);
