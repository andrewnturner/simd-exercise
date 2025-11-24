use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use simd_exercise::pop_count::{pop_count_native, pop_count_reference, pop_count_vectorised};

pub fn pop_count_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("pop_count");

    group.bench_function("pop_count_reference", |b| {
        b.iter(|| pop_count_reference(black_box(37)))
    });
    group.bench_function("pop_count_vectorised", |b| {
        b.iter(|| pop_count_vectorised(black_box(37)))
    });
    group.bench_function("pop_count_native", |b| {
        b.iter(|| pop_count_native(black_box(37)))
    });

    group.finish();
}

criterion_group!(benches, pop_count_benchmark);
criterion_main!(benches);
