use criterion::{
    black_box, criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, Criterion,
    Throughput,
};
use rand::Rng;

fn adler32(bytes: &[u8]) {
    let mut adler32 = adler32::RollingAdler32::new();
    adler32.update_buffer(bytes);
    adler32.hash();
}

fn adler32fast_baseline(bytes: &[u8]) {
    let mut adler32 = adler32fast::baseline::State::new(1);
    adler32.update(bytes);
    adler32.finalize();
}

fn adler32fast_ssse3(bytes: &[u8]) {
    let mut adler32 = adler32fast::specialized::ssse3::State::new(1).unwrap();
    adler32.update(bytes);
    adler32.finalize();
}

fn bench_all(mut group: BenchmarkGroup<WallTime>, bytes: &[u8]) {
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("adler32", |b| b.iter(|| adler32(black_box(&bytes))));
    group.bench_function("adler32fast-baseline", |b| {
        b.iter(|| adler32fast_baseline(black_box(&bytes)))
    });
    if adler32fast::specialized::ssse3::State::new(1).is_some() {
        group.bench_function("adler32fast-ssse3", |b| {
            b.iter(|| adler32fast_ssse3(black_box(&bytes)))
        });
    }
    group.finish();
}

macro_rules! benchmark {
    ($name:ident, $group:expr, $size:expr) => {
        fn $name(c: &mut Criterion) {
            let mut rng = rand::thread_rng();
            let bytes: Vec<u8> = (0..$size).map(|_| rng.gen_range(0..=255)).collect();
            let group = c.benchmark_group($group);
            bench_all(group, &bytes);
        }
    };
}

benchmark!(adler32_1kb, "adler32-1kb", 1_000);
benchmark!(adler32_100kb, "adler32-100kb", 100_000);
benchmark!(adler32_10mb, "adler32-10mb", 10_000_000);

criterion_group!(benches, adler32_1kb, adler32_100kb, adler32_10mb);
criterion_main!(benches);
