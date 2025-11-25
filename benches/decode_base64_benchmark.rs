use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use simd_exercise::decode_base64::decode_base64_reference;

pub fn decode_base64_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_base64");

    let input = "aaaa";

    group.bench_function("decode_base64_reference", |b| {
        b.iter(|| decode_base64_reference(black_box(input.as_bytes()), &mut Vec::new()))
    });

    group.finish();
}

criterion_group!(benches, decode_base64_benchmark);
criterion_main!(benches);
