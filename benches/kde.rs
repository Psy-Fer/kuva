use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use visus::render::render_utils::{simple_kde, silverman_bandwidth};

fn bench_kde(c: &mut Criterion) {
    let mut group = c.benchmark_group("kde");
    for &n in &[100usize, 1_000, 10_000, 100_000] {
        let data: Vec<f64> = (0..n).map(|i| i as f64 * 0.01).collect();
        let bw = silverman_bandwidth(&data);

        group.bench_with_input(BenchmarkId::new("simple_kde_256", n), &n, |b, _| {
            b.iter(|| criterion::black_box(simple_kde(&data, bw, 256)));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_kde);
criterion_main!(benches);
