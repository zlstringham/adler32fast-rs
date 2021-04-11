use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
        pub mod avx2;
        pub mod ssse3;

        #[derive(Copy, Clone, Debug)]
        pub enum Arch {
            Avx2(avx2::State),
            Ssse3(ssse3::State),
        }

        #[derive(Copy, Clone, Debug)]
        pub struct State {
            arch: Arch,
        }

        impl State {
            pub fn new(initial: u32) -> Option<Self> {
                avx2::State::new(initial)
                    .map(|a| Self {
                        arch: Arch::Avx2(a),
                    })
                    .or_else(|| {
                        ssse3::State::new(initial).map(|a| Self {
                            arch: Arch::Ssse3(a),
                        })
                    })
            }

            pub fn finalize(self) -> u32 {
                match self.arch {
                    Arch::Avx2(state) => state.finalize(),
                    Arch::Ssse3(state) => state.finalize(),
                }
            }

            pub fn reset(&mut self) {
                match self.arch {
                    Arch::Avx2(ref mut state) => state.reset(),
                    Arch::Ssse3(ref mut state) => state.reset(),
                };
            }

            pub fn update(&mut self, buf: &[u8]) {
                match self.arch {
                    Arch::Avx2(ref mut state) => state.update(buf),
                    Arch::Ssse3(ref mut state) => state.update(buf),
                }
            }
        }
    } else {
        #[derive(Copy, Clone, Debug)]
        pub enum State {}

        impl State {
            pub fn new(_: u32) -> Option<Self> {
                None
            }

            pub fn finalize(self) -> u32 {
                unimplemented!()
            }

            pub fn reset(&mut self) {
                unimplemented!()
            }

            pub fn update(&mut self, _: &[u8]) {
                unimplemented!()
            }
        }
    }
}
