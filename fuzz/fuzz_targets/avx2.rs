#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut adler32 = adler32fast::specialized::avx2::State::new(1)
        .expect("AVX2 implementation unavailable to fuzz");
    adler32.update(data);
    adler32.finalize();
});
