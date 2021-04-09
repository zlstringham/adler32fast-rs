use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
        mod sse;
        pub use sse::State;
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
