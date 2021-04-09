#![cfg_attr(not(any(feature = "std", test)), no_std)]

mod baseline;
mod specialized;

const DEFAULT_INIT_STATE: u32 = 1;

#[derive(Copy, Clone, Debug)]
enum State {
    Baseline(baseline::State),
    Specialized(specialized::State),
}

#[derive(Copy, Clone, Debug)]
pub struct Adler32 {
    state: State,
}

impl Adler32 {
    pub fn new() -> Self {
        Self::from(DEFAULT_INIT_STATE)
    }

    pub fn is_simd_enabled(&self) -> bool {
        match self.state {
            State::Specialized(_) => true,
            _ => false,
        }
    }

    pub fn finalize(&self) -> u32 {
        match self.state {
            State::Baseline(state) => state.finalize(),
            State::Specialized(state) => state.finalize(),
        }
    }

    pub fn reset(&mut self) {
        match self.state {
            State::Baseline(ref mut state) => state.reset(),
            State::Specialized(ref mut state) => state.reset(),
        }
    }

    pub fn update(&mut self, buf: &[u8]) {
        match self.state {
            State::Baseline(ref mut state) => state.update(buf),
            State::Specialized(ref mut state) => state.update(buf),
        }
    }

    #[doc(hidden)]
    pub fn internal_new_baseline(initial: u32) -> Self {
        Self {
            state: State::Baseline(baseline::State::new(initial)),
        }
    }

    #[doc(hidden)]
    pub fn internal_new_specialized(initial: u32) -> Option<Self> {
        if let Some(state) = specialized::State::new(initial) {
            Some(Self {
                state: State::Specialized(state),
            })
        } else {
            None
        }
    }
}

impl Default for Adler32 {
    fn default() -> Self {
        Self::new()
    }
}

impl From<u32> for Adler32 {
    fn from(initial: u32) -> Self {
        Self::internal_new_specialized(initial)
            .unwrap_or_else(|| Self::internal_new_baseline(initial))
    }
}
