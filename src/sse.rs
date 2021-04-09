#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

const BASE: u32 = 65521;
const NMAX: usize = 5536;

pub struct Adler32 {
    a: u32,
    b: u32,
}

impl Adler32 {
    pub fn new() -> Self {
        Self::from(1)
    }

    pub fn hash(&self) -> u32 {
        self.a | (self.b << 16)
    }

    pub unsafe fn update(&mut self, mut data: &[u8]) {
        let v_zeroes = _mm_set1_epi16(0);
        let v_ones = _mm_set1_epi16(1);
        let v_weights_left = _mm_set_epi8(
            17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
        );
        let v_weights_right = _mm_set_epi8(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
        while !data.is_empty() {
            let mut remaining = &[0; 0][..];
            if data.len() > NMAX {
                remaining = &data[NMAX..];
                data = &data[..NMAX];
            }
            let v_num_iterate_bytes = (data.len() & 0xffff_ffe0) as u32;
            self.b += self.a * v_num_iterate_bytes;
            let mut v_v1 = _mm_setzero_si128();
            let mut v_v2j = _mm_setzero_si128();
            let mut v_v2k = _mm_setzero_si128();

            while data.len() >= 32 {
                let v_p_left = _mm_lddqu_si128((&data).as_ptr() as *const __m128i);
                let v_p_right = _mm_lddqu_si128((&&data[16..]).as_ptr() as *const __m128i);
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
                data = &data[32..];
            }

            v_v1 = _mm_add_epi32(v_v1, _mm_shuffle_epi32(v_v1, 177));
            v_v1 = _mm_add_epi32(v_v1, _mm_shuffle_epi32(v_v1, 78));
            self.a += _mm_cvtsi128_si32(v_v1) as u32;
            let mut v_v2 = _mm_add_epi32(v_v2k, _mm_slli_epi32(v_v2j, 5));
            v_v2 = _mm_add_epi32(v_v2, _mm_shuffle_epi32(v_v2, 177));
            v_v2 = _mm_add_epi32(v_v2, _mm_shuffle_epi32(v_v2, 78));
            self.b += _mm_cvtsi128_si32(v_v2) as u32;

            for &b in data {
                self.a += u32::from(b);
                self.b += self.a;
            }

            self.a %= BASE;
            self.b %= BASE;
            data = remaining;
        }
    }
}

impl Default for Adler32 {
    fn default() -> Self {
        Self::new()
    }
}

impl From<u32> for Adler32 {
    fn from(adler32: u32) -> Self {
        Adler32 {
            a: adler32 & 0xffff,
            b: adler32 >> 16,
        }
    }
}

#[cfg(test)]
mod tests {
    use adler32::RollingAdler32;
    use quickcheck::quickcheck;

    quickcheck! {
        fn adler32fast_same_as_adler32(checksum: u32, bytes: Vec<u8>) -> bool {
            let mut expected = RollingAdler32::from_value(checksum);
            let mut actual = super::Adler32::from(checksum);

            expected.update_buffer(&bytes);
            unsafe {actual.update(&bytes);}
            expected.hash() == actual.hash()
        }
    }
}
