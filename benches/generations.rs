use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};

use polycube::Generation;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut generation = Generation::default();
    for _ in 1..13 {
        generation.advance();
    }

    c.bench_with_input(
        BenchmarkId::new("Generation", 14),
        &generation,
        |b, generation| {
            b.iter_batched_ref(
                || generation.clone(),
                Generation::advance,
                BatchSize::LargeInput,
            );
        },
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark
}
criterion_main!(benches);
