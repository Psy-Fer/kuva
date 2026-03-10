//! Raster (PNG byte-output) benchmarks.
//!
//! Measures the full data → PNG bytes pipeline for kuva and plotters,
//! which is the path that matters for IPC / webview transfer.
//!
//! Three kuva backends are compared:
//!   - PngBackend:    Scene → SVG string → usvg parse → tiny_skia raster → PNG encode
//!   - RasterBackend: Scene → direct tiny_skia draw → PNG encode  (new)
//!
//! Plotters comparison:
//!   - BitMapBackend: draw directly into RGB buffer → PNG encode

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

// ── Data generators ─────────────────────────────────────────────────────────

fn scatter_data(n: usize) -> Vec<(f64, f64)> {
    (0..n)
        .map(|i| (i as f64, (i as f64 * 0.001).sin()))
        .collect()
}

fn grid_data(n: usize) -> Vec<Vec<f64>> {
    (0..n)
        .map(|i| (0..n).map(|j| (i * n + j) as f64).collect())
        .collect()
}

// ── kuva helpers ────────────────────────────────────────────────────────────

fn kuva_scene_scatter(data: &[(f64, f64)]) -> kuva::render::render::Scene {
    use kuva::plot::scatter::ScatterPlot;
    use kuva::render::layout::Layout;
    use kuva::render::plots::Plot;
    use kuva::render::render::render_multiple;

    let n = data.len() as f64;
    let plot = ScatterPlot::new().with_data(data.to_vec());
    let layout = Layout::new((0.0, n), (-1.0, 1.0));
    render_multiple(vec![Plot::Scatter(plot)], layout)
}

fn kuva_scene_heatmap(data: &[Vec<f64>]) -> kuva::render::render::Scene {
    use kuva::plot::Heatmap;
    use kuva::render::layout::Layout;
    use kuva::render::plots::Plot;
    use kuva::render::render::render_multiple;

    let n = data.len();
    let plot = Heatmap::new().with_data(data.to_vec());
    let layout = Layout::new((0.5, n as f64 + 0.5), (0.5, n as f64 + 0.5));
    render_multiple(vec![Plot::Heatmap(plot)], layout)
}

// ── plotters helpers ────────────────────────────────────────────────────────

fn plotters_scatter_png(data: &[(f64, f64)]) -> Vec<u8> {
    use plotters::prelude::*;

    let n = data.len() as f64;
    let mut buf = vec![0u8; 800 * 600 * 3];
    {
        let root = BitMapBackend::with_buffer(&mut buf, (800, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(0.0..n, -1.0..1.0_f64)
            .unwrap();
        chart.configure_mesh().draw().unwrap();
        chart
            .draw_series(
                data.iter()
                    .map(|&(x, y)| Circle::new((x, y), 3, BLUE.filled())),
            )
            .unwrap();
        root.present().unwrap();
    }
    let mut png_bytes = Vec::new();
    {
        use image::ImageEncoder;
        let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
        encoder
            .write_image(&buf, 800, 600, image::ColorType::Rgb8)
            .unwrap();
    }
    png_bytes
}

fn plotters_heatmap_png(data: &[Vec<f64>]) -> Vec<u8> {
    use plotters::prelude::*;

    let n = data.len();
    let flat_max = data
        .iter()
        .flatten()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let mut buf = vec![0u8; 800 * 600 * 3];
    {
        let root = BitMapBackend::with_buffer(&mut buf, (800, 600)).into_drawing_area();
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
                    let color = RGBColor(0, g, 255 - g);
                    Rectangle::new([(col, row), (col + 1, row + 1)], color.filled())
                })
            }))
            .unwrap();
        root.present().unwrap();
    }
    let mut png_bytes = Vec::new();
    {
        use image::ImageEncoder;
        let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
        encoder
            .write_image(&buf, 800, 600, image::ColorType::Rgb8)
            .unwrap();
    }
    png_bytes
}

// ── Benchmarks ──────────────────────────────────────────────────────────────

fn bench_scatter_raster(c: &mut Criterion) {
    let mut group = c.benchmark_group("scatter_png");
    for &n in &[1_000usize, 10_000, 100_000] {
        let data = scatter_data(n);

        // kuva: build scene once, then benchmark just the raster backend
        let scene = kuva_scene_scatter(&data);

        group.bench_with_input(BenchmarkId::new("kuva_raster", n), &scene, |b, s| {
            b.iter(|| {
                criterion::black_box(
                    kuva::RasterBackend::new()
                        .with_scale(1.0)
                        .render_scene(s)
                        .unwrap(),
                )
            })
        });

        group.bench_with_input(BenchmarkId::new("kuva_png", n), &scene, |b, s| {
            b.iter(|| {
                criterion::black_box(
                    kuva::PngBackend::new()
                        .with_scale(1.0)
                        .render_scene(s)
                        .unwrap(),
                )
            })
        });

        group.bench_with_input(BenchmarkId::new("plotters_bitmap", n), &data, |b, d| {
            b.iter(|| criterion::black_box(plotters_scatter_png(d)))
        });
    }
    group.finish();
}

fn bench_heatmap_raster(c: &mut Criterion) {
    let mut group = c.benchmark_group("heatmap_png");
    for &n in &[50usize, 100, 200] {
        let data = grid_data(n);
        let scene = kuva_scene_heatmap(&data);

        group.bench_with_input(BenchmarkId::new("kuva_raster", n), &scene, |b, s| {
            b.iter(|| {
                criterion::black_box(
                    kuva::RasterBackend::new()
                        .with_scale(1.0)
                        .render_scene(s)
                        .unwrap(),
                )
            })
        });

        group.bench_with_input(BenchmarkId::new("kuva_png", n), &scene, |b, s| {
            b.iter(|| {
                criterion::black_box(
                    kuva::PngBackend::new()
                        .with_scale(1.0)
                        .render_scene(s)
                        .unwrap(),
                )
            })
        });

        group.bench_with_input(BenchmarkId::new("plotters_bitmap", n), &data, |b, d| {
            b.iter(|| criterion::black_box(plotters_heatmap_png(d)))
        });
    }
    group.finish();
}

// Also benchmark scene construction time separately, since the raster benches
// above only time the backend (scene is pre-built). This isolates where time
// is being spent.
fn bench_scene_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_build");
    for &n in &[1_000usize, 10_000, 100_000] {
        let data = scatter_data(n);
        group.bench_with_input(BenchmarkId::new("scatter", n), &data, |b, d| {
            b.iter(|| criterion::black_box(kuva_scene_scatter(d)))
        });
    }
    for &n in &[50usize, 100, 200] {
        let data = grid_data(n);
        group.bench_with_input(BenchmarkId::new("heatmap", n), &data, |b, d| {
            b.iter(|| criterion::black_box(kuva_scene_heatmap(d)))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_scatter_raster, bench_heatmap_raster, bench_scene_build);
criterion_main!(benches);
