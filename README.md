# adler32fast
[![adler32fast](https://img.shields.io/crates/v/adler32fast.svg)](https://crates.io/crates/adler32fast)
[![Documentation](https://docs.rs/adler32fast/badge.svg)](https://docs.rs/adler32fast)
[![License: MIT or Apache-2.0](https://img.shields.io/crates/l/adler32fast)](LICENSE)

_Fast, SIMD-accelerated Adler-32 checksum computation_

## Usage

```rust
use adler32fast::Adler32;

let mut adler32 = Adler32::new();
adler32.update(b"foo bar baz");
let checksum = adler32.as_u32();
```

## Performance

This crate contains multiple Adler-32 implementations:

- A fast baseline implementation which processes up to 16 bytes per iteration
- An optimized implementation for modern `x86`/`x86_64` using either `avx2` or `ssse3` instructions

Calling the `Adler32::new` or `Adler32::from` constructor at runtime will perform a feature detection to
select the most optimal implementation for the current CPU feature set.

The following was tested on a Razer Book 13 RZ09-0357, i7-1165G7 @ 2.80 GHz with 100kb of data.

| crate                                       | version | variant   | us/iter | GiB/s |
|---------------------------------------------|---------|-----------|---------|-------|
| [adler32](https://crates.io/crates/adler32) | 1.2.0   | n/a       |   23.50 |  3.96 |
| adler32fast (this crate)                    | 1.1.0   | baseline  |   22.69 |  4.10 |
| adler32fast (this crate)                    | 1.1.0   | ssse3     |    3.09 | 30.17 |
| adler32fast (this crate)                    | 1.1.0   | avx2      |    2.36 | 39.43 |

Benchmarks using [criterion](https://docs.rs/criterion) can be run on stable Rust with `cargo bench`.

Contributions are welcomed for more SIMD variants!

## Memory Safety

Due to the use of SIMD intrinsics for the optimized implementations, this crate contains some amount of `unsafe` code.

`adler32fast` is fuzz-tested with [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz).
(_NOTE: `cargo-fuzz` does not work on Windows, but can run under WSL._)

`cargo-fuzz` currently requires the nightly toolchain.
```shell
$ cargo install cargo-fuzz
$ rustup toolchain install nightly

$ cargo +nightly fuzz run ssse3
```

## Credits

This work is based on [crc32fast](https://crates.io/crates/crc32fast) as inspiration.

The AVX2 implementation was derived from [simd-adler32](https://github.com/mcountryman/simd-adler32)

The SSSE3 implementation has been derived from Google's [Wuffs](https://github.com/google/wuffs/tree/main/std/adler32)
implementation.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
