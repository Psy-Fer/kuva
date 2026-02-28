use criterion::{criterion_group, criterion_main, BenchmarkId, BatchSize, Criterion};

use kuva::plot::scatter::ScatterPlot;
use kuva::plot::line::LinePlot;
use kuva::plot::violin::ViolinPlot;
use kuva::plot::manhattan::ManhattanPlot;
use kuva::plot::Heatmap;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::{render_scatter, render_line, render_violin, render_manhattan, render_multiple};
use kuva::backend::svg::SvgBackend;

// ── scatter ──────────────────────────────────────────────────────────────────

fn bench_scatter(c: &mut Criterion) {
    let mut group = c.benchmark_group("scatter");
    for &n in &[100usize, 1_000, 10_000, 100_000, 1_000_000] {
        let data: Vec<(f64, f64)> = (0..n)
            .map(|i| (i as f64, (i as f64 * 0.001).sin()))
            .collect();
        let plot = ScatterPlot::new().with_data(data);

        // Full pipeline: scene build + SVG
        group.bench_with_input(BenchmarkId::new("scene_and_svg", n), &n, |b, &n| {
            b.iter(|| {
                let layout = Layout::new((0.0, n as f64), (-1.0, 1.0));
                let scene = render_scatter(&plot, layout);
                criterion::black_box(SvgBackend.render_scene(&scene))
            });
        });

        // SVG only (scene build is in setup, not timed)
        group.bench_with_input(BenchmarkId::new("svg_only", n), &n, |b, &n| {
            b.iter_batched(
                || {
                    let layout = Layout::new((0.0, n as f64), (-1.0, 1.0));
                    render_scatter(&plot, layout)
                },
                |scene| criterion::black_box(SvgBackend.render_scene(&scene)),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

// ── scatter with error bars ──────────────────────────────────────────────────

fn bench_scatter_errorbars(c: &mut Criterion) {
    let mut group = c.benchmark_group("scatter_errorbars");
    for &n in &[100usize, 1_000, 10_000, 100_000] {
        let data: Vec<(f64, f64)> = (0..n)
            .map(|i| (i as f64, (i as f64 * 0.001).sin()))
            .collect();
        let errs: Vec<f64> = (0..n).map(|i| i as f64 * 0.0001 + 0.05).collect();
        let plot = ScatterPlot::new().with_data(data).with_y_err(errs);

        group.bench_with_input(BenchmarkId::new("scene_and_svg", n), &n, |b, &n| {
            b.iter(|| {
                let layout = Layout::new((0.0, n as f64), (-1.5, 1.5));
                let scene = render_scatter(&plot, layout);
                criterion::black_box(SvgBackend.render_scene(&scene))
            });
        });
    }
    group.finish();
}

// ── line ─────────────────────────────────────────────────────────────────────

fn bench_line(c: &mut Criterion) {
    let mut group = c.benchmark_group("line");
    for &n in &[100usize, 1_000, 10_000, 100_000, 1_000_000] {
        let data: Vec<(f64, f64)> = (0..n)
            .map(|i| (i as f64, (i as f64 * 0.001).sin()))
            .collect();
        let plot = LinePlot::new().with_data(data);

        group.bench_with_input(BenchmarkId::new("scene_and_svg", n), &n, |b, &n| {
            b.iter(|| {
                let layout = Layout::new((0.0, n as f64), (-1.0, 1.0));
                let scene = render_line(&plot, layout);
                criterion::black_box(SvgBackend.render_scene(&scene))
            });
        });
    }
    group.finish();
}

// ── violin ───────────────────────────────────────────────────────────────────

fn make_violin(n: usize) -> ViolinPlot {
    let vals_a: Vec<f64> = (0..n).map(|i| (i as f64 * 0.01).sin()).collect();
    let vals_b: Vec<f64> = (0..n).map(|i| (i as f64 * 0.01).cos()).collect();
    let vals_c: Vec<f64> = (0..n).map(|i| (i as f64 * 0.01 + 1.0).sin() * 0.5).collect();
    ViolinPlot::new()
        .with_group("A", vals_a)
        .with_group("B", vals_b)
        .with_group("C", vals_c)
}

fn bench_violin(c: &mut Criterion) {
    let mut group = c.benchmark_group("violin");
    for &n in &[100usize, 1_000, 10_000, 100_000] {
        let plot = make_violin(n);
        let layout = Layout::new((0.0, 3.0), (-1.0, 1.0));

        // Full pipeline
        group.bench_with_input(BenchmarkId::new("scene_and_svg", n), &n, |b, _| {
            b.iter(|| {
                let scene = render_violin(&plot, &layout);
                criterion::black_box(SvgBackend.render_scene(&scene))
            });
        });

        // SVG only
        group.bench_with_input(BenchmarkId::new("svg_only", n), &n, |b, _| {
            b.iter_batched(
                || render_violin(&plot, &layout),
                |scene| criterion::black_box(SvgBackend.render_scene(&scene)),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

// ── manhattan ────────────────────────────────────────────────────────────────

const CHROMS: &[&str] = &[
    "1","2","3","4","5","6","7","8","9","10",
    "11","12","13","14","15","16","17","18","19","20","21","22",
];

fn make_manhattan(n: usize) -> ManhattanPlot {
    let data: Vec<(String, f64)> = (0..n)
        .map(|i| {
            let chrom = CHROMS[i % CHROMS.len()].to_string();
            // Vary p-values so we get a range of -log10 values
            let pvalue = 0.05_f64.powf(((i % 100) as f64 + 1.0) / 100.0);
            (chrom, pvalue)
        })
        .collect();
    ManhattanPlot::new().with_data(data)
}

fn bench_manhattan(c: &mut Criterion) {
    let mut group = c.benchmark_group("manhattan");

    for &n in &[1_000usize, 10_000, 100_000, 1_000_000] {
        let plot = make_manhattan(n);
        // Build a layout once from a small representative plot; ranges don't need to match exactly
        let layout = Layout::new((0.0, n as f64), (0.0, 10.0));

        group.bench_with_input(BenchmarkId::new("scene_and_svg", n), &n, |b, _| {
            b.iter(|| {
                let scene = render_manhattan(&plot, &layout);
                criterion::black_box(SvgBackend.render_scene(&scene))
            });
        });
    }
    group.finish();
}

// ── heatmap ──────────────────────────────────────────────────────────────────

fn make_heatmap_data(n: usize) -> Vec<Vec<f64>> {
    (0..n)
        .map(|i| (0..n).map(|j| (i * n + j) as f64).collect())
        .collect()
}

fn bench_heatmap(c: &mut Criterion) {
    let mut group = c.benchmark_group("heatmap");
    for &n in &[10usize, 50, 100, 200, 500] {
        let data = make_heatmap_data(n);
        // Use a fixed layout range; heatmap coordinate system is 0.5..(n+0.5)
        let x_range = (0.5, n as f64 + 0.5);
        let y_range = (0.5, n as f64 + 0.5);

        // Without show_values — use iter_batched to provide fresh Plot+Layout each call
        group.bench_with_input(BenchmarkId::new("no_values", n), &n, |b, _| {
            b.iter_batched(
                || {
                    let plot = Heatmap::new().with_data(data.clone());
                    let layout = Layout::new(x_range, y_range);
                    (plot, layout)
                },
                |(plot, layout)| {
                    let scene = render_multiple(vec![Plot::Heatmap(plot)], layout);
                    criterion::black_box(SvgBackend.render_scene(&scene))
                },
                BatchSize::SmallInput,
            );
        });

        // With show_values (only for smaller sizes)
        if n <= 100 {
            group.bench_with_input(BenchmarkId::new("with_values", n), &n, |b, _| {
                b.iter_batched(
                    || {
                        let plot = Heatmap::new().with_data(data.clone()).with_values();
                        let layout = Layout::new(x_range, y_range);
                        (plot, layout)
                    },
                    |(plot, layout)| {
                        let scene = render_multiple(vec![Plot::Heatmap(plot)], layout);
                        criterion::black_box(SvgBackend.render_scene(&scene))
                    },
                    BatchSize::SmallInput,
                );
            });
        }
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_scatter,
    bench_scatter_errorbars,
    bench_line,
    bench_violin,
    bench_manhattan,
    bench_heatmap,
);
criterion_main!(benches);
