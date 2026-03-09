//! Comprehensive kuva vs plotters benchmark across all output formats.
//!
//! Tests: SVG, PNG (encoded), raw pixel buffer — with and without text.
//! Run: `cargo bench --bench raster --features png`

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn scatter_data(n: usize) -> Vec<(f64, f64)> {
    (0..n).map(|i| (i as f64, (i as f64 * 0.001).sin())).collect()
}

fn grid_data(n: usize) -> Vec<Vec<f64>> {
    (0..n).map(|i| (0..n).map(|j| (i * n + j) as f64).collect()).collect()
}

// ── kuva scene builders ─────────────────────────────────────────────────────

fn kuva_scatter_scene(data: &[(f64, f64)]) -> kuva::render::render::Scene {
    use kuva::plot::scatter::ScatterPlot;
    use kuva::render::layout::Layout;
    use kuva::render::plots::Plot;
    use kuva::render::render::render_multiple;
    let n = data.len() as f64;
    let plot = ScatterPlot::new().with_data(data.to_vec());
    let layout = Layout::new((0.0, n), (-1.0, 1.0))
        .with_title("Scatter")
        .with_x_label("X")
        .with_y_label("Y");
    render_multiple(vec![Plot::Scatter(plot)], layout)
}

fn kuva_scatter_scene_no_labels(data: &[(f64, f64)]) -> kuva::render::render::Scene {
    use kuva::plot::scatter::ScatterPlot;
    use kuva::render::layout::Layout;
    use kuva::render::plots::Plot;
    use kuva::render::render::render_multiple;
    let n = data.len() as f64;
    let plot = ScatterPlot::new().with_data(data.to_vec());
    let layout = Layout::new((0.0, n), (-1.0, 1.0));
    render_multiple(vec![Plot::Scatter(plot)], layout)
}

fn kuva_heatmap_scene(data: &[Vec<f64>]) -> kuva::render::render::Scene {
    use kuva::plot::Heatmap;
    use kuva::render::layout::Layout;
    use kuva::render::plots::Plot;
    use kuva::render::render::render_multiple;
    let n = data.len();
    let plot = Heatmap::new().with_data(data.to_vec());
    let layout = Layout::new((0.5, n as f64 + 0.5), (0.5, n as f64 + 0.5))
        .with_title("Heatmap").with_x_label("X").with_y_label("Y");
    render_multiple(vec![Plot::Heatmap(plot)], layout)
}

// ── plotters helpers ────────────────────────────────────────────────────────

