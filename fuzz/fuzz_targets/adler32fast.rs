#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut adler32 = adler32fast::Adler32::new();
    adler32.update(data);
});
