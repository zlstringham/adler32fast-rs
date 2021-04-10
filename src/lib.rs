//! ## Example
//!
//! ```
//! use adler32fast::Adler32;
//!
//! let mut adler32 = Adler32::new();
//! adler32.update(b"foo bar baz");
//! let checksum = adler32.as_u32();
//! ```
//!
//! ## Performance
//!
//! This crate contains multiple Adler-32 implementations:
//!
//! - A fast baseline implementation which processes up to 16 bytes per iteration
//! - An optimized implementation for modern `x86`/`x86_64` using SSE instructions
//!
//! Calling the `Adler32::new`/`Adler32::from` constructors at runtime will perform a feature
//! detection to select the most optimal implementation for the current CPU feature set.
#![cfg_attr(not(any(feature = "std", test)), no_std)]

mod baseline;
mod specialized;

#[cfg(not(feature = "std"))]
use core::hash::Hasher;
#[cfg(feature = "std")]
use std::hash::Hasher;

const DEFAULT_INIT_STATE: u32 = 1;

#[derive(Copy, Clone, Debug)]
enum State {
    Baseline(baseline::State),
    Specialized(specialized::State),
}

#[derive(Copy, Clone, Debug)]
/// Represents an in-progress Adler-32 computation.
pub struct Adler32 {
    state: State,
}

impl Adler32 {
    /// Create a new `Adler32`.
    ///
    /// This will perform a CPU feature detection at runtime to select the most
    /// optimal implementation for the current processor architecture.
    pub fn new() -> Self {
        Self::from(DEFAULT_INIT_STATE)
    }

    /// Return the computed Adler-32 value.
    pub fn as_u32(&self) -> u32 {
        match self.state {
            State::Baseline(state) => state.finalize(),
            State::Specialized(state) => state.finalize(),
        }
    }

    /// Indicates whether the current implementation is SIMD-accelerated.
    pub fn is_simd_enabled(&self) -> bool {
        match self.state {
            State::Specialized(_) => true,
            _ => false,
        }
    }

    /// Reset the hash state.
    pub fn reset(&mut self) {
        match self.state {
            State::Baseline(ref mut state) => state.reset(),
            State::Specialized(ref mut state) => state.reset(),
        }
    }

    /// Process the given byte slice and update the hash state.
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

impl Hasher for Adler32 {
    fn finish(&self) -> u64 {
        u64::from(self.as_u32())
    }

    fn write(&mut self, bytes: &[u8]) {
        self.update(bytes);
    }
}

impl PartialEq<u32> for Adler32 {
    fn eq(&self, &other: &u32) -> bool {
        self.as_u32() == other
    }
}
