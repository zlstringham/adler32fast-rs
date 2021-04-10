#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut adler32 =
        adler32fast::Adler32::internal_new_specialized(1).expect("no SIMD implementation to fuzz");
    adler32.update(data);
    adler32.as_u32();
});
