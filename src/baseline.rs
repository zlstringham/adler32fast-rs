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

fn update_fast(mut a: u32, mut b: u32, buf: &[u8]) -> (u32, u32) {
    let chunks = buf.chunks(NMAX);
    for chunk in chunks {
        let inner_chunks = chunk.chunks_exact(16);
        let remainder = inner_chunks.remainder();
        for inner_chunk in inner_chunks {
            update_16(&mut a, &mut b, inner_chunk);
        }
        let updated = update_slow(a, b, remainder);
        a = updated.0;
        b = updated.1;
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
    #[test]
    fn baseline_is_valid() {
        fn golden(expected: u32, input: &[u8]) {
            let mut adler32 = super::State::new(1);
            adler32.update(input);
            assert_eq!(adler32.finalize(), expected);
        }

        // Goldens shamelessly borrowed from https://golang.org/src/hash/adler32/adler32_test.go
        golden(0x00000001, b"");
        golden(0x00620062, b"a");
        golden(0x012600c4, b"ab");
        golden(0x024d0127, b"abc");
        golden(0x03d8018b, b"abcd");
        golden(0x05c801f0, b"abcde");
        golden(0x081e0256, b"abcdef");
        golden(0x0adb02bd, b"abcdefg");
        golden(0x0e000325, b"abcdefgh");
        golden(0x118e038e, b"abcdefghi");
        golden(0x158603f8, b"abcdefghij");
        golden(0x3f090f02, b"Discard medicine more than two years old.");
        golden(
            0x46d81477,
            b"He who has a shady past knows that nice guys finish last.",
        );
        golden(0x40ee0ee1, b"I wouldn't marry him with a ten foot pole.");
        golden(
            0x16661315,
            b"Free! Free!/A trip/to Mars/for 900/empty jars/Burma Shave",
        );
        golden(
            0x5b2e1480,
            b"The days of the digital watch are numbered.  -Tom Stoppard",
        );
        golden(0x8c3c09ea, b"Nepal premier won't resign.");
        golden(
            0x45ac18fd,
            b"For every action there is an equal and opposite government program.",
        );
        golden(
            0x53c61462,
            b"His money is twice tainted: 'taint yours and 'taint mine.",
        );
        golden(0x7e511e63, b"There is no reason for any individual to have a computer in their home. -Ken Olsen, 1977");
        golden(
            0xe4801a6a,
            b"It's a tiny change to the code and not completely disgusting. - Bob Manchek",
        );
        golden(0x61b507df, b"size:  a.out:  bad magic");
        golden(
            0xb8631171,
            b"The major problem is with sendmail.  -Mark Horton",
        );
        golden(
            0x8b5e1904,
            b"Give me a rock, paper and scissors and I will move the world.  CCFestoon",
        );
        golden(
            0x7cc6102b,
            b"If the enemy is within range, then so are you.",
        );
        golden(
            0x700318e7,
            b"It's well we cannot hear the screams/That we create in others' dreams.",
        );
        golden(
            0x1e601747,
            b"You remind me of a TV show, but that's all right: I watch it anyway.",
        );
        golden(0xb55b0b09, b"C is as portable as Stonehedge!!");
        golden(0x39111dd0, b"Even if I could be Shakespeare, I think I should still choose to be Faraday. - A. Huxley");
        golden(0x91dd304f, b"The fugacity of a constituent in a mixture of gases at a given temperature is proportional to its mole fraction.  Lewis-Randall Rule");
        golden(
            0x2e5d1316,
            b"How can you write a big system without C++?  -Paul Glick",
        );
        golden(
            0xd0201df6,
            b"'Invariant assertions' is the most elegant programming technique!  -Tom Szymanski",
        );
        golden(0x86af0001, &b"\x00".repeat(100_000));
        golden(0x79660b4d, &b"a".repeat(100_000));
        golden(0x110588ee, &b"ABCDEFGHIJKLMNOPQRSTUVWXYZ".repeat(10_000));
    }
}
