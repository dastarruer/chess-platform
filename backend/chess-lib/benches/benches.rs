use chess_lib::fen::FENString;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn bench_fen_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("FEN Parsing");

    // Reduces possible noise
    group.warm_up_time(std::time::Duration::from_secs(5));
    group.measurement_time(std::time::Duration::from_secs(10));
    group.sample_size(150);

    let starting_pos = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let complex_pos = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

    group.bench_function("starting_pos", |b| {
        b.iter(|| FENString::try_parse(black_box(starting_pos)))
    });

    // Benchmark a position with lots of pieces/rights
    group.bench_function("complex_pos", |b| {
        b.iter(|| FENString::try_parse(black_box(complex_pos)))
    });

    group.finish();
}

criterion_group!(benches, bench_fen_parsing);
criterion_main!(benches);
