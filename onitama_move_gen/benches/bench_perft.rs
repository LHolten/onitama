use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use onitama_move_gen::perft::perft_test;

fn bench_perft(c: &mut Criterion) {
    let mut group = c.benchmark_group("perft");
    for i in 0..6usize {
        group.bench_with_input(BenchmarkId::new("perft_depth", i), &i, |b, i| {
            b.iter(|| perft_test(*i))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_perft);
criterion_main!(benches);
