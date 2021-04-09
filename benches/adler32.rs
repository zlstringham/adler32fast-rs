use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rand::Rng;

fn adler32(bytes: &[u8]) {
    let mut adler32 = adler32::RollingAdler32::new();
    adler32.update_buffer(bytes);
}

fn adler32fast_baseline(bytes: &[u8]) {
    let mut adler32 = adler32fast::Adler32::internal_new_baseline(1);
    adler32.update(bytes);
}

fn adler32fast_specialized(bytes: &[u8]) {
    let mut adler32 = adler32fast::Adler32::internal_new_specialized(1).unwrap();
    adler32.update(bytes);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..1_000_000).map(|_| rng.gen_range(0..=255)).collect();

    let mut group = c.benchmark_group("adler32");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("adler32", |b| b.iter(|| adler32(black_box(&bytes))));
    group.bench_function("adler32fast-baseline", |b| {
        b.iter(|| adler32fast_baseline(black_box(&bytes)))
    });
    if adler32fast::Adler32::internal_new_specialized(1).is_some() {
        group.bench_function("adler32fast-specialized", |b| {
            b.iter(|| adler32fast_specialized(black_box(&bytes)))
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
