use std::hint::black_box;

use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};

fn vector_add(a: &[f32], b: &[f32], output: &mut [f32]) {
    for ((out, x), y) in output.iter_mut().zip(a).zip(b) {
        *out = x + y;
    }
}

fn bench_vector_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("runtime/vector_add");

    for size in [1_024, 16_384, 262_144, 1_048_576] {
        let a = vec![1.0_f32; size];
        let b = vec![2.0_f32; size];
        let mut output = vec![0.0_f32; size];

        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &size,
            |bencher, _| {
                bencher.iter(|| {
                    vector_add(
                        black_box(&a),
                        black_box(&b),
                        black_box(&mut output),
                    );
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_vector_add);
criterion_main!(benches);
