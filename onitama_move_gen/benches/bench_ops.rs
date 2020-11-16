use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use onitama_move_gen::ops::{shift_or, shift_or_pdep};

fn bench_shift_or(c: &mut Criterion) {
    let mut group = c.benchmark_group("shift_or");
    for i in 0..6u32 {
        group.bench_with_input(BenchmarkId::new("lut", i), &i, |b, i| {
            let pieces = black_box((1u32 << (i + 1)) - 1);
            b.iter(|| shift_or(10, 0, pieces))
        });
        group.bench_with_input(BenchmarkId::new("lut_pdep", i), &i, |b, i| {
            let pieces = black_box((1u32 << (i + 1)) - 1);
            b.iter(|| shift_or_pdep(10, 0, pieces))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_shift_or);
criterion_main!(benches);
