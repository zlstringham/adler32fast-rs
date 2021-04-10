const NMAX: usize = 5536;

#[derive(Copy, Clone, Debug)]
pub struct State {
    state: (u32, u32),
}

impl State {
    #[cfg(not(feature = "std"))]
    pub fn new(initial: u32) -> Option<Self> {
        if cfg!(target_feature = "sse2") && cfg!(target_feature = "sse4.1") {
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
        if is_x86_feature_detected!("sse2") && is_x86_feature_detected!("sse4.1") {
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

#[target_feature(enable = "sse2", enable = "sse4.1")]
unsafe fn update_simd(mut a: u32, mut b: u32, buf: &[u8]) -> (u32, u32) {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;

    let v_zeroes = _mm_set1_epi16(0);
    let v_ones = _mm_set1_epi16(1);
    let v_weights_left = _mm_set_epi8(
        17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
    );
    let v_weights_right = _mm_set_epi8(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);

    let chunks = buf.chunks(NMAX);
    for chunk in chunks {
        let v_num_iterate_bytes = (buf.len() & 0xffff_ffe0) as u32;
        b += a * v_num_iterate_bytes;
        let mut v_v1 = _mm_setzero_si128();
        let mut v_v2j = _mm_setzero_si128();
        let mut v_v2k = _mm_setzero_si128();

        let inner_chunks = chunk.chunks_exact(32);
        let remainder = inner_chunks.remainder();
        for inner_chunk in inner_chunks {
            let v_p_left = _mm_lddqu_si128((&inner_chunk).as_ptr() as *const __m128i);
            let v_p_right = _mm_lddqu_si128((&&inner_chunk[16..]).as_ptr() as *const __m128i);
            v_v2j = _mm_add_epi32(v_v2j, v_v1);
            v_v1 = _mm_add_epi32(v_v1, _mm_sad_epu8(v_p_left, v_zeroes));
            v_v1 = _mm_add_epi32(v_v1, _mm_sad_epu8(v_p_right, v_zeroes));
            v_v2k = _mm_add_epi32(
                v_v2k,
                _mm_madd_epi16(v_ones, _mm_maddubs_epi16(v_p_left, v_weights_left)),
            );
            v_v2k = _mm_add_epi32(
                v_v2k,
                _mm_madd_epi16(v_ones, _mm_maddubs_epi16(v_p_right, v_weights_right)),
            );
        }

        v_v1 = _mm_add_epi32(v_v1, _mm_shuffle_epi32(v_v1, 177));
        v_v1 = _mm_add_epi32(v_v1, _mm_shuffle_epi32(v_v1, 78));
        a += _mm_cvtsi128_si32(v_v1) as u32;

        let mut v_v2 = _mm_add_epi32(v_v2k, _mm_slli_epi32(v_v2j, 5));
        v_v2 = _mm_add_epi32(v_v2, _mm_shuffle_epi32(v_v2, 177));
        v_v2 = _mm_add_epi32(v_v2, _mm_shuffle_epi32(v_v2, 78));
        b += _mm_cvtsi128_si32(v_v2) as u32;

        let updated = crate::baseline::update_slow(a, b, remainder);
        a = updated.0;
        b = updated.1;
    }
    (a, b)
}

#[cfg(test)]
mod tests {
    quickcheck::quickcheck! {
        fn sse_is_valid(init: u32, buf: Vec<u8>) -> bool {
            let mut expected = adler32::RollingAdler32::from_value(init);
            expected.update_buffer(&buf);

            let mut actual = super::State::new(init).expect("sse not supported");
            actual.update(&buf);

            expected.hash() == actual.finalize()
        }

        fn sse_supports_random_alignment(init: u32, chunks: Vec<(Vec<u8>, usize)>) -> bool {
            let mut expected = adler32::RollingAdler32::from_value(init);
            let mut actual = super::State::new(init).expect("sse not supported");
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
}
