const NMAX: usize = 5536;
const CHUNK_SIZE: usize = 32;

#[derive(Copy, Clone, Debug)]
pub struct State {
    state: (u32, u32),
}

impl State {
    #[cfg(not(feature = "std"))]
    pub fn new(initial: u32) -> Option<Self> {
        if cfg!(target_feature = "avx2") {
            // SAFETY: Ensure that all required instructions are supported by the CPU.
            Some(Self {
                state: (initial & 0xffff, initial >> 16),
            })
        } else {
            None
        }
    }

    #[cfg(feature = "std")]
    pub fn new(initial: u32) -> Option<Self> {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: Ensure that all required instructions are supported by the CPU.
            Some(Self {
                state: (initial & 0xffff, initial >> 16),
            })
        } else {
            None
        }
    }

    pub fn finalize(self) -> u32 {
        self.state.0 | (self.state.1 << 16)
    }

    pub fn reset(&mut self) {
        self.state = (1, 0)
    }

    pub fn update(&mut self, buf: &[u8]) {
        self.state = unsafe { update_simd(self.state.0, self.state.1, buf) }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn update_simd(mut a: u32, mut b: u32, buf: &[u8]) -> (u32, u32) {
    let chunks = buf.chunks_exact(NMAX);
    let mut remainder = chunks.remainder();
    for chunk in chunks {
        add_reduce(&mut a, &mut b, chunk);
        a %= crate::baseline::BASE;
        b %= crate::baseline::BASE;
    }
    remainder = add_reduce(&mut a, &mut b, remainder);
    crate::baseline::update_slow(a, b, remainder)
}

#[inline(always)]
unsafe fn add_reduce<'a>(a: &mut u32, b: &mut u32, chunk: &'a [u8]) -> &'a [u8] {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;

    if chunk.len() < CHUNK_SIZE {
        return chunk;
    }
    let v_zeroes = _mm256_setzero_si256();
    let v_ones = _mm256_set1_epi16(1);
    let v_weights = _mm256_set_epi8(
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30, 31, 32,
    );

    let inner_chunks = chunk.chunks_exact(CHUNK_SIZE);
    let remainder = inner_chunks.remainder();
    let mut p_v = _mm256_set_epi32(0, 0, 0, 0, 0, 0, 0, (*a * inner_chunks.len() as u32) as _);
    let mut a_v = _mm256_setzero_si256();
    let mut b_v = _mm256_set_epi32(0, 0, 0, 0, 0, 0, 0, *b as _);

    for inner_chunk in inner_chunks {
        let block = _mm256_lddqu_si256(inner_chunk.as_ptr() as *const __m256i);
        p_v = _mm256_add_epi32(p_v, a_v);
        a_v = _mm256_add_epi32(a_v, _mm256_sad_epu8(block, v_zeroes));
        let mad = _mm256_maddubs_epi16(block, v_weights);
        b_v = _mm256_add_epi32(b_v, _mm256_madd_epi16(mad, v_ones));
    }

    let mut sum = _mm_add_epi32(
        _mm256_castsi256_si128(a_v),
        _mm256_extracti128_si256(a_v, 1),
    );
    let mut hi = _mm_unpackhi_epi64(sum, sum);
    sum = _mm_add_epi32(hi, sum);
    hi = _mm_shuffle_epi32(sum, 177);
    sum = _mm_add_epi32(sum, hi);
    *a += _mm_cvtsi128_si32(sum) as u32;

    b_v = _mm256_add_epi32(b_v, _mm256_slli_epi32(p_v, 5));
    sum = _mm_add_epi32(
        _mm256_castsi256_si128(b_v),
        _mm256_extracti128_si256(b_v, 1),
    );
    hi = _mm_unpackhi_epi64(sum, sum);
    sum = _mm_add_epi32(hi, sum);
    hi = _mm_shuffle_epi32(sum, 177);
    sum = _mm_add_epi32(sum, hi);
    *b = _mm_cvtsi128_si32(sum) as u32;

    remainder
}

#[cfg(test)]
mod tests {
    quickcheck::quickcheck! {
        fn avx2_is_same_as_baseline(init: u32, buf: Vec<u8>) -> bool {
            let mut expected = crate::baseline::State::new(init);
            let mut actual = super::State::new(init).expect("avx2 not supported");
            expected.update(&buf);
            actual.update(&buf);
            expected.finalize() == actual.finalize()
        }

        fn avx2_supports_random_alignment(init: u32, chunks: Vec<(Vec<u8>, usize)>) -> bool {
            let mut expected = adler32::RollingAdler32::from_value(init);
            let mut actual = super::State::new(init).expect("avx2 not supported");
            for (chunk, mut offset) in chunks {
                // Simulate random alignments by offsetting the slice by up to 15 bytes
                offset &= 0xf;
                if chunk.len() <= offset {
                    expected.update_buffer(&chunk);
                    actual.update(&chunk);
                } else {
                    expected.update_buffer(&chunk[offset..]);
                    actual.update(&chunk[offset..]);
                }
            }
            expected.hash() == actual.finalize()
        }
    }

    #[test]
    fn avx2_is_valid_for_large_input() {
        let v = vec![100; super::NMAX * 4];
        let mut expected = crate::baseline::State::new(1);
        let mut actual = super::State::new(1).expect("avx2 not supported");
        expected.update(&v);
        actual.update(&v);
        assert_eq!(expected.finalize(), actual.finalize())
    }
}
