pub(crate) const BASE: u32 = 65521;
const NMAX: usize = 5552;

#[derive(Copy, Clone, Debug)]
pub struct State {
    state: (u32, u32),
}

impl State {
    pub fn new(initial: u32) -> Self {
        Self {
            state: (initial & 0xffff, initial >> 16),
        }
    }

    pub fn finalize(&self) -> u32 {
        self.state.0 | (self.state.1 << 16)
    }

    pub fn reset(&mut self) {
        self.state = (1, 0);
    }

    pub fn update(&mut self, buf: &[u8]) {
        self.state = update_fast(self.state.0, self.state.1, buf);
    }
}

pub(crate) fn update_slow(mut a: u32, mut b: u32, buf: &[u8]) -> (u32, u32) {
    for &byte in buf {
        a += u32::from(byte);
        b += a;
    }
    (a % BASE, b % BASE)
}

fn update_fast(mut a: u32, mut b: u32, mut buf: &[u8]) -> (u32, u32) {
    while !buf.is_empty() {
        let mut remaining = &[0; 0][..];
        if buf.len() > NMAX {
            remaining = &buf[NMAX..];
            buf = &buf[..NMAX];
        }
        while buf.len() >= 16 {
            update_16(&mut a, &mut b, &buf);
            buf = &buf[16..];
        }
        let updated = update_slow(a, b, buf);
        a = updated.0;
        b = updated.1;
        buf = remaining;
    }
    (a, b)
}

#[inline(always)]
fn update_16(a: &mut u32, b: &mut u32, buf: &[u8]) {
    debug_assert!(buf.len() >= 16);
    for i in 0..16 {
        *a += u32::from(buf[i]);
        *b += *a;
    }
}

#[cfg(test)]
mod tests {
    quickcheck::quickcheck! {
        fn baseline_is_valid(initial: u32, buf: Vec<u8>) -> bool {
            let mut expected = adler32::RollingAdler32::from_value(initial);
            expected.update_buffer(&buf[..]);

            let mut actual = super::State::new(initial);
            actual.update(&buf[..]);

            expected.hash() == actual.finalize()
        }
    }
}
