// use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
// use onitama_move_gen::gen::loop_moves;

// fn bench_loop_moves(c: &mut Criterion) {
//     let mut group = c.benchmark_group("loop_moves");
//     for i in 0..6u32 {
//         group.bench_with_input(BenchmarkId::new("count_moves", i), &i, |b, i| {
//             let pieces = black_box((1u32 << (i + 1)) - 1);
//             let player = black_box(0);
//             let cards = black_box((1 << 4) + (1 << 5));
//             b.iter(|| {
//                 let mut total = 0;
//                 loop_moves(player, cards, pieces, |_, _, _| {
//                     total += 1;
//                 });
//                 total
//             })
//         });
//     }
//     group.finish();
// }

// criterion_group!(benches, bench_loop_moves);
// criterion_main!(benches);
