//! Seedable random number generation shared by every generation engine.
//!
//! Engines expose `&self` generation methods, so the RNG lives behind a
//! `RefCell` and is borrowed for the duration of each draw. Constructing an
//! engine with an explicit seed makes the whole pipeline reproducible:
//! the same seed and the same configuration always yield the same language.
//!
//! Borrows are short-lived and never nested — helper methods that need
//! randomness take `&mut StdRng` rather than reaching back into the engine.

use rand::rngs::StdRng;
use rand::SeedableRng;
use std::cell::RefCell;

/// An interior-mutable, optionally seeded RNG handle.
///
/// This is deliberately not `Sync`; engines are single-threaded values that
/// are cheap to clone-construct per thread if parallel generation is needed.
#[derive(Debug)]
pub struct SharedRng(RefCell<StdRng>);

impl SharedRng {
    /// Deterministic RNG: the same seed always produces the same stream.
    pub fn from_seed(seed: u64) -> Self {
        Self(RefCell::new(StdRng::seed_from_u64(seed)))
    }

    /// Non-deterministic RNG seeded from the operating system.
    pub fn from_entropy() -> Self {
        Self(RefCell::new(StdRng::from_entropy()))
    }

    /// `Some(seed)` gives a reproducible stream, `None` falls back to entropy.
    pub fn from_optional_seed(seed: Option<u64>) -> Self {
        match seed {
            Some(seed) => Self::from_seed(seed),
            None => Self::from_entropy(),
        }
    }

    /// Borrow the underlying RNG for a single operation.
    ///
    /// # Panics
    /// Panics if called re-entrantly from inside another `with` closure on the
    /// same handle. Pass the `&mut StdRng` down instead of nesting calls.
    pub fn with<R>(&self, f: impl FnOnce(&mut StdRng) -> R) -> R {
        f(&mut self.0.borrow_mut())
    }
}

impl Default for SharedRng {
    fn default() -> Self {
        Self::from_entropy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn same_seed_gives_same_stream() {
        let a = SharedRng::from_seed(42);
        let b = SharedRng::from_seed(42);
        for _ in 0..32 {
            assert_eq!(a.with(|r| r.gen::<u64>()), b.with(|r| r.gen::<u64>()));
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let a = SharedRng::from_seed(1);
        let b = SharedRng::from_seed(2);
        let a: Vec<u64> = (0..16).map(|_| a.with(|r| r.gen())).collect();
        let b: Vec<u64> = (0..16).map(|_| b.with(|r| r.gen())).collect();
        assert_ne!(a, b);
    }

    #[test]
    fn optional_seed_none_is_usable() {
        let rng = SharedRng::from_optional_seed(None);
        let _: u64 = rng.with(|r| r.gen());
    }
}