fn plotters_scatter_svg(data: &[(f64, f64)], with_text: bool) -> String {
    use plotters::prelude::*;
    let n = data.len() as f64;
    let mut buf = String::new();
    {
        let root = SVGBackend::with_string(&mut buf, (800, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let mut builder = ChartBuilder::on(&root);
        if with_text {
            builder.caption("Scatter", ("sans-serif", 20));
            builder.x_label_area_size(30).y_label_area_size(40);
        }
        let mut chart = builder.build_cartesian_2d(0.0..n, -1.0..1.0_f64).unwrap();
        if with_text { chart.configure_mesh().draw().unwrap(); }
        chart.draw_series(data.iter().map(|&(x, y)| Circle::new((x, y), 3, BLUE.filled()))).unwrap();
        root.present().unwrap();
    }
    buf
}

fn plotters_scatter_buffer(data: &[(f64, f64)], with_text: bool) -> Vec<u8> {
    use plotters::prelude::*;
    let n = data.len() as f64;
    let mut buf = vec![0u8; 800 * 600 * 3];
    {
        let root = BitMapBackend::with_buffer(&mut buf, (800, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let mut builder = ChartBuilder::on(&root);
        if with_text {
            builder.caption("Scatter", ("sans-serif", 20));
            builder.x_label_area_size(30).y_label_area_size(40);
        }
        let mut chart = builder.build_cartesian_2d(0.0..n, -1.0..1.0_f64).unwrap();
        if with_text { chart.configure_mesh().draw().unwrap(); }
        chart.draw_series(data.iter().map(|&(x, y)| Circle::new((x, y), 3, BLUE.filled()))).unwrap();
        root.present().unwrap();
    }
    buf
}

fn plotters_scatter_png(data: &[(f64, f64)], with_text: bool) -> Vec<u8> {
    let buf = plotters_scatter_buffer(data, with_text);
    let mut png = Vec::new();
    {
        use image::ImageEncoder;
        let enc = image::codecs::png::PngEncoder::new(&mut png);
        enc.write_image(&buf, 800, 600, image::ColorType::Rgb8).unwrap();
    }
    png
}

// ── Benchmarks ──────────────────────────────────────────────────────────────

fn bench_scatter_comprehensive(c: &mut Criterion) {
    // Warm up fontdue font cache
    {
        let data = scatter_data(10);
        let scene = kuva_scatter_scene(&data);
        let _ = kuva::RasterBackend::new().with_scale(1.0).render_scene(&scene);
    }

    for &n in &[1_000usize, 10_000, 100_000] {
        let data = scatter_data(n);
        let scene_text = kuva_scatter_scene(&data);
        let scene_bare = kuva_scatter_scene_no_labels(&data);

        // ── SVG ─────────────────────────────────────────────────────────
        let mut g = c.benchmark_group(format!("scatter_{n}_svg"));
        g.bench_function("kuva", |b| {
            b.iter(|| criterion::black_box(kuva::backend::svg::SvgBackend.render_scene(&scene_text)))
        });
        g.bench_function("kuva_no_text", |b| {
            b.iter(|| criterion::black_box(kuva::backend::svg::SvgBackend.render_scene(&scene_bare)))
        });
        g.bench_function("plotters", |b| {
            b.iter(|| criterion::black_box(plotters_scatter_svg(&data, true)))
        });
        g.bench_function("plotters_no_text", |b| {
            b.iter(|| criterion::black_box(plotters_scatter_svg(&data, false)))
        });
        g.finish();

        // ── PNG (encoded bytes) ─────────────────────────────────────────
        let mut g = c.benchmark_group(format!("scatter_{n}_png"));
        g.bench_function("kuva_raster", |b| {
            b.iter(|| criterion::black_box(
                kuva::RasterBackend::new().with_scale(1.0).render_scene(&scene_text).unwrap()
            ))
        });
        g.bench_function("kuva_raster_no_text", |b| {
            b.iter(|| criterion::black_box(
                kuva::RasterBackend::new().with_scale(1.0).with_skip_text(true).render_scene(&scene_text).unwrap()
            ))
        });
        g.bench_function("plotters", |b| {
            b.iter(|| criterion::black_box(plotters_scatter_png(&data, true)))
        });
        g.bench_function("plotters_no_text", |b| {
            b.iter(|| criterion::black_box(plotters_scatter_png(&data, false)))
        });
        g.finish();

        // ── Raw buffer (no PNG encoding) ────────────────────────────────
        let mut g = c.benchmark_group(format!("scatter_{n}_raw"));
        g.bench_function("kuva_pixmap", |b| {
            b.iter(|| criterion::black_box(
                kuva::RasterBackend::new().with_scale(1.0).render_scene_to_pixmap(&scene_text).unwrap()
            ))
        });
        g.bench_function("kuva_pixmap_no_text", |b| {
            b.iter(|| criterion::black_box(
                kuva::RasterBackend::new().with_scale(1.0).with_skip_text(true).render_scene_to_pixmap(&scene_text).unwrap()
            ))
        });
        g.bench_function("plotters_buffer", |b| {
            b.iter(|| criterion::black_box(plotters_scatter_buffer(&data, true)))
        });
        g.bench_function("plotters_buffer_no_text", |b| {
            b.iter(|| criterion::black_box(plotters_scatter_buffer(&data, false)))
        });
        g.finish();
    }
}

fn bench_heatmap_comprehensive(c: &mut Criterion) {
    for &n in &[50usize, 100, 200] {
        let data = grid_data(n);
        let scene = kuva_heatmap_scene(&data);

        let mut g = c.benchmark_group(format!("heatmap_{n}_png"));
        g.bench_function("kuva_raster", |b| {
            b.iter(|| criterion::black_box(
                kuva::RasterBackend::new().with_scale(1.0).render_scene(&scene).unwrap()
            ))
        });
        g.bench_function("kuva_raster_no_text", |b| {
            b.iter(|| criterion::black_box(
                kuva::RasterBackend::new().with_scale(1.0).with_skip_text(true).render_scene(&scene).unwrap()
            ))
        });
        g.finish();
    }
}

criterion_group!(benches, bench_scatter_comprehensive, bench_heatmap_comprehensive);
criterion_main!(benches);
