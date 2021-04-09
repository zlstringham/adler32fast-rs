# adler32fast

_Fast, SIMD-accelerated Adler-32 checksum computation_

## Usage

```rust
use adler32fast::Adler32;

let mut adler32 = adler32::new();
adler32.update(b"foo bar baz");
let checksum = adler32.finalize();
```

## Performance

This crate contains multiple Adler-32 implementations:

- A fast baseline implementation which processes up to 16 bytes per iteration
- An optimized implementation for modern `x86`/`x86_64` using `sse` instructions

Calling the `Adler32::new` or `Adler32::from` constructor at runtime will perform a feature detection to
select the most optimal implementation for the current CPU feature set.

| crate                                       | version | variant   | ns/iter | MB/s |
|---------------------------------------------|---------|-----------|---------|------|
| [adler32](https://crates.io/crates/adler32) | 1.2.0   | n/a       |    TODO | TODO |
| adler32fast (this crate)                    | 0.1.0   | baseline  |    TODO | TODO |
| adler32fast (this crate)                    | 0.1.0   | sse       |    TODO | TODO |

## Memory Safety

Due to the use of SIMD intrinsics for the optimized implementations, this crate contains some amount of `unsafe` code.

TODO: Fuzz testing.

## Credits

This work is based on [crc32fast](https://crates.io/crates/crc32fast) as inspiration.

The SSE implementation has been derived from Google's [Wuffs](https://github.com/google/wuffs/tree/main/std/adler32)
implementation.

## TODO:
- [] Fuzz-testing
- [] Benchmarking
- [] Documentation
- [] Implement more traits on `Adler32` struct

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
